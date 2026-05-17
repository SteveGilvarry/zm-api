use std::time::Duration;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tracing::info;

use crate::configure::AppConfig;
use crate::error::AppResult;
use crate::util;

pub type DatabaseClient = DatabaseConnection;

pub trait DatabaseClientExt: Sized {
    fn build_from_config(config: &AppConfig) -> impl std::future::Future<Output = AppResult<Self>>;
}

impl DatabaseClientExt for DatabaseClient {
    async fn build_from_config(config: &AppConfig) -> AppResult<Self> {
        let db_cfg = &config.db;
        let mut opt = ConnectOptions::new(db_cfg.get_url());
        opt.max_connections(db_cfg.max_connections)
            .min_connections(db_cfg.min_connections)
            .connect_timeout(Duration::from_secs(db_cfg.connect_timeout_secs))
            .acquire_timeout(Duration::from_secs(db_cfg.acquire_timeout_secs))
            .idle_timeout(Duration::from_secs(db_cfg.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(db_cfg.max_lifetime_secs))
            // Per-statement logging is off unless explicitly enabled — see the
            // perf note on `DatabaseConfig::sqlx_logging`.
            .sqlx_logging(db_cfg.sqlx_logging)
            .sqlx_logging_level(log::LevelFilter::Debug);
        let db = Database::connect(opt).await?;
        Ok(db)
    }
}

async fn create_database(db: &DatabaseConnection, database_name: &str) -> AppResult {
    db.execute_unprepared(&format!("CREATE DATABASE {database_name}"))
        .await?;
    tracing::info!("Create new database: {database_name}.");
    Ok(())
}

pub async fn setup_new_database(config: &mut AppConfig) -> AppResult<DatabaseClient> {
    info!("Setup new database for the test.");
    let db = DatabaseClient::build_from_config(config).await?;
    config.db.database_name =
        util::random::generate_random_string_with_prefix("test_db").to_lowercase();
    create_database(&db, &config.db.database_name).await?;
    Ok(db)
}

pub async fn drop_database(db: &DatabaseConnection, database_name: &str) -> AppResult {
    let drop_query = format!("DROP DATABASE {database_name} WITH (FORCE);");
    db.execute_unprepared(&drop_query).await?;
    info!("Drop database: {database_name}.");
    Ok(())
}

pub async fn migrate_database(db: &DatabaseConnection) -> AppResult {
    info!("Start migrate database.");
    crate::migration::Migrator::up(db, None).await?;
    info!("Migrate database successfully done.");
    Ok(())
}
