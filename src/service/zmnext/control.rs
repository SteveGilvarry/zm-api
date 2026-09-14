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
use serde_json::{json, Value};
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
        match p.schema_sha256.as_deref() {
            // No schema: the kind is valid, only its `$secret` references
            // can be checked.
            None => {
                out.insert(p.kind.clone(), Value::Null);
            }
            Some(sha) => match cache.get(sha) {
                Some(schema) => {
                    out.insert(p.kind.clone(), schema);
                }
                None => missing.push(p.kind.clone()),
            },
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
        if let Some(sha) = hello
            .plugins
            .iter()
            .find(|p| &p.kind == kind)
            .and_then(|p| p.schema_sha256.as_deref())
        {
            if !schema.is_null() {
                cache.insert(sha, schema.clone());
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
    // zm-core accepts only these node keys; `path` in particular would name a
    // library to load.
    for key in obj.keys() {
        if !matches!(
            key.as_str(),
            "id" | "kind" | "cfg" | "config" | "children" | "queue_depth"
        ) {
            errors.push(PathError::new(
                format!("{path}.{key}"),
                "not accepted in configure; name the plugin with kind",
            ));
        }
    }
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
        if let Some(min) = s.get("exclusiveMinimum").and_then(Value::as_f64) {
            if n <= min {
                errors.push(PathError::new(path, format!("must be > {min}")));
            }
        }
        if let Some(max) = s.get("exclusiveMaximum").and_then(Value::as_f64) {
            if n >= max {
                errors.push(PathError::new(path, format!("must be < {max}")));
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
pub fn configure_command(pipeline: &Value, secrets: &SecretMap, salt: &str, apply: &str) -> Value {
    json!({
        "cmd": "configure",
        "pipeline": pipeline,
        "secrets": secrets,
        "secrets_salt": salt,
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
    if !errors.is_empty() {
        return errors;
    }
    // apply_failed: the pipeline validated but a plugin wouldn't load.
    match data.get("error").and_then(Value::as_str) {
        Some(error) => vec![PathError::new("", format!("{message}: {error}"))],
        None => vec![PathError::new("", message)],
    }
}

/// Replace the capture streams' `username`/`password` literals with `$secret`
/// references (`camera.stream<N>.username` / `.password`) and return the
/// values. Everything else in the pipeline is left alone.
pub fn split_camera_credentials(pipeline: &mut Value) -> SecretMap {
    fn walk(nodes: &mut Value, out: &mut SecretMap) {
        let Some(arr) = nodes.as_array_mut() else {
            return;
        };
        for node in arr {
            let is_capture = node
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|k| k.starts_with("capture_"));
            if is_capture {
                if let Some(streams) = node
                    .pointer_mut("/cfg/streams")
                    .and_then(Value::as_array_mut)
                {
                    for (i, stream) in streams.iter_mut().enumerate() {
                        for key in ["username", "password"] {
                            if let Some(v) =
                                stream.get(key).and_then(Value::as_str).map(str::to_string)
                            {
                                let name = format!("camera.stream{i}.{key}");
                                stream[key] = json!({ "$secret": name });
                                out.insert(name, v);
                            }
                        }
                    }
                }
            }
            if let Some(children) = node.get_mut("children") {
                walk(children, out);
            }
        }
    }
    let mut out = SecretMap::new();
    if let Some(plugins) = pipeline.get_mut("plugins") {
        walk(plugins, &mut out);
    }
    out
}

/// Keys every plugin's secrets live under, whatever its schema says (zm-core
/// `default_secret_keys()`).
const DEFAULT_SECRET_KEYS: &[&str] = &[
    "password",
    "username",
    "auth_header",
    "api_key",
    "token",
    "secret",
];

/// Keys a plugin schema marks `x-secret`, at any depth (properties, items,
/// additionalProperties, oneOf/anyOf/allOf), as zm-core collects them.
pub fn schema_secret_keys(schema: &Value) -> std::collections::BTreeSet<String> {
    fn collect(node: &Value, out: &mut std::collections::BTreeSet<String>) {
        let Some(obj) = node.as_object() else {
            return;
        };
        if let Some(props) = obj.get("properties").and_then(Value::as_object) {
            for (name, sub) in props {
                if sub.get("x-secret").and_then(Value::as_bool) == Some(true) {
                    out.insert(name.clone());
                }
                collect(sub, out);
            }
        }
        for k in ["items", "additionalProperties"] {
            if let Some(sub) = obj.get(k) {
                collect(sub, out);
            }
        }
        for k in ["oneOf", "anyOf", "allOf"] {
            if let Some(arr) = obj.get(k).and_then(Value::as_array) {
                arr.iter().for_each(|sub| collect(sub, out));
            }
        }
    }
    let mut out = std::collections::BTreeSet::new();
    collect(schema, &mut out);
    out
}

/// The pipeline as zm-core hashes it, and its secret values with their JSON
/// pointers. Mirrors zm-core `redact_pipeline` (core/src/WorkerHello.cpp):
/// resolve `$secret` references, replace every string or number under a
/// secret key (defaults plus the plugin's `x-secret` keys) with `"<secret>"`,
/// then do the same at every position a reference occupied.
fn redact_like_zm_core(
    pipeline: &Value,
    secrets: &SecretMap,
    schemas: &BTreeMap<String, Value>,
) -> (Value, Vec<(String, String)>) {
    fn resolve(v: &mut Value, path: &str, secrets: &SecretMap, refs: &mut Vec<String>) {
        if let Some(name) = reference_name(v).map(str::to_string) {
            refs.push(path.to_string());
            *v = Value::String(secrets.get(&name).cloned().unwrap_or_default());
            return;
        }
        match v {
            Value::Object(m) => {
                for (k, child) in m.iter_mut() {
                    resolve(
                        child,
                        &format!("{path}/{}", pointer_escape(k)),
                        secrets,
                        refs,
                    );
                }
            }
            Value::Array(a) => {
                for (i, child) in a.iter_mut().enumerate() {
                    resolve(child, &format!("{path}/{i}"), secrets, refs);
                }
            }
            _ => {}
        }
    }
    fn redact_cfg(
        v: &mut Value,
        keys: &std::collections::BTreeSet<String>,
        path: &str,
        out: &mut Vec<(String, String)>,
    ) {
        match v {
            Value::Object(m) => {
                for (k, child) in m.iter_mut() {
                    let here = format!("{path}/{k}");
                    if keys.contains(k) && (child.is_string() || child.is_number()) {
                        let value = child
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| child.to_string());
                        out.push((here, value));
                        *child = Value::String("<secret>".into());
                    } else {
                        redact_cfg(child, keys, &here, out);
                    }
                }
            }
            Value::Array(a) => {
                for (i, child) in a.iter_mut().enumerate() {
                    redact_cfg(child, keys, &format!("{path}/{i}"), out);
                }
            }
            _ => {}
        }
    }
    fn redact_nodes(
        nodes: &mut Value,
        path: &str,
        schemas: &BTreeMap<String, Value>,
        out: &mut Vec<(String, String)>,
    ) {
        let Some(arr) = nodes.as_array_mut() else {
            return;
        };
        for (i, node) in arr.iter_mut().enumerate() {
            let Some(obj) = node.as_object_mut() else {
                continue;
            };
            let here = format!("{path}/{i}");
            let mut keys: std::collections::BTreeSet<String> =
                DEFAULT_SECRET_KEYS.iter().map(|k| k.to_string()).collect();
            if let Some(schema) = obj
                .get("kind")
                .and_then(Value::as_str)
                .and_then(|k| schemas.get(k))
            {
                keys.extend(schema_secret_keys(schema));
            }
            for cfg_key in ["cfg", "config"] {
                if let Some(cfg) = obj.get_mut(cfg_key) {
                    redact_cfg(cfg, &keys, &format!("{here}/{cfg_key}"), out);
                }
            }
            if let Some(children) = obj.get_mut("children") {
                redact_nodes(children, &format!("{here}/children"), schemas, out);
            }
        }
    }

    let mut copy = pipeline.clone();
    let mut refs = Vec::new();
    resolve(&mut copy, "", secrets, &mut refs);
    let mut found = Vec::new();
    if let Some(plugins) = copy.get_mut("plugins") {
        redact_nodes(plugins, "/plugins", schemas, &mut found);
    }
    for pointer in refs {
        if let Some(v) = copy.pointer_mut(&pointer) {
            if v.as_str() == Some("<secret>") {
                continue;
            }
            let value = v
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string());
            found.push((pointer, value));
            *v = Value::String("<secret>".into());
        }
    }
    (copy, found)
}

fn pointer_escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The `pipeline_hash` a worker reports once configured with `pipeline`
/// (references included) and `secrets`: SHA-256 of the pipeline with every
/// secret value replaced by `"<secret>"`, keys sorted, compact. zm-api's
/// serde_json has no `preserve_order`, so objects serialise sorted like
/// nlohmann::json.
pub fn pipeline_hash(
    pipeline: &Value,
    secrets: &SecretMap,
    schemas: &BTreeMap<String, Value>,
) -> String {
    let (redacted, _) = redact_like_zm_core(pipeline, secrets, schemas);
    let text = serde_json::to_string(&redacted).expect("Value serialises");
    format!("sha256:{}", sha256_hex(text.as_bytes()))
}

/// The `secrets_fingerprint` a worker reports for the same configure with
/// `salt`: SHA-256 of `salt` followed by the sorted `[pointer, value]` list.
/// `None` when the pipeline holds no secrets.
pub fn secrets_fingerprint(
    pipeline: &Value,
    secrets: &SecretMap,
    schemas: &BTreeMap<String, Value>,
    salt: &str,
) -> Option<String> {
    let (_, mut found) = redact_like_zm_core(pipeline, secrets, schemas);
    if found.is_empty() {
        return None;
    }
    found.sort();
    let list: Vec<[&str; 2]> = found
        .iter()
        .map(|(p, v)| [p.as_str(), v.as_str()])
        .collect();
    let text = serde_json::to_string(&list).expect("list serialises");
    Some(format!(
        "sha256:{}",
        sha256_hex(format!("{salt}{text}").as_bytes())
    ))
}

/// Whether a running worker already has the configuration zm-api would send:
/// same pipeline hash and, when the hello carries one, the same secrets
/// fingerprint. A worker without a hello can't be checked.
pub fn worker_has_configuration(
    hello: Option<&WorkerHello>,
    pipeline: &Value,
    secrets: &SecretMap,
    schemas: &BTreeMap<String, Value>,
    salt: &str,
) -> Option<bool> {
    let hello = hello?;
    if hello.state != "running" {
        return Some(false);
    }
    let same_pipeline =
        hello.pipeline_hash.as_deref() == Some(pipeline_hash(pipeline, secrets, schemas).as_str());
    let same_secrets = match &hello.secrets_fingerprint {
        Some(fp) => Some(fp.clone()) == secrets_fingerprint(pipeline, secrets, schemas, salt),
        None => true,
    };
    Some(same_pipeline && same_secrets)
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
    fn schemaless_kinds_node_keys_and_exclusive_bounds() {
        let mut s = schemas();
        s.insert("capture_file".into(), Value::Null);
        s.insert(
            "zones".into(),
            json!({"type": "object", "properties": {"ratio": {"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 1}}}),
        );
        let g = json!({"plugins": [
            {"kind": "capture_file", "cfg": {"anything": 1}},
            {"kind": "zones", "cfg": {"ratio": 1}, "path": "/tmp/x.so"}
        ]});
        let errors = validate_graph_with_schemas(&g, &s, SecretRule::ReferenceOnly);
        assert_eq!(
            errors,
            vec![
                PathError::new(
                    "plugins[1].path",
                    "not accepted in configure; name the plugin with kind"
                ),
                PathError::new("plugins[1].cfg.ratio", "must be < 1"),
            ]
        );
    }

    #[test]
    fn configure_message_keeps_secrets_apart() {
        let mut secrets = SecretMap::new();
        secrets.insert("cam.user".into(), "admin".into());
        secrets.insert("cam.pass".into(), "p@ss:w/d".into());
        let cmd = configure_command(&doc_pipeline(), &secrets, "c2FsdA", "restart");
        assert_eq!(cmd["secrets_salt"], "c2FsdA");
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
    fn camera_credentials_become_references() {
        let mut p = json!({"plugins": [{"id": "capture", "kind": "capture_rtsp_multi",
            "cfg": {"streams": [{"url": "rtsp://10.0.0.5/102", "username": "admin", "password": "p@ss:w/d"}]},
            "children": [{"kind": "output_webhook", "cfg": {"auth_header": {"$secret": "notify.auth_header"}}}]}]});
        let map = split_camera_credentials(&mut p);
        assert_eq!(map["camera.stream0.username"], "admin");
        assert_eq!(map["camera.stream0.password"], "p@ss:w/d");
        assert!(!p.to_string().contains("p@ss"));
        assert_eq!(
            p["plugins"][0]["cfg"]["streams"][0]["password"],
            json!({"$secret": "camera.stream0.password"})
        );
        assert_eq!(
            p["plugins"][0]["children"][0]["cfg"]["auth_header"],
            json!({"$secret": "notify.auth_header"})
        );
    }

    /// The configure sent to a real zm-next Phase 1 zm-core (586f282) and the
    /// hashes it reported: `pipeline_hash` in the Response and the next hello,
    /// `secrets_fingerprint` in the hello. Captured 2026-09-14; replace with
    /// zm-next's tests/contract/ transcript when published.
    fn zm_core_vector() -> (Value, SecretMap) {
        let pipeline = json!({"name": "vec", "root": true, "plugins": [
        {"id": "capture", "kind": "capture_rtsp_multi",
         "cfg": {"streams": [{"stream_id": 0, "url": "rtsp://127.0.0.1:9/none", "transport": "tcp",
                              "max_retry_attempts": -1,
                              "username": {"$secret": "camera.stream0.username"},
                              "password": {"$secret": "camera.stream0.password"}}]},
         "children": [
            {"id": "track", "kind": "tracker", "cfg": {"iou_threshold": 0.35, "max_age": 30}},
            {"id": "notify", "kind": "output_webhook",
             "cfg": {"url": "http://127.0.0.1:9/h", "auth_header": {"$secret": "notify.auth_header"}}}
         ]}]});
        let secrets: SecretMap = [
            ("camera.stream0.username", "admin"),
            ("camera.stream0.password", "p@ss:w/d"),
            ("notify.auth_header", "Bearer t0k3n"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        (pipeline, secrets)
    }

    #[test]
    fn pipeline_hash_and_fingerprint_match_zm_core() {
        let (pipeline, secrets) = zm_core_vector();
        let schemas = BTreeMap::new();
        assert_eq!(
            pipeline_hash(&pipeline, &secrets, &schemas),
            "sha256:39ea29e81ccd705da371f29368ff8a3aa2f36f9553acb01c8746535709677151"
        );
        assert_eq!(
            secrets_fingerprint(&pipeline, &secrets, &schemas, "c2FsdC1mcm9tLXptLWFwaQ").as_deref(),
            Some("sha256:b04af9f4110614d736421c4773cb3decac2f5fa9a3687578e2606bdf10f6c65f")
        );

        // A password change leaves the hash alone and moves the fingerprint.
        let mut rotated = secrets.clone();
        rotated.insert("camera.stream0.password".into(), "n3w".into());
        assert_eq!(
            pipeline_hash(&pipeline, &rotated, &schemas),
            pipeline_hash(&pipeline, &secrets, &schemas)
        );
        assert_ne!(
            secrets_fingerprint(&pipeline, &rotated, &schemas, "c2FsdC1mcm9tLXptLWFwaQ"),
            secrets_fingerprint(&pipeline, &secrets, &schemas, "c2FsdC1mcm9tLXptLWFwaQ")
        );
        // Literal secrets under default keys hash the same as references.
        let mut literal = pipeline.clone();
        literal["plugins"][0]["cfg"]["streams"][0]["password"] = json!("p@ss:w/d");
        assert_eq!(
            pipeline_hash(&literal, &secrets, &schemas),
            pipeline_hash(&pipeline, &secrets, &schemas)
        );
    }

    #[test]
    fn worker_has_configuration_checks_hash_and_fingerprint() {
        let (pipeline, secrets) = zm_core_vector();
        let schemas = BTreeMap::new();
        let salt = "c2FsdC1mcm9tLXptLWFwaQ";
        let hello = |state: &str, fp: &str| {
            parse_worker_hello(
                format!(
                    r#"{{"state":"{state}","pipeline_hash":"sha256:39ea29e81ccd705da371f29368ff8a3aa2f36f9553acb01c8746535709677151","secrets_fingerprint":"{fp}"}}"#
                )
                .as_bytes(),
            )
            .unwrap()
        };
        let good = "sha256:b04af9f4110614d736421c4773cb3decac2f5fa9a3687578e2606bdf10f6c65f";
        assert_eq!(
            worker_has_configuration(
                Some(&hello("running", good)),
                &pipeline,
                &secrets,
                &schemas,
                salt
            ),
            Some(true)
        );
        assert_eq!(
            worker_has_configuration(
                Some(&hello("running", "sha256:old")),
                &pipeline,
                &secrets,
                &schemas,
                salt
            ),
            Some(false),
            "same pipeline, different secrets"
        );
        assert_eq!(
            worker_has_configuration(
                Some(&hello("failed", good)),
                &pipeline,
                &secrets,
                &schemas,
                salt
            ),
            Some(false)
        );
        assert_eq!(
            worker_has_configuration(None, &pipeline, &secrets, &schemas, salt),
            None
        );
    }

    #[test]
    fn schema_x_secret_keys_are_collected_at_any_depth() {
        let schema = json!({"type": "object", "properties": {
            "streams": {"type": "array", "items": {"type": "object", "properties": {
                "passphrase": {"type": "string", "x-secret": true}}}},
            "auth": {"oneOf": [{"properties": {"bearer": {"type": "string", "x-secret": true}}}]}
        }});
        let keys: Vec<String> = schema_secret_keys(&schema).into_iter().collect();
        assert_eq!(keys, ["bearer", "passphrase"]);
    }

    /// zm-core's refusal of a configure with a bad value and a `path` node key
    /// (captured from 586f282).
    #[test]
    fn zm_core_refusals_become_path_errors() {
        let data: Value = serde_json::from_str(
            r#"{"errors":[{"message":"not accepted in configure; name the plugin with kind","path":"plugins[0].path"},{"message":"must be <= 1","path":"plugins[0].cfg.iou_threshold"}]}"#,
        )
        .unwrap();
        let errors = configure_errors("invalid_config", &data);
        assert_eq!(errors[0].path, "plugins[0].path");
        assert_eq!(errors[1].message, "must be <= 1");
        let failed = configure_errors(
            "apply_failed",
            &json!({"error": "decode_detect: no backend"}),
        );
        assert_eq!(
            failed,
            vec![PathError::new(
                "",
                "apply_failed: decode_detect: no backend"
            )]
        );
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
