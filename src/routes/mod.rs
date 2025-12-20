use axum::{Router, routing::any, http::{Method, HeaderName}};
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;
use axum::extract::MatchedPath;
use tower_http::cors::CorsLayer;
use crate::handlers::openapi::ApiDoc;
use crate::server::state::AppState;

pub mod server;
pub mod auth;
pub mod monitors;
pub mod streaming;
pub mod events; // Add events module
pub mod mse; // Add MSE module
pub mod webrtc; // Add WebRTC module
pub mod configs; // Config management
pub mod zones; // Zones
pub mod filters; // Filters
pub mod users; // Users
pub mod groups; // Groups
pub mod servers; // Server info list
pub mod logs; // Logs
pub mod storage; // Storage
pub mod manufacturers; // Manufacturers
pub mod models; // Models
pub mod zone_presets; // Zone Presets
pub mod controls; // Controls
pub mod control_presets; // Control Presets
pub mod devices; // Devices
pub mod monitor_presets; // Monitor Presets
pub mod montage_layouts; // Montage Layouts
pub mod snapshots; // Snapshots
pub mod tags; // Tags
pub mod triggers_x10; // X10 Triggers
pub mod user_preferences; // User Preferences
pub mod sessions; // Sessions
pub mod states; // States
pub mod stats; // Stats
pub mod frames; // Frames
pub mod monitor_status; // Monitor Status
pub mod object_types; // Object Types
pub mod server_stats; // Server Stats
pub mod reports; // Reports
pub mod groups_monitors; // Groups Monitors
pub mod groups_permissions; // Groups Permissions
pub mod monitors_permissions; // Monitors Permissions
pub mod snapshots_events; // Snapshots Events
pub mod event_data; // Event Data
pub mod events_tags; // Events Tags

async fn fallback_handler(path: MatchedPath) -> &'static str {
    tracing::error!("Unknown route: {}", path.as_str());
    "Unknown route"
}

pub fn create_router_app(state: AppState) -> Router {
    // Get frontend URL from environment variable or use default localhost addresses
    let frontend_urls = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173,http://localhost:8000".to_string());
    
    // Parse the URLs into a Vec of HeaderValues for CORS configuration
    let origins = frontend_urls
        .split(',')
        .filter_map(|origin| origin.parse().ok())
        .collect::<Vec<_>>();
    
    tracing::info!("Configuring CORS with allowed origins: {:?}", frontend_urls);
    
    // Configure CORS to allow requests from the frontend(s)
    let cors = CorsLayer::new()
        // Allow frontend origins to access the API
        .allow_origin(origins)
        // Allow common HTTP methods needed for a RESTful API
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
        // Allow common HTTP headers used in API requests
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("accept"),
            HeaderName::from_static("origin"),
        ])
        .allow_credentials(true);

    let server_routes = server::add_server_routes(Router::new());
    let auth_routes = auth::add_routers(Router::new());
    let monitors_routes = monitors::add_monitor_routes(Router::new());
    let streaming_routes = streaming::add_streaming_routes(Router::new());
    let events_routes = events::add_event_routes(Router::new()); // Add events routes
    let mse_routes = mse::add_mse_routes(Router::new()); // Add MSE routes
    let webrtc_routes = webrtc::add_webrtc_routes(Router::new()); // Add WebRTC routes
    let config_routes = configs::add_config_routes(Router::new()); // Add Config routes
    let zone_routes = zones::add_zone_routes(Router::new());
    let filter_routes = filters::add_filter_routes(Router::new());
    let user_routes = users::add_user_routes(Router::new());
    let group_routes = groups::add_group_routes(Router::new());
    let server_info_routes = servers::add_server_info_routes(Router::new());
    let log_routes = logs::add_log_routes(Router::new());
    let storage_routes = storage::add_storage_routes(Router::new());
    let manufacturer_routes = manufacturers::add_manufacturer_routes(Router::new());
    let model_routes = models::add_model_routes(Router::new());
    let zone_preset_routes = zone_presets::add_zone_preset_routes(Router::new());
    let control_routes = controls::add_control_routes(Router::new());
    let control_preset_routes = control_presets::add_control_preset_routes(Router::new());
    let device_routes = devices::add_device_routes(Router::new());
    let monitor_preset_routes = monitor_presets::add_monitor_preset_routes(Router::new());
    let montage_layout_routes = montage_layouts::add_montage_layout_routes(Router::new());
    let snapshot_routes = snapshots::add_snapshot_routes(Router::new());
    let tag_routes = tags::add_tag_routes(Router::new());
    let trigger_x10_routes = triggers_x10::add_trigger_x10_routes(Router::new());
    let user_preference_routes = user_preferences::add_user_preference_routes(Router::new());
    let session_routes = sessions::add_session_routes(Router::new());
    let state_routes = states::add_state_routes(Router::new());
    let stat_routes = stats::add_stat_routes(Router::new());
    let frame_routes = frames::add_frames_routes(Router::new());
    let monitor_status_routes = monitor_status::add_monitor_status_routes(Router::new());
    let object_type_routes = object_types::add_object_type_routes(Router::new());
    let server_stat_routes = server_stats::add_server_stat_routes(Router::new());
    let report_routes = reports::add_report_routes(Router::new());
    let group_monitor_routes = groups_monitors::add_group_monitor_routes(Router::new());
    let group_permission_routes = groups_permissions::add_group_permission_routes(Router::new());
    let monitor_permission_routes = monitors_permissions::add_monitor_permission_routes(Router::new());
    let snapshot_event_routes = snapshots_events::add_snapshot_event_routes(Router::new());
    let event_data_routes = event_data::add_event_data_routes(Router::new());
    let event_tag_routes = events_tags::add_event_tag_routes(Router::new());

    Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        )
        .merge(server_routes)
        .merge(auth_routes)
        .merge(monitors_routes)
        .merge(streaming_routes)
        .merge(events_routes) // Merge events routes
        .merge(mse_routes) // Merge MSE routes
        .merge(webrtc_routes) // Merge WebRTC routes
        .merge(config_routes)
        .merge(zone_routes)
        .merge(filter_routes)
        .merge(user_routes)
        .merge(group_routes)
        .merge(server_info_routes)
        .merge(log_routes)
        .merge(storage_routes)
        .merge(manufacturer_routes)
        .merge(model_routes)
        .merge(zone_preset_routes)
        .merge(control_routes)
        .merge(control_preset_routes)
        .merge(device_routes)
        .merge(monitor_preset_routes)
        .merge(montage_layout_routes)
        .merge(snapshot_routes)
        .merge(tag_routes)
        .merge(trigger_x10_routes)
        .merge(user_preference_routes)
        .merge(session_routes)
        .merge(state_routes)
        .merge(stat_routes)
        .merge(frame_routes)
        .merge(monitor_status_routes)
        .merge(object_type_routes)
        .merge(server_stat_routes)
        .merge(report_routes)
        .merge(group_monitor_routes)
        .merge(group_permission_routes)
        .merge(monitor_permission_routes)
        .merge(snapshot_event_routes)
        .merge(event_data_routes)
        .merge(event_tag_routes)
        .fallback(any(fallback_handler))
        .layer(cors)  // Apply CORS middleware to all routes
        .with_state(state)
}
