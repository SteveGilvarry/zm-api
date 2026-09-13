//! Mirrors `db/legacy/zm_update-1.39.3.sql`: `Notifications.Profile`, zone
//! thresholds become percentages of the zone area (DECIMAL columns plus the
//! `zm_convert_zone_thresholds_to_percent` procedure as a Rust loop), and the
//! `Menu_Items` table with its seed rows.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const THRESHOLDS: [&str; 6] = [
    "MinAlarmPixels",
    "MaxAlarmPixels",
    "MinFilterPixels",
    "MaxFilterPixels",
    "MinBlobPixels",
    "MaxBlobPixels",
];

const MENU_ITEMS: [(&str, i32); 14] = [
    ("Console", 10),
    ("Watch", 15),
    ("Montage", 20),
    ("MontageReview", 30),
    ("Events", 40),
    ("Options", 50),
    ("Log", 60),
    ("Devices", 70),
    ("IntelGpu", 80),
    ("Groups", 90),
    ("Filters", 100),
    ("Snapshots", 110),
    ("Reports", 120),
    ("ReportEventAudit", 130),
    // ("Map", 140) follows in the same upstream list.
];

/// Area of a percent-coordinate polygon by the shoelace formula, as the
/// procedure computes it.
pub(crate) fn percent_area(coords: &str) -> f64 {
    let mut pts = Vec::new();
    for pair in coords.split_whitespace() {
        if let Some((x, y)) = pair.split_once(',') {
            if let (Ok(x), Ok(y)) = (x.trim().parse::<f64>(), y.trim().parse::<f64>()) {
                pts.push((x, y));
            }
        }
    }
    if pts.len() < 2 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..pts.len() {
        let (x0, y0) = pts[i];
        let (x1, y1) = pts[(i + 1) % pts.len()];
        area += x0 * y1 - x1 * y0;
    }
    area.abs() / 2.0
}

/// A pixel-count threshold as a percentage of the zone's pixel area, capped
/// at 100 and rounded to two decimals; values already ≤ 100 are percentages.
pub(crate) fn threshold_to_percent(value: Option<f64>, pixel_area: f64) -> Option<f64> {
    let v = value?;
    if v <= 100.0 || pixel_area <= 0.0 {
        return Some(v);
    }
    Some(((v * 100.0 / pixel_area * 100.0).round() / 100.0).min(100.0))
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Notifications",
            "Profile",
            ColumnDef::new(c("Profile")).string_len(128).to_owned(),
        )
        .await?;

        for table in ["Zones", "ZonePresets"] {
            let already = matches!(
                column_data_type(m, table, "MinAlarmPixels")
                    .await?
                    .as_deref(),
                Some("decimal") | Some("numeric")
            );
            if !already {
                for col in THRESHOLDS {
                    modify_column(m, table, decimal_unsigned(m, col, 10, 2)).await?;
                }
            }
        }

        let conn = m.get_connection();
        let mut sel = Query::select();
        sel.expr_as(as_i64_of(m, "Zones", "Id"), Alias::new("zone_id"))
            .column((t("Zones"), c("Coords")))
            .expr_as(
                Expr::col((t("Monitors"), c("Width"))).cast_as(cast_i64_target(m)),
                Alias::new("width"),
            )
            .expr_as(
                Expr::col((t("Monitors"), c("Height"))).cast_as(cast_i64_target(m)),
                Alias::new("height"),
            );
        for col in THRESHOLDS {
            sel.expr_as(as_f64_of(m, "Zones", col), Alias::new(col.to_lowercase()));
        }
        sel.from(t("Zones"))
            .inner_join(
                t("Monitors"),
                Expr::col((t("Zones"), c("MonitorId"))).equals((t("Monitors"), c("Id"))),
            )
            .and_where(Expr::col((t("Zones"), c("Coords"))).like("%.%"))
            .and_where(Expr::col((t("Monitors"), c("Width"))).gt(0))
            .and_where(Expr::col((t("Monitors"), c("Height"))).gt(0));
        for row in conn.query_all(backend(m).build(&sel)).await? {
            let id: i64 = row.try_get("", "zone_id")?;
            let coords: String = row.try_get("", "Coords")?;
            let width: i64 = row.try_get("", "width")?;
            let height: i64 = row.try_get("", "height")?;
            let values: Vec<Option<f64>> = THRESHOLDS
                .iter()
                .map(|col| row.try_get::<Option<f64>>("", &col.to_lowercase()))
                .collect::<Result<_, _>>()?;
            if !values.iter().any(|v| v.is_some_and(|v| v > 100.0)) {
                continue;
            }
            let pixel_area = percent_area(&coords) * width as f64 * height as f64 / 10000.0;
            if pixel_area <= 0.0 {
                continue;
            }
            let mut upd = Query::update();
            upd.table(t("Zones")).and_where(Expr::col(c("Id")).eq(id));
            for (col, v) in THRESHOLDS.iter().zip(values) {
                upd.value(c(col), threshold_to_percent(v, pixel_area));
            }
            exec_stmt(m, &upd).await?;
        }

        if !table_exists(m, "Menu_Items").await? {
            ensure_enum_type(
                m,
                "menu_items_icon_type",
                &["material", "fontawesome", "image", "none"],
            )
            .await?;
            create_table(
                m,
                Table::create()
                    .table(t("Menu_Items"))
                    .col(autoinc_pk(m, "Id", false))
                    .col(ColumnDef::new(c("MenuKey")).string_len(32).not_null())
                    .col(tinyint1(m, "Enabled", Some(true)).not_null())
                    .col(ColumnDef::new(c("Label")).string_len(64))
                    .col(
                        ColumnDef::new(c("SortOrder"))
                            .small_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(c("Icon")).string_len(128))
                    .col(
                        enum_col(
                            "IconType",
                            "menu_items_icon_type",
                            &["material", "fontawesome", "image", "none"],
                        )
                        .not_null()
                        .default("material"),
                    )
                    .to_owned(),
                "Menu_Items",
                &[("Menu_Items_MenuKey_idx", &["MenuKey"], true)],
            )
            .await?;
        }
        add_column_if_missing(
            m,
            "Menu_Items",
            "Icon",
            ColumnDef::new(c("Icon")).string_len(128).to_owned(),
        )
        .await?;
        ensure_enum_type(
            m,
            "menu_items_icon_type",
            &["material", "fontawesome", "image", "none"],
        )
        .await?;
        add_column_if_missing(
            m,
            "Menu_Items",
            "IconType",
            enum_col(
                "IconType",
                "menu_items_icon_type",
                &["material", "fontawesome", "image", "none"],
            )
            .not_null()
            .default("material")
            .to_owned(),
        )
        .await?;
        for (key, order) in MENU_ITEMS.iter().copied().chain([("Map", 140)]) {
            insert_if_absent(
                m,
                probe_eq("Menu_Items", "MenuKey", key),
                Query::insert()
                    .into_table(t("Menu_Items"))
                    .columns([c("MenuKey"), c("Enabled"), c("SortOrder")])
                    .values_panic([key.into(), true.into(), order.into()])
                    .to_owned(),
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

fn cast_i64_target(m: &SchemaManager<'_>) -> Alias {
    match backend(m) {
        sea_orm_migration::sea_orm::DatabaseBackend::MySql => Alias::new("SIGNED"),
        _ => Alias::new("BIGINT"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shoelace_area_of_a_percent_square() {
        assert_eq!(percent_area("0,0 50,0 50,50 0,50"), 2500.0);
        assert_eq!(percent_area("0,0"), 0.0);
    }

    #[test]
    fn pixel_thresholds_scale_to_percent_of_the_zone_and_cap_at_100() {
        // A 50%×50% zone on 1000×1000 is 250 000 px²; 25 000 px is 10%.
        let px = 2500.0 * 1000.0 * 1000.0 / 10000.0;
        assert_eq!(threshold_to_percent(Some(25_000.0), px), Some(10.0));
        assert_eq!(threshold_to_percent(Some(9_000_000.0), px), Some(100.0));
        assert_eq!(
            threshold_to_percent(Some(75.0), px),
            Some(75.0),
            "already a percentage"
        );
        assert_eq!(threshold_to_percent(None, px), None);
    }
}
