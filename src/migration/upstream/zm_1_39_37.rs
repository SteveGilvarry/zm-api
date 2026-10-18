//! Mirrors `db/legacy/zm_update-1.39.37.sql`: Remote/rtsp monitors become
//! Ffmpeg with the equivalent url, and `Monitors.RTSPDescribe` is dropped
//! with the rtsp client that was its only reader.
//!
//! Upstream builds the url with `CONCAT`/`IF` in one UPDATE; this reads the
//! affected rows and writes each back, which is the same on both backends.
//! Credentials stay in `User`/`Pass`, as upstream leaves them.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// `rtsp://Host[:Port]/Path`, with upstream's rules: an empty Port adds
/// nothing, an empty Path becomes `/`, and a Path without a leading slash
/// gets one.
pub(super) fn rtsp_url(host: &str, port: Option<&str>, path: Option<&str>) -> String {
    let port = match port {
        None | Some("") => String::new(),
        Some(p) => format!(":{p}"),
    };
    let path = match path {
        None | Some("") => "/".to_string(),
        Some(p) if p.starts_with('/') => p.to_string(),
        Some(p) => format!("/{p}"),
    };
    format!("rtsp://{host}{port}{path}")
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        let rows = conn
            .query_all_raw(
                backend(m).build(
                    Query::select()
                        .expr_as(as_i64(m, "Id"), c("Id"))
                        .columns([c("Host"), c("Port"), c("Path")])
                        .from(t("Monitors"))
                        .and_where(Expr::col(c("Type")).eq(enum_val("monitors_type", "Remote")))
                        .and_where(Expr::col(c("Protocol")).eq("rtsp")),
                ),
            )
            .await?;
        for row in rows {
            let id: i64 = row.try_get("", "Id")?;
            let host: Option<String> = row.try_get("", "Host")?;
            let port: Option<String> = row.try_get("", "Port")?;
            let path: Option<String> = row.try_get("", "Path")?;
            let url = rtsp_url(
                host.as_deref().unwrap_or(""),
                port.as_deref(),
                path.as_deref(),
            );
            exec_stmt(
                m,
                Query::update()
                    .table(t("Monitors"))
                    .value(c("Type"), enum_val("monitors_type", "Ffmpeg"))
                    .value(c("Path"), url)
                    .and_where(Expr::col(c("Id")).eq(id)),
            )
            .await?;
        }
        drop_column_if_present(m, "Monitors", "RTSPDescribe").await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::rtsp_url;

    #[test]
    fn builds_the_url_the_way_upstream_does() {
        assert_eq!(
            rtsp_url("cam", Some("554"), Some("/live")),
            "rtsp://cam:554/live"
        );
        assert_eq!(rtsp_url("cam", Some(""), Some("live")), "rtsp://cam/live");
        assert_eq!(rtsp_url("cam", None, None), "rtsp://cam/");
        assert_eq!(rtsp_url("cam", None, Some("")), "rtsp://cam/");
        assert_eq!(
            rtsp_url("u:p@cam", Some("8554"), Some("/s")),
            "rtsp://u:p@cam:8554/s"
        );
    }
}
