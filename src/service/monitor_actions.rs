//! ZoneMinder 1.39.30's MonitorActions (#179): what a monitor does to another
//! monitor's device on a trigger, mirroring `web/includes/MonitorAction.php`.
//!
//! An action type is only stored when the target's control can perform it.
//! zmc reads the list at start, so a change restarts the monitor's daemons,
//! as ZoneMinder's monitor editor does.

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, NotSet, QueryFilter, QueryOrder, Set};

use crate::dto::request::monitor_actions::{
    ActionType, CreateMonitorActionRequest, TriggerOn, UpdateMonitorActionRequest,
};
use crate::dto::response::monitor_actions::{ActionTargetResponse, MonitorActionResponse};
use crate::dto::response::ptz::PtzCommandResponse;
use crate::entity::{controls, monitor_actions, monitors};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::service::monitor_daemons;
use crate::util::authz::Level;

fn monitor_not_found(id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("monitor_id".into(), id.to_string())],
        resource_type: ResourceType::Monitor,
    })
}

fn action_not_found(id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("action_id".into(), id.to_string())],
        resource_type: ResourceType::Monitor,
    })
}

/// The action types `monitor` can perform (`MonitorAction::typesForMonitor`):
/// none unless it is controllable and its control has the capability.
pub fn types_for(monitor: &monitors::Model, control: Option<&controls::Model>) -> Vec<ActionType> {
    let Some(c) = control.filter(|_| monitor.controllable != 0 && monitor.control_id.is_some())
    else {
        return Vec::new();
    };
    let mut types = Vec::new();
    if c.can_light != 0 {
        types.extend([ActionType::LightOn, ActionType::LightOff]);
    }
    if c.can_indicator_light != 0 {
        types.extend([ActionType::IndicatorLightOn, ActionType::IndicatorLightOff]);
    }
    if c.can_audio_play != 0 {
        types.extend([ActionType::AudioPlay, ActionType::AudioStop]);
    }
    types
}

async fn live_monitor(state: &AppState, id: u32) -> AppResult<monitors::Model> {
    monitors::Entity::find_by_id(id)
        .one(state.db())
        .await?
        .filter(|m| m.deleted == 0)
        .ok_or_else(|| monitor_not_found(id))
}

async fn find_action(
    state: &AppState,
    monitor_id: u32,
    action_id: u32,
) -> AppResult<monitor_actions::Model> {
    monitor_actions::Entity::find_by_id(action_id)
        .filter(monitor_actions::Column::MonitorId.eq(monitor_id))
        .one(state.db())
        .await?
        .ok_or_else(|| action_not_found(action_id))
}

/// Refuse an action the target can't perform, an `AudioPlay` file outside
/// the control's range, or a target the caller can't see.
async fn check_target(
    state: &AppState,
    scope: &MonitorScope,
    target: u32,
    action: ActionType,
    audio_file: Option<u32>,
) -> AppResult<()> {
    let bad = |msg: String| Err(AppError::BadRequestError(msg));
    let Some((monitor, control)) =
        crate::repo::ptz::get_monitor_with_control(state.db(), target).await?
    else {
        return bad(format!("target monitor {target} does not exist"));
    };
    if monitor.deleted != 0 || !scope.allows(target, Level::View) {
        return bad(format!("target monitor {target} does not exist"));
    }
    if !types_for(&monitor, control.as_ref()).contains(&action) {
        return bad(format!(
            "monitor {target} has no control that can {}",
            action.as_str()
        ));
    }
    if let (ActionType::AudioPlay, Some(file), Some(c)) = (action, audio_file, control) {
        if let (Some(min), Some(max)) = (c.min_audio_file, c.max_audio_file) {
            if !(min..=max).contains(&file) {
                return bad(format!("audio_file must be within {min}..={max}"));
            }
        }
    }
    Ok(())
}

/// zmc reads its action list at start.
async fn restart(state: &AppState, monitor: &monitors::Model) {
    if monitor_daemons::should_run(monitor) {
        monitor_daemons::set_state(state, monitor, monitor_daemons::Action::Restart).await;
    }
}

pub async fn list(state: &AppState, monitor_id: u32) -> AppResult<Vec<MonitorActionResponse>> {
    live_monitor(state, monitor_id).await?;
    Ok(monitor_actions::Entity::find()
        .filter(monitor_actions::Column::MonitorId.eq(monitor_id))
        .order_by_asc(monitor_actions::Column::Sequence)
        .order_by_asc(monitor_actions::Column::Id)
        .all(state.db())
        .await?
        .iter()
        .map(MonitorActionResponse::from)
        .collect())
}

pub async fn get(
    state: &AppState,
    monitor_id: u32,
    action_id: u32,
) -> AppResult<MonitorActionResponse> {
    Ok(MonitorActionResponse::from(
        &find_action(state, monitor_id, action_id).await?,
    ))
}

pub async fn create(
    state: &AppState,
    scope: &MonitorScope,
    monitor_id: u32,
    req: CreateMonitorActionRequest,
) -> AppResult<MonitorActionResponse> {
    let monitor = live_monitor(state, monitor_id).await?;
    check_target(
        state,
        scope,
        req.target_monitor_id,
        req.action_type,
        req.audio_file,
    )
    .await?;
    let am = monitor_actions::ActiveModel {
        id: NotSet,
        monitor_id: Set(monitor_id),
        trigger_on: Set(req
            .trigger_on
            .unwrap_or(TriggerOn::EventStart)
            .as_str()
            .into()),
        action_type: Set(req.action_type.as_str().into()),
        target_monitor_id: Set(req.target_monitor_id),
        audio_file: Set(req.audio_file),
        label: Set(req.label.unwrap_or_default()),
        enabled: Set(u8::from(req.enabled.unwrap_or(true))),
        sequence: Set(req.sequence.unwrap_or(0)),
    };
    let saved = am.insert(state.db()).await?;
    restart(state, &monitor).await;
    Ok(MonitorActionResponse::from(&saved))
}

pub async fn update(
    state: &AppState,
    scope: &MonitorScope,
    monitor_id: u32,
    action_id: u32,
    req: UpdateMonitorActionRequest,
) -> AppResult<MonitorActionResponse> {
    let monitor = live_monitor(state, monitor_id).await?;
    let row = find_action(state, monitor_id, action_id).await?;
    let target = req.target_monitor_id.unwrap_or(row.target_monitor_id);
    let action = match req.action_type {
        Some(a) => a,
        None => ActionType::parse(&row.action_type).ok_or_else(|| {
            AppError::BadRequestError(format!("unknown stored action type {}", row.action_type))
        })?,
    };
    let audio_file = req.audio_file.unwrap_or(row.audio_file);
    check_target(state, scope, target, action, audio_file).await?;

    let mut am: monitor_actions::ActiveModel = row.into();
    am.target_monitor_id = Set(target);
    am.action_type = Set(action.as_str().into());
    am.audio_file = Set(audio_file);
    if let Some(t) = req.trigger_on {
        am.trigger_on = Set(t.as_str().into());
    }
    if let Some(v) = req.label {
        am.label = Set(v);
    }
    if let Some(v) = req.enabled {
        am.enabled = Set(u8::from(v));
    }
    if let Some(v) = req.sequence {
        am.sequence = Set(v);
    }
    let saved = am.update(state.db()).await?;
    restart(state, &monitor).await;
    Ok(MonitorActionResponse::from(&saved))
}

pub async fn delete(state: &AppState, monitor_id: u32, action_id: u32) -> AppResult<()> {
    let monitor = live_monitor(state, monitor_id).await?;
    let res = monitor_actions::Entity::delete_many()
        .filter(monitor_actions::Column::Id.eq(action_id))
        .filter(monitor_actions::Column::MonitorId.eq(monitor_id))
        .exec(state.db())
        .await?;
    if res.rows_affected == 0 {
        return Err(action_not_found(action_id));
    }
    restart(state, &monitor).await;
    Ok(())
}

/// The monitors an action can drive, among those the caller can see.
pub async fn targets(
    state: &AppState,
    scope: &MonitorScope,
) -> AppResult<Vec<ActionTargetResponse>> {
    let candidates = monitors::Entity::find()
        .filter(monitors::Column::Deleted.eq(0))
        .filter(monitors::Column::Controllable.ne(0))
        .filter(monitors::Column::ControlId.is_not_null())
        .order_by_asc(monitors::Column::Sequence)
        .order_by_asc(monitors::Column::Id)
        .all(state.db())
        .await?;
    let ids: Vec<u32> = candidates.iter().filter_map(|m| m.control_id).collect();
    let controls = controls::Entity::find()
        .filter(controls::Column::Id.is_in(ids))
        .all(state.db())
        .await?;
    Ok(candidates
        .into_iter()
        .filter(|m| scope.allows(m.id, Level::View))
        .filter_map(|m| {
            let c = controls.iter().find(|c| Some(c.id) == m.control_id);
            let types = types_for(&m, c);
            (!types.is_empty()).then(|| ActionTargetResponse {
                id: m.id,
                name: m.name.clone(),
                device_class: format!("{:?}", m.device_class),
                types,
                min_audio_file: c.and_then(|c| c.min_audio_file),
                max_audio_file: c.and_then(|c| c.max_audio_file),
            })
        })
        .collect())
}

/// Run an enabled `Manual` action. The command is built from the stored
/// row, never the request, and the caller needs Control on both the monitor
/// and the target (ZoneMinder's `ajax/control.php`).
pub async fn run(
    state: &AppState,
    scope: &MonitorScope,
    monitor_id: u32,
    action_id: u32,
) -> AppResult<PtzCommandResponse> {
    if !scope.allows(monitor_id, Level::View) {
        return Err(monitor_not_found(monitor_id));
    }
    let row = find_action(state, monitor_id, action_id).await?;
    if row.trigger_on != TriggerOn::Manual.as_str() || row.enabled == 0 {
        return Err(AppError::BadRequestError(format!(
            "action {action_id} is not an enabled Manual action"
        )));
    }
    let target = row.target_monitor_id;
    if !scope.allows(target, Level::View) {
        return Err(monitor_not_found(target));
    }
    let pm = state.ptz_manager();
    use crate::service::ptz;
    match ActionType::parse(&row.action_type) {
        Some(ActionType::LightOn) => ptz::light(state, pm, target, false, true).await,
        Some(ActionType::LightOff) => ptz::light(state, pm, target, false, false).await,
        Some(ActionType::IndicatorLightOn) => ptz::light(state, pm, target, true, true).await,
        Some(ActionType::IndicatorLightOff) => ptz::light(state, pm, target, true, false).await,
        Some(ActionType::AudioPlay) => {
            // No file stored: the control's first, where ZoneMinder sends none.
            let file = match row.audio_file {
                Some(f) => Some(f),
                None => crate::repo::ptz::get_monitor_with_control(state.db(), target)
                    .await?
                    .and_then(|(_, c)| c)
                    .and_then(|c| c.min_audio_file),
            };
            let file = file.ok_or_else(|| {
                AppError::BadRequestError(format!("action {action_id} names no audio file"))
            })?;
            ptz::audio(state, pm, target, Some(file)).await
        }
        Some(ActionType::AudioStop) => ptz::audio(state, pm, target, None).await,
        None => Err(AppError::BadRequestError(format!(
            "unknown action type {}",
            row.action_type
        ))),
    }
}
