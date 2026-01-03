use crate::dto::request::groups_monitors::CreateGroupMonitorRequest;
use crate::entity::groups_monitors::{
    ActiveModel, Column, Entity as GroupsMonitors, Model as GroupMonitorModel,
};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<GroupMonitorModel>> {
    Ok(GroupsMonitors::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<GroupMonitorModel>> {
    Ok(GroupsMonitors::find_by_id(id).one(db).await?)
}

pub async fn find_by_group_id(
    db: &DatabaseConnection,
    group_id: u32,
) -> AppResult<Vec<GroupMonitorModel>> {
    Ok(GroupsMonitors::find()
        .filter(Column::GroupId.eq(group_id))
        .all(db)
        .await?)
}

pub async fn find_by_monitor_id(
    db: &DatabaseConnection,
    monitor_id: u32,
) -> AppResult<Vec<GroupMonitorModel>> {
    Ok(GroupsMonitors::find()
        .filter(Column::MonitorId.eq(monitor_id))
        .all(db)
        .await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateGroupMonitorRequest,
) -> AppResult<GroupMonitorModel> {
    let am = ActiveModel {
        id: Default::default(),
        group_id: Set(req.group_id),
        monitor_id: Set(req.monitor_id),
    };
    Ok(am.insert(db).await?)
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = GroupsMonitors::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
