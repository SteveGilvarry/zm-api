use crate::dto::PaginationParams;
use crate::entity::models::{Entity as Models, Model as ModelModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(
    db: &DatabaseConnection,
    manufacturer_id: Option<u32>,
) -> AppResult<Vec<ModelModel>> {
    let mut query = Models::find();
    if let Some(mid) = manufacturer_id {
        query = query.filter(crate::entity::models::Column::ManufacturerId.eq(mid as i32));
    }
    Ok(query.all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
    manufacturer_id: Option<u32>,
) -> AppResult<(Vec<ModelModel>, u64)> {
    let mut query = Models::find();
    if let Some(mid) = manufacturer_id {
        query = query.filter(crate::entity::models::Column::ManufacturerId.eq(mid as i32));
    }
    let paginator = query.paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ModelModel>> {
    Ok(Models::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateModelRequest,
) -> AppResult<ModelModel> {
    use crate::entity::models::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    let am = AM {
        id: Default::default(),
        name: Set(req.name.clone()),
        manufacturer_id: Set(req.manufacturer_id),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    name: Option<String>,
    manufacturer_id: Option<i32>,
) -> AppResult<Option<ModelModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: crate::entity::models::ActiveModel = model.into();
    if let Some(v) = name {
        am.name = Set(v);
    }
    if let Some(v) = manufacturer_id {
        am.manufacturer_id = Set(Some(v));
    }
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Models::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str, man: Option<i32>) -> ModelModel {
        ModelModel {
            id,
            name: name.to_string(),
            manufacturer_id: man,
        }
    }

    #[tokio::test]
    async fn test_update_happy_path() {
        let initial = mk(2, "old", None);
        let after = mk(2, "new", Some(5));
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ModelModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ModelModel, _, _>(vec![vec![after.clone()]])
            .into_connection();

        let updated = update(&db, 2, Some("new".into()), Some(5))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "new");
        assert_eq!(updated.manufacturer_id, Some(5));
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
