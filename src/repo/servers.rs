use crate::entity::servers::{Entity as Servers, Model as ServerModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ServerModel>> {
    Ok(Servers::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ServerModel>> {
    Ok(Servers::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateServerRequest,
) -> AppResult<ServerModel> {
    use crate::entity::servers::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    fn parse_status(s: &str) -> crate::entity::sea_orm_active_enums::Status {
        use crate::entity::sea_orm_active_enums::Status::*;
        match s.to_lowercase().as_str() {
            "running" => Running,
            "notrunning" => NotRunning,
            _ => Unknown,
        }
    }
    let am = AM {
        id: Default::default(),
        protocol: Set(None),
        hostname: Set(req.hostname.clone()),
        port: Set(req.port),
        path_to_index: Set(None),
        path_to_zms: Set(None),
        path_to_api: Set(None),
        name: Set(req.name.clone()),
        state_id: Set(None),
        status: Set(req
            .status
            .as_deref()
            .map(parse_status)
            .unwrap_or(crate::entity::sea_orm_active_enums::Status::Unknown)),
        cpu_load: Set(None),
        cpu_user_percent: Set(None),
        cpu_nice_percent: Set(None),
        cpu_system_percent: Set(None),
        cpu_idle_percent: Set(None),
        cpu_usage_percent: Set(None),
        total_mem: Set(None),
        free_mem: Set(None),
        total_swap: Set(None),
        free_swap: Set(None),
        zmstats: Set(0),
        zmaudit: Set(0),
        zmtrigger: Set(0),
        zmeventnotification: Set(0),
        latitude: Set(None),
        longitude: Set(None),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    name: Option<String>,
    hostname: Option<String>,
    port: Option<u32>,
    status: Option<String>,
) -> AppResult<Option<ServerModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: crate::entity::servers::ActiveModel = model.into();
    fn parse_status(s: &str) -> crate::entity::sea_orm_active_enums::Status {
        use crate::entity::sea_orm_active_enums::Status::*;
        match s.to_lowercase().as_str() {
            "running" => Running,
            "notrunning" => NotRunning,
            _ => Unknown,
        }
    }
    if let Some(v) = name {
        am.name = Set(v);
    }
    if let Some(v) = hostname {
        am.hostname = Set(Some(v));
    }
    if let Some(v) = port {
        am.port = Set(Some(v));
    }
    if let Some(v) = status {
        am.status = Set(parse_status(&v));
    }
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Servers::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::sea_orm_active_enums::Status;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str) -> ServerModel {
        ServerModel {
            id,
            protocol: None,
            hostname: None,
            port: None,
            path_to_index: None,
            path_to_zms: None,
            path_to_api: None,
            name: name.to_string(),
            state_id: None,
            status: Status::Unknown,
            cpu_load: None,
            cpu_user_percent: None,
            cpu_nice_percent: None,
            cpu_system_percent: None,
            cpu_idle_percent: None,
            cpu_usage_percent: None,
            total_mem: None,
            free_mem: None,
            total_swap: None,
            free_swap: None,
            zmstats: 0,
            zmaudit: 0,
            zmtrigger: 0,
            zmeventnotification: 0,
            latitude: None,
            longitude: None,
        }
    }

    #[tokio::test]
    async fn test_update_happy_path() {
        let initial = mk(10, "old");
        let mut after = initial.clone();
        after.name = "new".into();
        after.hostname = Some("host".into());
        after.status = Status::Running;
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ServerModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ServerModel, _, _>(vec![vec![after.clone()]])
            .into_connection();

        let updated = update(
            &db,
            10,
            Some("new".into()),
            Some("host".into()),
            Some(8080),
            Some("running".into()),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(updated.name, "new");
        assert_eq!(updated.hostname.as_deref(), Some("host"));
        assert_eq!(updated.status, Status::Running);
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
