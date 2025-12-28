use sea_orm::*;
use crate::entity::users::{Entity as Users, Model as UserModel};
use crate::error::AppResult;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<UserModel>> {
    Ok(Users::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<UserModel>> {
    Ok(Users::find_by_id(id).one(db).await?)
}

// Find a user by username and enabled flag (ZoneMinder Users table)
#[tracing::instrument(skip_all)]
pub async fn find_by_username_and_status(
    db: &DatabaseConnection,
    username: &str,
    is_enabled: bool,
) -> AppResult<Option<UserModel>> {
    Ok(
        Users::find()
            .filter(
                crate::entity::users::Column::Username
                    .eq(username)
                    .and(crate::entity::users::Column::Enabled.eq(is_enabled)),
            )
            .one(db)
            .await?,
    )
}

pub async fn update(db: &DatabaseConnection, id: u32, email: Option<String>, enabled: Option<u8>) -> AppResult<Option<UserModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    if let Some(model) = find_by_id(db, id).await? {
        let mut active: crate::entity::users::ActiveModel = model.into();
        if let Some(e) = email { active.email = Set(e); }
        if let Some(en) = enabled { active.enabled = Set(en); }
        let updated = active.update(db).await?;
        Ok(Some(updated))
    } else { Ok(None) }
}

pub async fn create(db: &DatabaseConnection, req: &crate::dto::request::CreateUserRequest) -> AppResult<UserModel> {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entity::users::ActiveModel as AM;
    use crate::entity::sea_orm_active_enums as E;
    let am = AM {
        id: Default::default(),
        username: Set(req.username.clone()),
        password: Set(req.password.clone()),
        name: Set(req.name.clone().unwrap_or_default()),
        email: Set(req.email.clone()),
        phone: Set(req.phone.clone().unwrap_or_default()),
        language: Set(None),
        enabled: Set(req.enabled.unwrap_or(1)),
        stream: Set(E::Stream::View),
        events: Set(E::Events::View),
        control: Set(E::Control::View),
        monitors: Set(E::Monitors::View),
        groups: Set(E::Groups::View),
        devices: Set(E::Devices::View),
        snapshots: Set(E::Snapshots::View),
        system: Set(E::System::View),
        max_bandwidth: Set(None),
        token_min_expiry: Set(0),
        api_enabled: Set(1),
        home_view: Set("console".to_string()),
    };
    Ok(am.insert(db).await?)
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Users::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use crate::entity::sea_orm_active_enums as E;

    fn mk_user(id: u32, username: &str, enabled: u8) -> UserModel {
        UserModel {
            id,
            username: username.to_string(),
            password: "pass".to_string(),
            name: "Name".to_string(),
            email: "user@example.com".to_string(),
            phone: "".to_string(),
            language: None,
            enabled,
            stream: E::Stream::View,
            events: E::Events::View,
            control: E::Control::View,
            monitors: E::Monitors::View,
            groups: E::Groups::View,
            devices: E::Devices::View,
            snapshots: E::Snapshots::View,
            system: E::System::View,
            max_bandwidth: None,
            token_min_expiry: 0,
            api_enabled: 1,
            home_view: "console".to_string(),
        }
    }

    #[tokio::test]
    async fn test_find_by_username_and_status_found() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![mk_user(1, "admin", 1)]])
            .into_connection();

        let res = find_by_username_and_status(&db, "admin", true).await.unwrap();
        assert!(res.is_some());
        let user = res.unwrap();
        assert_eq!(user.username, "admin");
        assert_eq!(user.enabled, 1);
    }

    #[tokio::test]
    async fn test_find_by_username_and_status_not_found() {
        let empty: Vec<UserModel> = Vec::new();
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<UserModel, _, _>(vec![empty])
            .into_connection();

        let res = find_by_username_and_status(&db, "missing", true).await.unwrap();
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_update_happy_path() {
        let initial = mk_user(42, "user", 1);
        let mut after = initial.clone();
        after.email = "new@example.com".into();
        after.enabled = 0;
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<UserModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .append_query_results::<UserModel, _, _>(vec![vec![after.clone()]])
            .into_connection();

        let updated = update(&db, 42, Some("new@example.com".into()), Some(0)).await.unwrap().unwrap();
        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.enabled, 0);
    }

    #[tokio::test]
    async fn test_delete_by_id_affects_rows() {
        let db_true = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .into_connection();
        assert!(delete_by_id(&db_true, 1).await.unwrap());

        let db_false = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 0 }])
            .into_connection();
        assert!(!delete_by_id(&db_false, 1).await.unwrap());
    }
}
