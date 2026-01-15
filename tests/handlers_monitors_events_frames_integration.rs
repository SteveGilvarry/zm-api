// Integration tests for monitors, events, and frames with a real database
// Run with: cargo test --test handlers_monitors_events_frames_integration -- --include-ignored
#![allow(clippy::needless_borrows_for_generic_args)]

mod common;

use axum::body::{self, Body};
use axum::http::{header, Request, StatusCode};
use common::test_db::{get_test_db, test_prefix};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};
use tower::ServiceExt;
use zm_api::dto::request::{AlarmControlRequest, CreateMonitorRequest, UpdateStateRequest};
use zm_api::dto::response::{
    EventResponse, FrameResponse, MonitorResponse, PaginatedEventsResponse,
};
use zm_api::entity::sea_orm_active_enums::{
    Analysing, AnalysisImage, AnalysisSource, Capturing, Decoding, DefaultCodec, EventCloseMode,
    Function, Importance, MonitorType, Orientation, OutputContainer, Recording, RecordingSource,
    Rtsp2WebType,
};
use zm_api::entity::{events, frames, monitors};

fn auth_header() -> String {
    let token = zm_api::service::token::generate_tokens("tester".to_string())
        .expect("token")
        .access_token;
    format!("Bearer {}", token)
}

fn build_app(db: DatabaseConnection) -> axum::Router {
    let state = zm_api::server::state::AppState::for_test_with_db(db);
    zm_api::routes::create_router_app(state)
}

fn build_create_monitor_request(name: String) -> CreateMonitorRequest {
    CreateMonitorRequest {
        name,
        deleted: false,
        notes: None,
        server_id: None,
        storage_id: 1,
        manufacturer_id: None,
        model_id: None,
        r#type: MonitorType::Local,
        function: Function::Monitor,
        capturing: Capturing::None,
        enabled: 1,
        decoding_enabled: 0,
        decoding: Decoding::None,
        rtsp2_web_enabled: 0,
        rtsp2_web_type: Rtsp2WebType::Hls,
        janus_enabled: 0,
        janus_audio_enabled: 0,
        janus_profile_override: Some(String::new()),
        janus_use_rtsp_restream: 0,
        janus_rtsp_user: None,
        janus_rtsp_session_timeout: Some(0),
        linked_monitors: None,
        triggers: String::new(),
        event_start_command: String::new(),
        event_end_command: String::new(),
        onvif_url: String::new(),
        onvif_events_path: "/Events".to_string(),
        onvif_username: String::new(),
        onvif_password: String::new(),
        onvif_options: String::new(),
        onvif_event_listener: 0,
        onvif_alarm_text: None,
        use_amcrest_api: 0,
        device: String::new(),
        channel: 0,
        format: 0,
        v4l_multi_buffer: None,
        v4l_captures_per_frame: None,
        protocol: None,
        method: None,
        host: None,
        port: String::new(),
        sub_path: String::new(),
        path: None,
        second_path: None,
        options: None,
        user: None,
        pass: None,
        width: 640,
        height: 480,
        colours: 3,
        palette: 0,
        orientation: Orientation::Rotate0,
        deinterlacing: 0,
        decoder: None,
        decoder_hw_accel_name: None,
        decoder_hw_accel_device: None,
        save_jpe_gs: 0,
        video_writer: 0,
        output_codec: Some(0),
        encoder: None,
        output_container: OutputContainer::Auto,
        encoder_parameters: None,
        record_audio: 0,
        recording_source: RecordingSource::Primary,
        rtsp_describe: None,
        brightness: 0,
        contrast: 0,
        hue: 0,
        colour: 0,
        event_prefix: "Event-".to_string(),
        label_format: None,
        label_x: 0,
        label_y: 0,
        label_size: 1,
        image_buffer_count: 1,
        max_image_buffer_count: 1,
        warmup_count: 0,
        pre_event_count: 0,
        post_event_count: 0,
        stream_replay_buffer: 1,
        alarm_frame_count: 1,
        section_length: 1,
        section_length_warn: 0,
        event_close_mode: EventCloseMode::System,
        min_section_length: 1,
        frame_skip: 0,
        motion_frame_skip: 0,
        analysis_fps_limit: None,
        analysis_update_delay: 0,
        max_fps: None,
        alarm_max_fps: None,
        fps_report_interval: 0,
        ref_blend_perc: 0,
        alarm_ref_blend_perc: 0,
        controllable: 0,
        control_id: None,
        control_device: None,
        control_address: None,
        auto_stop_timeout: None,
        track_motion: 0,
        track_delay: None,
        return_location: 0,
        return_delay: None,
        modect_during_ptz: 0,
        default_rate: "100".to_string(),
        default_scale: "100".to_string(),
        default_codec: DefaultCodec::Auto,
        signal_check_points: 0,
        signal_check_colour: String::new(),
        web_colour: String::new(),
        exif: 0,
        sequence: None,
        zone_count: 0,
        refresh: None,
        latitude: None,
        longitude: None,
        rtsp_server: 0,
        rtsp_stream_name: String::new(),
        soap_wsa_compl: 0,
        importance: Importance::Normal,
        mqtt_enabled: 0,
        mqtt_subscriptions: String::new(),
        startup_delay: 0,
        analysing: Analysing::None,
        analysis_source: AnalysisSource::Primary,
        analysis_image: AnalysisImage::FullColour,
        recording: Recording::None,
    }
}

async fn create_monitor(app: &axum::Router, name: String) -> MonitorResponse {
    let create_body = serde_json::to_vec(&build_create_monitor_request(name))
        .expect("serialize monitor create request");
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v3/monitors")
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).expect("parse monitor response")
}

async fn get_monitor(app: &axum::Router, id: u32) -> MonitorResponse {
    let response = app
        .clone()
        .oneshot(
            Request::get(&format!("/api/v3/monitors/{}", id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).expect("parse monitor response")
}

async fn create_monitor_db(db: &DatabaseConnection) -> Result<monitors::Model, DbErr> {
    let name = format!("{}monitor", test_prefix());
    let model = monitors::ActiveModel {
        name: Set(name),
        ..Default::default()
    };
    model.insert(db).await
}

async fn create_event_db(db: &DatabaseConnection, monitor_id: u32) -> Result<events::Model, DbErr> {
    let name = format!("{}event", test_prefix());
    let model = events::ActiveModel {
        monitor_id: Set(monitor_id),
        state_id: Set(1),
        name: Set(name),
        ..Default::default()
    };
    model.insert(db).await
}

async fn create_frame_db(db: &DatabaseConnection, event_id: u64) -> Result<frames::Model, DbErr> {
    let model = frames::ActiveModel {
        event_id: Set(event_id),
        frame_id: Set(1),
        ..Default::default()
    };
    model.insert(db).await
}

async fn cleanup_monitor_db(db: &DatabaseConnection, id: u32) -> Result<(), DbErr> {
    monitors::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

async fn cleanup_event_db(db: &DatabaseConnection, id: u64) -> Result<(), DbErr> {
    events::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

async fn cleanup_frame_db(db: &DatabaseConnection, id: u64) -> Result<(), DbErr> {
    frames::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_monitors_create_update_delete() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let app = build_app(db);

    let name = format!("{}create_monitor", test_prefix());
    let created = create_monitor(&app, name).await;
    let fetched = get_monitor(&app, created.id).await;
    assert_eq!(fetched.name, created.name);

    let updated_name = format!("{}updated_monitor", test_prefix());
    let update_body = serde_json::to_vec(&serde_json::json!({
        "name": updated_name,
        "enabled": 0
    }))
    .expect("serialize update body");
    let response = app
        .clone()
        .oneshot(
            Request::patch(&format!("/api/v3/monitors/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    let updated: MonitorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, updated_name);

    let fetched = get_monitor(&app, created.id).await;
    assert_eq!(fetched.name, updated_name);
    assert_eq!(fetched.enabled, 0);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/monitors/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_monitors_state_alarm() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let app = build_app(db);

    let name = format!("{}state_monitor", test_prefix());
    let created = create_monitor(&app, name).await;

    let state_body = serde_json::to_vec(&UpdateStateRequest {
        state: "start".to_string(),
    })
    .expect("serialize state request");
    let response = app
        .clone()
        .oneshot(
            Request::patch(&format!("/api/v3/monitors/{}/state", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(state_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let alarm_body = serde_json::to_vec(&AlarmControlRequest {
        action: "status".to_string(),
    })
    .expect("serialize alarm request");
    let response = app
        .clone()
        .oneshot(
            Request::patch(&format!("/api/v3/monitors/{}/alarm", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(alarm_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::delete(&format!("/api/v3/monitors/{}", created.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_monitors_list_get() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let monitor = create_monitor_db(&db)
        .await
        .expect("Failed to create monitor");
    let app = build_app(db);

    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v3/monitors")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    let body: Vec<MonitorResponse> = serde_json::from_slice(&bytes).unwrap();
    assert!(body.iter().any(|m| m.id == monitor.id));

    let response = app
        .oneshot(
            Request::get(&format!("/api/v3/monitors/{}", monitor.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    let body: MonitorResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.id, monitor.id);

    let cleanup_db = get_test_db()
        .await
        .expect("Failed to get cleanup connection");
    cleanup_monitor_db(&cleanup_db, monitor.id)
        .await
        .expect("Failed to cleanup monitor");
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_events_list_get() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let monitor = create_monitor_db(&db)
        .await
        .expect("Failed to create monitor");
    let event = create_event_db(&db, monitor.id)
        .await
        .expect("Failed to create event");
    let app = build_app(db);

    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v3/events?page=1&page_size=100")
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    let body: PaginatedEventsResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(body.events.iter().any(|e| e.id == event.id));

    let response = app
        .oneshot(
            Request::get(&format!("/api/v3/events/{}", event.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .unwrap();
    let body: EventResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.id, event.id);

    let cleanup_db = get_test_db()
        .await
        .expect("Failed to get cleanup connection");
    cleanup_event_db(&cleanup_db, event.id)
        .await
        .expect("Failed to cleanup event");
    cleanup_monitor_db(&cleanup_db, monitor.id)
        .await
        .expect("Failed to cleanup monitor");
}

#[tokio::test]
#[ignore = "Requires running test database - run with: ./scripts/db-manager.sh mysql"]
async fn test_api_frames_list_get() {
    let db = get_test_db()
        .await
        .expect("Failed to connect to test database");
    let monitor = create_monitor_db(&db)
        .await
        .expect("Failed to create monitor");
    let event = create_event_db(&db, monitor.id)
        .await
        .expect("Failed to create event");
    let frame = create_frame_db(&db, event.id)
        .await
        .expect("Failed to create frame");
    let app = build_app(db);

    let response = app
        .clone()
        .oneshot(
            Request::get(&format!("/frames?event_id={}", event.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: Vec<FrameResponse> = serde_json::from_slice(&bytes).unwrap();
    assert!(body.iter().any(|f| f.id == frame.id));

    let response = app
        .oneshot(
            Request::get(&format!("/frames/{}", frame.id))
                .header(header::AUTHORIZATION, auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: FrameResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.id, frame.id);

    let cleanup_db = get_test_db()
        .await
        .expect("Failed to get cleanup connection");
    cleanup_frame_db(&cleanup_db, frame.id)
        .await
        .expect("Failed to cleanup frame");
    cleanup_event_db(&cleanup_db, event.id)
        .await
        .expect("Failed to cleanup event");
    cleanup_monitor_db(&cleanup_db, monitor.id)
        .await
        .expect("Failed to cleanup monitor");
}
