//! Integration tests for the AI object-detection registry (GH #47).
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_ai_registry -- --include-ignored

mod common;

use axum::http::{Method, StatusCode};
use common::assertions::assert_status;
use common::fixtures::{unique_name, RowGuard};
use common::harness::{superuser_token, TestApp};
use serde_json::json;
use zm_api::dto::response::ai::{
    AiDatasetResponse, AiModelResponse, AiObjectClassResponse, PaginatedAiObjectClassesResponse,
};

fn guard_dataset(id: u32) -> RowGuard {
    RowGuard::new(format!("AI_Datasets#{id}"), move |db| async move {
        use sea_orm::EntityTrait;
        let _ = zm_api::entity::ai_datasets::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

fn guard_model(id: u32) -> RowGuard {
    RowGuard::new(format!("AI_Models#{id}"), move |db| async move {
        use sea_orm::EntityTrait;
        let _ = zm_api::entity::ai_models::Entity::delete_by_id(id)
            .exec(&db)
            .await;
    })
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn dataset_model_and_class_round_trip() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // --- dataset create → get → update
    let ds_name = unique_name("ds");
    let resp = app
        .post_json(
            "/api/v3/ai/datasets",
            &token,
            &json!({ "name": ds_name, "num_classes": 3, "version": "1.0" }),
        )
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "create dataset; body: {}",
        resp.text()
    );
    let ds: AiDatasetResponse = resp.json();
    let _dg = guard_dataset(ds.id);
    assert_eq!(ds.num_classes, 3);

    let got: AiDatasetResponse = app
        .get(&format!("/api/v3/ai/datasets/{}", ds.id), &token)
        .await
        .json();
    assert_eq!(got.name, ds_name);

    let resp = app
        .request(Method::PATCH, &format!("/api/v3/ai/datasets/{}", ds.id))
        .bearer(&token)
        .json(&json!({ "num_classes": 5 }))
        .send()
        .await;
    assert_status(&resp, StatusCode::OK);
    let updated: AiDatasetResponse = resp.json();
    assert_eq!(updated.num_classes, 5, "partial update applied");
    assert_eq!(updated.name, ds_name, "untouched field preserved");

    // --- model create, with the dataset name joined on read
    let resp = app
        .post_json(
            "/api/v3/ai/models",
            &token,
            &json!({
                "name": unique_name("mdl"),
                "framework": "ONNX",
                "dataset_id": ds.id,
                "model_path": "/var/lib/zm/models/yolo.onnx",
                "enabled": 1
            }),
        )
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "create model; body: {}",
        resp.text()
    );
    let model: AiModelResponse = resp.json();
    let _mg = guard_model(model.id);
    assert_eq!(model.framework, "ONNX", "framework round-trips by name");

    let fetched: AiModelResponse = app
        .get(&format!("/api/v3/ai/models/{}", model.id), &token)
        .await
        .json();
    assert_eq!(
        fetched.dataset_name.as_deref(),
        Some(ds_name.as_str()),
        "model listing resolves the dataset name"
    );

    // --- object class create + dataset-scoped listing
    let resp = app
        .post_json(
            "/api/v3/ai/object-classes",
            &token,
            &json!({ "dataset_id": ds.id, "class_name": "sasquatch", "class_index": 0 }),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::CREATED, "body: {}", resp.text());
    let class: AiObjectClassResponse = resp.json();
    assert_eq!(class.dataset_id, ds.id);

    let listed: PaginatedAiObjectClassesResponse = app
        .get(
            &format!(
                "/api/v3/ai/object-classes?dataset_id={}&page_size=100",
                ds.id
            ),
            &token,
        )
        .await
        .json();
    assert_eq!(listed.total, 1, "filter scopes to the dataset");
    assert_eq!(listed.items[0].class_name, "sasquatch");

    // --- delete model, then dataset (its classes cascade)
    let del = app
        .delete(&format!("/api/v3/ai/models/{}", model.id), &token)
        .await;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    let del = app
        .delete(&format!("/api/v3/ai/datasets/{}", ds.id), &token)
        .await;
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    let gone = app
        .get(&format!("/api/v3/ai/datasets/{}", ds.id), &token)
        .await;
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn model_with_unknown_dataset_is_rejected() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // A dangling FK should surface as a 400, not a raw DB error.
    let resp = app
        .post_json(
            "/api/v3/ai/models",
            &token,
            &json!({ "name": unique_name("orphan"), "dataset_id": 999_000_111u32 }),
        )
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "unknown dataset must be a 400; body: {}",
        resp.text()
    );
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn coco_seed_is_readable() {
    let app = TestApp::spawn().await;
    let token = superuser_token();

    // The install seeds COCO with 80 classes; the registry should surface them.
    let listed: PaginatedAiObjectClassesResponse = app
        .get("/api/v3/ai/object-classes?page_size=200", &token)
        .await
        .json();
    assert!(
        listed.total >= 80,
        "expected the seeded COCO classes, got {}",
        listed.total
    );
}
