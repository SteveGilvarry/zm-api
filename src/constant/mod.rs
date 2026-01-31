use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::Lazy;
use std::{path::PathBuf, time::Duration};
use utoipa::OpenApi;

use crate::{
    client::{http::HttpClient, ClientBuilder},
    configure::{env::get_env_source, get_static_dir},
    handlers::openapi::ApiDoc,
};

pub const API_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const API_NAME: &str = env!("CARGO_PKG_NAME");
pub const ENV_PREFIX: &str = "APP";
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(120);
// Token expiry constants
pub const EXPIRE_BEARER_TOKEN_SECS: Duration = Duration::from_secs(600);
pub const EXPIRE_REFRESH_TOKEN_SECS: Duration = Duration::from_secs(3600);
pub const AUTHORIZATION: &str = "Authorization";
pub const BEARER: &str = "Bearer";
pub static IMAGES_PATH: Lazy<PathBuf> = Lazy::new(|| get_static_dir().unwrap().join("images"));
pub static APP_IMAGE: Lazy<PathBuf> =
    Lazy::new(|| get_static_dir().unwrap().join("images/logo.jpg"));
pub static CONFIG: Lazy<crate::configure::AppConfig> =
    Lazy::new(|| crate::configure::AppConfig::read(get_env_source(ENV_PREFIX)).unwrap());
pub static HTTP: Lazy<reqwest::Client> =
    Lazy::new(|| HttpClient::build_from_config(&CONFIG).unwrap());
pub const MAX_RETRY: u32 = 10;
pub const MINIMUM_DELAY_TIME: std::time::Duration = std::time::Duration::from_millis(100);
pub static REFRESH_TOKEN_ENCODE_KEY: Lazy<EncodingKey> = Lazy::new(|| {
    let key = CONFIG.secret.read_private_refresh_key().unwrap();
    EncodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});
pub static REFRESH_TOKEN_DECODE_KEY: Lazy<DecodingKey> = Lazy::new(|| {
    let key = CONFIG.secret.read_public_refresh_key().unwrap();
    DecodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});
pub static ACCESS_TOKEN_ENCODE_KEY: Lazy<EncodingKey> = Lazy::new(|| {
    let key = CONFIG.secret.read_private_access_key().unwrap();
    EncodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});
pub static ACCESS_TOKEN_DECODE_KEY: Lazy<DecodingKey> = Lazy::new(|| {
    let key = CONFIG.secret.read_public_access_key().unwrap();
    DecodingKey::from_rsa_pem(key.as_bytes()).unwrap()
});
pub static API_DOC: Lazy<utoipa::openapi::OpenApi> = Lazy::new(ApiDoc::openapi);
