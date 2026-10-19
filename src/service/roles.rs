//! ZoneMinder 1.39 roles (`User_Roles`) (#143).

use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryOrder, Set};

use crate::dto::request::roles::{CreateRoleRequest, UpdateRoleRequest};
use crate::dto::response::roles::RoleResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::entity::sea_orm_active_enums as E;
use crate::entity::user_roles::{ActiveModel, Column, Entity as Roles};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::server::state::AppState;

fn not_found(id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("role_id".into(), id.to_string())],
        resource_type: ResourceType::User,
    })
}

pub async fn list(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<RoleResponse>> {
    let paginator = Roles::find()
        .order_by_asc(Column::Name)
        .paginate(state.db(), params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok(PaginatedResponse::from_params(
        items.iter().map(RoleResponse::from).collect(),
        total,
        params,
    ))
}

pub async fn get(state: &AppState, id: u32) -> AppResult<RoleResponse> {
    Roles::find_by_id(id)
        .one(state.db())
        .await?
        .map(|r| RoleResponse::from(&r))
        .ok_or_else(|| not_found(id))
}

pub async fn create(state: &AppState, req: CreateRoleRequest) -> AppResult<RoleResponse> {
    let p = &req.permissions;
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        description: Set(req.description.clone()),
        stream: Set(p.stream_level()?.unwrap_or(E::Stream::None)),
        events: Set(p.events_level()?.unwrap_or(E::Events::None)),
        control: Set(p.control_level()?.unwrap_or(E::Control::None)),
        monitors: Set(p.monitors_level()?.unwrap_or(E::Monitors::None)),
        groups: Set(p.groups_level()?.unwrap_or(E::Groups::None)),
        devices: Set(p.devices_level()?.unwrap_or(E::Devices::None)),
        snapshots: Set(p.snapshots_level()?.unwrap_or(E::Snapshots::None)),
        system: Set(p.system_level()?.unwrap_or(E::System::None)),
    };
    Ok(RoleResponse::from(&am.insert(state.db()).await?))
}

pub async fn update(state: &AppState, id: u32, req: UpdateRoleRequest) -> AppResult<RoleResponse> {
    let model = Roles::find_by_id(id)
        .one(state.db())
        .await?
        .ok_or_else(|| not_found(id))?;
    let mut am: ActiveModel = model.into();
    if let Some(name) = req.name {
        am.name = Set(name);
    }
    if let Some(description) = req.description {
        am.description = Set(description);
    }
    let p = &req.permissions;
    if let Some(v) = p.stream_level()? {
        am.stream = Set(v);
    }
    if let Some(v) = p.events_level()? {
        am.events = Set(v);
    }
    if let Some(v) = p.control_level()? {
        am.control = Set(v);
    }
    if let Some(v) = p.monitors_level()? {
        am.monitors = Set(v);
    }
    if let Some(v) = p.groups_level()? {
        am.groups = Set(v);
    }
    if let Some(v) = p.devices_level()? {
        am.devices = Set(v);
    }
    if let Some(v) = p.snapshots_level()? {
        am.snapshots = Set(v);
    }
    if let Some(v) = p.system_level()? {
        am.system = Set(v);
    }
    Ok(RoleResponse::from(&am.update(state.db()).await?))
}

/// Delete a role. Its users keep their own levels: `Users.RoleId` is
/// `ON DELETE SET NULL`, and the role's monitor and group rows cascade.
pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let res = Roles::delete_by_id(id).exec(state.db()).await?;
    if res.rows_affected == 0 {
        return Err(not_found(id));
    }
    Ok(())
}
