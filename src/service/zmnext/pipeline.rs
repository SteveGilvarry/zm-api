//! Generates a zm-next worker pipeline JSON from a Monitor row and its Zones.
//!
//! zm-next consumes a recursive plugin tree — `{id, kind, cfg, children}` under
//! a top-level `{name, root, plugins}` — and never reads the database itself;
//! zm-api is the sole DB reader and hands the worker a fully-resolved pipeline
//! file via `--pipeline`. The shape is `capture_rtsp_multi → decode_detect →
//! {store[, output_mqtt]}`, where `store` is zm-next's merged recorder (it
//! folded the old `store_filesystem` + `store_event` into one mode-switched
//! plugin).
//!
//! The Monitor's `Path` column already holds the full RTSP URL (credentials
//! embedded) that zm-api uses everywhere else, so it is used verbatim as the
//! capture source. ZoneMinder Zones are translated into a best-effort `zones`
//! array on the detect stage (polygon points + pixel thresholds); evolution is
//! additive, so unknown keys are harmless.
//!
//! The generator works on small plain inputs ([`ZoneSpec`]) rather than the
//! `monitors`/`zones` entities directly, so it is fully unit-testable; the thin
//! [`zone_specs_from_models`] mapper bridges the SeaORM models at the call site.

use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::configure::zmnext::PipelineConfig;
use crate::entity::sea_orm_active_enums::{Function, ZoneType};
use crate::entity::zones;

/// Recording mode for the merged `store` plugin, derived from the monitor's
/// ZoneMinder function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreMode {
    /// Gapless segment recording (ZM `Record`).
    Continuous,
    /// Triggered pre/post-roll clips only (ZM `Modect`/`Nodect`).
    Event,
    /// Both continuous segments and triggered clips (ZM `Mocord`).
    Both,
}

impl StoreMode {
    /// Map a monitor's `Function` to a store mode. Non-recording functions
    /// (`Monitor`/`None`) default to continuous, matching the worker default.
    pub fn from_function(function: &Function) -> Self {
        match function {
            Function::Modect | Function::Nodect => StoreMode::Event,
            Function::Mocord => StoreMode::Both,
            // Record, Monitor, None
            _ => StoreMode::Continuous,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            StoreMode::Continuous => "continuous",
            StoreMode::Event => "event",
            StoreMode::Both => "both",
        }
    }

    fn records_continuous(self) -> bool {
        matches!(self, StoreMode::Continuous | StoreMode::Both)
    }

    fn records_events(self) -> bool {
        matches!(self, StoreMode::Event | StoreMode::Both)
    }
}

/// A ZoneMinder zone reduced to what the detect stage needs.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSpec {
    pub name: String,
    /// Raw `Coords` string ("x1,y1 x2,y2 ...").
    pub coords: String,
    pub min_pixel_threshold: Option<u16>,
    pub max_pixel_threshold: Option<u16>,
}

/// Reduce SeaORM zone models to [`ZoneSpec`]s, dropping Inactive and Privacy
/// zones (which never trigger motion).
pub fn zone_specs_from_models(zones: &[zones::Model]) -> Vec<ZoneSpec> {
    zones
        .iter()
        .filter(|z| !matches!(z.r#type, ZoneType::Inactive | ZoneType::Privacy))
        .map(|z| ZoneSpec {
            name: z.name.clone(),
            coords: z.coords.clone(),
            min_pixel_threshold: z.min_pixel_threshold,
            max_pixel_threshold: z.max_pixel_threshold,
        })
        .collect()
}

/// Build the pipeline JSON document for one monitor.
///
/// * `source_url` — the capture RTSP URL (the monitor's `Path`).
/// * `mode` — recording mode for the merged `store` plugin (from the monitor's
///   function).
/// * `events_root` — media root the `store` plugin writes under. **Must be on
///   the same filesystem as the directory zm-api hands back in
///   `assign_recording`**, because the worker renames the in-progress file into
///   that directory with an open fd (a cross-fs target fails and the clip keeps
///   the worker's own name).
pub fn generate_pipeline(
    monitor_id: u32,
    source_url: &str,
    zones: &[ZoneSpec],
    cfg: &PipelineConfig,
    mode: StoreMode,
    events_root: &Path,
) -> Value {
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

    // The merged `store` plugin (zm-next folded store_filesystem + store_event
    // into one). Common keys always; segment / event keys only for the modes
    // that use them. Every segment and every clip is one ZM event and runs the
    // id-assignment handshake.
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
    let mut detect_children = vec![json!({
        "id": "record",
        "kind": "store",
        "cfg": store_cfg,
        "queue_depth": 120,
    })];
    if let Some((host, port)) = mqtt_host_port(cfg.mqtt_url.as_deref()) {
        detect_children.push(json!({
            "id": "notify",
            "kind": "output_mqtt",
            "cfg": { "host": host, "port": port, "base_topic": "zm-next" },
            "queue_depth": 8,
        }));
    }

    json!({
        "name": format!("monitor_{monitor_id}"),
        "root": true,
        "plugins": [{
            "id": "capture",
            "kind": "capture_rtsp_multi",
            "cfg": {
                "streams": [{
                    "stream_id": 0,
                    "url": source_url,
                    "transport": cfg.rtsp_transport,
                    "max_retry_attempts": -1,
                }],
            },
            "children": [{
                "id": "detect",
                "kind": "decode_detect",
                "cfg": detect_cfg,
                "queue_depth": 2,
                "children": detect_children,
            }],
        }],
    })
}

/// Serialize and write the pipeline for `monitor_id` to `dir`, returning the
/// file path. Creates `dir` if it does not exist.
pub fn write_pipeline_file(dir: &Path, monitor_id: u32, pipeline: &Value) -> io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("monitor_{monitor_id}.json"));
    let body = serde_json::to_vec_pretty(pipeline).map_err(io::Error::other)?;
    std::fs::write(&path, body)?;
    Ok(path)
}

/// Turn [`ZoneSpec`]s into detect-stage zone JSON, dropping zones whose polygon
/// fails to parse.
fn translate_zones(zones: &[ZoneSpec]) -> Vec<Value> {
    zones
        .iter()
        .filter_map(|z| {
            let points = parse_coords(&z.coords);
            if points.is_empty() {
                return None;
            }
            Some(json!({
                "name": z.name,
                "points": points,
                "min_pixel_threshold": z.min_pixel_threshold,
                "max_pixel_threshold": z.max_pixel_threshold,
            }))
        })
        .collect()
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
            min_pixel_threshold: Some(25),
            max_pixel_threshold: None,
        }];
        let p = generate_pipeline(
            7,
            "rtsp://admin:pw@cam:554/Streaming/Channels/101",
            &zones,
            &cfg,
            StoreMode::Event,
            Path::new("/var/lib/zm/events"),
        );

        assert_eq!(p["name"], "monitor_7");
        assert_eq!(p["root"], true);
        let capture = &p["plugins"][0];
        assert_eq!(capture["kind"], "capture_rtsp_multi");
        assert_eq!(
            capture["cfg"]["streams"][0]["url"],
            "rtsp://admin:pw@cam:554/Streaming/Channels/101"
        );

        let detect = &capture["children"][0];
        assert_eq!(detect["kind"], "decode_detect");
        // The active zone is translated onto the detect stage.
        assert_eq!(detect["cfg"]["zones"][0]["name"], "Front");
        assert_eq!(detect["cfg"]["zones"][0]["points"][1], json!([640, 0]));
        assert_eq!(detect["cfg"]["zones"][0]["min_pixel_threshold"], 25);

        // One merged `store` plugin in event mode: roll/trigger keys, no max_secs.
        let record = &detect["children"][0];
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
        // No MQTT broker configured → no output_mqtt child.
        assert_eq!(detect["children"].as_array().unwrap().len(), 1);
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
        assert_eq!(
            StoreMode::from_function(&Function::Monitor),
            StoreMode::Continuous
        );

        let cfg = test_cfg();
        // store lives at capture → detect → store.
        // Continuous: segment key present, event keys absent.
        let cont = generate_pipeline(
            1,
            "rtsp://c",
            &[],
            &cfg,
            StoreMode::Continuous,
            Path::new("/e"),
        );
        let store = &cont["plugins"][0]["children"][0]["children"][0]["cfg"];
        assert_eq!(store["mode"], "continuous");
        assert_eq!(store["max_secs"], 300);
        assert!(store.get("pre_roll_sec").is_none());
        assert!(store.get("trigger_types").is_none());

        // Both: segment AND event keys present.
        let both = generate_pipeline(1, "rtsp://c", &[], &cfg, StoreMode::Both, Path::new("/e"));
        let store = &both["plugins"][0]["children"][0]["children"][0]["cfg"];
        assert_eq!(store["mode"], "both");
        assert_eq!(store["max_secs"], 300);
        assert_eq!(store["post_roll_sec"], 10);
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
            &[],
            &cfg,
            StoreMode::Continuous,
            Path::new("/events"),
        );
        let detect = &p["plugins"][0]["children"][0];
        let children = detect["children"].as_array().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[1]["kind"], "output_mqtt");
        assert_eq!(children[1]["cfg"]["port"], 1883);
        // No zones supplied → the detect cfg omits the zones key entirely.
        assert!(detect["cfg"].get("zones").is_none());
    }
}
