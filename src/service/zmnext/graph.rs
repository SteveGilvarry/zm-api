//! Validation for the stored zm-next **processing plugin graph**.
//!
//! The graph stored in `monitor_pipeline.graph_json` is the part of the pipeline
//! ZoneMinder has no schema for — everything from `decode` downward
//! (detect/track/analytics/describe/audio/outputs). Capture, credentials, the
//! `store` (recording) node, and zones are NOT part of it; they are composed in
//! at spawn from `Monitors`/`Zones` (see [`super::pipeline::compose_pipeline`]).
//!
//! Document shape: `{ "plugins": [ <node>, ... ] }`, where each node is the
//! standard `{ id?, kind, cfg?|config?, queue_depth?, children? }`. These nodes
//! become the children of the composed `capture` node.
//!
//! Validation is intentionally shallow: well-formed tree, a known plugin `kind`,
//! no capture/recording nodes (those are composed), and no secret/capture keys
//! in any `cfg`. zm-next remains the deep per-plugin validator at spawn.

use serde_json::Value;

/// Plugin kinds zm-api refuses inside a stored graph: capture is built from the
/// monitor's `Path`/creds, and `store` (recording) is built from `Function`.
const FORBIDDEN_KINDS: &[&str] = &["capture_rtsp_multi", "capture_file", "store"];

/// Plugin kinds a stored graph may use: every plugin zm-next builds, minus
/// [`FORBIDDEN_KINDS`]. Unknown kinds are rejected here early, because zm-core
/// refuses the whole pipeline when one plugin fails to load. A test checks this
/// list against `zmnext_plugins.txt`, a copy of zm-next's plugin build list.
const KNOWN_KINDS: &[&str] = &[
    "decode_ffmpeg",
    "decode_detect",
    "detect_onnx",
    "detect_openvocab",
    "detect_pose",
    "detect_seg",
    "motion_gate",
    "motion_pixel_diff",
    "zones",
    "tracker",
    "analytics_rules",
    "alert_policy",
    "describe_vlm",
    "llm_event_review",
    "audio_detect",
    "recognize_face",
    "lpr",
    "output_mqtt",
    "output_webhook",
    "store_snapshot",
    "review_export",
    "overlay",
    "privacy_mask",
    "encode_ffmpeg",
    "hello",
];

/// cfg keys that must never be persisted in a stored graph: capture URLs and
/// credentials are injected at spawn and kept out of any persisted form.
const FORBIDDEN_CFG_KEYS: &[&str] = &[
    "url",
    "username",
    "password",
    "pass",
    "credentials",
    "streams",
    // Shared-inference routing is injected by zm-api at compose time (never from a
    // stored graph), so a user cannot point a monitor at an arbitrary daemon.
    "infer_endpoint",
    "gpu_id",
];

/// Validate a stored processing-graph document. Returns a human-readable reason
/// on the first problem found.
pub fn validate_graph(doc: &Value) -> Result<(), String> {
    let plugins = doc
        .get("plugins")
        .and_then(Value::as_array)
        .ok_or("graph must be an object with a `plugins` array")?;
    if plugins.is_empty() {
        return Err("graph `plugins` must not be empty".to_string());
    }
    for node in plugins {
        validate_node(node)?;
    }
    Ok(())
}

fn validate_node(node: &Value) -> Result<(), String> {
    let obj = node
        .as_object()
        .ok_or("each plugin node must be a JSON object")?;

    let kind = obj
        .get("kind")
        .and_then(Value::as_str)
        .ok_or("each plugin node needs a non-empty string `kind`")?;
    if kind.is_empty() {
        return Err("plugin `kind` must not be empty".to_string());
    }
    if FORBIDDEN_KINDS.contains(&kind) {
        return Err(format!(
            "`{kind}` is composed by zm-api and cannot appear in a stored graph"
        ));
    }
    if !KNOWN_KINDS.contains(&kind) {
        return Err(format!("unknown plugin kind `{kind}`"));
    }

    // Reject capture/secret keys in either `cfg` or `config`.
    if let Some(cfg) = obj.get("cfg").or_else(|| obj.get("config")) {
        if let Some(cfg_obj) = cfg.as_object() {
            for (key, value) in cfg_obj {
                if super::secrets::is_secret_key(kind, key) {
                    // A literal is moved into the secret store on save; a
                    // reference keeps the stored value.
                    if !value.is_string() && super::secrets::reference_name(value).is_none() {
                        return Err(format!(
                            "`{kind}` `{key}` must be a string or a {{\"$secret\": \"<name>\"}} reference"
                        ));
                    }
                    continue;
                }
                if kind == "output_mqtt" && key == "username" {
                    continue;
                }
                if FORBIDDEN_CFG_KEYS.contains(&key.as_str()) {
                    return Err(format!(
                        "`{kind}` cfg may not contain `{key}` — capture/credentials are injected at spawn, never stored"
                    ));
                }
            }
        }
    }

    if let Some(children) = obj.get("children") {
        let arr = children
            .as_array()
            .ok_or("`children` must be an array when present")?;
        for child in arr {
            validate_node(child)?;
        }
    }
    Ok(())
}

/// zm-next's plugin kinds, from the checked-in copy of its plugin build list.
#[cfg(test)]
pub(crate) fn zmnext_plugin_kinds() -> std::collections::BTreeSet<&'static str> {
    include_str!("zmnext_plugins.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeSet;

    /// Fails when `KNOWN_KINDS` drifts from zm-next: a plugin was added or
    /// removed there, or a kind here never existed.
    #[test]
    fn known_kinds_match_zm_next_plugins() {
        let zmnext = zmnext_plugin_kinds();
        let ours: BTreeSet<&str> = KNOWN_KINDS.iter().chain(FORBIDDEN_KINDS).copied().collect();
        let missing: Vec<_> = zmnext.difference(&ours).collect();
        let stale: Vec<_> = ours.difference(&zmnext).collect();
        assert!(
            missing.is_empty() && stale.is_empty(),
            "KNOWN_KINDS + FORBIDDEN_KINDS out of sync with zmnext_plugins.txt: \
             missing {missing:?}, not in zm-next {stale:?}"
        );
    }

    /// Fails when `zmnext_plugins.txt` itself is stale. Runs only with
    /// `ZMNEXT_SRC` pointing at a zm-next checkout.
    #[test]
    fn zmnext_plugins_copy_matches_zm_next() {
        let Ok(src) = std::env::var("ZMNEXT_SRC") else {
            return;
        };
        let cmake = std::fs::read_to_string(format!("{src}/plugins/CMakeLists.txt"))
            .expect("read zm-next plugins/CMakeLists.txt");
        let actual: BTreeSet<&str> = cmake
            .lines()
            .filter_map(|l| {
                l.trim()
                    .strip_prefix("add_subdirectory(")?
                    .strip_suffix(')')
            })
            .collect();
        assert_eq!(
            zmnext_plugin_kinds(),
            actual,
            "refresh src/service/zmnext/zmnext_plugins.txt from {src}/plugins/CMakeLists.txt"
        );
    }

    #[test]
    fn plugin_secrets_are_allowed_as_strings_or_references() {
        let doc = json!({ "plugins": [ { "kind": "decode_detect", "children": [
            { "kind": "output_mqtt", "cfg": { "username": "u", "password": "p" } },
            { "kind": "output_webhook", "cfg": { "auth_header": { "$secret": "notify.auth_header" } } },
            { "kind": "llm_event_review", "cfg": { "api_key": "sk" } }
        ] } ] });
        assert!(validate_graph(&doc).is_ok(), "{:?}", validate_graph(&doc));

        let bad =
            json!({ "plugins": [ { "kind": "output_webhook", "cfg": { "auth_header": 42 } } ] });
        assert!(validate_graph(&bad).unwrap_err().contains("reference"));
        // Camera-style credentials stay forbidden everywhere else.
        let cam = json!({ "plugins": [ { "kind": "decode_detect", "cfg": { "password": "p" } } ] });
        assert!(validate_graph(&cam).is_err());
    }

    #[test]
    fn rejects_kinds_zm_next_removed() {
        for kind in ["output_webrtc", "output_mse", "plate_export"] {
            let doc = json!({ "plugins": [ { "kind": kind } ] });
            assert!(
                validate_graph(&doc)
                    .unwrap_err()
                    .contains("unknown plugin kind"),
                "{kind} should be rejected"
            );
        }
    }

    #[test]
    fn accepts_a_valid_processing_graph() {
        let doc = json!({
            "plugins": [
                { "id": "detect", "kind": "decode_detect", "cfg": { "conf_threshold": 0.4 },
                  "children": [
                      { "id": "track", "kind": "tracker", "cfg": { "max_age": 30 },
                        "children": [ { "id": "rules", "kind": "analytics_rules", "cfg": {} } ] }
                  ] }
            ]
        });
        assert!(validate_graph(&doc).is_ok());
    }

    #[test]
    fn rejects_missing_or_empty_plugins() {
        assert!(validate_graph(&json!({})).is_err());
        assert!(validate_graph(&json!({ "plugins": [] })).is_err());
        assert!(validate_graph(&json!({ "plugins": "nope" })).is_err());
    }

    #[test]
    fn rejects_capture_and_store_kinds() {
        let cap = json!({ "plugins": [ { "kind": "capture_rtsp_multi" } ] });
        assert!(validate_graph(&cap)
            .unwrap_err()
            .contains("capture_rtsp_multi"));
        let store = json!({ "plugins": [ { "kind": "store", "cfg": {} } ] });
        assert!(validate_graph(&store).unwrap_err().contains("store"));
    }

    #[test]
    fn rejects_unknown_kind() {
        let doc = json!({ "plugins": [ { "kind": "detect_aliens" } ] });
        assert!(validate_graph(&doc)
            .unwrap_err()
            .contains("unknown plugin kind"));
    }

    #[test]
    fn rejects_secret_or_capture_keys_in_cfg() {
        for key in [
            "url",
            "username",
            "password",
            "pass",
            "credentials",
            "streams",
        ] {
            let doc = json!({ "plugins": [ { "kind": "detect_onnx", "cfg": { key: "x" } } ] });
            assert!(
                validate_graph(&doc).is_err(),
                "expected `{key}` in cfg to be rejected"
            );
        }
        // store_snapshot is allowed (distinct from the composed `store`); a
        // benign cfg key passes.
        let ok =
            json!({ "plugins": [ { "kind": "store_snapshot", "cfg": { "interval_sec": 5 } } ] });
        assert!(validate_graph(&ok).is_ok());
    }

    #[test]
    fn rejects_secret_keys_in_nested_children() {
        let doc = json!({
            "plugins": [ { "kind": "decode_detect", "children": [
                { "kind": "output_webhook", "cfg": { "password": "leak" } }
            ] } ]
        });
        assert!(validate_graph(&doc).is_err());
    }
}
