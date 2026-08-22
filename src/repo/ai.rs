//! Query layer for the AI object-detection registry.

use sea_orm::*;

use crate::dto::request::ai::{
    CreateAiDatasetRequest, CreateAiModelRequest, CreateAiObjectClassRequest,
    UpdateAiDatasetRequest, UpdateAiModelRequest, UpdateAiObjectClassRequest,
};
use crate::dto::PaginationParams;
use crate::entity::ai_datasets::{
    ActiveModel as DatasetAm, Entity as Datasets, Model as DatasetModel,
};
use crate::entity::ai_models::{ActiveModel as ModelAm, Entity as Models, Model as AiModel};
use crate::entity::ai_object_classes::{
    ActiveModel as ClassAm, Column as ClassColumn, Entity as Classes, Model as ClassModel,
};
use crate::error::AppResult;

// --- Datasets --------------------------------------------------------------

pub async fn datasets_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<DatasetModel>, u64)> {
    let paginator = Datasets::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn dataset_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<DatasetModel>> {
    Ok(Datasets::find_by_id(id).one(db).await?)
}

pub async fn create_dataset(
    db: &DatabaseConnection,
    req: &CreateAiDatasetRequest,
) -> AppResult<DatasetModel> {
    let am = DatasetAm {
        id: NotSet,
        name: Set(req.name.clone()),
        description: Set(req.description.clone()),
        version: Set(req.version.clone()),
        num_classes: Set(req.num_classes),
    };
    Ok(am.insert(db).await?)
}

pub async fn update_dataset(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateAiDatasetRequest,
) -> AppResult<Option<DatasetModel>> {
    let Some(model) = dataset_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: DatasetAm = model.into();
    if let Some(v) = &req.name {
        am.name = Set(v.clone());
    }
    if req.description.is_some() {
        am.description = Set(req.description.clone());
    }
    if req.version.is_some() {
        am.version = Set(req.version.clone());
    }
    if let Some(v) = req.num_classes {
        am.num_classes = Set(v);
    }
    Ok(Some(am.update(db).await?))
}

pub async fn delete_dataset(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    Ok(Datasets::delete_by_id(id).exec(db).await?.rows_affected > 0)
}

// --- Models ----------------------------------------------------------------

pub async fn models_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<AiModel>, u64)> {
    let paginator = Models::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn model_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<AiModel>> {
    Ok(Models::find_by_id(id).one(db).await?)
}

pub async fn create_model(
    db: &DatabaseConnection,
    req: &CreateAiModelRequest,
) -> AppResult<AiModel> {
    use crate::entity::sea_orm_active_enums::Framework;
    let am = ModelAm {
        id: NotSet,
        name: Set(req.name.clone()),
        description: Set(req.description.clone()),
        model_path: Set(req.model_path.clone()),
        framework: Set(req.framework.clone().unwrap_or(Framework::Onnx)),
        version: Set(req.version.clone()),
        dataset_id: Set(req.dataset_id),
        enabled: Set(req.enabled.unwrap_or(0)),
    };
    Ok(am.insert(db).await?)
}

pub async fn update_model(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateAiModelRequest,
) -> AppResult<Option<AiModel>> {
    let Some(model) = model_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ModelAm = model.into();
    if let Some(v) = &req.name {
        am.name = Set(v.clone());
    }
    if req.description.is_some() {
        am.description = Set(req.description.clone());
    }
    if req.model_path.is_some() {
        am.model_path = Set(req.model_path.clone());
    }
    if let Some(v) = &req.framework {
        am.framework = Set(v.clone());
    }
    if req.version.is_some() {
        am.version = Set(req.version.clone());
    }
    // Outer Some = "change it"; inner None clears the link.
    if let Some(v) = req.dataset_id {
        am.dataset_id = Set(v);
    }
    if let Some(v) = req.enabled {
        am.enabled = Set(v);
    }
    Ok(Some(am.update(db).await?))
}

pub async fn delete_model(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    Ok(Models::delete_by_id(id).exec(db).await?.rows_affected > 0)
}

// --- Object classes --------------------------------------------------------

pub async fn classes_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
    dataset_id: Option<u32>,
) -> AppResult<(Vec<ClassModel>, u64)> {
    let mut query = Classes::find();
    if let Some(d) = dataset_id {
        query = query.filter(ClassColumn::DatasetId.eq(d));
    }
    // Stable ordering: classes are read by index far more than by id.
    let paginator = query
        .order_by_asc(ClassColumn::DatasetId)
        .order_by_asc(ClassColumn::ClassIndex)
        .paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn class_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ClassModel>> {
    Ok(Classes::find_by_id(id).one(db).await?)
}

pub async fn create_class(
    db: &DatabaseConnection,
    req: &CreateAiObjectClassRequest,
) -> AppResult<ClassModel> {
    let am = ClassAm {
        id: NotSet,
        dataset_id: Set(req.dataset_id),
        class_name: Set(req.class_name.clone()),
        class_index: Set(req.class_index),
        description: Set(req.description.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update_class(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateAiObjectClassRequest,
) -> AppResult<Option<ClassModel>> {
    let Some(model) = class_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ClassAm = model.into();
    if let Some(v) = req.dataset_id {
        am.dataset_id = Set(v);
    }
    if let Some(v) = &req.class_name {
        am.class_name = Set(v.clone());
    }
    if let Some(v) = req.class_index {
        am.class_index = Set(v);
    }
    if req.description.is_some() {
        am.description = Set(req.description.clone());
    }
    Ok(Some(am.update(db).await?))
}

pub async fn delete_class(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    Ok(Classes::delete_by_id(id).exec(db).await?.rows_affected > 0)
}

/// Fetch datasets by id (used to join names onto a model listing).
pub async fn datasets_by_ids(db: &DatabaseConnection, ids: &[u32]) -> AppResult<Vec<DatasetModel>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    use crate::entity::ai_datasets::Column as DatasetColumn;
    Ok(Datasets::find()
        .filter(DatasetColumn::Id.is_in(ids.iter().copied()))
        .all(db)
        .await?)
}
