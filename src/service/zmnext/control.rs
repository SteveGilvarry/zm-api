//! zm-api's side of zm-next's worker control protocol (Phase 2).
//!
//! Contract: zm-next `docs/Worker_Control_Protocol.md`. This module holds the
//! parts that don't depend on how the worker was started:
//!
//! * [`SchemaCache`] / [`fetch_schemas`]: plugin config schemas from
//!   `describe_plugins`, cached by `schema_sha256` from the hello;
//! * [`validate_graph_with_schemas`]: validation against those schemas (the
//!   doc's JSON Schema subset), with path-specific errors;
//! * [`configure_command`] / [`configure_errors`]: the `configure` message and
//!   its rejection;
//! * [`pipeline_hash`]: the hash a hello reports, for adopting a running worker.
//!
//! Every test built from the doc's JSON examples says so, to be replaced by
//! zm-next's `tests/contract/` transcripts when they exist.

use std::collections::BTreeMap;
use std::time::Duration;

use dashmap::DashMap;
use serde::Serialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use super::secrets::{reference_name, SecretMap};
use crate::streaming::source::command::CommandError;
use crate::streaming::source::protocol::WorkerHello;
use crate::streaming::source::SourceRouter;

/// One validation or configure error, located by path, e.g.
/// `plugins[0].children[1].cfg.iou_threshold`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize, ToSchema)]
pub struct PathError {
    pub path: String,
    pub message: String,
}

impl PathError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Plugin config schemas, shared by every monitor: keyed by `schema_sha256`,
/// so a schema is fetched once per plugin version across all workers.
#[derive(Default)]
pub struct SchemaCache {
    by_sha: DashMap<String, Value>,
}

impl SchemaCache {
    pub fn get(&self, sha: &str) -> Option<Value> {
        self.by_sha.get(sha).map(|v| v.clone())
    }

    pub fn insert(&self, sha: &str, schema: Value) {
        self.by_sha.insert(sha.to_string(), schema);
    }
}

/// The process-wide schema cache.
pub fn schema_cache() -> &'static SchemaCache {
    static CACHE: std::sync::OnceLock<SchemaCache> = std::sync::OnceLock::new();
    CACHE.get_or_init(SchemaCache::default)
}

/// Kind → schema for every plugin in `hello`, fetching the ones `cache`
/// doesn't hold with a single `describe_plugins`.
pub async fn fetch_schemas(
    router: &SourceRouter,
    monitor_id: u32,
    hello: &WorkerHello,
    cache: &SchemaCache,
    timeout: Duration,
) -> Result<BTreeMap<String, Value>, CommandError> {
    let mut out = BTreeMap::new();
    let mut missing = Vec::new();
    for p in &hello.plugins {
        match cache
            .get(&p.schema_sha256)
            .filter(|_| !p.schema_sha256.is_empty())
        {
            Some(schema) => {
                out.insert(p.kind.clone(), schema);
            }
            None => missing.push(p.kind.clone()),
        }
    }
    if !missing.is_empty() {
        let data = router
            .send_control(
                monitor_id,
                json!({ "cmd": "describe_plugins", "kinds": missing }),
                timeout,
            )
            .await?;
        store_described(&data, hello, cache, &mut out);
    }
    Ok(out)
}

/// Fold a `describe_plugins` result (`{kind: {version, schema}}`) into `out`
/// and the cache.
fn store_described(
    data: &Value,
    hello: &WorkerHello,
    cache: &SchemaCache,
    out: &mut BTreeMap<String, Value>,
) {
    let Some(map) = data.as_object() else {
        return;
    };
    for (kind, entry) in map {
        let Some(schema) = entry.get("schema") else {
            continue;
        };
        if let Some(p) = hello.plugins.iter().find(|p| &p.kind == kind) {
            if !p.schema_sha256.is_empty() {
                cache.insert(&p.schema_sha256, schema.clone());
            }
        }
        out.insert(kind.clone(), schema.clone());
    }
}

/// How `x-secret` keys are checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretRule {
    /// A stored graph being saved: a literal is allowed (it is moved to the
    /// secret store before the graph is written).
    LiteralOrReference,
    /// A configure payload: the doc requires a `$secret` reference.
    ReferenceOnly,
}

/// Validate a graph's nodes against plugin schemas. Returns every error, not
/// just the first, so an editor can mark them all.
pub fn validate_graph_with_schemas(
    graph: &Value,
    schemas: &BTreeMap<String, Value>,
    rule: SecretRule,
) -> Vec<PathError> {
    let mut errors = Vec::new();
    match graph.get("plugins").and_then(Value::as_array) {
        Some(nodes) => {
            for (i, node) in nodes.iter().enumerate() {
                validate_node(node, &format!("plugins[{i}]"), schemas, rule, &mut errors);
            }
        }
        None => errors.push(PathError::new(
            "plugins",
            "must be an array of plugin nodes",
        )),
    }
    errors
}

fn validate_node(
    node: &Value,
    path: &str,
    schemas: &BTreeMap<String, Value>,
    rule: SecretRule,
    errors: &mut Vec<PathError>,
) {
    let Some(obj) = node.as_object() else {
        errors.push(PathError::new(path, "must be an object"));
        return;
    };
    let kind = obj.get("kind").and_then(Value::as_str).unwrap_or_default();
    match schemas.get(kind) {
        None => errors.push(PathError::new(
            format!("{path}.kind"),
            format!("`{kind}` is not a plugin this worker has"),
        )),
        Some(schema) => {
            let cfg = obj
                .get("cfg")
                .or_else(|| obj.get("config"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            check(&cfg, schema, &format!("{path}.cfg"), rule, errors);
        }
    }
    if let Some(children) = obj.get("children").and_then(Value::as_array) {
        for (i, child) in children.iter().enumerate() {
            validate_node(
                child,
                &format!("{path}.children[{i}]"),
                schemas,
                rule,
                errors,
            );
        }
    }
}

fn type_matches(v: &Value, ty: &str) -> bool {
    match ty {
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "boolean" => v.is_boolean(),
        "null" => v.is_null(),
        "number" => v.is_number(),
        "integer" => v.as_i64().is_some() || v.as_u64().is_some(),
        _ => true,
    }
}

/// The doc's schema subset: `type`, `properties`, `required`, `enum`,
/// `minimum`/`maximum`, `items`, `additionalProperties`, plus `x-secret`.
/// `default` and `description` don't constrain.
fn check(v: &Value, schema: &Value, path: &str, rule: SecretRule, errors: &mut Vec<PathError>) {
    let Some(s) = schema.as_object() else {
        return;
    };

    if s.get("x-secret").and_then(Value::as_bool) == Some(true) {
        match (reference_name(v).is_some(), v.is_string(), rule) {
            (true, _, _) => return,
            (false, true, SecretRule::LiteralOrReference) => {}
            _ => {
                errors.push(PathError::new(path, "secret must be a $secret reference"));
                return;
            }
        }
    }

    if let Some(ty) = s.get("type") {
        let ok = match ty {
            Value::String(t) => type_matches(v, t),
            Value::Array(ts) => ts
                .iter()
                .filter_map(Value::as_str)
                .any(|t| type_matches(v, t)),
            _ => true,
        };
        if !ok {
            errors.push(PathError::new(path, format!("must be of type {ty}")));
            return;
        }
    }

    if let Some(options) = s.get("enum").and_then(Value::as_array) {
        if !options.contains(v) {
            errors.push(PathError::new(
                path,
                format!("must be one of {}", Value::Array(options.clone())),
            ));
        }
    }

    if let Some(n) = v.as_f64() {
        if let Some(min) = s.get("minimum").and_then(Value::as_f64) {
            if n < min {
                errors.push(PathError::new(path, format!("must be >= {min}")));
            }
        }
        if let Some(max) = s.get("maximum").and_then(Value::as_f64) {
            if n > max {
                errors.push(PathError::new(path, format!("must be <= {max}")));
            }
        }
    }

    if let (Some(obj), true) = (
        v.as_object(),
        s.contains_key("properties")
            || s.contains_key("required")
            || s.contains_key("additionalProperties"),
    ) {
        let props = s.get("properties").and_then(Value::as_object);
        if let Some(required) = s.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                if !obj.contains_key(key) {
                    errors.push(PathError::new(format!("{path}.{key}"), "is required"));
                }
            }
        }
        for (key, value) in obj {
            let child = format!("{path}.{key}");
            match props.and_then(|p| p.get(key)) {
                Some(sub) => check(value, sub, &child, rule, errors),
                None => match s.get("additionalProperties") {
                    Some(Value::Bool(false)) => {
                        errors.push(PathError::new(child, "is not a known setting"))
                    }
                    Some(extra @ Value::Object(_)) => check(value, extra, &child, rule, errors),
                    _ => {}
                },
            }
        }
    }

    if let (Some(arr), Some(items)) = (v.as_array(), s.get("items")) {
        for (i, item) in arr.iter().enumerate() {
            check(item, items, &format!("{path}[{i}]"), rule, errors);
        }
    }
}

/// The `configure` command, with secrets separate from the pipeline.
pub fn configure_command(pipeline: &Value, secrets: &SecretMap, apply: &str) -> Value {
    json!({
        "cmd": "configure",
        "pipeline": pipeline,
        "secrets": secrets,
        "apply": apply,
    })
}

/// The path errors from a refused configure (`data.errors`), or one error
/// carrying the refusal message when the worker gave none.
pub fn configure_errors(message: &str, data: &Value) -> Vec<PathError> {
    let errors: Vec<PathError> = data
        .get("errors")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|e| serde_json::from_value(e.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    if errors.is_empty() {
        vec![PathError::new("", message)]
    } else {
        errors
    }
}

/// `sha256:<hex>` of the pipeline's canonical JSON (object keys sorted, no
/// whitespace), with `$secret` references as they are and no secret values.
///
/// Provisional: the doc says "canonical JSON of the active pipeline with
/// secret values removed" without defining canonical, and zm-next's
/// nlohmann::json may format numbers differently from serde_json. Until the
/// doc pins the rule (proposed: RFC 8785 over the pipeline as sent in
/// configure), a mismatch only means zm-api reconfigures instead of adopting.
pub fn pipeline_hash(pipeline: &Value) -> String {
    fn sorted(v: &Value) -> Value {
        match v {
            Value::Object(m) => {
                let mut out = Map::new();
                let mut keys: Vec<_> = m.keys().collect();
                keys.sort();
                for k in keys {
                    out.insert(k.clone(), sorted(&m[k]));
                }
                Value::Object(out)
            }
            Value::Array(a) => Value::Array(a.iter().map(sorted).collect()),
            other => other.clone(),
        }
    }
    let canonical = serde_json::to_vec(&sorted(pipeline)).expect("Value serialises");
    let digest = Sha256::digest(&canonical);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

/// What to do with a worker found running when zm-api starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Adoption {
    /// Its pipeline is the one zm-api would send: leave it running.
    Adopt,
    /// Running with a different or unknown pipeline: send configure.
    Reconfigure,
    /// No hello (older worker): zm-api can't tell what it runs.
    Unknown,
}

/// Decide from the hello whether a running worker can be adopted as is.
pub fn adoption(hello: Option<&WorkerHello>, wanted: &Value) -> Adoption {
    match hello {
        None => Adoption::Unknown,
        Some(h)
            if h.state == "running"
                && h.pipeline_hash.as_deref() == Some(&pipeline_hash(wanted)) =>
        {
            Adoption::Adopt
        }
        Some(_) => Adoption::Reconfigure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::source::protocol::parse_worker_hello;

    fn schemas() -> BTreeMap<String, Value> {
        let mut m = BTreeMap::new();
        m.insert(
            "capture_rtsp_multi".into(),
            json!({"type": "object", "required": ["streams"], "properties": {
                "streams": {"type": "array", "items": {"type": "object", "required": ["url"],
                    "additionalProperties": false,
                    "properties": {
                        "url": {"type": "string"},
                        "username": {"type": "string", "x-secret": true},
                        "password": {"type": "string", "x-secret": true},
                        "stream_id": {"type": "integer", "minimum": 0}
                    }}}
            }}),
        );
        m.insert(
            "tracker".into(),
            json!({"type": "object", "properties": {
                "iou_threshold": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.3},
                "mode": {"enum": ["sort", "bytetrack"]}
            }}),
        );
        m
    }

    /// The configure pipeline from the doc's "Configure" example.
    fn doc_pipeline() -> Value {
        json!({"plugins": [
            {"id": "capture", "kind": "capture_rtsp_multi",
             "cfg": {"streams": [{"url": "rtsp://10.0.0.5/Streaming/Channels/102",
                                  "username": {"$secret": "cam.user"},
                                  "password": {"$secret": "cam.pass"}}]},
             "children": [
                {"id": "a", "kind": "tracker", "cfg": {}},
                {"id": "b", "kind": "tracker", "cfg": {"iou_threshold": 1.5}}
             ]}
        ]})
    }

    #[test]
    fn reports_every_error_with_the_docs_paths() {
        let errors =
            validate_graph_with_schemas(&doc_pipeline(), &schemas(), SecretRule::ReferenceOnly);
        assert_eq!(
            errors,
            vec![PathError::new(
                "plugins[0].children[1].cfg.iou_threshold",
                "must be <= 1"
            )]
        );
    }

    #[test]
    fn a_literal_secret_is_refused_in_a_configure_but_allowed_before_saving() {
        let mut p = doc_pipeline();
        p["plugins"][0]["cfg"]["streams"][0]["password"] = json!("p@ss:w/d");
        p["plugins"][0]["children"][1]["cfg"]["iou_threshold"] = json!(0.5);
        assert_eq!(
            validate_graph_with_schemas(&p, &schemas(), SecretRule::ReferenceOnly),
            vec![PathError::new(
                "plugins[0].cfg.streams[0].password",
                "secret must be a $secret reference"
            )]
        );
        assert!(
            validate_graph_with_schemas(&p, &schemas(), SecretRule::LiteralOrReference).is_empty()
        );
    }

    #[test]
    fn unknown_kinds_types_enums_required_and_extra_keys() {
        let g = json!({"plugins": [
            {"kind": "tracker", "cfg": {"mode": "hungarian", "iou_threshold": "high"}},
            {"kind": "capture_rtsp_multi", "cfg": {"streams": [{"stream_id": -1, "codec": "h265"}]}},
            {"kind": "output_webrtc"}
        ]});
        let errors = validate_graph_with_schemas(&g, &schemas(), SecretRule::ReferenceOnly);
        let paths: Vec<&str> = errors.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "plugins[0].cfg.iou_threshold",
                "plugins[0].cfg.mode",
                "plugins[1].cfg.streams[0].url",
                "plugins[1].cfg.streams[0].codec",
                "plugins[1].cfg.streams[0].stream_id",
                "plugins[2].kind",
            ],
            "{errors:#?}"
        );
    }

    #[test]
    fn configure_message_keeps_secrets_apart() {
        let mut secrets = SecretMap::new();
        secrets.insert("cam.user".into(), "admin".into());
        secrets.insert("cam.pass".into(), "p@ss:w/d".into());
        let cmd = configure_command(&doc_pipeline(), &secrets, "restart");
        assert_eq!(cmd["cmd"], "configure");
        assert_eq!(cmd["apply"], "restart");
        assert_eq!(cmd["secrets"]["cam.pass"], "p@ss:w/d");
        assert!(!cmd["pipeline"].to_string().contains("p@ss"));
    }

    #[test]
    fn configure_rejection_errors_come_from_data() {
        let data = json!({"errors": [
            {"path": "plugins[0].children[1].cfg.iou_threshold", "message": "must be <= 1"},
            {"path": "plugins[0].cfg.streams[0].password", "message": "secret must be a $secret reference"}
        ]});
        let errors = configure_errors("invalid pipeline", &data);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[1].path, "plugins[0].cfg.streams[0].password");
        assert_eq!(
            configure_errors("unknown_command: configure", &Value::Null),
            vec![PathError::new("", "unknown_command: configure")]
        );
    }

    #[test]
    fn pipeline_hash_ignores_key_order_and_whitespace() {
        let a = json!({"plugins": [{"kind": "tracker", "id": "t", "cfg": {"b": 1, "a": 2}}]});
        let b: Value = serde_json::from_str(
            r#"{ "plugins" : [ {"cfg":{"a":2,"b":1}, "id":"t", "kind":"tracker"} ] }"#,
        )
        .unwrap();
        assert_eq!(pipeline_hash(&a), pipeline_hash(&b));
        assert!(pipeline_hash(&a).starts_with("sha256:"));
        assert_eq!(pipeline_hash(&a).len(), "sha256:".len() + 64);
        let c = json!({"plugins": [{"kind": "tracker", "id": "t", "cfg": {"b": 1, "a": 3}}]});
        assert_ne!(pipeline_hash(&a), pipeline_hash(&c));
    }

    #[test]
    fn adoption_decision() {
        let wanted = doc_pipeline();
        let hash = pipeline_hash(&wanted);
        let hello = |state: &str, h: &str| {
            parse_worker_hello(format!(r#"{{"state":"{state}","pipeline_hash":"{h}"}}"#).as_bytes())
                .unwrap()
        };
        assert_eq!(
            adoption(Some(&hello("running", &hash)), &wanted),
            Adoption::Adopt
        );
        assert_eq!(
            adoption(Some(&hello("running", "sha256:other")), &wanted),
            Adoption::Reconfigure
        );
        assert_eq!(
            adoption(Some(&hello("unconfigured", &hash)), &wanted),
            Adoption::Reconfigure
        );
        assert_eq!(adoption(None, &wanted), Adoption::Unknown);
    }

    #[test]
    fn describe_plugins_result_fills_the_cache_by_sha() {
        let hello = parse_worker_hello(
            br#"{"state":"running","plugins":[{"kind":"tracker","version":"1.2.0","schema_sha256":"77e0"}]}"#,
        )
        .unwrap();
        let cache = SchemaCache::default();
        let mut out = BTreeMap::new();
        // The doc's describe_plugins data shape.
        let data = json!({"tracker": {"version": "1.2.0", "schema": {"type": "object"}}});
        store_described(&data, &hello, &cache, &mut out);
        assert_eq!(out["tracker"]["type"], "object");
        assert_eq!(cache.get("77e0").unwrap()["type"], "object");
    }
}
