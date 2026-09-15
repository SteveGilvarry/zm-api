//! `GET /api/v3/events/{id}/frames/{fid}/image` and `GET /api/v3/frames/{id}/image`
//! (GH #26).
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo nextest run --test it_frame_images --run-ignored all

mod common;

use axum::http::{Method, StatusCode};
use common::fixtures::{insert_monitor, insert_storage, unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use zm_api::entity::sea_orm_active_enums::{FrameType, Scheme};

fn guard_event(id: u64) -> RowGuard {
    RowGuard::new(format!("Events#{id}"), move |db| async move {
        let _ = zm_api::entity::frames::Entity::delete_many()
            .filter(zm_api::entity::frames::Column::EventId.eq(id))
            .exec(&db)
            .await;
        let _ = zm_api::entity::events::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}
struct Fixture {
    app: TestApp,
    token: String,
    tmp: tempfile::TempDir,
    event_id: u64,
    dir: std::path::PathBuf,
    _guards: Vec<RowGuard>,
}

/// A 2-second, 20-frame event on shallow storage under a temp dir.
async fn fixture(label: &str) -> Fixture {
    let app = TestApp::spawn().await;
    let token = superuser_token();
    let tmp = tempfile::tempdir().expect("tempdir");
    let monitor = insert_monitor(&app.db, label).await.expect("monitor");
    let storage = insert_storage(
        &app.db,
        label,
        tmp.path().to_str().unwrap(),
        Scheme::Shallow,
    )
    .await
    .expect("storage");
    let event = zm_api::entity::events::ActiveModel {
        monitor_id: Set(monitor.id),
        storage_id: Set(Some(storage.id)),
        scheme: Set(Scheme::Shallow),
        state_id: Set(1),
        name: Set(unique_name(label)),
        frames: Set(Some(20)),
        length: Set(Decimal::new(200, 2)),
        max_score_frame_id: Set(Some(7)),
        default_video: Set(String::new()),
        ..Default::default()
    }
    .insert(&app.db)
    .await
    .expect("event");
    let guards = vec![
        guard_event(event.id),
        RowGuard::storage(storage.id),
        RowGuard::monitor(monitor.id),
    ];
    let dir = tmp
        .path()
        .join(monitor.id.to_string())
        .join(event.id.to_string());
    std::fs::create_dir_all(&dir).unwrap();
    Fixture {
        app,
        token,
        tmp,
        event_id: event.id,
        dir,
        _guards: guards,
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn saved_jpegs_are_served_by_number_name_and_frame_row() {
    let f = fixture("FrameImgJpeg").await;
    std::fs::write(f.dir.join("00003-capture.jpg"), b"frame-three").unwrap();
    std::fs::write(f.dir.join("00007-capture.jpg"), b"frame-seven").unwrap();
    std::fs::write(f.dir.join("snapshot.jpg"), b"the-snapshot").unwrap();

    let get = |path: String| {
        let app = &f.app;
        let token = f.token.clone();
        async move { app.get(&path, &token).await }
    };
    let base = format!("/api/v3/events/{}/frames", f.event_id);

    let resp = get(format!("{base}/3/image")).await;
    assert_eq!(resp.status(), StatusCode::OK, "{}", resp.text());
    assert_eq!(resp.text(), "frame-three");

    // No alarm.jpg: falls back to the event's highest-scoring frame.
    let resp = get(format!("{base}/alarm/image")).await;
    assert_eq!(resp.text(), "frame-seven");
    assert_eq!(
        get(format!("{base}/snapshot/image")).await.text(),
        "the-snapshot"
    );

    // objdetect has no fallback; numbers past the last frame and junk fids fail.
    assert_eq!(
        get(format!("{base}/objdetect/image")).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(format!("{base}/21/image")).await.status(),
        StatusCode::NOT_FOUND
    );
    for bad in ["0", "abc", "-1"] {
        assert_eq!(
            get(format!("{base}/{bad}/image")).await.status(),
            StatusCode::BAD_REQUEST,
            "fid {bad}"
        );
    }

    // By Frames row id, with the token in the query string like other media.
    let row = zm_api::entity::frames::ActiveModel {
        event_id: Set(f.event_id),
        frame_id: Set(3),
        r#type: Set(FrameType::Normal),
        time_stamp: Set(chrono::Utc::now()),
        delta: Set(Decimal::new(30, 2)),
        score: Set(0),
        ..Default::default()
    }
    .insert(&f.app.db)
    .await
    .expect("frame row");
    let resp = f
        .app
        .request(
            Method::GET,
            &format!("/api/v3/frames/{}/image?token={}", row.id, f.token),
        )
        .send()
        .await;
    assert_eq!(resp.status(), StatusCode::OK, "{}", resp.text());
    assert_eq!(resp.text(), "frame-three");
    assert_eq!(
        f.app
            .request(Method::GET, &format!("/api/v3/frames/{}/image", row.id))
            .send()
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    drop(f.tmp);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn video_only_events_decode_the_frame_from_the_mp4() {
    let f = fixture("FrameImgVideo").await;
    // One second black, one second white, at 10fps: frame 15 is white.
    let mp4 = f.dir.join(format!("{}-video.mp4", f.event_id));
    let made = std::process::Command::new("ffmpeg")
        .args(["-y", "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i"])
        .arg("color=c=black:s=64x64:r=10:d=1[b];color=c=white:s=64x64:r=10:d=1[w];[b][w]concat=n=2:v=1")
        .args(["-c:v", "libx264", "-g", "20", "-pix_fmt", "yuv420p"])
        .arg(&mp4)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !made {
        eprintln!("Skipping: ffmpeg binary not available to build fixture");
        return;
    }
    let luma = |jpeg: &[u8]| {
        let img = image::load_from_memory(jpeg).expect("jpeg").to_luma8();
        img.pixels().map(|p| f64::from(p.0[0])).sum::<f64>() / f64::from(img.width() * img.height())
    };

    for (fid, white) in [(3, false), (15, true)] {
        let resp = f
            .app
            .get(
                &format!("/api/v3/events/{}/frames/{fid}/image", f.event_id),
                &f.token,
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK, "fid {fid}: {}", resp.text());
        let l = luma(&resp.body);
        assert_eq!(l > 128.0, white, "fid {fid} luma {l}");
    }
}
