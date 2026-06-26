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
use tower_http::cors::{AllowOrigin, CorsLayer};
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
#[cfg(feature = "onvif-discovery")]
pub mod discovery; // ONVIF camera discovery
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
pub mod search; // Natural-language / semantic event search
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

/// One allowed-origin rule parsed from `ALLOWED_ORIGINS`.
enum OriginRule {
    /// Exact origin match, e.g. `https://app.example.com`.
    Exact(HeaderValue),
    /// A host with any explicit port, written `scheme://host:*` (e.g.
    /// `http://localhost:*`). Matches `scheme://host:<digits>` only. This is
    /// the dev convenience that avoids re-listing every Vite/CRA/preview port,
    /// without resorting to a wildcard (illegal here — see below).
    AnyPort { prefix: String },
}

impl OriginRule {
    fn matches(&self, origin: &HeaderValue) -> bool {
        match self {
            OriginRule::Exact(allowed) => allowed == origin,
            OriginRule::AnyPort { prefix } => {
                let o = origin.as_bytes();
                let p = prefix.as_bytes();
                // `scheme://host:` followed by a non-empty, all-digits port.
                o.len() > p.len() && o.starts_with(p) && o[p.len()..].iter().all(u8::is_ascii_digit)
            }
        }
    }
}

/// Build the CORS layer from the `ALLOWED_ORIGINS` environment variable.
///
/// `ALLOWED_ORIGINS` is a comma-separated list. Each entry is either an exact
/// origin (`https://app.example.com`) or a host with a `:*` port wildcard
/// (`http://localhost:*`) that matches that host on any port. The default
/// (dev) allows any localhost / 127.0.0.1 port so new frontend dev ports don't
/// need to be registered one by one.
///
/// Note: the layer sets `Access-Control-Allow-Credentials: true`, and the CORS
/// spec forbids pairing credentials with a literal `*` origin (browsers reject
/// it). So "all origins" is intentionally not offered; the `:*` rule reflects
/// the specific matching origin back instead, which is credentials-safe and,
/// for `localhost`, does not expose the API to other hosts. In production set
/// `ALLOWED_ORIGINS` to the exact deployed front-end origin(s).
fn build_cors_layer() -> CorsLayer {
    let frontend_urls = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:*,http://127.0.0.1:*".to_string());

    let rules = parse_origin_rules(&frontend_urls);

    tracing::info!("Configuring CORS with allowed origins: {:?}", frontend_urls);

    let allow_origin = AllowOrigin::predicate(move |origin: &HeaderValue, _req| {
        rules.iter().any(|r| r.matches(origin))
    });

    CorsLayer::new()
        // Allow frontend origins to access the API
        .allow_origin(allow_origin)
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

/// Parse `ALLOWED_ORIGINS` into a set of [`OriginRule`]s. Entries ending in
/// `:*` become any-port host rules; the rest are exact origins. Entries that
/// are neither parseable as a header value nor a `:*` rule are skipped.
fn parse_origin_rules(raw: &str) -> Vec<OriginRule> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|entry| {
            if let Some(host) = entry.strip_suffix(":*") {
                // Store the prefix including the colon, e.g. "http://localhost:".
                Some(OriginRule::AnyPort {
                    prefix: format!("{host}:"),
                })
            } else {
                entry.parse::<HeaderValue>().ok().map(OriginRule::Exact)
            }
        })
        .collect()
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
    // Natural-language / semantic event search. JSON (compressible), so it lives
    // in the `api` group rather than the streaming group. Row-level ACL is
    // enforced inside the handlers via `MonitorScope`.
    let search_routes = protect(search::add_search_routes(Router::new()), Feature::Events);

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

    // Build the served OpenAPI document, merging in feature-gated fragments
    // (utoipa's derive cannot `#[cfg]` individual path/schema entries, so the
    // ONVIF discovery fragment is composed here at runtime).
    #[allow(unused_mut)]
    let mut openapi = ApiDoc::openapi();
    #[cfg(feature = "onvif-discovery")]
    openapi.merge(crate::handlers::discovery::DiscoveryApiDoc::openapi());

    // Regular JSON API endpoints — safe to gzip/brotli compress.
    let api = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
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
        .merge(search_routes)
        .merge(daemon_routes) // Daemon control
        .merge(ptz_routes); // PTZ control

    // ONVIF discovery (feature-gated). Feeds monitor creation, so gated like
    // monitor management; merged before the compression layer so its JSON
    // responses are compressed like the rest of the API. The probe/inspect
    // endpoints have no `{monitor_id}` path param, so no row-level
    // `monitor_path_guard` is needed — the handlers' own `scope.is_restricted()`
    // check is the row-level gate.
    #[cfg(feature = "onvif-discovery")]
    let api = api.merge(protect(
        discovery::add_discovery_routes(Router::new()),
        Feature::Monitors,
    ));

    let api = api.layer(CompressionLayer::new());

    let app = Router::new()
        .merge(api)
        .merge(streaming)
        .fallback(any(fallback_handler));

    apply_common_middleware(app, cors).with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hv(s: &str) -> HeaderValue {
        HeaderValue::from_str(s).unwrap()
    }

    fn allows(rules: &[OriginRule], origin: &str) -> bool {
        let o = hv(origin);
        rules.iter().any(|r| r.matches(&o))
    }

    #[test]
    fn any_port_rule_matches_localhost_on_any_port() {
        let rules = parse_origin_rules("http://localhost:*,http://127.0.0.1:*");
        assert!(allows(&rules, "http://localhost:3000"));
        assert!(allows(&rules, "http://localhost:5173"));
        assert!(allows(&rules, "http://localhost:49152")); // ephemeral
        assert!(allows(&rules, "http://127.0.0.1:8080"));
    }

    #[test]
    fn any_port_rule_rejects_lookalikes_and_other_hosts() {
        let rules = parse_origin_rules("http://localhost:*");
        // Non-numeric / empty port must not match (so a forged
        // "localhost:3000.evil.com"-style origin can't slip through).
        assert!(!allows(&rules, "http://localhost:3000.evil.com"));
        assert!(!allows(&rules, "http://localhost:"));
        assert!(!allows(&rules, "http://localhost")); // bare host, no port
                                                      // Different host, and the subdomain-confusion case.
        assert!(!allows(&rules, "http://evil.com:3000"));
        assert!(!allows(&rules, "http://localhost.evil.com:3000"));
        // Scheme must match too.
        assert!(!allows(&rules, "https://localhost:3000"));
    }

    #[test]
    fn exact_rule_matches_only_that_origin() {
        let rules = parse_origin_rules("https://app.example.com,http://localhost:3000");
        assert!(allows(&rules, "https://app.example.com"));
        assert!(allows(&rules, "http://localhost:3000"));
        assert!(!allows(&rules, "https://app.example.com:8443"));
        assert!(!allows(&rules, "http://localhost:3001"));
    }

    #[test]
    fn mixed_exact_and_wildcard_entries() {
        let rules = parse_origin_rules("https://prod.example.com, http://localhost:*");
        assert!(allows(&rules, "https://prod.example.com"));
        assert!(allows(&rules, "http://localhost:1234"));
        assert!(!allows(&rules, "https://staging.example.com"));
    }

    #[test]
    fn blank_and_unparseable_entries_are_skipped() {
        let rules = parse_origin_rules(" , ,http://localhost:*, ");
        assert_eq!(rules.len(), 1);
        assert!(allows(&rules, "http://localhost:9999"));
    }
}
