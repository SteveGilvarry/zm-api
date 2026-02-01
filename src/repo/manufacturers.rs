use crate::dto::PaginationParams;
use crate::entity::manufacturers::{Entity as Manufacturers, Model as ManufacturerModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ManufacturerModel>> {
    Ok(Manufacturers::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<ManufacturerModel>, u64)> {
    let paginator = Manufacturers::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ManufacturerModel>> {
    Ok(Manufacturers::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateManufacturerRequest,
) -> AppResult<ManufacturerModel> {
    use crate::entity::manufacturers::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    let am = AM {
        id: Default::default(),
        name: Set(req.name.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    name: Option<String>,
) -> AppResult<Option<ManufacturerModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: crate::entity::manufacturers::ActiveModel = model.into();
    if let Some(v) = name {
        am.name = Set(v);
    }
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Manufacturers::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str) -> ManufacturerModel {
        ManufacturerModel {
            id,
            name: name.to_string(),
        }
    }

    #[tokio::test]
    async fn test_update_happy_path() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ManufacturerModel, _, _>(vec![vec![mk(1, "old")]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ManufacturerModel, _, _>(vec![vec![mk(1, "new")]])
            .into_connection();

        let updated = update(&db, 1, Some("new".into())).await.unwrap().unwrap();
        assert_eq!(updated.name, "new");
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
