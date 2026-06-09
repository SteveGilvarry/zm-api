//! HAP accessory database (HAP spec ch. 6–9).
//!
//! Exposes the JSON returned by `GET /accessories`: a bridge accessory
//! (`aid 1`) plus one camera accessory (`aid 2`). Each accessory is a list of
//! services, each service a list of characteristics keyed by an instance id
//! (`iid`) unique within the accessory.
//!
//! Phase 1 keeps the model small and mostly static. The dynamic camera
//! characteristics (`SetupEndpoints`, `SelectedRTPStreamConfiguration`,
//! `StreamingStatus`) are owned by [`crate::homekit::camera`]; here we only
//! declare them and their iids so the controller can address them.

use serde_json::{json, Value};

// --- HAP service type short-UUIDs (HAP spec ch. 8). ---
const SVC_ACCESSORY_INFORMATION: &str = "3E";
const SVC_PROTOCOL_INFORMATION: &str = "A2";
const SVC_CAMERA_RTP_STREAM_MANAGEMENT: &str = "110";
const SVC_MICROPHONE: &str = "112";

// --- HAP characteristic type short-UUIDs (HAP spec ch. 9). ---
const CH_IDENTIFY: &str = "14";
const CH_MANUFACTURER: &str = "20";
const CH_MODEL: &str = "21";
const CH_NAME: &str = "23";
const CH_SERIAL_NUMBER: &str = "30";
const CH_FIRMWARE_REVISION: &str = "52";
const CH_VERSION: &str = "37";
const CH_MUTE: &str = "11C";

const CH_STREAMING_STATUS: &str = "120";
const CH_SUPPORTED_VIDEO_STREAM_CONFIG: &str = "114";
const CH_SUPPORTED_AUDIO_STREAM_CONFIG: &str = "115";
const CH_SUPPORTED_RTP_CONFIG: &str = "116";
const CH_SELECTED_RTP_STREAM_CONFIG: &str = "117";
const CH_SETUP_ENDPOINTS: &str = "118";

/// Well-known instance ids for the camera accessory's stream-management
/// characteristics. Stable so the controller and our handlers agree.
pub mod iid {
    pub const CAMERA_STREAMING_STATUS: u64 = 0x21;
    pub const CAMERA_SUPPORTED_VIDEO: u64 = 0x22;
    pub const CAMERA_SUPPORTED_AUDIO: u64 = 0x23;
    pub const CAMERA_SUPPORTED_RTP: u64 = 0x24;
    pub const CAMERA_SELECTED_RTP: u64 = 0x25;
    pub const CAMERA_SETUP_ENDPOINTS: u64 = 0x26;
}

/// Aid of the camera accessory.
pub const CAMERA_AID: u64 = 2;

/// A characteristic entry in the `/accessories` JSON.
fn ch(iid: u64, ty: &str, perms: &[&str], format: &str, value: Value) -> Value {
    json!({
        "iid": iid,
        "type": ty,
        "perms": perms,
        "format": format,
        "value": value,
    })
}

/// Build the static `/accessories` document.
///
/// `bridge_name` is the user-facing name; `serial`/`model`/`firmware` describe
/// both accessories. The camera's dynamic TLV characteristics are emitted with
/// empty base64 values — the controller writes/reads them at stream time.
#[allow(clippy::too_many_arguments)]
pub fn accessories_json(
    bridge_name: &str,
    camera_name: &str,
    model: &str,
    serial: &str,
    firmware: &str,
    supported_video_b64: &str,
    supported_audio_b64: &str,
    supported_rtp_b64: &str,
) -> Value {
    let bridge = json!({
        "aid": 1,
        "services": [
            accessory_information(1, bridge_name, model, serial, firmware),
            {
                "iid": 10,
                "type": SVC_PROTOCOL_INFORMATION,
                "characteristics": [
                    ch(11, CH_VERSION, &["pr"], "string", json!("1.1.0")),
                ],
            },
        ],
    });

    let camera = json!({
        "aid": CAMERA_AID,
        "services": [
            accessory_information(1, camera_name, model, serial, firmware),
            {
                "iid": 0x20,
                "type": SVC_CAMERA_RTP_STREAM_MANAGEMENT,
                "characteristics": [
                    ch(iid::CAMERA_STREAMING_STATUS, CH_STREAMING_STATUS, &["pr", "ev"], "tlv8", json!("")),
                    ch(iid::CAMERA_SUPPORTED_VIDEO, CH_SUPPORTED_VIDEO_STREAM_CONFIG, &["pr"], "tlv8", json!(supported_video_b64)),
                    ch(iid::CAMERA_SUPPORTED_AUDIO, CH_SUPPORTED_AUDIO_STREAM_CONFIG, &["pr"], "tlv8", json!(supported_audio_b64)),
                    ch(iid::CAMERA_SUPPORTED_RTP, CH_SUPPORTED_RTP_CONFIG, &["pr"], "tlv8", json!(supported_rtp_b64)),
                    ch(iid::CAMERA_SELECTED_RTP, CH_SELECTED_RTP_STREAM_CONFIG, &["pr", "pw"], "tlv8", json!("")),
                    ch(iid::CAMERA_SETUP_ENDPOINTS, CH_SETUP_ENDPOINTS, &["pr", "pw"], "tlv8", json!("")),
                ],
            },
            {
                "iid": 0x30,
                "type": SVC_MICROPHONE,
                "characteristics": [
                    ch(0x31, CH_MUTE, &["pr", "pw", "ev"], "bool", json!(false)),
                ],
            },
        ],
    });

    json!({ "accessories": [bridge, camera] })
}

fn accessory_information(
    base_iid: u64,
    name: &str,
    model: &str,
    serial: &str,
    firmware: &str,
) -> Value {
    json!({
        "iid": base_iid,
        "type": SVC_ACCESSORY_INFORMATION,
        "characteristics": [
            ch(base_iid + 1, CH_IDENTIFY, &["pw"], "bool", Value::Null),
            ch(base_iid + 2, CH_MANUFACTURER, &["pr"], "string", json!("ZoneMinder")),
            ch(base_iid + 3, CH_MODEL, &["pr"], "string", json!(model)),
            ch(base_iid + 4, CH_NAME, &["pr"], "string", json!(name)),
            ch(base_iid + 5, CH_SERIAL_NUMBER, &["pr"], "string", json!(serial)),
            ch(base_iid + 6, CH_FIRMWARE_REVISION, &["pr"], "string", json!(firmware)),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_two_accessories_with_camera_service() {
        let doc = accessories_json(
            "ZoneMinder",
            "Front Door",
            "zm-api",
            "ZM-0001",
            "3.0.0",
            "v",
            "a",
            "r",
        );
        let acc = doc["accessories"].as_array().unwrap();
        assert_eq!(acc.len(), 2);
        assert_eq!(acc[0]["aid"], 1);
        assert_eq!(acc[1]["aid"], CAMERA_AID);

        // Camera has the RTP stream-management service with the setup-endpoints char.
        let svcs = acc[1]["services"].as_array().unwrap();
        let has_stream_mgmt = svcs
            .iter()
            .any(|s| s["type"] == SVC_CAMERA_RTP_STREAM_MANAGEMENT);
        assert!(has_stream_mgmt);
        let stream = svcs
            .iter()
            .find(|s| s["type"] == SVC_CAMERA_RTP_STREAM_MANAGEMENT)
            .unwrap();
        let setup = stream["characteristics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["iid"] == iid::CAMERA_SETUP_ENDPOINTS);
        assert!(setup);
    }

    #[test]
    fn supported_configs_are_embedded() {
        let doc = accessories_json("B", "C", "m", "s", "f", "VIDEO_B64", "AUDIO_B64", "RTP_B64");
        let s = doc.to_string();
        assert!(s.contains("VIDEO_B64"));
        assert!(s.contains("AUDIO_B64"));
        assert!(s.contains("RTP_B64"));
    }
}
