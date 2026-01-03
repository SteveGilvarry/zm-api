use futures::FutureExt;
use tracing::info;
use zm_api::constant::CONFIG;
use zm_api::error::AppResult;
use zm_api::server::AppServer;
use zm_api::{configure, util};

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> AppResult<()> {
    let _file_appender_guard = configure::tracing::init()?;
    info!("The initialization of Tracing was successful.");
    let config = CONFIG.clone();
    info!("Reading the config file was successful.");
    info!("Create a new server.");
    let server = AppServer::new(config).await?;
    info!("Run the server.");
    util::task::join_all(vec![(true, server.run().boxed())]).await?;
    Ok(())
}
