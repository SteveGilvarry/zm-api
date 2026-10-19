//! `POST /api/v3/events/export` (#30): a streamed zip with a folder per
//! event and the parts asked for.
//!
//! Requires the test database:
//!   APP_PROFILE=test-db cargo nextest run --test it_event_export --run-ignored all

mod common;

use axum::http::StatusCode;
use common::fixtures::{insert_monitor, insert_storage, unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use zm_api::entity::sea_orm_active_enums::Scheme;

/// Names and contents of the zip's entries.
async fn entries(zip: Vec<u8>) -> Vec<(String, Vec<u8>)> {
    let reader = async_zip::base::read::mem::ZipFileReader::new(zip)
        .await
        .expect("a readable zip");
    let mut out = Vec::new();
    for i in 0..reader.file().entries().len() {
        let name = reader.file().entries()[i]
            .filename()
            .as_str()
            .unwrap()
            .to_string();
        let mut buf = Vec::new();
        reader
            .reader_with_entry(i)
            .await
            .unwrap()
            .read_to_end_checked(&mut buf)
            .await
            .unwrap();
        out.push((name, buf));
    }
    out
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn exports_the_parts_asked_for() {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let tmp = tempfile::tempdir().unwrap();
    let monitor = insert_monitor(&app.db, "Export").await.unwrap();
    let storage = insert_storage(
        &app.db,
        "Export",
        tmp.path().to_str().unwrap(),
        Scheme::Shallow,
    )
    .await
    .unwrap();
    let event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor.id),
        storage_id: Set(Some(storage.id)),
        scheme: Set(Scheme::Shallow),
        state_id: Set(1),
        name: Set(unique_name("Export")),
        frames: Set(Some(2)),
        length: Set(Decimal::new(200, 2)),
        default_video: Set("1-video.mp4".into()),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .unwrap();
    let id = event.id;
    let _g = (
        RowGuard::new(format!("Events#{id}"), move |db| async move {
            let _ = zm_api::entity::events::Entity::delete_many()
                .filter(zm_api::entity::events::Column::Id.eq(id))
                .exec(&db)
                .await;
        }),
        RowGuard::storage(storage.id),
        RowGuard::monitor(monitor.id),
    );
    let dir = tmp.path().join(monitor.id.to_string()).join(id.to_string());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("1-video.mp4"), vec![7u8; 300_000]).unwrap();
    std::fs::write(dir.join("00001-capture.jpg"), b"jpeg").unwrap();
    std::fs::write(dir.join("objdetect.json"), b"{}").unwrap();

    // Default: details and video.
    let r = app
        .post_json("/api/v3/events/export", &token, &json!({ "ids": [id] }))
        .await;
    assert_eq!(r.status(), StatusCode::OK, "{}", r.text());
    let files = entries(r.body).await;
    let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [format!("{id}/event.json"), format!("{id}/1-video.mp4")]
    );
    let details: serde_json::Value = serde_json::from_slice(&files[0].1).unwrap();
    assert_eq!(details["id"], id);
    assert_eq!(files[1].1, vec![7u8; 300_000], "the video, byte for byte");

    let r = app
        .post_json(
            "/api/v3/events/export",
            &token,
            &json!({ "ids": [id], "include": ["images", "misc"] }),
        )
        .await;
    let files = entries(r.body).await;
    let names: Vec<String> = files.into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        names,
        [
            format!("{id}/00001-capture.jpg"),
            format!("{id}/objdetect.json")
        ]
    );

    // An unknown id: nothing is streamed.
    let r = app
        .post_json(
            "/api/v3/events/export",
            &token,
            &json!({ "ids": [id, 999_999_999] }),
        )
        .await;
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
    let r = app
        .post_json("/api/v3/events/export", &token, &json!({ "ids": [] }))
        .await;
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
