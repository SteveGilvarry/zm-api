#![allow(clippy::result_large_err)]
use std::path::PathBuf;

use tracing::{subscriber, Subscriber};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

use crate::error::AppResult;

/// Get the log directory from environment or use default.
///
/// Reads from `APP_LOG_DIR` environment variable. If not set:
/// - Returns `/var/log/zm_api` if it exists (production)
/// - Otherwise returns `./logs` (development)
pub fn get_log_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("APP_LOG_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    // Default to /var/log/zm_api for production if it exists
    let prod_path = PathBuf::from("/var/log/zm_api");
    if prod_path.exists() {
        return prod_path;
    }
    // Fall back to relative path for development
    PathBuf::from("logs")
}

fn create_subscriber<W>(
    name: &str,
    env_filter: EnvFilter,
    writer: W,
) -> impl Subscriber + Sync + Send
where
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(BunyanFormattingLayer::new(name.into(), std::io::stdout))
        .with(BunyanFormattingLayer::new(name.into(), writer))
}

pub fn init_subscriber<S>(subscriber: S) -> anyhow::Result<()>
where
    S: Subscriber + Send + Sync + 'static,
{
    LogTracer::init()?;
    subscriber::set_global_default(subscriber)?;
    Ok(())
}

pub fn init() -> AppResult<WorkerGuard> {
    let log_dir = get_log_dir();
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "zm_api.log");
    let (file_appender, file_appender_guard) = tracing_appender::non_blocking(file_appender);
    init_subscriber(create_subscriber(
        "zm_api",
        EnvFilter::from_default_env(),
        file_appender,
    ))?;
    Ok(file_appender_guard)
}
