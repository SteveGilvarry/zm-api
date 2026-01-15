use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::client::{
    database::{DatabaseClient, DatabaseClientExt},
    email::EmailClient,
    http::HttpClient,
    webrtc_signaling::WebRtcSignalingClient,
    ClientBuilder,
};
use crate::configure::{self, env::get_env_source, AppConfig};
use crate::constant::ENV_PREFIX;
use crate::daemon::DaemonManager;
use crate::error::AppResult;
use crate::mse_client::MseStreamManager;
use crate::streaming::hls::HlsSessionManager;
use crate::streaming::webrtc::{session::SessionManager, WebRtcEngine};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Arc<DatabaseClient>,
    pub email: Arc<EmailClient>,
    pub http: HttpClient,
    pub webrtc_client: WebRtcSignalingClient,
    pub mse_manager: Arc<MseStreamManager>,
    // Native WebRTC (Phase 2)
    pub native_webrtc_engine: Option<Arc<WebRtcEngine>>,
    pub native_session_manager: Option<Arc<SessionManager>>,
    // HLS Streaming (Phase 3)
    pub hls_session_manager: Option<Arc<HlsSessionManager>>,
    // Daemon Controller
    pub daemon_manager: Option<Arc<DaemonManager>>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        let email = Arc::new(EmailClient::build_from_config(&config)?);
        let db = Arc::new(DatabaseClient::build_from_config(&config).await?);
        let http = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("http client");

        // Initialize WebRTC signaling client
        let webrtc_client = WebRtcSignalingClient::new("127.0.0.1:9050".to_string());

        // Initialize MSE stream manager
        let mse_manager = Arc::new(MseStreamManager::new());

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
            Some(Arc::new(HlsSessionManager::new(
                config.streaming.hls.clone(),
                "/api/v3/hls",
            )))
        } else {
            tracing::info!("HLS streaming disabled in configuration");
            None
        };

        // Initialize daemon manager if enabled
        let daemon_manager = if config.daemon.enabled {
            tracing::info!("Daemon controller enabled, initializing manager");
            Some(Arc::new(DaemonManager::with_database(
                config.daemon.clone(),
                None, // Server ID can be set from DB config later
                db.clone(),
            )))
        } else {
            tracing::info!("Daemon controller disabled in configuration");
            None
        };

        Ok(Self {
            config: Arc::new(config),
            db,
            email,
            http,
            webrtc_client,
            mse_manager,
            native_webrtc_engine,
            native_session_manager,
            hls_session_manager,
            daemon_manager,
        })
    }

    /// Returns a reference to the DatabaseConnection for use with Sea-ORM.
    /// This handles dereferencing the Arc<DatabaseClient> automatically.
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Returns a reference to the MSE stream manager
    pub fn mse_manager(&self) -> &Arc<MseStreamManager> {
        &self.mse_manager
    }

    pub fn for_test_with_db(db: DatabaseConnection) -> Self {
        let config =
            configure::AppConfig::read(get_env_source(ENV_PREFIX)).expect("read config for test");
        let email = std::sync::Arc::new(EmailClient::builder_dangerous("127.0.0.1").build());
        let http = crate::client::http::HttpClient::builder()
            .no_proxy()
            .build()
            .expect("http client");
        let webrtc_client =
            crate::client::webrtc_signaling::WebRtcSignalingClient::new("127.0.0.1:0".to_string());
        let mse_manager = std::sync::Arc::new(crate::mse_client::MseStreamManager::new());
        Self {
            config: std::sync::Arc::new(config),
            db: std::sync::Arc::new(db),
            email,
            http,
            webrtc_client,
            mse_manager,
            native_webrtc_engine: None,
            native_session_manager: None,
            hls_session_manager: None,
            daemon_manager: None,
        }
    }
}
