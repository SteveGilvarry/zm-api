use sentry::ClientInitGuard;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SentryConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    dsn: String,
}

/// Initialize Sentry error reporting if enabled and DSN is configured.
/// Returns None if Sentry is disabled or DSN is empty.
pub fn init(config: &SentryConfig) -> Option<ClientInitGuard> {
    if !config.enabled || config.dsn.is_empty() {
        tracing::info!("Sentry error reporting is disabled");
        return None;
    }

    tracing::info!("Initializing Sentry error reporting");
    Some(sentry::init((
        config.dsn.clone(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    )))
}
