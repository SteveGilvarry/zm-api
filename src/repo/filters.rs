use crate::dto::PaginationParams;
use crate::entity::filters::{Entity as Filters, Model as FilterModel};
use crate::error::AppResult;
use sea_orm::*;

// Repos accept a database connection (preferred)
#[tracing::instrument(skip_all)]
pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<FilterModel>> {
    Ok(Filters::find().all(db).await?)
}

#[tracing::instrument(skip_all)]
pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<FilterModel>, u64)> {
    let paginator = Filters::find().paginate(db, params.page_size());
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
    name: Option<String>,
    query_json: Option<String>,
) -> AppResult<Option<FilterModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    if let Some(model) = find_by_id(db, id).await? {
        let mut active: crate::entity::filters::ActiveModel = model.into();
        if let Some(n) = name {
            active.name = Set(n);
        }
        if let Some(q) = query_json {
            active.query_json = Set(q);
        }
        let updated = active.update(db).await?;
        Ok(Some(updated))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateFilterRequest,
) -> AppResult<FilterModel> {
    use crate::entity::filters::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    fn parse_email_format(s: &str) -> crate::entity::sea_orm_active_enums::EmailFormat {
        use crate::entity::sea_orm_active_enums::EmailFormat::*;
        match s.to_lowercase().as_str() {
            "individual" => Individual,
            _ => Summary,
        }
    }
    let am = AM {
        id: Default::default(),
        name: Set(req.name.clone()),
        user_id: Set(req.user_id),
        execute_interval: Set(req.execute_interval.unwrap_or(0)),
        query_json: Set(req.query_json.clone()),
        auto_archive: Set(0),
        auto_unarchive: Set(0),
        auto_video: Set(0),
        auto_upload: Set(0),
        auto_email: Set(0),
        email_to: Set(None),
        email_subject: Set(None),
        email_body: Set(None),
        email_format: Set(req
            .email_format
            .as_deref()
            .map(parse_email_format)
            .unwrap_or(crate::entity::sea_orm_active_enums::EmailFormat::Summary)),
        auto_message: Set(0),
        auto_execute: Set(0),
        auto_execute_cmd: Set(None),
        auto_delete: Set(0),
        auto_move: Set(0),
        auto_copy: Set(0),
        auto_copy_to: Set(0),
        auto_move_to: Set(0),
        update_disk_space: Set(0),
        background: Set(0),
        concurrent: Set(0),
        lock_rows: Set(0),
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

        let rows = find_all(&db).await.unwrap();
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

        let res = update(
            &db,
            7,
            Some("new".to_string()),
            Some("{\"k\":\"v\"}".to_string()),
        )
        .await
        .unwrap();
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
