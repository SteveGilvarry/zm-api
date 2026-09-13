//! Mirrors `db/legacy/zm_update-1.39.2.sql`: zone coordinates become
//! percentages of the monitor frame (the `zm_update_zone_coords_to_percent`
//! stored procedure, done here as a Rust loop so it runs on any backend),
//! `Zones.Area` is rescaled to the 100×100 percent frame, `Units` flips to
//! Percent, and the `Notifications` table is created.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// `"x,y x,y ..."` in pixels → the same in percent with two decimals, as the
/// procedure's `CAST(... AS DECIMAL(10,2))` rendered it. Coordinates that
/// already contain a decimal point are already percentages and are skipped.
pub(crate) fn coords_to_percent(coords: &str, width: i64, height: i64) -> Option<String> {
    if coords.contains('.') || width <= 0 || height <= 0 {
        return None;
    }
    let mut out = Vec::new();
    for pair in coords.split_whitespace() {
        if pair.is_empty() || pair == "," {
            continue;
        }
        let (x, y) = pair.split_once(',')?;
        let x: f64 = x.trim().parse().ok()?;
        let y: f64 = y.trim().parse().ok()?;
        out.push(format!(
            "{:.2},{:.2}",
            (x / width as f64 * 100.0 * 100.0).round() / 100.0,
            (y / height as f64 * 100.0 * 100.0).round() / 100.0
        ));
    }
    (!out.is_empty()).then(|| out.join(" "))
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();

        // Coords and Area, per zone, from the monitor's pixel dimensions.
        let rows = conn
            .query_all(
                backend(m).build(
                    Query::select()
                        .expr_as(as_i64_of(m, "Zones", "Id"), Alias::new("zone_id"))
                        .column((t("Zones"), c("Coords")))
                        .expr_as(as_i64_of(m, "Zones", "Area"), Alias::new("area"))
                        .expr_as(
                            Expr::col((t("Monitors"), c("Width"))).cast_as(cast_target(m)),
                            Alias::new("width"),
                        )
                        .expr_as(
                            Expr::col((t("Monitors"), c("Height"))).cast_as(cast_target(m)),
                            Alias::new("height"),
                        )
                        .from(t("Zones"))
                        .inner_join(
                            t("Monitors"),
                            Expr::col((t("Zones"), c("MonitorId")))
                                .equals((t("Monitors"), c("Id"))),
                        )
                        .and_where(Expr::col((t("Monitors"), c("Width"))).gt(0))
                        .and_where(Expr::col((t("Monitors"), c("Height"))).gt(0)),
                ),
            )
            .await?;
        for row in rows {
            let id: i64 = row.try_get("", "zone_id")?;
            let coords: String = row.try_get("", "Coords")?;
            let area: i64 = row.try_get("", "area")?;
            let width: i64 = row.try_get("", "width")?;
            let height: i64 = row.try_get("", "height")?;
            let mut upd = Query::update();
            upd.table(t("Zones")).and_where(Expr::col(c("Id")).eq(id));
            let mut changed = false;
            if let Some(pct) = coords_to_percent(&coords, width, height) {
                upd.value(c("Coords"), pct);
                changed = true;
            }
            if area > 0 {
                let scaled =
                    (area as f64 * 10000.0 / (width as f64 * height as f64)).round() as i64;
                upd.value(c("Area"), scaled);
                changed = true;
            }
            if changed {
                exec_stmt(m, &upd).await?;
            }
        }

        exec_stmt(
            m,
            Query::update()
                .table(t("Zones"))
                .value(c("Units"), enum_val("zones_units", "Percent"))
                .and_where(Expr::col(c("Units")).eq(enum_val("zones_units", "Pixels"))),
        )
        .await?;
        modify_column(
            m,
            "Zones",
            enum_col("Units", "zones_units", &["Pixels", "Percent"])
                .not_null()
                .default("Percent")
                .to_owned(),
        )
        .await?;

        if !table_exists(m, "Notifications").await? {
            ensure_enum_type(m, "notifications_platform", &["android", "ios", "web"]).await?;
            ensure_enum_type(m, "notifications_push_state", &["enabled", "disabled"]).await?;
            create_table(
                m,
                Table::create()
                    .table(t("Notifications"))
                    .col(autoinc_pk(m, "Id", false))
                    .col(ColumnDef::new(c("UserId")).unsigned())
                    .col(ColumnDef::new(c("Token")).string_len(512).not_null())
                    .col(
                        enum_col(
                            "Platform",
                            "notifications_platform",
                            &["android", "ios", "web"],
                        )
                        .not_null(),
                    )
                    .col(ColumnDef::new(c("MonitorList")).text())
                    .col(
                        ColumnDef::new(c("Interval"))
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        enum_col(
                            "PushState",
                            "notifications_push_state",
                            &["enabled", "disabled"],
                        )
                        .not_null()
                        .default("enabled"),
                    )
                    .col(ColumnDef::new(c("AppVersion")).string_len(32))
                    .col(
                        ColumnDef::new(c("BadgeCount"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(c("LastNotifiedAt")).date_time())
                    .col(ColumnDef::new(c("CreatedOn")).date_time())
                    .col(timestamp_on_update(m, "UpdatedOn"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("Notifications_ibfk_1")
                            .from(t("Notifications"), c("UserId"))
                            .to(t("Users"), c("Id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
                "Notifications",
                &[
                    ("Notifications_Token_idx", &["Token"], true),
                    ("Notifications_UserId_idx", &["UserId"], false),
                ],
            )
            .await?;
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}

fn cast_target(m: &SchemaManager<'_>) -> Alias {
    match backend(m) {
        sea_orm_migration::sea_orm::DatabaseBackend::MySql => Alias::new("SIGNED"),
        _ => Alias::new("BIGINT"),
    }
}

#[cfg(test)]
mod tests {
    use super::coords_to_percent;

    #[test]
    fn pixel_coordinates_become_percentages_with_two_decimals() {
        assert_eq!(
            coords_to_percent("0,0 640,0 640,480 0,480", 1280, 960).as_deref(),
            Some("0.00,0.00 50.00,0.00 50.00,50.00 0.00,50.00")
        );
        assert_eq!(
            coords_to_percent("10,10  ,  20,20", 100, 100).as_deref(),
            Some("10.00,10.00 20.00,20.00")
        );
    }

    #[test]
    fn percentages_and_bad_input_are_left_alone() {
        assert_eq!(coords_to_percent("12.5,7.25 50.00,50.00", 1280, 960), None);
        assert_eq!(coords_to_percent("1,1", 0, 100), None);
        assert_eq!(coords_to_percent("garbage", 100, 100), None);
    }
}
