use std::sync::Arc;

use tokio::sync::Notify;
use sea_orm::DatabaseConnection;

use crate::client::{
  database::{DatabaseClient, DatabaseClientExt},
  email::EmailClient,
  http::HttpClient,
  redis::RedisClient,
  webrtc_signaling::WebRtcSignalingClient,
  ClientBuilder,
};
use crate::configure::AppConfig;
use crate::error::AppResult;
use crate::mse_client::MseStreamManager;

#[derive(Clone)]
pub struct AppState {
  pub config: Arc<AppConfig>,
  pub redis: Arc<RedisClient>,
  pub db: Arc<DatabaseClient>,
  pub email: Arc<EmailClient>,
  pub messenger_notify: Arc<Notify>,
  pub http: HttpClient,
  pub webrtc_client: WebRtcSignalingClient,
  pub mse_manager: Arc<MseStreamManager>,
}

impl AppState {
  pub async fn new(config: AppConfig) -> AppResult<Self> {
    let redis = Arc::new(RedisClient::build_from_config(&config)?);
    let email = Arc::new(EmailClient::build_from_config(&config)?);
    let db = Arc::new(DatabaseClient::build_from_config(&config).await?);
    let http = HttpClient::build_from_config(&config)?;
    
    // Initialize WebRTC signaling client
    let webrtc_client = WebRtcSignalingClient::new("127.0.0.1:9050".to_string());
    
    // Initialize MSE stream manager
    let mse_manager = Arc::new(MseStreamManager::new());
    
    Ok(Self {
      config: Arc::new(config),
      db,
      redis,
      email,
      messenger_notify: Default::default(),
      http,
      webrtc_client,
      mse_manager,
    })
  }
  
  /// Returns a reference to the DatabaseConnection for use with Sea-ORM.
  /// This handles dereferencing the Arc<DatabaseClient> automatically.
  pub fn db(&self) -> &DatabaseConnection {
    &*self.db
  }
  
  /// Returns a reference to the MSE stream manager
  pub fn mse_manager(&self) -> &Arc<MseStreamManager> {
    &self.mse_manager
  }
}
