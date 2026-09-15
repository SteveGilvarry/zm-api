//! `Zones.Area` is recomputed from `Coords` (GH #43).
//!
//! Area is not decorative: when a zone's `Units` are `Percent`, the alarm
//! thresholds are stored relative to it. Every zone created through this API
//! used to start at `Area = 0`, and changing a zone's coordinates never
//! updated it — so percent thresholds quietly meant something other than what
//! they said.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_zone_area -- --include-ignored

mod common;

use common::fixtures::insert_monitor;
use common::test_db::get_test_db;
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, FromQueryResult, Statement};

async fn exec(db: &DatabaseConnection, sql: impl Into<String>) {
    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.into(),
    ))
    .await
    .expect("statement");
}

#[derive(FromQueryResult)]
struct AreaRow {
    area: i64,
    coords: String,
}

async fn zone(db: &DatabaseConnection, id: u32) -> AreaRow {
    AreaRow::find_by_statement(Statement::from_string(
        db.get_database_backend(),
        format!("SELECT CAST(Area AS SIGNED) AS area, Coords AS coords FROM Zones WHERE Id = {id}"),
    ))
    .one(db)
    .await
    .expect("query")
    .expect("zone exists")
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn area_is_computed_on_create_and_recomputed_on_coord_change() {
    let db = get_test_db().await.expect("test db");
    // Zones.MonitorId is a real foreign key, so the monitor has to exist.
    let monitor = insert_monitor(&db, "zone_area").await.expect("monitor");
    let mon = monitor.id;

    let req = zm_api::dto::request::CreateZoneRequest {
        name: "area-test".to_string(),
        r#type: "Active".to_string(),
        units: "Percent".to_string(),
        num_coords: 4,
        coords: "0,0 639,0 639,479 0,479".to_string(),
        check_method: None,
        ..Default::default()
    };
    let created = zm_api::repo::zones::create_for_monitor(&db, mon, &req)
        .await
        .expect("create");

    let row = zone(&db, created.id).await;
    assert_eq!(
        row.area,
        639 * 479,
        "Area must be computed from Coords on create, not left at 0"
    );

    // Halve the width; the area must follow.
    zm_api::repo::zones::update_coords(
        &db,
        created.id,
        None,
        Some("0,0 319,0 319,479 0,479".to_string()),
    )
    .await
    .expect("update");

    let row = zone(&db, created.id).await;
    assert_eq!(row.coords, "0,0 319,0 319,479 0,479");
    assert_eq!(
        row.area,
        319 * 479,
        "changing Coords must recompute Area, not leave the old one"
    );

    exec(&db, format!("DELETE FROM Zones WHERE MonitorId = {mon}")).await;
    let _ = zm_api::entity::monitors::Entity::delete_by_id(mon)
        .exec(&db)
        .await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn malformed_coords_are_rejected_rather_than_stored_with_a_zero_area() {
    let db = get_test_db().await.expect("test db");
    // A real monitor, so a failure here can only be the coords — an earlier
    // version of this test passed because the foreign key rejected it first,
    // which proved nothing about the validation being tested.
    let monitor = insert_monitor(&db, "zone_bad").await.expect("monitor");
    let mon = monitor.id;

    let req = zm_api::dto::request::CreateZoneRequest {
        name: "bad-coords".to_string(),
        r#type: "Active".to_string(),
        units: "Percent".to_string(),
        num_coords: 2,
        // Two points enclose nothing — storing this with Area = 0 is how the
        // old behaviour produced silently meaningless thresholds.
        coords: "0,0 100,0".to_string(),
        check_method: None,
        ..Default::default()
    };
    let result = zm_api::repo::zones::create_for_monitor(&db, mon, &req).await;
    let err = result.expect_err("a non-polygon must be refused");
    assert!(
        err.to_string().contains("polygon"),
        "it must be refused for the coords, not incidentally: {err}"
    );

    let remaining = AreaRow::find_by_statement(Statement::from_string(
        db.get_database_backend(),
        format!("SELECT CAST(COUNT(*) AS SIGNED) AS area, '' AS coords FROM Zones WHERE MonitorId = {mon}"),
    ))
    .one(&db)
    .await
    .expect("query")
    .expect("row");
    assert_eq!(remaining.area, 0, "nothing should have been written");

    let _ = zm_api::entity::monitors::Entity::delete_by_id(mon)
        .exec(&db)
        .await;
}
