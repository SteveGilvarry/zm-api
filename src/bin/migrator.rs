//! SeaORM migration runner CLI.
//!
//! The seed of the future `zm-api db` subcommand (issue #11): runs the
//! migration set (baseline + incremental) against the database given by
//! `-u <url>` or `DATABASE_URL`.
//!
//! ```text
//! cargo run --bin migrator -- up -u mysql://root:pass@127.0.0.1:3307/somedb
//! cargo run --bin migrator -- status
//! ```

#[tokio::main]
async fn main() {
    sea_orm_migration::cli::run_cli(zm_api::migration::Migrator).await;
}
