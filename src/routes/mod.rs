use crate::handlers::openapi::ApiDoc;
use crate::server::state::AppState;
use crate::util::authz;
use axum::extract::{DefaultBodyLimit, MatchedPath};
use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    routing::any,
    Router,
};
use tower::ServiceBuilder;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod configs; // Config management
pub mod control_presets; // Control Presets
pub mod controls; // Controls
pub mod daemon; // Daemon control
pub mod devices; // Devices
pub mod event_data; // Event Data
pub mod event_summaries; // Event Summaries (pre-calculated counts)
pub mod events; // Add events module
pub mod events_playback; // Event playback (video streaming)
pub mod events_tags; // Events Tags
pub mod filters; // Filters
pub mod frames; // Frames
pub mod groups; // Groups
pub mod groups_monitors; // Groups Monitors
pub mod groups_permissions; // Groups Permissions
pub mod live; // Live streaming (unified)
pub mod logs; // Logs
pub mod manufacturers; // Manufacturers
pub mod models; // Models
pub mod monitor_presets; // Monitor Presets
pub mod monitor_status; // Monitor Status
pub mod monitors;
pub mod monitors_permissions; // Monitors Permissions
pub mod montage_layouts; // Montage Layouts
pub mod object_types; // Object Types
pub mod ptz; // PTZ control
pub mod reports; // Reports
pub mod server;
pub mod server_stats; // Server Stats
pub mod servers; // Server info list
pub mod sessions; // Sessions
pub mod snapshots; // Snapshots
pub mod snapshots_events; // Snapshots Events
pub mod states; // States
pub mod stats; // Stats
pub mod storage; // Storage
pub mod tags; // Tags
pub mod triggers_x10; // X10 Triggers
pub mod user_preferences; // User Preferences
pub mod users; // Users
pub mod zone_presets; // Zone Presets
pub mod zones; // Zones // go2rtc WebSocket proxy

async fn fallback_handler(path: MatchedPath) -> &'static str {
    tracing::error!("Unknown route: {}", path.as_str());
    "Unknown route"
}

/// Build the CORS layer from the `ALLOWED_ORIGINS` environment variable.
fn build_cors_layer() -> CorsLayer {
    // Get frontend URL from environment variable or use default localhost addresses
    let frontend_urls = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
        "http://localhost:3000,http://localhost:5173,http://localhost:8000".to_string()
    });

    // Parse the URLs into a Vec of HeaderValues for CORS configuration
    let origins = frontend_urls
        .split(',')
        .filter_map(|origin| origin.parse().ok())
        .collect::<Vec<_>>();

    tracing::info!("Configuring CORS with allowed origins: {:?}", frontend_urls);

    CorsLayer::new()
        // Allow frontend origins to access the API
        .allow_origin(origins)
        // Allow common HTTP methods needed for a RESTful API
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        // Allow common HTTP headers used in API requests
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("accept"),
            HeaderName::from_static("origin"),
        ])
        .allow_credentials(true)
}

/// Apply the cross-cutting middleware stack to the fully-merged router.
///
/// Outermost → innermost: tracing, security headers, body-size limit,
/// optional per-IP rate limiting, then CORS.
fn apply_common_middleware(router: Router<AppState>, cors: CorsLayer) -> Router<AppState> {
    let cfg = &*crate::constant::CONFIG;
    let mw = &cfg.server.middleware;
    let tls_enabled = cfg.server.tls.as_ref().is_some_and(|t| t.enabled)
        || cfg.server.acme.as_ref().is_some_and(|a| a.enabled);

    // Per-IP rate limiting — opt-in via config (`rate_limit_per_second = 0`
    // disables it). `SmartIpKeyExtractor` reads X-Forwarded-For / X-Real-IP,
    // so it works behind a reverse proxy.
    let rate_limit_layer = if mw.rate_limiting_enabled() {
        match GovernorConfigBuilder::default()
            .per_second(mw.rate_limit_per_second)
            .burst_size(mw.rate_limit_burst.max(1))
            .key_extractor(SmartIpKeyExtractor)
            .finish()
        {
            Some(conf) => {
                tracing::info!(
                    "Rate limiting enabled: {} req/s per IP, burst {}",
                    mw.rate_limit_per_second,
                    mw.rate_limit_burst.max(1),
                );
                Some(GovernorLayer::new(conf))
            }
            None => {
                tracing::warn!("Invalid rate-limit configuration; rate limiting disabled");
                None
            }
        }
    } else {
        None
    };

    // HSTS only makes sense when the service itself terminates TLS.
    let hsts_layer = tls_enabled.then(|| {
        SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        )
    });

    let stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        ))
        .option_layer(hsts_layer)
        .layer(DefaultBodyLimit::max(mw.body_limit_bytes))
        .option_layer(rate_limit_layer)
        .layer(cors);

    router.layer(stack)
}

pub fn create_router_app(state: AppState) -> Router {
    let cors = build_cors_layer();

    // `auth` and `server` expose a mix of public and protected endpoints
    // (login, health check, version) and manage their own auth per-route, so
    // they are deliberately not wrapped with a blanket RBAC feature gate.
    let server_routes = server::add_server_routes(Router::new());
    let auth_routes = auth::add_routers(Router::new());

    // Every other router is gated on the ZoneMinder permission feature it
    // belongs to. `authz::protect` derives the required level from the HTTP
    // method (read -> View, write -> Edit).
    use authz::Feature;
    let protect = authz::protect;

    let monitors_routes = protect(
        monitors::add_monitor_routes(Router::new()),
        Feature::Monitors,
    );
    let monitor_preset_routes = protect(
        monitor_presets::add_monitor_preset_routes(Router::new()),
        Feature::Monitors,
    );
    let monitor_status_routes = protect(
        monitor_status::add_monitor_status_routes(Router::new()),
        Feature::Monitors,
    );
    // Permission CRUD is administrative: a Groups:Edit / Monitors:Edit user
    // would otherwise be able to grant themselves or others elevated row-level
    // access via this endpoint. Require System (admin-tier) instead.
    let monitor_permission_routes = protect(
        monitors_permissions::add_monitor_permission_routes(Router::new()),
        Feature::System,
    );
    let zone_routes = protect(zones::add_zone_routes(Router::new()), Feature::Monitors);
    let zone_preset_routes = protect(
        zone_presets::add_zone_preset_routes(Router::new()),
        Feature::Monitors,
    );

    let events_routes = protect(events::add_event_routes(Router::new()), Feature::Events);
    let event_data_routes = protect(
        event_data::add_event_data_routes(Router::new()),
        Feature::Events,
    );
    let event_summary_routes = protect(
        event_summaries::add_event_summaries_routes(Router::new()),
        Feature::Events,
    );
    let event_tag_routes = protect(
        events_tags::add_event_tag_routes(Router::new()),
        Feature::Events,
    );
    let frame_routes = protect(frames::add_frames_routes(Router::new()), Feature::Events);
    let filter_routes = protect(filters::add_filter_routes(Router::new()), Feature::Events);
    let tag_routes = protect(tags::add_tag_routes(Router::new()), Feature::Events);
    let object_type_routes = protect(
        object_types::add_object_type_routes(Router::new()),
        Feature::Events,
    );
    let events_playback_routes = protect(
        events_playback::add_events_playback_routes(Router::new()),
        Feature::Events,
    );

    let control_routes = protect(
        controls::add_control_routes(Router::new()),
        Feature::Control,
    );
    let control_preset_routes = protect(
        control_presets::add_control_preset_routes(Router::new()),
        Feature::Control,
    );
    // PTZ acts on a monitor named in the path (`{id}`); guard it row-level.
    // Order matters: `protect` must wrap *outside* the row-level guard so the
    // feature-level RBAC check runs first and the guard's DB query is only
    // reached after the caller has at least `Control:View`.
    let ptz_routes = protect(
        ptz::add_ptz_routes(Router::new()).route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::service::monitor_acl::monitor_path_guard,
        )),
        Feature::Control,
    );
    let trigger_x10_routes = protect(
        triggers_x10::add_trigger_x10_routes(Router::new()),
        Feature::Control,
    );

    let group_routes = protect(groups::add_group_routes(Router::new()), Feature::Groups);
    let group_monitor_routes = protect(
        groups_monitors::add_group_monitor_routes(Router::new()),
        Feature::Groups,
    );
    // Permission CRUD is administrative — see `monitor_permission_routes` above.
    let group_permission_routes = protect(
        groups_permissions::add_group_permission_routes(Router::new()),
        Feature::System,
    );

    let device_routes = protect(devices::add_device_routes(Router::new()), Feature::Devices);
    let manufacturer_routes = protect(
        manufacturers::add_manufacturer_routes(Router::new()),
        Feature::Devices,
    );
    let model_routes = protect(models::add_model_routes(Router::new()), Feature::Devices);

    let snapshot_routes = protect(
        snapshots::add_snapshot_routes(Router::new()),
        Feature::Snapshots,
    );
    let snapshot_event_routes = protect(
        snapshots_events::add_snapshot_event_routes(Router::new()),
        Feature::Snapshots,
    );

    // Live streaming serves a monitor named in the path (`{monitor_id}`);
    // guard it row-level. `/live/sessions` and `/live/sources` have no path
    // monitor id, so the guard passes them through.
    //
    // Order matters: `protect` must wrap *outside* the row-level guard so the
    // feature-level RBAC check runs first and the guard's DB query is only
    // reached after the caller has at least `Stream:View`.
    let live_routes = protect(
        live::add_live_routes(Router::new()).route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::service::monitor_acl::monitor_path_guard,
        )),
        Feature::Stream,
    );

    let config_routes = protect(configs::add_config_routes(Router::new()), Feature::System);
    let log_routes = protect(logs::add_log_routes(Router::new()), Feature::System);
    let storage_routes = protect(storage::add_storage_routes(Router::new()), Feature::System);
    let server_info_routes = protect(
        servers::add_server_info_routes(Router::new()),
        Feature::System,
    );
    let server_stat_routes = protect(
        server_stats::add_server_stat_routes(Router::new()),
        Feature::System,
    );
    let stat_routes = protect(stats::add_stat_routes(Router::new()), Feature::System);
    let state_routes = protect(states::add_state_routes(Router::new()), Feature::System);
    let report_routes = protect(reports::add_report_routes(Router::new()), Feature::System);
    let daemon_routes = protect(daemon::add_daemon_routes(Router::new()), Feature::System);
    let user_routes = protect(users::add_user_routes(Router::new()), Feature::System);
    let user_preference_routes = protect(
        user_preferences::add_user_preference_routes(Router::new()),
        Feature::System,
    );
    let session_routes = protect(sessions::add_session_routes(Router::new()), Feature::System);
    let montage_layout_routes = protect(
        montage_layouts::add_montage_layout_routes(Router::new()),
        Feature::System,
    );

    // Streaming endpoints must bypass response compression: they serve
    // byte-range video, chunked HLS playlists, JPEG snapshots, SSE and
    // WebSocket upgrades, all of which `CompressionLayer` would buffer,
    // invalidate (Range) or corrupt.
    let streaming = Router::new()
        .merge(live_routes) // Live streaming (unified)
        .merge(events_playback_routes) // Event playback
        .merge(snapshot_routes)
        .merge(snapshot_event_routes);

    // Regular JSON API endpoints — safe to gzip/brotli compress.
    let api = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(server_routes)
        .merge(auth_routes)
        .merge(monitors_routes)
        .merge(events_routes)
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
        .merge(event_data_routes)
        .merge(event_summary_routes)
        .merge(event_tag_routes)
        .merge(daemon_routes) // Daemon control
        .merge(ptz_routes) // PTZ control
        .layer(CompressionLayer::new());

    let app = Router::new()
        .merge(api)
        .merge(streaming)
        .fallback(any(fallback_handler));

    apply_common_middleware(app, cors).with_state(state)
}
