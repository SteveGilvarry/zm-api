use futures::FutureExt;
use zm_api::constant::CONFIG;
use zm_api::error::AppResult;
//use zm_api::server::worker::MessengerTask;
use zm_api::server::AppServer;
use zm_api::{configure, util};
use tracing::info;


#[tokio::main]
async fn main() -> AppResult<()> {
    let _file_appender_guard = configure::tracing::init()?;
    info!("The initialization of Tracing was successful.");
    let config = CONFIG.clone();
    info!("Reading the config file was successful.");
    info!("Create a new server.");
    let server = AppServer::new(config).await?;
    info!("Create a new messenger task.");
    //let messenger = MessengerTask::new(server.state.clone());
    info!("Run the server.");
    util::task::join_all(vec![
        (true, server.run().boxed()),
       // (true, messenger.run().boxed()),
    ])
        .await?;
    Ok(())
}
