use crate::dto::PaginationParams;
use crate::entity::storage::{Entity as Storage, Model as StorageModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<StorageModel>> {
    Ok(Storage::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<StorageModel>, u64)> {
    let paginator = Storage::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u16) -> AppResult<Option<StorageModel>> {
    Ok(Storage::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateStorageRequest,
) -> AppResult<StorageModel> {
    use crate::entity::storage::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    fn parse_type(s: &str) -> crate::entity::sea_orm_active_enums::StorageType {
        use crate::entity::sea_orm_active_enums::StorageType::*;
        match s.to_lowercase().as_str() {
            "s3fs" => S3fs,
            _ => Local,
        }
    }
    fn parse_scheme(s: &str) -> crate::entity::sea_orm_active_enums::Scheme {
        use crate::entity::sea_orm_active_enums::Scheme::*;
        match s.to_lowercase().as_str() {
            "medium" => Medium,
            "shallow" => Shallow,
            _ => Deep,
        }
    }
    let am = AM {
        id: Default::default(),
        path: Set(req.path.clone()),
        name: Set(req.name.clone()),
        r#type: Set(parse_type(&req.r#type)),
        url: Set(req.url.clone()),
        disk_space: Set(None),
        scheme: Set(req
            .scheme
            .as_deref()
            .map(parse_scheme)
            .unwrap_or(crate::entity::sea_orm_active_enums::Scheme::Deep)),
        server_id: Set(req.server_id),
        do_delete: Set(0),
        enabled: Set(req.enabled),
    };
    Ok(am.insert(db).await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    db: &DatabaseConnection,
    id: u16,
    name: Option<String>,
    path: Option<String>,
    r#type: Option<String>,
    enabled: Option<i8>,
    scheme: Option<String>,
    server_id: Option<u32>,
    url: Option<String>,
) -> AppResult<Option<StorageModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: crate::entity::storage::ActiveModel = model.into();
    fn parse_type(s: &str) -> crate::entity::sea_orm_active_enums::StorageType {
        use crate::entity::sea_orm_active_enums::StorageType::*;
        match s.to_lowercase().as_str() {
            "s3fs" => S3fs,
            _ => Local,
        }
    }
    fn parse_scheme(s: &str) -> crate::entity::sea_orm_active_enums::Scheme {
        use crate::entity::sea_orm_active_enums::Scheme::*;
        match s.to_lowercase().as_str() {
            "medium" => Medium,
            "shallow" => Shallow,
            _ => Deep,
        }
    }
    if let Some(v) = name {
        am.name = Set(v);
    }
    if let Some(v) = path {
        am.path = Set(v);
    }
    if let Some(v) = r#type {
        am.r#type = Set(parse_type(&v));
    }
    if let Some(v) = enabled {
        am.enabled = Set(v);
    }
    if let Some(v) = scheme {
        am.scheme = Set(parse_scheme(&v));
    }
    if let Some(v) = server_id {
        am.server_id = Set(Some(v));
    }
    if let Some(v) = url {
        am.url = Set(Some(v));
    }
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u16) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Storage::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::sea_orm_active_enums::{Scheme, StorageType};
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u16, name: &str) -> StorageModel {
        StorageModel {
            id,
            path: "/tmp".to_string(),
            name: name.to_string(),
            r#type: StorageType::Local,
            url: None,
            disk_space: None,
            scheme: Scheme::Deep,
            server_id: None,
            do_delete: 0,
            enabled: 1,
        }
    }

    #[tokio::test]
    async fn test_update_happy_path() {
        let initial = mk(3, "old");
        let after = StorageModel {
            name: "new".to_string(),
            path: "/data".to_string(),
            ..initial.clone()
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<StorageModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<StorageModel, _, _>(vec![vec![after.clone()]])
            .into_connection();

        let updated = update(
            &db,
            3,
            Some("new".into()),
            Some("/data".into()),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(updated.name, "new");
        assert_eq!(updated.path, "/data");
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
