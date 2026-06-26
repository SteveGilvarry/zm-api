use std::sync::Arc;
use std::time::Duration;

use sea_orm::DatabaseConnection;

use crate::client::{
    database::{DatabaseClient, DatabaseClientExt},
    http::HttpClient,
};
use crate::configure::{self, env::get_env_source, AppConfig};
use crate::constant::ENV_PREFIX;
use crate::daemon::DaemonManager;
use crate::error::AppResult;
use crate::ptz::PtzManager;
use crate::service::search::SearchService;
use crate::service::synopsis::SynopsisService;
use crate::streaming::hls::HlsSessionManager;
use crate::streaming::live::LiveStreamCoordinator;
use crate::streaming::snapshot::SnapshotService;
use crate::streaming::source::SourceRouter;
use crate::streaming::webrtc::{session::SessionManager, WebRtcEngine};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Arc<DatabaseClient>,
    pub http: HttpClient,
    // Native WebRTC (Phase 2)
    pub native_webrtc_engine: Option<Arc<WebRtcEngine>>,
    pub native_session_manager: Option<Arc<SessionManager>>,
    // HLS Streaming (Phase 3)
    pub hls_session_manager: Option<Arc<HlsSessionManager>>,
    // Live Streaming Coordinator
    pub source_router: Option<Arc<SourceRouter>>,
    pub live_coordinator: Option<Arc<LiveStreamCoordinator>>,
    // Daemon Controller
    pub daemon_manager: Option<Arc<DaemonManager>>,
    // Snapshot Service
    pub snapshot_service: Option<Arc<SnapshotService>>,
    // Motion-synopsis renderer/serving
    pub synopsis_service: Option<Arc<SynopsisService>>,
    // Natural-language / semantic event search
    pub search_service: Option<Arc<SearchService>>,
    // PTZ Manager
    pub ptz_manager: Arc<PtzManager>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        let db = Arc::new(DatabaseClient::build_from_config(&config).await?);
        let http = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("http client");

        // Initialize native WebRTC engine (Phase 2)
        let native_webrtc_engine = match WebRtcEngine::new(Default::default()) {
            Ok(engine) => {
                tracing::info!("Native WebRTC engine initialized successfully");
                Some(Arc::new(engine))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize native WebRTC engine: {}", e);
                None
            }
        };

        // Initialize native session manager with default max sessions
        let native_session_manager = Some(Arc::new(SessionManager::new(100)));

        // Initialize HLS session manager (Phase 3)
        let hls_session_manager = if config.streaming.hls.enabled {
            tracing::info!("HLS streaming enabled, initializing session manager");
            let manager = Arc::new(HlsSessionManager::new(
                config.streaming.hls.clone(),
                "/api/v3/live", // Changed to use /api/v3/live/ prefix
            ));
            // Start the background segment-cleanup loop (previously never
            // started — its handle was dropped on the floor). See §3.1.
            manager.start_cleanup_task();
            Some(manager)
        } else {
            tracing::info!("HLS streaming disabled in configuration");
            None
        };

        // Initialize source router and live coordinator
        let (source_router, live_coordinator) = if config.streaming.enabled {
            tracing::info!("Live streaming enabled, initializing source router and coordinator");
            let mut router =
                SourceRouter::from_zoneminder_config(config.streaming.zoneminder.clone());

            // zm-next event ingest: the router forwards decoded monitor EVENTs
            // to a bounded channel drained by a background ingest task that
            // writes Events/Frames rows. Only wired when zm-next is enabled.
            if config.zmnext.enabled {
                let (event_tx, event_rx) =
                    tokio::sync::mpsc::channel(config.zmnext.ingest.channel_capacity);
                router.set_event_sink(event_tx);
                let ingestor = crate::service::zmnext::EventIngestor::new(
                    db.clone(),
                    config.zmnext.ingest.clone(),
                    config.synopsis.clone(),
                );
                tokio::spawn(ingestor.run(event_rx));
                tracing::info!("zm-next event ingest enabled");
            }

            let router = Arc::new(router);

            // Keep configured monitors' readers hot so the first viewer skips
            // cold spin-up (pre-populated keyframe cache → instant codec + a
            // ready keyframe). No-op when the list is empty.
            router.spawn_prewarm_task(
                config.streaming.source.prewarm_monitors.clone(),
                Duration::from_secs(config.streaming.source.prewarm_interval_seconds),
            );

            let coordinator = Arc::new(LiveStreamCoordinator::new(
                Arc::clone(&router),
                hls_session_manager.clone(),
            ));
            // Reap HLS sessions abandoned by viewers who navigated away, so the
            // socket reader + segmenter don't run forever (§3.2). Disabled when
            // idle_timeout_seconds is 0.
            if let Some(hls) = &hls_session_manager {
                coordinator.start_idle_watchdog(hls.idle_timeout(), Duration::from_secs(15));
            }
            (Some(router), Some(coordinator))
        } else {
            tracing::info!("Live streaming disabled in configuration");
            (None, None)
        };

        // Initialize snapshot service (reuses source router)
        let snapshot_service = source_router
            .as_ref()
            .map(|r| Arc::new(SnapshotService::with_defaults(Arc::clone(r))));

        // Initialize the motion-synopsis service. Always constructed (it only
        // needs the db + config) so the endpoints can report a clear "disabled"
        // status rather than 404 when `[synopsis].enabled` is false.
        let synopsis_service = Some(Arc::new(SynopsisService::new(
            db.clone(),
            config.synopsis.clone(),
        )));
        // Hourly retention sweep of expired rendered synopses (only when enabled
        // with a finite retention window).
        if config.synopsis.enabled && config.synopsis.retention_days > 0 {
            if let Some(svc) = &synopsis_service {
                Arc::clone(svc).spawn_retention_task(Duration::from_secs(3600));
                tracing::info!(
                    "synopsis retention enabled ({} day window)",
                    config.synopsis.retention_days
                );
            }
        }

        // Natural-language / semantic event search. Resolves its vector backend
        // (probing the DB only when enabled) and creates its schema; off by
        // default → a no-op store.
        let search_service = Some(Arc::new(
            SearchService::new(db.clone(), config.search.clone()).await,
        ));

        // Initialize daemon manager if enabled
        let daemon_manager = if config.daemon.enabled {
            tracing::info!("Daemon controller enabled, initializing manager");
            let mut manager = DaemonManager::with_database(
                config.daemon.clone(),
                None, // Server ID can be set from DB config later
                db.clone(),
            );
            // Enable zm-next worker control (no-op unless [zmnext].enabled).
            // Synopsis-opted-in monitors get the extra export stages in their
            // generated pipeline.
            let synopsis_monitors: std::collections::HashSet<u32> = if config.synopsis.enabled {
                config.synopsis.enabled_monitors.iter().copied().collect()
            } else {
                std::collections::HashSet::new()
            };
            manager.set_zmnext(
                config.zmnext.clone(),
                config.streaming.zoneminder.socks_path.clone(),
                synopsis_monitors,
            );
            Some(Arc::new(manager))
        } else {
            tracing::info!("Daemon controller disabled in configuration");
            None
        };

        // Initialize PTZ manager
        let ptz_manager = Arc::new(PtzManager::with_defaults());
        tracing::info!("PTZ manager initialized");

        Ok(Self {
            config: Arc::new(config),
            db,
            http,
            native_webrtc_engine,
            native_session_manager,
            hls_session_manager,
            source_router,
            live_coordinator,
            snapshot_service,
            synopsis_service,
            search_service,
            daemon_manager,
            ptz_manager,
        })
    }

    /// Returns a reference to the DatabaseConnection for use with Sea-ORM.
    /// This handles dereferencing the Arc<DatabaseClient> automatically.
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn for_test_with_db(db: DatabaseConnection) -> Self {
        let config =
            configure::AppConfig::read(get_env_source(ENV_PREFIX)).expect("read config for test");
        let http = crate::client::http::HttpClient::builder()
            .no_proxy()
            .build()
            .expect("http client");
        let db = std::sync::Arc::new(db);
        let synopsis_service = Some(std::sync::Arc::new(SynopsisService::new(
            db.clone(),
            config.synopsis.clone(),
        )));
        let search_service = Some(std::sync::Arc::new(SearchService::disabled(
            config.search.clone(),
        )));
        Self {
            config: std::sync::Arc::new(config),
            db,
            http,
            native_webrtc_engine: None,
            native_session_manager: None,
            hls_session_manager: None,
            source_router: None,
            live_coordinator: None,
            snapshot_service: None,
            synopsis_service,
            search_service,
            daemon_manager: None,
            ptz_manager: std::sync::Arc::new(PtzManager::with_defaults()),
        }
    }

    /// Returns a reference to the PTZ manager
    pub fn ptz_manager(&self) -> &PtzManager {
        &self.ptz_manager
    }

    /// Spawn one ONVIF PullPoint event listener per monitor that has the ONVIF
    /// event listener enabled (`onvif_event_listener != 0`) and a usable ONVIF
    /// URL. Each listener subscribes, long-polls `PullMessages`, and translates
    /// alarm on/off notifications into ZoneMinder events, quiescing with the
    /// daemon manager on shutdown. No-op when the daemon manager is disabled.
    #[cfg(feature = "onvif-events")]
    pub async fn spawn_onvif_event_listeners(&self) {
        let Some(manager) = self.daemon_manager.clone() else {
            return;
        };
        let monitors = match crate::repo::monitors::find_all(self.db(), None).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("ONVIF event listeners: failed to load monitors: {e}");
                return;
            }
        };

        let mut spawned = 0usize;
        for m in monitors {
            if m.onvif_event_listener == 0 || m.onvif_url.trim().is_empty() {
                continue;
            }
            // Events service endpoint = onvif_url joined with onvif_events_path.
            let events_xaddr = join_onvif_url(&m.onvif_url, &m.onvif_events_path);
            let creds = (!m.onvif_username.is_empty()).then(|| {
                crate::onvif::types::Credentials::new(
                    m.onvif_username.clone(),
                    m.onvif_password.clone(),
                )
            });
            let transport = crate::onvif::transport::OnvifTransport::new(self.http.clone());
            let client = crate::onvif::events::EventsClient::new(transport, events_xaddr, creds);
            let alarm_cause = m
                .onvif_alarm_text
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "ONVIF Alarm".to_string());

            crate::daemon::onvif_event_listener::spawn_monitor_event_listener(
                self.clone(),
                m.id,
                m.name.clone(),
                alarm_cause,
                client,
                manager.clone(),
                crate::daemon::onvif_event_listener::OnvifEventListenerConfig::default(),
            );
            spawned += 1;
        }

        if spawned > 0 {
            tracing::info!("Spawned {spawned} ONVIF event listener(s)");
        }
    }
}

/// Join an ONVIF base URL with an events service path, tolerating an empty path
/// (returns the base) and avoiding duplicate slashes at the join.
#[cfg(feature = "onvif-events")]
fn join_onvif_url(base: &str, path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        return base.to_string();
    }
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}
