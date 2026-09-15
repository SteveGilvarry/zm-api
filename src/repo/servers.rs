use crate::dto::PaginationParams;
use crate::entity::servers::{Entity as Servers, Model as ServerModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ServerModel>> {
    Ok(Servers::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<ServerModel>, u64)> {
    let paginator = Servers::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ServerModel>> {
    Ok(Servers::find_by_id(id).one(db).await?)
}

fn parse_status(s: &str) -> crate::entity::sea_orm_active_enums::Status {
    use crate::entity::sea_orm_active_enums::Status::*;
    match s.to_lowercase().as_str() {
        "running" => Running,
        "notrunning" => NotRunning,
        _ => Unknown,
    }
}

fn to_decimal(v: f64, places: u32) -> Option<rust_decimal::Decimal> {
    rust_decimal::Decimal::from_f64_retain(v).map(|d| d.round_dp(places))
}

pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateServerRequest,
) -> AppResult<ServerModel> {
    use crate::entity::servers::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    let flag = |v: Option<bool>| i8::from(v.unwrap_or(false));
    let am = AM {
        id: Default::default(),
        protocol: Set(req.protocol.clone()),
        hostname: Set(req.hostname.clone()),
        port: Set(req.port),
        path_to_index: Set(req.path_to_index.clone()),
        path_to_zms: Set(req.path_to_zms.clone()),
        path_to_api: Set(req.path_to_api.clone()),
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
        zmstats: Set(flag(req.zmstats)),
        zmaudit: Set(flag(req.zmaudit)),
        zmtrigger: Set(flag(req.zmtrigger)),
        zmeventnotification: Set(flag(req.zmeventnotification)),
        // Servers.Latitude is decimal(10,8), Longitude decimal(11,8).
        latitude: Set(req.latitude.and_then(|v| to_decimal(v, 8))),
        longitude: Set(req.longitude.and_then(|v| to_decimal(v, 8))),
    };
    Ok(am.insert(db).await?)
}

/// Apply a partial update: fields left out are kept, `null` clears nullable ones.
pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &crate::dto::request::servers::UpdateServerRequest,
) -> AppResult<Option<ServerModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: crate::entity::servers::ActiveModel = model.into();
    if let Some(v) = &req.name {
        am.name = Set(v.clone());
    }
    if let Some(v) = &req.hostname {
        am.hostname = Set(v.clone());
    }
    if let Some(v) = req.port {
        am.port = Set(v);
    }
    if let Some(v) = &req.status {
        am.status = Set(parse_status(v));
    }
    if let Some(v) = &req.protocol {
        am.protocol = Set(v.clone());
    }
    if let Some(v) = &req.path_to_index {
        am.path_to_index = Set(v.clone());
    }
    if let Some(v) = &req.path_to_zms {
        am.path_to_zms = Set(v.clone());
    }
    if let Some(v) = &req.path_to_api {
        am.path_to_api = Set(v.clone());
    }
    for (value, column) in [
        (req.zmstats, &mut am.zmstats),
        (req.zmaudit, &mut am.zmaudit),
        (req.zmtrigger, &mut am.zmtrigger),
        (req.zmeventnotification, &mut am.zmeventnotification),
    ] {
        if let Some(v) = value {
            *column = Set(i8::from(v));
        }
    }
    if let Some(v) = req.latitude {
        am.latitude = Set(v.and_then(|x| to_decimal(x, 8)));
    }
    if let Some(v) = req.longitude {
        am.longitude = Set(v.and_then(|x| to_decimal(x, 8)));
    }
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

/// How many monitors each server has, keyed by server id. Servers with none
/// are absent.
pub async fn monitor_counts(
    db: &DatabaseConnection,
) -> AppResult<std::collections::HashMap<u32, u64>> {
    use crate::entity::monitors;
    #[derive(FromQueryResult)]
    struct Row {
        server_id: u32,
        count: i64,
    }
    let rows = monitors::Entity::find()
        .select_only()
        .column_as(monitors::Column::ServerId, "server_id")
        .column_as(monitors::Column::Id.count(), "count")
        .filter(monitors::Column::ServerId.is_not_null())
        .group_by(monitors::Column::ServerId)
        .into_model::<Row>()
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.server_id, r.count.max(0) as u64))
        .collect())
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
            &crate::dto::request::servers::UpdateServerRequest {
                name: Some("new".into()),
                hostname: Some(Some("host".into())),
                port: Some(Some(8080)),
                status: Some("running".into()),
                ..Default::default()
            },
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
