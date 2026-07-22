//! SeaORM migration runner CLI.
//!
//! The seed of the future `zm-api db` subcommand (issues #11/#12):
//!
//! ```text
//! # fresh install / normal migrations (sea-orm-migration CLI)
//! cargo run --bin migrator -- up -u mysql://root:pass@127.0.0.1:3307/somedb
//! cargo run --bin migrator -- status
//!
//! # bring an existing ZoneMinder database (>= 1.26.0) into the migration
//! # system: walk the embedded zm_update chain, converge triggers, stamp
//! # the baseline, run remaining migrations
//! cargo run --bin migrator -- bridge -u mysql://root:pass@host:3306/zm
//! ```

use sea_orm::{ConnectOptions, Database};

fn database_url(args: &[String]) -> String {
    for (i, a) in args.iter().enumerate() {
        if (a == "-u" || a == "--database-url") && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("error: pass -u <url> or set DATABASE_URL");
        std::process::exit(2);
    })
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().map(String::as_str) == Some("bridge") {
        tracing_subscriber::fmt().with_target(false).init();
        let url = database_url(&args);
        // Single connection: the legacy chain uses session state
        // (SET @s ... PREPARE stmt FROM @s), which must not hop between
        // pooled connections.
        let mut opts = ConnectOptions::new(&url);
        opts.max_connections(1).sqlx_logging(false);
        let conn = match Database::connect(opts).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: cannot connect: {e}");
                std::process::exit(1);
            }
        };
        if let Err(e) = zm_api::migration::legacy_bridge::run(&conn).await {
            eprintln!("bridge failed: {e}");
            std::process::exit(1);
        }
        println!("bridge complete: database is now on the migration system");
        return;
    }

    sea_orm_migration::cli::run_cli(zm_api::migration::Migrator).await;
}
