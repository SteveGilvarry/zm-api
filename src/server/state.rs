use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::client::{
    database::{DatabaseClient, DatabaseClientExt},
    email::EmailClient,
    http::HttpClient,
    webrtc_signaling::WebRtcSignalingClient,
    ClientBuilder,
};
use crate::configure::AppConfig;
use crate::error::AppResult;
use crate::mse_client::MseStreamManager;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: Arc<DatabaseClient>,
    pub email: Arc<EmailClient>,
    pub http: HttpClient,
    pub webrtc_client: WebRtcSignalingClient,
    pub mse_manager: Arc<MseStreamManager>,
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

        Ok(Self {
            config: Arc::new(config),
            db,
            email,
            http,
            webrtc_client,
            mse_manager,
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
        use crate::client::email::EmailClient;
        use crate::configure::{self, env::get_env_source};
        use crate::constant::ENV_PREFIX;

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
        }
    }
}
