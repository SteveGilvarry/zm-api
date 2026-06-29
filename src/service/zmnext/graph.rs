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

/// Known zm-next plugin kinds — mirrors the zm-next plugin catalog
/// (`zm-next/plugins/*`). Unknown kinds are rejected here early; zm-next also
/// fails to `dlopen` an unknown kind at spawn. Keep in sync as plugins are added.
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
    "output_webrtc",
    "output_mse",
    "output_mqtt",
    "output_webhook",
    "store_snapshot",
    "review_export",
    "plate_export",
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
            for key in cfg_obj.keys() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
