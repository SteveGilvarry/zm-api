//! Generates a zm-next worker pipeline JSON from a Monitor row and its Zones.
//!
//! zm-next consumes a recursive plugin tree — `{id, kind, cfg, children}` under
//! a top-level `{name, root, plugins}` — and never reads the database itself;
//! zm-api is the sole DB reader and hands the worker a fully-resolved pipeline
//! in-memory over the worker's stdin (`--pipeline -`), so the config — camera
//! credentials included — never lands on disk. The shape is a
//! `capture_rtsp_multi` root with
//! `decode_detect`, `store`, and (optionally) `output_mqtt` as **siblings**
//! under it. `store` is zm-next's merged recorder (it folded the old
//! `store_filesystem` + `store_event` into one mode-switched plugin); it hangs
//! off `capture` — not under `decode_detect` — so it records the captured main
//! stream rather than the detector's substream (triggers reach it over
//! zm-next's process-global event bus regardless of tree position).
//!
//! The Monitor's `Path` column may hold a full RTSP URL with credentials
//! embedded. [`split_url_credentials`] strips any userinfo so the worker
//! receives a credential-free `url` plus separate `username`/`password` fields,
//! keeping secrets out of logs and any persisted form. ZoneMinder Zones are
//! translated into a best-effort `zones` array on the detect stage (polygon
//! points + pixel thresholds); evolution is additive, so unknown keys are
//! harmless.
//!
//! The generator works on small plain inputs ([`ZoneSpec`]) rather than the
//! `monitors`/`zones` entities directly, so it is fully unit-testable; the thin
//! [`zone_specs_from_models`] mapper bridges the SeaORM models at the call site.

use std::path::Path;

use serde_json::{json, Value};

use crate::configure::zmnext::PipelineConfig;
use crate::entity::sea_orm_active_enums::{Function, ZoneType};
use crate::entity::zones;

/// Split any RTSP userinfo out of `url`, returning `(clean_url, credentials)`.
/// The returned credentials are percent-decoded (raw); the clean URL preserves
/// everything else verbatim (scheme, host, port, path, query) — deliberately
/// done with plain string slicing rather than a URL parser so RTSP query params
/// (e.g. `?transportmode=mcast`) are not normalized or re-encoded. Returns
/// `(url, None)` when there is no `scheme://` or no userinfo.
pub fn split_url_credentials(url: &str) -> (String, Option<(String, String)>) {
    let Some(scheme_end) = url.find("://") else {
        return (url.to_string(), None);
    };
    let host_start = scheme_end + 3;
    let rest = &url[host_start..];
    // The authority ends at the first '/', '?' or '#'; userinfo is everything
    // before the last '@' within it.
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let Some(at) = authority.rfind('@') else {
        return (url.to_string(), None);
    };
    let userinfo = &authority[..at];
    let (user_enc, pass_enc) = match userinfo.split_once(':') {
        Some((u, p)) => (u, p),
        None => (userinfo, ""),
    };
    let decode = |s: &str| {
        urlencoding::decode(s)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| s.to_string())
    };
    let clean = format!("{}{}", &url[..host_start], &rest[at + 1..]);
    (clean, Some((decode(user_enc), decode(pass_enc))))
}

/// Recording mode for the merged `store` plugin, derived from the monitor's
/// ZoneMinder function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreMode {
    /// No recording at all — live view / detection only (ZM `Monitor`/`None`).
    /// The generated pipeline omits the `store` plugin entirely.
    None,
    /// Gapless segment recording (ZM `Record`).
    Continuous,
    /// Triggered pre/post-roll clips only (ZM `Modect`/`Nodect`).
    Event,
    /// Both continuous segments and triggered clips (ZM `Mocord`).
    Both,
}

impl StoreMode {
    /// Map a monitor's `Function` to a store mode. `Monitor`/`None` are
    /// view-only in ZoneMinder and so record nothing here ([`StoreMode::None`]);
    /// only the explicit recording functions produce a `store` node.
    pub fn from_function(function: &Function) -> Self {
        match function {
            Function::Modect | Function::Nodect => StoreMode::Event,
            Function::Mocord => StoreMode::Both,
            Function::Record => StoreMode::Continuous,
            // Monitor, None: live view / detection only, no recording.
            _ => StoreMode::None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            StoreMode::None => "none",
            StoreMode::Continuous => "continuous",
            StoreMode::Event => "event",
            StoreMode::Both => "both",
        }
    }

    /// Whether this mode emits a `store` node at all.
    fn records(self) -> bool {
        !matches!(self, StoreMode::None)
    }

    fn records_continuous(self) -> bool {
        matches!(self, StoreMode::Continuous | StoreMode::Both)
    }

    fn records_events(self) -> bool {
        matches!(self, StoreMode::Event | StoreMode::Both)
    }
}

/// A ZoneMinder zone reduced to what a zm-next zone-consumer needs.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSpec {
    pub name: String,
    /// Raw `Coords` string ("x1,y1 x2,y2 ...").
    pub coords: String,
    /// ZoneMinder zone type carried through verbatim ("Active"/"Inclusive"/
    /// "Exclusive"/"Preclusive") — zm-next's `zones` plugin understands these. See
    /// the type→ROI reinterpretation note in `docs/ZMNEXT_PROVISIONING_PLAN.md`.
    pub zm_type: String,
    pub min_pixel_threshold: Option<u16>,
    pub max_pixel_threshold: Option<u16>,
}

/// Map a ZoneMinder `ZoneType` to the string the zm-next `zones` plugin expects.
fn zone_type_str(t: &ZoneType) -> &'static str {
    match t {
        ZoneType::Active => "Active",
        ZoneType::Inclusive => "Inclusive",
        ZoneType::Exclusive => "Exclusive",
        ZoneType::Preclusive => "Preclusive",
        ZoneType::Inactive => "Inactive",
        ZoneType::Privacy => "Privacy",
    }
}

/// Reduce SeaORM zone models to [`ZoneSpec`]s, dropping only Inactive zones
/// (disabled). Detection zones and `Privacy` zones are both kept: the generator
/// routes detection zones to the `zones`/detect stage and `Privacy` zones to a
/// `privacy_mask` stage ([`privacy_regions`]).
pub fn zone_specs_from_models(zones: &[zones::Model]) -> Vec<ZoneSpec> {
    zones
        .iter()
        .filter(|z| !matches!(z.r#type, ZoneType::Inactive))
        .map(|z| ZoneSpec {
            name: z.name.clone(),
            coords: z.coords.clone(),
            zm_type: zone_type_str(&z.r#type).to_string(),
            min_pixel_threshold: z.min_pixel_threshold,
            max_pixel_threshold: z.max_pixel_threshold,
        })
        .collect()
}

/// Build the pipeline JSON document for one monitor.
///
/// * `source_url` — the **credential-free** capture RTSP URL (the monitor's
///   `Path` with any userinfo stripped, via [`split_url_credentials`]).
/// * `username` / `password` — RTSP credentials delivered as separate fields so
///   they never appear in the (logged/persisted) URL. Empty ⇒ omitted; the
///   capture plugin composes a transient credentialed URL only at open time.
/// * `mode` — recording mode for the merged `store` plugin (from the monitor's
///   function).
/// * `events_root` — media root the `store` plugin writes under. **Must be on
///   the same filesystem as the directory zm-api hands back in
///   `assign_recording`**, because the worker renames the in-progress file into
///   that directory with an open fd (a cross-fs target fails and the clip keeps
///   the worker's own name).
// Builder-style generator: each parameter is an independent, named pipeline
// input. Bundling them into a struct would not improve clarity here.
#[allow(clippy::too_many_arguments)]
pub fn generate_pipeline(
    monitor_id: u32,
    source_url: &str,
    username: &str,
    password: &str,
    zones: &[ZoneSpec],
    cfg: &PipelineConfig,
    mode: StoreMode,
    events_root: &Path,
    synopsis: bool,
) -> Value {
    // Privacy zones can't be masked on a passthrough recording (store records the
    // compressed capture). When any are present, switch to a transcode topology so
    // the mask reaches detection, live, AND the stored clip.
    let privacy = privacy_regions(zones);
    if !privacy.is_empty() {
        return build_privacy_transcode_pipeline(
            monitor_id,
            source_url,
            username,
            password,
            cfg,
            mode,
            events_root,
            privacy,
        );
    }

    let mut detect_cfg = json!({
        "model_path": cfg.model_path.to_string_lossy(),
        "hw": cfg.detect_hw,
        "input_size": cfg.detect_input_size,
        "conf_threshold": cfg.detect_conf_threshold,
        "roi_motion": true,
    });
    let zone_specs = translate_zones(zones);
    if !zone_specs.is_empty() {
        detect_cfg["zones"] = Value::Array(zone_specs);
    }
    if synopsis {
        // Motion synopsis needs per-object **polygon** masks (not bbox-only) and
        // stable **track ids** so the worker can build object "tubes" and emit
        // them as `review_assets` (0x0306). `mask_format:"none"` would degrade
        // tubes to bbox-only — see the hand-off spec §6.
        detect_cfg["mask_format"] = json!("polygon");
        detect_cfg["tracker"] = json!(true);
    }

    // The merged `store` plugin (zm-next folded store_filesystem + store_event
    // into one). Common keys always; segment / event keys only for the modes
    // that use them. Every segment and every clip is one ZM event and runs the
    // id-assignment handshake.
    // `store` and `output_mqtt` are siblings of `decode_detect` under `capture`,
    // not children of it: in zm-next, EVENTs (triggers, assign_recording) flow
    // on a process-global bus regardless of tree position, while tree position
    // only decides which FRAMES a plugin sees. `store` must record the captured
    // (main) stream, so it hangs off `capture`; were it under `decode_detect` it
    // would record whatever that stage is fed (the low-res substream).
    let mut capture_children = vec![json!({
        "id": "detect",
        "kind": "decode_detect",
        "cfg": detect_cfg,
        "queue_depth": 2,
    })];
    // `StoreMode::None` (ZM Monitor/None) records nothing — omit the store node.
    if mode.records() {
        capture_children.push(build_store_node(monitor_id, cfg, mode, events_root));
    }
    if synopsis {
        // Export the synopsis ingredients: object cutouts/tubes (review_export →
        // 0x0306 review_assets) and clean background plates (plate_export). Only
        // synopsis-enabled cameras pay this; the rest never emit it.
        capture_children.push(json!({
            "id": "review",
            "kind": "review_export",
            "cfg": {
                "monitor_id": monitor_id,
                "root": events_root.to_string_lossy(),
                "mask_format": "polygon",
            },
            "queue_depth": 8,
        }));
        capture_children.push(json!({
            "id": "plate",
            "kind": "plate_export",
            "cfg": {
                "monitor_id": monitor_id,
                "root": events_root.to_string_lossy(),
            },
            "queue_depth": 4,
        }));
    }
    if let Some((host, port)) = mqtt_host_port(cfg.mqtt_url.as_deref()) {
        capture_children.push(json!({
            "id": "notify",
            "kind": "output_mqtt",
            "cfg": { "host": host, "port": port, "base_topic": "zm-next" },
            "queue_depth": 8,
        }));
    }

    let capture = build_capture_node(
        source_url,
        username,
        password,
        &cfg.rtsp_transport,
        capture_children,
    );
    pipeline_doc(monitor_id, capture)
}

/// Build the **default processing graph** to seed a monitor that's being switched
/// to zm-next ("make it zm-next"). This is exactly the default generated pipeline
/// minus the composed `capture`/`store` nodes (compose re-adds those), so a
/// freshly-migrated monitor behaves identically to the legacy default until the
/// operator edits its graph. Returns `{ "plugins": [ <processing node>, ... ] }`.
pub fn default_processing_graph(cfg: &PipelineConfig, synopsis: bool) -> Value {
    // Reuse the default generator (single source of truth) with throwaway
    // capture/store inputs, then keep only the processing children.
    let full = generate_pipeline(
        0,
        "",
        "",
        "",
        &[],
        cfg,
        StoreMode::Continuous,
        Path::new(""),
        synopsis,
    );
    let processing: Vec<Value> = full
        .get("plugins")
        .and_then(|p| p.get(0))
        .and_then(|c| c.get("children"))
        .and_then(Value::as_array)
        .map(|children| {
            children
                .iter()
                .filter(|n| n.get("kind").and_then(Value::as_str) != Some("store"))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    json!({ "plugins": processing })
}

/// Compose a worker pipeline from a **stored processing graph** (the free graph)
/// plus the monitor-derived capture + store nodes. Capture/credentials and the
/// `store` recorder are always re-derived here (never persisted); the stored
/// graph supplies the processing plugins, which become `capture`'s children. ZM
/// `zones` are injected into any `zones`-kind node in the graph (the motion-path
/// consumer); object-path rules (`analytics_rules`) are configured in the graph
/// directly and left untouched.
///
/// The graph should already be validated ([`super::graph::validate_graph`]);
/// returns `None` if it has no `plugins` array so the caller can fall back to
/// [`generate_pipeline`].
#[allow(clippy::too_many_arguments)]
pub fn compose_pipeline(
    monitor_id: u32,
    source_url: &str,
    username: &str,
    password: &str,
    graph_doc: &Value,
    zones: &[ZoneSpec],
    cfg: &PipelineConfig,
    mode: StoreMode,
    events_root: &Path,
) -> Option<Value> {
    let mut nodes: Vec<Value> = graph_doc.get("plugins")?.as_array()?.clone();

    let zone_json = translate_zones(zones);
    let privacy = privacy_regions(zones);
    if !zone_json.is_empty() || !privacy.is_empty() {
        for node in &mut nodes {
            inject_zone_config(node, &zone_json, &privacy);
        }
    }

    // The recorder is composed from the monitor's Function, never stored.
    // `StoreMode::None` (ZM Monitor/None) records nothing — omit the store node.
    if mode.records() {
        nodes.push(build_store_node(monitor_id, cfg, mode, events_root));
    }

    let capture = build_capture_node(source_url, username, password, &cfg.rtsp_transport, nodes);
    Some(pipeline_doc(monitor_id, capture))
}

/// Build the merged `store` recorder node for the monitor's recording mode.
fn build_store_node(
    monitor_id: u32,
    cfg: &PipelineConfig,
    mode: StoreMode,
    events_root: &Path,
) -> Value {
    let mut store_cfg = json!({
        "mode": mode.as_str(),
        "root": events_root.to_string_lossy(),
        "monitor_id": monitor_id,
        "stream_filter": [0],
    });
    if mode.records_continuous() {
        store_cfg["max_secs"] = json!(cfg.segment_max_secs);
    }
    if mode.records_events() {
        store_cfg["pre_roll_sec"] = json!(cfg.pre_roll_sec);
        store_cfg["post_roll_sec"] = json!(cfg.post_roll_sec);
        store_cfg["max_buffer_sec"] = json!(cfg.max_buffer_sec);
        store_cfg["trigger_types"] = json!(cfg.trigger_types);
    }
    json!({
        "id": "record",
        "kind": "store",
        "cfg": store_cfg,
        "queue_depth": 120,
    })
}

/// Build the `capture_rtsp_multi` node. Credentials ride as separate fields
/// (never baked into `url`); the capture plugin applies them as transient
/// userinfo only at `avformat_open_input` time.
fn build_capture_node(
    source_url: &str,
    username: &str,
    password: &str,
    rtsp_transport: &str,
    children: Vec<Value>,
) -> Value {
    let mut stream = json!({
        "stream_id": 0,
        "url": source_url,
        "transport": rtsp_transport,
        "max_retry_attempts": -1,
    });
    if !username.is_empty() {
        stream["username"] = json!(username);
        stream["password"] = json!(password);
    }
    json!({
        "id": "capture",
        "kind": "capture_rtsp_multi",
        "cfg": { "streams": [stream] },
        "children": children,
    })
}

/// Wrap a `capture` node in the top-level pipeline document.
fn pipeline_doc(monitor_id: u32, capture: Value) -> Value {
    json!({
        "name": format!("monitor_{monitor_id}"),
        "root": true,
        "plugins": [capture],
    })
}

/// Recursively inject ZM zone config into the graph: detection zones into any
/// `zones`-kind node's `cfg.zones`, and `Privacy` regions into any
/// `privacy_mask`-kind node's `cfg.regions`. Both stay ZM-authoritative.
fn inject_zone_config(node: &mut Value, zone_json: &[Value], privacy: &[Value]) {
    let kind = node.get("kind").and_then(Value::as_str);
    if kind == Some("zones") && !zone_json.is_empty() {
        set_cfg_array(node, "zones", zone_json);
    } else if kind == Some("privacy_mask") && !privacy.is_empty() {
        set_cfg_array(node, "regions", privacy);
    }
    if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
        for child in children.iter_mut() {
            inject_zone_config(child, zone_json, privacy);
        }
    }
}

/// Set `node.cfg[key] = values`, creating `cfg` if absent.
fn set_cfg_array(node: &mut Value, key: &str, values: &[Value]) {
    if let Some(obj) = node.as_object_mut() {
        let cfg = obj.entry("cfg").or_insert_with(|| json!({}));
        if let Some(cfg_obj) = cfg.as_object_mut() {
            cfg_obj.insert(key.to_string(), Value::Array(values.to_vec()));
        }
    }
}

/// Inject shared-inference routing into a generated/composed pipeline document:
/// set `infer_endpoint` + `gpu_id` on every detect node that supports the remote
/// daemon (`decode_detect`, `detect_onnx`). Applied AFTER graph validation, so
/// these keys never live in a stored graph. No-op when there are no such detect
/// nodes. `roi_motion` is deliberately left as configured: the motion gate runs
/// locally on the worker and only motion-region crops are sent to the daemon, so
/// detection stays motion-gated (it must NOT be forced to whole-frame here).
pub fn inject_shared_inference(doc: &mut Value, endpoint: &str, gpu_id: u32) {
    fn walk(node: &mut Value, endpoint: &str, gpu_id: u32) {
        let kind = node.get("kind").and_then(Value::as_str).map(str::to_string);
        if matches!(kind.as_deref(), Some("decode_detect") | Some("detect_onnx")) {
            if let Some(obj) = node.as_object_mut() {
                let cfg = obj.entry("cfg").or_insert_with(|| json!({}));
                if let Some(c) = cfg.as_object_mut() {
                    c.insert(
                        "infer_endpoint".to_string(),
                        Value::String(endpoint.to_string()),
                    );
                    c.insert("gpu_id".to_string(), json!(gpu_id));
                }
            }
        }
        if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
            for child in children.iter_mut() {
                walk(child, endpoint, gpu_id);
            }
        }
    }
    if let Some(plugins) = doc.get_mut("plugins").and_then(Value::as_array_mut) {
        for p in plugins.iter_mut() {
            walk(p, endpoint, gpu_id);
        }
    }
}

/// Turn detection [`ZoneSpec`]s into `zones`-plugin JSON, excluding `Privacy`
/// zones (those drive `privacy_mask`, see [`privacy_regions`]) and dropping zones
/// whose polygon fails to parse.
fn translate_zones(zones: &[ZoneSpec]) -> Vec<Value> {
    zones
        .iter()
        .filter(|z| z.zm_type != "Privacy")
        .filter_map(|z| {
            let points = parse_coords(&z.coords);
            if points.is_empty() {
                return None;
            }
            Some(json!({
                "name": z.name,
                "type": z.zm_type,
                "points": points,
                "min_pixel_threshold": z.min_pixel_threshold,
                "max_pixel_threshold": z.max_pixel_threshold,
            }))
        })
        .collect()
}

/// Turn `Privacy` [`ZoneSpec`]s into `privacy_mask` `regions` (each a polygon of
/// `[x,y]` points), dropping any whose polygon fails to parse.
fn privacy_regions(zones: &[ZoneSpec]) -> Vec<Value> {
    zones
        .iter()
        .filter(|z| z.zm_type == "Privacy")
        .filter_map(|z| {
            let points = parse_coords(&z.coords);
            if points.is_empty() {
                return None;
            }
            Some(Value::Array(
                points.into_iter().map(|[x, y]| json!([x, y])).collect(),
            ))
        })
        .collect()
}

/// Build a `privacy_mask` node (PROCESS plugin on decoded frames) that blacks out
/// the given regions, with `children` hung beneath it.
fn build_privacy_mask_node(regions: Vec<Value>, children: Vec<Value>) -> Value {
    json!({
        "id": "privacy",
        "kind": "privacy_mask",
        "cfg": { "mode": "black", "regions": regions },
        "queue_depth": 4,
        "children": children,
    })
}

/// Build the **transcode** pipeline used when a monitor has `Privacy` zones:
/// `capture → decode_ffmpeg → privacy_mask → { detect_onnx, encode_ffmpeg →
/// store }`. The mask is applied to decoded frames, so it reaches detection,
/// live, and (via re-encode) the stored clip — unlike the passthrough default,
/// which records the unmasked compressed capture. This costs a per-camera decode
/// + re-encode.
///
/// NOTE: this is the intended topology; end-to-end verification against a camera
/// with privacy zones (and the zm-next `encode_ffmpeg → store` path) is still
/// required — see `docs/ZMNEXT_PROVISIONING_PLAN.md`, Phase 4.
#[allow(clippy::too_many_arguments)]
fn build_privacy_transcode_pipeline(
    monitor_id: u32,
    source_url: &str,
    username: &str,
    password: &str,
    cfg: &PipelineConfig,
    mode: StoreMode,
    events_root: &Path,
    privacy: Vec<Value>,
) -> Value {
    // Detector on the masked decoded frames (separate detect_onnx, since decode is
    // already done by decode_ffmpeg rather than the fused decode_detect).
    let detect = json!({
        "id": "detect",
        "kind": "detect_onnx",
        "cfg": {
            "model_path": cfg.model_path.to_string_lossy(),
            "ep": cfg.detect_hw,
            "conf_threshold": cfg.detect_conf_threshold,
            "input_size": cfg.detect_input_size,
        },
        "queue_depth": 2,
    });
    // Re-encode the masked frames, then record the encoded stream. The encode
    // exists only to feed the recorder, so when `StoreMode::None` (ZM
    // Monitor/None) records nothing we drop the encode→store branch and keep
    // just the masked detector.
    let mut masked_children = vec![detect];
    if mode.records() {
        masked_children.push(json!({
            "id": "encode",
            "kind": "encode_ffmpeg",
            "cfg": { "codec": "h264", "hwaccel": "none" },
            "queue_depth": 8,
            "children": [ build_store_node(monitor_id, cfg, mode, events_root) ],
        }));
    }
    let privacy_node = build_privacy_mask_node(privacy, masked_children);
    let decode = json!({
        "id": "decode",
        "kind": "decode_ffmpeg",
        "cfg": { "output_format": "yuv420p" },
        "queue_depth": 8,
        "children": [ privacy_node ],
    });
    let capture = build_capture_node(
        source_url,
        username,
        password,
        &cfg.rtsp_transport,
        vec![decode],
    );
    pipeline_doc(monitor_id, capture)
}

/// Parse ZoneMinder's `Coords` string ("x1,y1 x2,y2 ...") into `[[x,y], ...]`.
fn parse_coords(coords: &str) -> Vec<[i64; 2]> {
    coords
        .split_whitespace()
        .filter_map(|pair| {
            let (x, y) = pair.split_once(',')?;
            Some([x.trim().parse().ok()?, y.trim().parse().ok()?])
        })
        .collect()
}

/// Split an `mqtt://host:port` (or `host:port`) URL into host + port.
fn mqtt_host_port(url: Option<&str>) -> Option<(String, u16)> {
    let url = url?;
    let authority = url.strip_prefix("mqtt://").unwrap_or(url);
    let authority = authority.split('/').next().unwrap_or(authority);
    match authority.rsplit_once(':') {
        Some((host, port)) => Some((host.to_string(), port.parse().ok()?)),
        None => Some((authority.to_string(), 1883)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cfg() -> PipelineConfig {
        PipelineConfig {
            mqtt_url: None,
            ..PipelineConfig::default()
        }
    }

    #[test]
    fn inject_shared_inference_targets_detect_nodes_only() {
        let mut doc = json!({
            "plugins": [{
                "kind": "capture_rtsp_multi",
                "children": [
                    { "kind": "decode_detect", "cfg": { "roi_motion": true }, "children": [
                        { "kind": "tracker", "cfg": {} }
                    ]},
                    { "kind": "store", "cfg": {} }
                ]
            }]
        });
        inject_shared_inference(&mut doc, "/run/zm/zm_infer_gpu0.sock", 0);

        let det = &doc["plugins"][0]["children"][0];
        assert_eq!(det["cfg"]["infer_endpoint"], "/run/zm/zm_infer_gpu0.sock");
        assert_eq!(det["cfg"]["gpu_id"], 0);
        // roi_motion is left as configured (motion gating stays local on the worker).
        assert_eq!(det["cfg"]["roi_motion"], true);
                                                     // Non-detect siblings/children are untouched.
        assert!(doc["plugins"][0]["children"][1]["cfg"]
            .get("infer_endpoint")
            .is_none());
        assert!(det["children"][0]["cfg"].get("infer_endpoint").is_none());
    }

    #[test]
    fn inject_shared_inference_generated_pipeline() {
        // The default generated pipeline (decode_detect under capture) gets routed.
        let doc_owned = generate_pipeline(
            7,
            "rtsp://cam/stream",
            "",
            "",
            &[],
            &test_cfg(),
            StoreMode::Continuous,
            Path::new("/tmp/events"),
            false,
        );
        let mut doc = doc_owned;
        inject_shared_inference(&mut doc, "/run/zm/zm_infer_gpu0.sock", 0);
        // decode_detect hangs off capture as the first child (see generate_pipeline).
        let detect = &doc["plugins"][0]["children"][0];
        assert_eq!(detect["kind"], "decode_detect");
        assert_eq!(
            detect["cfg"]["infer_endpoint"],
            "/run/zm/zm_infer_gpu0.sock"
        );
    }

    #[test]
    fn parse_coords_handles_polygon_and_garbage() {
        assert_eq!(
            parse_coords("0,0 640,0 640,480 0,480"),
            vec![[0, 0], [640, 0], [640, 480], [0, 480]]
        );
        // Skips malformed pairs, keeps valid ones.
        assert_eq!(parse_coords("10,20 bad 30,40"), vec![[10, 20], [30, 40]]);
        assert!(parse_coords("").is_empty());
    }

    #[test]
    fn mqtt_host_port_parses_forms() {
        assert_eq!(
            mqtt_host_port(Some("mqtt://broker:1884")),
            Some(("broker".to_string(), 1884))
        );
        assert_eq!(
            mqtt_host_port(Some("localhost")),
            Some(("localhost".to_string(), 1883))
        );
        assert_eq!(mqtt_host_port(None), None);
    }

    #[test]
    fn pipeline_has_capture_detect_store_chain() {
        let cfg = test_cfg();
        let zones = vec![ZoneSpec {
            name: "Front".to_string(),
            coords: "0,0 640,0 640,480 0,480".to_string(),
            zm_type: "Active".to_string(),
            min_pixel_threshold: Some(25),
            max_pixel_threshold: None,
        }];
        let p = generate_pipeline(
            7,
            "rtsp://cam:554/Streaming/Channels/101",
            "admin",
            "p@ss:w/d",
            &zones,
            &cfg,
            StoreMode::Event,
            Path::new("/var/lib/zm/events"),
            false,
        );

        assert_eq!(p["name"], "monitor_7");
        assert_eq!(p["root"], true);
        let capture = &p["plugins"][0];
        assert_eq!(capture["kind"], "capture_rtsp_multi");
        // URL is credential-free; creds ride as separate fields.
        assert_eq!(
            capture["cfg"]["streams"][0]["url"],
            "rtsp://cam:554/Streaming/Channels/101"
        );
        assert_eq!(capture["cfg"]["streams"][0]["username"], "admin");
        assert_eq!(capture["cfg"]["streams"][0]["password"], "p@ss:w/d");

        // detect, store, (mqtt) are siblings under capture. detect is first and
        // has no children of its own.
        let detect = &capture["children"][0];
        assert_eq!(detect["kind"], "decode_detect");
        assert!(detect.get("children").is_none());
        // The active zone is translated onto the detect stage.
        assert_eq!(detect["cfg"]["zones"][0]["name"], "Front");
        assert_eq!(detect["cfg"]["zones"][0]["points"][1], json!([640, 0]));
        assert_eq!(detect["cfg"]["zones"][0]["min_pixel_threshold"], 25);

        // One merged `store` plugin in event mode: roll/trigger keys, no max_secs.
        let record = &capture["children"][1];
        assert_eq!(record["kind"], "store");
        assert_eq!(record["cfg"]["mode"], "event");
        assert_eq!(record["cfg"]["root"], "/var/lib/zm/events");
        assert_eq!(record["cfg"]["monitor_id"], 7);
        assert_eq!(record["cfg"]["stream_filter"], json!([0]));
        assert_eq!(record["cfg"]["pre_roll_sec"], 5);
        assert_eq!(record["cfg"]["post_roll_sec"], 10);
        assert_eq!(record["cfg"]["max_buffer_sec"], 15);
        assert_eq!(record["cfg"]["trigger_types"][0], "detection");
        assert!(record["cfg"].get("max_secs").is_none());
        // No MQTT broker configured → just detect + store under capture.
        assert_eq!(capture["children"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn store_mode_maps_from_function_and_emits_right_keys() {
        assert_eq!(
            StoreMode::from_function(&Function::Record),
            StoreMode::Continuous
        );
        assert_eq!(StoreMode::from_function(&Function::Mocord), StoreMode::Both);
        assert_eq!(
            StoreMode::from_function(&Function::Modect),
            StoreMode::Event
        );
        assert_eq!(
            StoreMode::from_function(&Function::Nodect),
            StoreMode::Event
        );
        // View-only functions record nothing.
        assert_eq!(
            StoreMode::from_function(&Function::Monitor),
            StoreMode::None
        );
        assert_eq!(StoreMode::from_function(&Function::None), StoreMode::None);

        let cfg = test_cfg();
        // store is a sibling of detect under capture: plugins[0].children[1].
        // Continuous: segment key present, event keys absent.
        let cont = generate_pipeline(
            1,
            "rtsp://c",
            "",
            "",
            &[],
            &cfg,
            StoreMode::Continuous,
            Path::new("/e"),
            false,
        );
        let store = &cont["plugins"][0]["children"][1]["cfg"];
        assert_eq!(cont["plugins"][0]["children"][1]["kind"], "store");
        assert_eq!(store["mode"], "continuous");
        assert_eq!(store["max_secs"], 300);
        assert!(store.get("pre_roll_sec").is_none());
        assert!(store.get("trigger_types").is_none());

        // Both: segment AND event keys present.
        let both = generate_pipeline(
            1,
            "rtsp://c",
            "",
            "",
            &[],
            &cfg,
            StoreMode::Both,
            Path::new("/e"),
            false,
        );
        let store = &both["plugins"][0]["children"][1]["cfg"];
        assert_eq!(store["mode"], "both");
        assert_eq!(store["max_secs"], 300);
        assert_eq!(store["post_roll_sec"], 10);
    }

    #[test]
    fn store_mode_none_omits_recorder() {
        let cfg = test_cfg();
        let p = generate_pipeline(
            1,
            "rtsp://c",
            "",
            "",
            &[],
            &cfg,
            StoreMode::None,
            Path::new("/e"),
            false,
        );
        let children = p["plugins"][0]["children"].as_array().unwrap();
        let kinds: Vec<&str> = children
            .iter()
            .map(|n| n["kind"].as_str().unwrap())
            .collect();
        // Detector still runs (live view / detection), but nothing records.
        assert!(kinds.contains(&"decode_detect"), "kinds: {kinds:?}");
        assert!(!kinds.contains(&"store"), "kinds: {kinds:?}");
    }

    #[test]
    fn mqtt_broker_adds_output_stage() {
        let cfg = PipelineConfig {
            mqtt_url: Some("mqtt://localhost:1883".to_string()),
            ..PipelineConfig::default()
        };
        let p = generate_pipeline(
            1,
            "rtsp://cam/stream",
            "",
            "",
            &[],
            &cfg,
            StoreMode::Continuous,
            Path::new("/events"),
            false,
        );
        let capture = &p["plugins"][0];
        let children = capture["children"].as_array().unwrap();
        // capture's children: detect, store, output_mqtt (all siblings).
        assert_eq!(children.len(), 3);
        assert_eq!(children[0]["kind"], "decode_detect");
        assert_eq!(children[1]["kind"], "store");
        assert_eq!(children[2]["kind"], "output_mqtt");
        assert_eq!(children[2]["cfg"]["port"], 1883);
        let detect = &children[0];
        // No zones supplied → the detect cfg omits the zones key entirely.
        assert!(detect["cfg"].get("zones").is_none());
    }

    #[test]
    fn synopsis_flag_adds_review_and_plate_export_with_polygon_masks() {
        let cfg = test_cfg();
        // synopsis disabled → plain detect+store, bbox-only detect (no mask_format).
        let plain = generate_pipeline(
            3,
            "rtsp://c",
            "",
            "",
            &[],
            &cfg,
            StoreMode::Event,
            Path::new("/e"),
            false,
        );
        let plain_children = plain["plugins"][0]["children"].as_array().unwrap();
        assert_eq!(plain_children.len(), 2);
        assert!(plain_children[0]["cfg"].get("mask_format").is_none());

        // synopsis enabled → polygon masks + tracker on detect, plus review_export
        // and plate_export siblings under capture.
        let syn = generate_pipeline(
            3,
            "rtsp://c",
            "",
            "",
            &[],
            &cfg,
            StoreMode::Event,
            Path::new("/e"),
            true,
        );
        let detect = &syn["plugins"][0]["children"][0];
        assert_eq!(detect["cfg"]["mask_format"], "polygon");
        assert_eq!(detect["cfg"]["tracker"], true);

        let kinds: Vec<&str> = syn["plugins"][0]["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["kind"].as_str().unwrap())
            .collect();
        assert!(kinds.contains(&"review_export"), "kinds: {kinds:?}");
        assert!(kinds.contains(&"plate_export"), "kinds: {kinds:?}");
    }

    #[test]
    fn split_url_credentials_strips_userinfo_and_decodes() {
        // Percent-encoded userinfo is split out and decoded; the rest is verbatim.
        let (clean, creds) = split_url_credentials(
            "rtsp://admin:plmokn09%29@192.168.0.225:554/Streaming/Channels/101?transportmode=mcast",
        );
        assert_eq!(
            clean,
            "rtsp://192.168.0.225:554/Streaming/Channels/101?transportmode=mcast"
        );
        assert_eq!(creds, Some(("admin".to_string(), "plmokn09)".to_string())));

        // No userinfo → unchanged, no creds.
        let (clean, creds) = split_url_credentials("rtsp://cam:554/s?x=1");
        assert_eq!(clean, "rtsp://cam:554/s?x=1");
        assert_eq!(creds, None);

        // Username only (no ':').
        let (clean, creds) = split_url_credentials("rtsp://user@host/s");
        assert_eq!(clean, "rtsp://host/s");
        assert_eq!(creds, Some(("user".to_string(), String::new())));

        // An '@' in the path must not be mistaken for userinfo.
        let (clean, creds) = split_url_credentials("rtsp://host/path@weird");
        assert_eq!(clean, "rtsp://host/path@weird");
        assert_eq!(creds, None);
    }

    #[test]
    fn compose_wraps_stored_graph_with_capture_and_store() {
        let cfg = test_cfg();
        // A stored processing graph: decode_detect -> tracker (no capture/store).
        let graph = json!({
            "plugins": [
                { "id": "detect", "kind": "decode_detect", "cfg": { "conf_threshold": 0.5 },
                  "children": [ { "id": "track", "kind": "tracker", "cfg": {} } ] }
            ]
        });
        let p = compose_pipeline(
            9,
            "rtsp://cam:554/main",
            "admin",
            "p@ss",
            &graph,
            &[],
            &cfg,
            StoreMode::Continuous,
            Path::new("/var/lib/zm/events"),
        )
        .expect("compose");

        // Top-level wraps a single capture node with the credentials split out.
        assert_eq!(p["name"], "monitor_9");
        let capture = &p["plugins"][0];
        assert_eq!(capture["kind"], "capture_rtsp_multi");
        assert_eq!(capture["cfg"]["streams"][0]["url"], "rtsp://cam:554/main");
        assert_eq!(capture["cfg"]["streams"][0]["username"], "admin");

        // capture children = [stored detect node ..., composed store node].
        let children = capture["children"].as_array().unwrap();
        assert_eq!(children[0]["kind"], "decode_detect");
        assert_eq!(children[0]["children"][0]["kind"], "tracker");
        let last = children.last().unwrap();
        assert_eq!(last["kind"], "store");
        assert_eq!(last["cfg"]["mode"], "continuous");
        assert_eq!(last["cfg"]["monitor_id"], 9);
    }

    #[test]
    fn compose_injects_zm_zones_into_zones_node() {
        let cfg = test_cfg();
        let graph = json!({
            "plugins": [
                { "id": "z", "kind": "zones",
                  "children": [ { "id": "m", "kind": "motion_pixel_diff", "cfg": {} } ] }
            ]
        });
        let zones = vec![
            ZoneSpec {
                name: "Yard".to_string(),
                coords: "0,0 100,0 100,100 0,100".to_string(),
                zm_type: "Active".to_string(),
                min_pixel_threshold: Some(25),
                max_pixel_threshold: None,
            },
            ZoneSpec {
                name: "Road".to_string(),
                coords: "0,0 10,0 10,10 0,10".to_string(),
                zm_type: "Preclusive".to_string(),
                min_pixel_threshold: None,
                max_pixel_threshold: None,
            },
        ];
        let p = compose_pipeline(
            1,
            "rtsp://c",
            "",
            "",
            &graph,
            &zones,
            &cfg,
            StoreMode::Event,
            Path::new("/e"),
        )
        .expect("compose");

        let zones_node = &p["plugins"][0]["children"][0];
        assert_eq!(zones_node["kind"], "zones");
        let injected = zones_node["cfg"]["zones"].as_array().unwrap();
        assert_eq!(injected.len(), 2);
        assert_eq!(injected[0]["name"], "Yard");
        assert_eq!(injected[0]["type"], "Active");
        assert_eq!(injected[1]["type"], "Preclusive");
    }

    #[test]
    fn default_processing_graph_is_a_valid_capture_free_seed() {
        let cfg = test_cfg();
        let g = default_processing_graph(&cfg, false);
        let kinds: Vec<&str> = g["plugins"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["kind"].as_str().unwrap())
            .collect();
        // Seeds the detector but never the composed capture/store nodes.
        assert!(kinds.contains(&"decode_detect"), "kinds: {kinds:?}");
        assert!(!kinds.contains(&"store"), "kinds: {kinds:?}");
        assert!(!kinds.contains(&"capture_rtsp_multi"), "kinds: {kinds:?}");
        // And it passes the stored-graph validator (so PUT/enable accept it).
        assert!(crate::service::zmnext::graph::validate_graph(&g).is_ok());
    }

    #[test]
    fn privacy_zone_switches_default_to_transcode_topology() {
        let cfg = test_cfg();
        let zones = vec![
            ZoneSpec {
                name: "Front".to_string(),
                coords: "0,0 640,0 640,480 0,480".to_string(),
                zm_type: "Active".to_string(),
                min_pixel_threshold: Some(25),
                max_pixel_threshold: None,
            },
            ZoneSpec {
                name: "Neighbour".to_string(),
                coords: "10,10 50,10 50,50 10,50".to_string(),
                zm_type: "Privacy".to_string(),
                min_pixel_threshold: None,
                max_pixel_threshold: None,
            },
        ];
        let p = generate_pipeline(
            5,
            "rtsp://cam/main",
            "",
            "",
            &zones,
            &cfg,
            StoreMode::Continuous,
            Path::new("/e"),
            false,
        );
        // capture -> decode_ffmpeg -> privacy_mask -> { detect_onnx, encode -> store }
        let capture = &p["plugins"][0];
        let decode = &capture["children"][0];
        assert_eq!(decode["kind"], "decode_ffmpeg");
        let privacy = &decode["children"][0];
        assert_eq!(privacy["kind"], "privacy_mask");
        // The Privacy zone became one masked region (a polygon of points).
        let regions = privacy["cfg"]["regions"].as_array().unwrap();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0][0], json!([10, 10]));
        let pchildren = privacy["children"].as_array().unwrap();
        let kinds: Vec<&str> = pchildren
            .iter()
            .map(|c| c["kind"].as_str().unwrap())
            .collect();
        assert!(kinds.contains(&"detect_onnx"), "kinds: {kinds:?}");
        let encode = pchildren
            .iter()
            .find(|c| c["kind"] == "encode_ffmpeg")
            .unwrap();
        // The recorder hangs off the encoder, so it stores the masked stream.
        assert_eq!(encode["children"][0]["kind"], "store");
    }

    #[test]
    fn compose_injects_privacy_regions_into_privacy_mask_node() {
        let cfg = test_cfg();
        let graph = json!({
            "plugins": [
                { "id": "decode", "kind": "decode_ffmpeg",
                  "children": [ { "id": "pm", "kind": "privacy_mask", "cfg": { "mode": "black" } } ] }
            ]
        });
        let zones = vec![ZoneSpec {
            name: "Window".to_string(),
            coords: "1,1 2,1 2,2 1,2".to_string(),
            zm_type: "Privacy".to_string(),
            min_pixel_threshold: None,
            max_pixel_threshold: None,
        }];
        let p = compose_pipeline(
            1,
            "rtsp://c",
            "",
            "",
            &graph,
            &zones,
            &cfg,
            StoreMode::Event,
            Path::new("/e"),
        )
        .expect("compose");
        let pm = &p["plugins"][0]["children"][0]["children"][0];
        assert_eq!(pm["kind"], "privacy_mask");
        let regions = pm["cfg"]["regions"].as_array().unwrap();
        assert_eq!(regions.len(), 1);
        // mode is preserved; regions injected.
        assert_eq!(pm["cfg"]["mode"], "black");
    }
}
