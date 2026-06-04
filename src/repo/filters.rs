use crate::dto::PaginationParams;
use crate::entity::filters::{Column as FilterColumn, Entity as Filters, Model as FilterModel};
use crate::error::AppResult;
use sea_orm::*;

// Repos accept a database connection (preferred)
//
// `owner` scopes the result to a single user's filters (`Some(uid)`); `None` is
// unrestricted (used for admins / System-level access). See `service::filters`.
#[tracing::instrument(skip_all)]
pub async fn find_all(db: &DatabaseConnection, owner: Option<u32>) -> AppResult<Vec<FilterModel>> {
    let mut query = Filters::find();
    if let Some(uid) = owner {
        query = query.filter(FilterColumn::UserId.eq(uid));
    }
    Ok(query.all(db).await?)
}

#[tracing::instrument(skip_all)]
pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
    owner: Option<u32>,
) -> AppResult<(Vec<FilterModel>, u64)> {
    let mut query = Filters::find();
    if let Some(uid) = owner {
        query = query.filter(FilterColumn::UserId.eq(uid));
    }
    let paginator = query.paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

#[tracing::instrument(skip_all)]
pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<FilterModel>> {
    Ok(Filters::find_by_id(id).one(db).await?)
}

#[tracing::instrument(skip_all)]
pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &crate::dto::request::UpdateFilterRequest,
) -> AppResult<Option<FilterModel>> {
    use crate::dto::request::parse_email_format;
    use sea_orm::{ActiveModelTrait, Set};
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut active: crate::entity::filters::ActiveModel = model.into();
    // Only fields present in the request are changed; everything else keeps
    // its current value.
    if let Some(v) = req.name.clone() {
        active.name = Set(v);
    }
    if let Some(v) = req.query_json.clone() {
        active.query_json = Set(v);
    }
    if let Some(v) = req.execute_interval {
        active.execute_interval = Set(v);
    }
    if req.user_id.is_some() {
        active.user_id = Set(req.user_id);
    }
    if let Some(v) = req.email_format.as_deref() {
        active.email_format = Set(parse_email_format(v));
    }
    if let Some(v) = req.auto_archive {
        active.auto_archive = Set(v);
    }
    if let Some(v) = req.auto_unarchive {
        active.auto_unarchive = Set(v);
    }
    if let Some(v) = req.auto_video {
        active.auto_video = Set(v);
    }
    if let Some(v) = req.auto_upload {
        active.auto_upload = Set(v);
    }
    if let Some(v) = req.auto_email {
        active.auto_email = Set(v);
    }
    if req.email_to.is_some() {
        active.email_to = Set(req.email_to.clone());
    }
    if req.email_subject.is_some() {
        active.email_subject = Set(req.email_subject.clone());
    }
    if req.email_body.is_some() {
        active.email_body = Set(req.email_body.clone());
    }
    if req.email_server.is_some() {
        active.email_server = Set(req.email_server.clone());
    }
    if let Some(v) = req.auto_message {
        active.auto_message = Set(v);
    }
    if let Some(v) = req.auto_execute {
        active.auto_execute = Set(v);
    }
    if req.auto_execute_cmd.is_some() {
        active.auto_execute_cmd = Set(req.auto_execute_cmd.clone());
    }
    if let Some(v) = req.auto_delete {
        active.auto_delete = Set(v);
    }
    if let Some(v) = req.auto_move {
        active.auto_move = Set(v);
    }
    if let Some(v) = req.auto_move_to {
        active.auto_move_to = Set(v);
    }
    if let Some(v) = req.auto_copy {
        active.auto_copy = Set(v);
    }
    if let Some(v) = req.auto_copy_to {
        active.auto_copy_to = Set(v);
    }
    if let Some(v) = req.update_disk_space {
        active.update_disk_space = Set(v);
    }
    if let Some(v) = req.background {
        active.background = Set(v);
    }
    if let Some(v) = req.concurrent {
        active.concurrent = Set(v);
    }
    if let Some(v) = req.lock_rows {
        active.lock_rows = Set(v);
    }
    let updated = active.update(db).await?;
    Ok(Some(updated))
}

#[tracing::instrument(skip_all)]
pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateFilterRequest,
) -> AppResult<FilterModel> {
    use crate::dto::request::parse_email_format;
    use crate::entity::filters::ActiveModel as AM;
    use crate::entity::sea_orm_active_enums::EmailFormat;
    use sea_orm::{ActiveModelTrait, Set};
    // Omitted fields fall back to the `Filters` table's schema defaults.
    let am = AM {
        id: Default::default(),
        name: Set(req.name.clone()),
        user_id: Set(req.user_id),
        execute_interval: Set(req.execute_interval.unwrap_or(60)),
        query_json: Set(req.query_json.clone()),
        auto_archive: Set(req.auto_archive.unwrap_or(0)),
        auto_unarchive: Set(req.auto_unarchive.unwrap_or(0)),
        auto_video: Set(req.auto_video.unwrap_or(0)),
        auto_upload: Set(req.auto_upload.unwrap_or(0)),
        auto_email: Set(req.auto_email.unwrap_or(0)),
        email_server: Set(req.email_server.clone()),
        email_to: Set(req.email_to.clone()),
        email_subject: Set(req.email_subject.clone()),
        email_body: Set(req.email_body.clone()),
        email_format: Set(req
            .email_format
            .as_deref()
            .map(parse_email_format)
            .unwrap_or(EmailFormat::Individual)),
        auto_message: Set(req.auto_message.unwrap_or(0)),
        auto_execute: Set(req.auto_execute.unwrap_or(0)),
        auto_execute_cmd: Set(req.auto_execute_cmd.clone()),
        auto_delete: Set(req.auto_delete.unwrap_or(0)),
        auto_move: Set(req.auto_move.unwrap_or(0)),
        auto_copy: Set(req.auto_copy.unwrap_or(0)),
        auto_copy_to: Set(req.auto_copy_to.unwrap_or(0)),
        auto_move_to: Set(req.auto_move_to.unwrap_or(0)),
        update_disk_space: Set(req.update_disk_space.unwrap_or(0)),
        background: Set(req.background.unwrap_or(0)),
        concurrent: Set(req.concurrent.unwrap_or(0)),
        lock_rows: Set(req.lock_rows.unwrap_or(0)),
    };
    Ok(am.insert(db).await?)
}

#[tracing::instrument(skip_all)]
pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Filters::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk_filter(id: u32, name: &str) -> FilterModel {
        use crate::entity::sea_orm_active_enums::EmailFormat;
        FilterModel {
            id,
            name: name.to_string(),
            user_id: Some(1),
            execute_interval: 0,
            query_json: "{}".to_string(),
            auto_archive: 0,
            auto_unarchive: 0,
            auto_video: 0,
            auto_upload: 0,
            auto_email: 0,
            email_server: None,
            email_to: None,
            email_subject: None,
            email_body: None,
            email_format: EmailFormat::Summary,
            auto_message: 0,
            auto_execute: 0,
            auto_execute_cmd: None,
            auto_delete: 0,
            auto_move: 0,
            auto_copy: 0,
            auto_copy_to: 0,
            auto_move_to: 0,
            update_disk_space: 0,
            background: 0,
            concurrent: 0,
            lock_rows: 0,
        }
    }

    #[tokio::test]
    async fn test_find_all_returns_rows() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![vec![
                mk_filter(1, "f1"),
                mk_filter(2, "f2"),
            ]])
            .into_connection();

        let rows = find_all(&db, None).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "f1");
        assert_eq!(rows[1].name, "f2");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let empty: Vec<FilterModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![empty])
            .into_connection();

        let row = find_by_id(&db, 42).await.unwrap();
        assert!(row.is_none());
    }

    #[tokio::test]
    async fn test_update_updates_fields() {
        let initial = mk_filter(7, "old");
        let updated = FilterModel {
            name: "new".to_string(),
            query_json: "{\"k\":\"v\"}".to_string(),
            ..initial.clone()
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            // find_by_id
            .append_query_results::<FilterModel, _, _>(vec![vec![initial]])
            // exec UPDATE
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            // follow-up fetch of updated row
            .append_query_results::<FilterModel, _, _>(vec![vec![updated.clone()]])
            .into_connection();

        let req = crate::dto::request::UpdateFilterRequest {
            name: Some("new".to_string()),
            query_json: Some("{\"k\":\"v\"}".to_string()),
            ..Default::default()
        };
        let res = update(&db, 7, &req).await.unwrap();
        assert!(res.is_some());
        let row = res.unwrap();
        assert_eq!(row.name, "new");
        assert_eq!(row.query_json, "{\"k\":\"v\"}");
    }

    #[tokio::test]
    async fn test_delete_by_id_affects_rows() {
        let db_true = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        assert!(delete_by_id(&db_true, 1).await.unwrap());

        let db_false = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .into_connection();
        assert!(!delete_by_id(&db_false, 1).await.unwrap());
    }
}
