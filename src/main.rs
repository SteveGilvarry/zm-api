use futures::FutureExt;
use tracing::info;
use zm_api::constant::CONFIG;
use zm_api::error::AppResult;
use zm_api::server::AppServer;
use zm_api::{configure, util};

const USAGE: &str = "\
zm_api — REST API server for a ZoneMinder installation

Usage: zm_api [OPTIONS]

Options:
  -h, --help       Print this help and exit
  -V, --version    Print the version and exit
      --openapi    Write the OpenAPI 3 spec to stdout and exit

The server takes no other arguments; it is configured entirely from files and
environment variables, loaded in this order (last wins):

  1. ZoneMinder's /etc/zm/zm.conf (database settings only)
  2. <config dir>/base.toml
  3. <config dir>/{APP_PROFILE}.toml
  4. APP_* environment variables

The config dir is $APP_CONFIG_DIR, else ./settings. Nested keys use a double
underscore: APP_DB__HOST=10.0.0.5 sets db.host.

Key variables:
  APP_PROFILE                  dev | test | test-db | prod   (default: dev)
  APP_CONFIG_DIR               config directory              (packaged: /etc/zm_api)
  APP_STATIC_DIR               static assets                 (packaged: /usr/share/zm_api/static)
  APP_SERVER__ALLOWED_ORIGINS  CORS origins for a browser dashboard
  APP_DAEMON__ENABLED          false = passive (REST only), true = supervise ZM daemons
  RUST_LOG                     log filter                    (e.g. info, zm_api=debug)

Docs: https://github.com/SteveGilvarry/zm-api
";

/// Handle `--help`/`--version` before anything else initialises. Returns true
/// when the process should exit — deliberately ahead of config loading and
/// tracing setup, so both work on a host whose config is broken.
fn handle_cli_args() -> bool {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => false,
        Some("-h" | "--help") => {
            print!("{USAGE}");
            true
        }
        Some("-V" | "--version") => {
            println!("zm_api {}", env!("CARGO_PKG_VERSION"));
            true
        }
        // Lets the spec be diffed between releases, attached to a release, and
        // fed to a client generator without standing a server up.
        Some("--openapi") => {
            use utoipa::OpenApi;
            match zm_api::handlers::openapi::ApiDoc::openapi().to_pretty_json() {
                Ok(json) => println!("{json}"),
                Err(e) => {
                    eprintln!("zm_api: could not serialise the OpenAPI spec: {e}");
                    std::process::exit(1);
                }
            }
            true
        }
        Some(other) => {
            eprintln!("zm_api: unrecognised argument '{other}'\n");
            eprint!("{USAGE}");
            std::process::exit(2);
        }
    }
}

#[tokio::main]
#[allow(clippy::result_large_err)]
async fn main() -> AppResult<()> {
    if handle_cli_args() {
        return Ok(());
    }

    // Install the rustls CryptoProvider before any TLS/DTLS usage.
    // Required by rustls 0.23+ (used by webrtc-rs DTLS, axum-server TLS, sqlx).
    zm_api::install_crypto_provider();

    // Initialize the ffmpeg libraries once at startup (idempotent, thread-safe).
    // Snapshot/VOD decode paths rely on registered codecs/demuxers; doing it
    // here means production requests never race first-use registration.
    // REVIEW_FIXES_PLAN §5.2.
    ffmpeg_next::init().ok();

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
