//! Regression for #55: request fields that land in bounded columns are capped
//! at the column width, so an over-long value is a 400 naming the field rather
//! than MySQL's 1406 surfacing as a 500. `varchar(N)` is N *characters*, so
//! those caps count chars; `tinytext` is 255 *bytes*, so that one counts bytes.

use garde::Validate;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use zm_api::dto::request::daemon::ApplyStateRequest;
use zm_api::dto::request::monitor::UpdateMonitorRequest;
use zm_api::dto::request::monitor_presets::UpdateMonitorPresetRequest;
use zm_api::dto::request::object_types::UpdateObjectTypeRequest;
use zm_api::dto::request::sessions::CreateSessionRequest;
use zm_api::dto::request::states::CreateStateRequest;
use zm_api::dto::request::tags::UpdateTagRequest;
use zm_api::dto::request::users::UpdateUserRequest;

fn valid<T: DeserializeOwned + Validate<Context = ()>>(body: Value) -> bool {
    serde_json::from_value::<T>(body)
        .expect("body deserialises")
        .validate()
        .is_ok()
}

#[test]
fn varchar_caps_count_characters_not_bytes() {
    // 64 two-byte characters is 128 bytes: legal in a varchar(64).
    assert!(valid::<UpdateTagRequest>(json!({ "name": "é".repeat(64) })));
    assert!(!valid::<UpdateTagRequest>(
        json!({ "name": "a".repeat(65) })
    ));
    assert!(valid::<UpdateMonitorRequest>(
        json!({ "name": "é".repeat(64) })
    ));
}

#[test]
fn the_short_columns_are_the_easy_ones_to_overflow() {
    assert!(valid::<UpdateUserRequest>(
        json!({ "language": "a".repeat(8) })
    ));
    assert!(!valid::<UpdateUserRequest>(
        json!({ "language": "a".repeat(9) })
    ));
    assert!(!valid::<UpdateUserRequest>(
        json!({ "max_bandwidth": "a".repeat(17) })
    ));

    assert!(valid::<UpdateMonitorPresetRequest>(
        json!({ "port": "a".repeat(8) })
    ));
    assert!(!valid::<UpdateMonitorPresetRequest>(
        json!({ "port": "a".repeat(9) })
    ));

    assert!(valid::<UpdateObjectTypeRequest>(
        json!({ "name": "a".repeat(32) })
    ));
    assert!(!valid::<UpdateObjectTypeRequest>(
        json!({ "name": "a".repeat(33) })
    ));

    assert!(valid::<CreateSessionRequest>(
        json!({ "id": "a".repeat(32) })
    ));
    assert!(!valid::<CreateSessionRequest>(
        json!({ "id": "a".repeat(33) })
    ));

    assert!(valid::<ApplyStateRequest>(
        json!({ "state_name": "a".repeat(64) })
    ));
    assert!(!valid::<ApplyStateRequest>(
        json!({ "state_name": "a".repeat(65) })
    ));

    let state = |n: usize| json!({ "name": "a".repeat(n), "definition": "", "is_active": 0 });
    assert!(valid::<CreateStateRequest>(state(64)));
    assert!(!valid::<CreateStateRequest>(state(65)));
}

#[test]
fn monitor_fields_that_were_skipped_are_now_bounded() {
    assert!(valid::<UpdateMonitorRequest>(
        json!({ "onvif_events_path": "a".repeat(20) })
    ));
    assert!(!valid::<UpdateMonitorRequest>(
        json!({ "onvif_events_path": "a".repeat(21) })
    ));
    assert!(valid::<UpdateMonitorRequest>(
        json!({ "janus_profile_override": "a".repeat(30) })
    ));
    assert!(!valid::<UpdateMonitorRequest>(
        json!({ "janus_profile_override": "a".repeat(31) })
    ));
    assert!(!valid::<UpdateMonitorRequest>(
        json!({ "decoder": "a".repeat(33) })
    ));
    assert!(!valid::<UpdateMonitorRequest>(
        json!({ "user": "a".repeat(65) })
    ));
    assert!(!valid::<UpdateMonitorRequest>(
        json!({ "options": "a".repeat(256) })
    ));
}

#[test]
fn tinytext_caps_count_bytes() {
    // 127 two-byte characters plus one ASCII byte is exactly 255 bytes.
    let fits = format!("{}a", "é".repeat(127));
    assert!(valid::<UpdateMonitorPresetRequest>(
        json!({ "device": fits })
    ));
    // 128 two-byte characters is 256 bytes: too long for tinytext even
    // though it is only 128 characters.
    assert!(!valid::<UpdateMonitorPresetRequest>(
        json!({ "device": "é".repeat(128) })
    ));
    assert!(!valid::<UpdateMonitorRequest>(
        json!({ "device": "é".repeat(128) })
    ));
}
