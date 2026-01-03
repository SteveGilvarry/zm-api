use crate::entity::groups::{Entity as Groups, Model as GroupModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<GroupModel>> {
    Ok(Groups::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<GroupModel>> {
    Ok(Groups::find_by_id(id).one(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    name: Option<String>,
) -> AppResult<Option<GroupModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    if let Some(model) = find_by_id(db, id).await? {
        let mut active: crate::entity::groups::ActiveModel = model.into();
        if let Some(n) = name {
            active.name = Set(n);
        }
        let updated = active.update(db).await?;
        Ok(Some(updated))
    } else {
        Ok(None)
    }
}

pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateGroupRequest,
) -> AppResult<GroupModel> {
    use crate::entity::groups::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    let am = AM {
        id: Default::default(),
        name: Set(req.name.clone()),
        parent_id: Set(req.parent_id),
    };
    Ok(am.insert(db).await?)
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Groups::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str) -> GroupModel {
        GroupModel {
            id,
            name: name.into(),
            parent_id: None,
        }
    }

    #[tokio::test]
    async fn test_update_happy_path() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<GroupModel, _, _>(vec![vec![mk(5, "old")]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<GroupModel, _, _>(vec![vec![mk(5, "new")]])
            .into_connection();

        let updated = update(&db, 5, Some("new".into())).await.unwrap().unwrap();
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
