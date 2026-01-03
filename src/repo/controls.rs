use crate::dto::request::controls::{CreateControlRequest, UpdateControlRequest};
use crate::entity::controls::{ActiveModel, Entity as Controls, Model as ControlModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ControlModel>> {
    Ok(Controls::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ControlModel>> {
    Ok(Controls::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateControlRequest,
) -> AppResult<ControlModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        r#type: Set(req.r#type.clone()),
        protocol: Set(req.protocol.clone()),
        can_wake: Set(req.can_wake.unwrap_or(0)),
        can_sleep: Set(req.can_sleep.unwrap_or(0)),
        can_reset: Set(req.can_reset.unwrap_or(0)),
        can_reboot: Set(req.can_reboot.unwrap_or(0)),
        can_zoom: Set(req.can_zoom.unwrap_or(0)),
        can_auto_zoom: Set(req.can_auto_zoom.unwrap_or(0)),
        can_zoom_abs: Set(req.can_zoom_abs.unwrap_or(0)),
        can_zoom_rel: Set(req.can_zoom_rel.unwrap_or(0)),
        can_zoom_con: Set(req.can_zoom_con.unwrap_or(0)),
        min_zoom_range: Set(req.min_zoom_range),
        max_zoom_range: Set(req.max_zoom_range),
        min_zoom_step: Set(req.min_zoom_step),
        max_zoom_step: Set(req.max_zoom_step),
        has_zoom_speed: Set(req.has_zoom_speed.unwrap_or(0)),
        min_zoom_speed: Set(req.min_zoom_speed),
        max_zoom_speed: Set(req.max_zoom_speed),
        can_focus: Set(req.can_focus.unwrap_or(0)),
        can_auto_focus: Set(req.can_auto_focus.unwrap_or(0)),
        can_focus_abs: Set(req.can_focus_abs.unwrap_or(0)),
        can_focus_rel: Set(req.can_focus_rel.unwrap_or(0)),
        can_focus_con: Set(req.can_focus_con.unwrap_or(0)),
        min_focus_range: Set(req.min_focus_range),
        max_focus_range: Set(req.max_focus_range),
        min_focus_step: Set(req.min_focus_step),
        max_focus_step: Set(req.max_focus_step),
        has_focus_speed: Set(req.has_focus_speed.unwrap_or(0)),
        min_focus_speed: Set(req.min_focus_speed),
        max_focus_speed: Set(req.max_focus_speed),
        can_iris: Set(req.can_iris.unwrap_or(0)),
        can_auto_iris: Set(req.can_auto_iris.unwrap_or(0)),
        can_iris_abs: Set(req.can_iris_abs.unwrap_or(0)),
        can_iris_rel: Set(req.can_iris_rel.unwrap_or(0)),
        can_iris_con: Set(req.can_iris_con.unwrap_or(0)),
        min_iris_range: Set(req.min_iris_range),
        max_iris_range: Set(req.max_iris_range),
        min_iris_step: Set(req.min_iris_step),
        max_iris_step: Set(req.max_iris_step),
        has_iris_speed: Set(req.has_iris_speed.unwrap_or(0)),
        min_iris_speed: Set(req.min_iris_speed),
        max_iris_speed: Set(req.max_iris_speed),
        can_gain: Set(req.can_gain.unwrap_or(0)),
        can_auto_gain: Set(req.can_auto_gain.unwrap_or(0)),
        can_gain_abs: Set(req.can_gain_abs.unwrap_or(0)),
        can_gain_rel: Set(req.can_gain_rel.unwrap_or(0)),
        can_gain_con: Set(req.can_gain_con.unwrap_or(0)),
        min_gain_range: Set(req.min_gain_range),
        max_gain_range: Set(req.max_gain_range),
        min_gain_step: Set(req.min_gain_step),
        max_gain_step: Set(req.max_gain_step),
        has_gain_speed: Set(req.has_gain_speed.unwrap_or(0)),
        min_gain_speed: Set(req.min_gain_speed),
        max_gain_speed: Set(req.max_gain_speed),
        can_white: Set(req.can_white.unwrap_or(0)),
        can_auto_white: Set(req.can_auto_white.unwrap_or(0)),
        can_white_abs: Set(req.can_white_abs.unwrap_or(0)),
        can_white_rel: Set(req.can_white_rel.unwrap_or(0)),
        can_white_con: Set(req.can_white_con.unwrap_or(0)),
        min_white_range: Set(req.min_white_range),
        max_white_range: Set(req.max_white_range),
        min_white_step: Set(req.min_white_step),
        max_white_step: Set(req.max_white_step),
        has_white_speed: Set(req.has_white_speed.unwrap_or(0)),
        min_white_speed: Set(req.min_white_speed),
        max_white_speed: Set(req.max_white_speed),
        has_presets: Set(req.has_presets.unwrap_or(0)),
        num_presets: Set(req.num_presets.unwrap_or(0)),
        has_home_preset: Set(req.has_home_preset.unwrap_or(0)),
        can_set_presets: Set(req.can_set_presets.unwrap_or(0)),
        can_move: Set(req.can_move.unwrap_or(0)),
        can_move_diag: Set(req.can_move_diag.unwrap_or(0)),
        can_move_map: Set(req.can_move_map.unwrap_or(0)),
        can_move_abs: Set(req.can_move_abs.unwrap_or(0)),
        can_move_rel: Set(req.can_move_rel.unwrap_or(0)),
        can_move_con: Set(req.can_move_con.unwrap_or(0)),
        can_pan: Set(req.can_pan.unwrap_or(0)),
        min_pan_range: Set(req.min_pan_range),
        max_pan_range: Set(req.max_pan_range),
        min_pan_step: Set(req.min_pan_step),
        max_pan_step: Set(req.max_pan_step),
        has_pan_speed: Set(req.has_pan_speed.unwrap_or(0)),
        min_pan_speed: Set(req.min_pan_speed),
        max_pan_speed: Set(req.max_pan_speed),
        has_turbo_pan: Set(req.has_turbo_pan.unwrap_or(0)),
        turbo_pan_speed: Set(req.turbo_pan_speed),
        can_tilt: Set(req.can_tilt.unwrap_or(0)),
        min_tilt_range: Set(req.min_tilt_range),
        max_tilt_range: Set(req.max_tilt_range),
        min_tilt_step: Set(req.min_tilt_step),
        max_tilt_step: Set(req.max_tilt_step),
        has_tilt_speed: Set(req.has_tilt_speed.unwrap_or(0)),
        min_tilt_speed: Set(req.min_tilt_speed),
        max_tilt_speed: Set(req.max_tilt_speed),
        has_turbo_tilt: Set(req.has_turbo_tilt.unwrap_or(0)),
        turbo_tilt_speed: Set(req.turbo_tilt_speed),
        can_auto_scan: Set(req.can_auto_scan.unwrap_or(0)),
        num_scan_paths: Set(req.num_scan_paths.unwrap_or(0)),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateControlRequest,
) -> AppResult<Option<ControlModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = &req.name {
        am.name = Set(v.clone());
    }
    if let Some(v) = &req.r#type {
        am.r#type = Set(v.clone());
    }
    if let Some(v) = &req.protocol {
        am.protocol = Set(Some(v.clone()));
    }
    if let Some(v) = req.can_wake {
        am.can_wake = Set(v);
    }
    if let Some(v) = req.can_sleep {
        am.can_sleep = Set(v);
    }
    if let Some(v) = req.can_reset {
        am.can_reset = Set(v);
    }
    if let Some(v) = req.can_reboot {
        am.can_reboot = Set(v);
    }
    if let Some(v) = req.can_zoom {
        am.can_zoom = Set(v);
    }
    if let Some(v) = req.can_auto_zoom {
        am.can_auto_zoom = Set(v);
    }
    if let Some(v) = req.can_zoom_abs {
        am.can_zoom_abs = Set(v);
    }
    if let Some(v) = req.can_zoom_rel {
        am.can_zoom_rel = Set(v);
    }
    if let Some(v) = req.can_zoom_con {
        am.can_zoom_con = Set(v);
    }
    if let Some(v) = req.min_zoom_range {
        am.min_zoom_range = Set(Some(v));
    }
    if let Some(v) = req.max_zoom_range {
        am.max_zoom_range = Set(Some(v));
    }
    if let Some(v) = req.min_zoom_step {
        am.min_zoom_step = Set(Some(v));
    }
    if let Some(v) = req.max_zoom_step {
        am.max_zoom_step = Set(Some(v));
    }
    if let Some(v) = req.has_zoom_speed {
        am.has_zoom_speed = Set(v);
    }
    if let Some(v) = req.min_zoom_speed {
        am.min_zoom_speed = Set(Some(v));
    }
    if let Some(v) = req.max_zoom_speed {
        am.max_zoom_speed = Set(Some(v));
    }
    if let Some(v) = req.can_focus {
        am.can_focus = Set(v);
    }
    if let Some(v) = req.can_auto_focus {
        am.can_auto_focus = Set(v);
    }
    if let Some(v) = req.can_focus_abs {
        am.can_focus_abs = Set(v);
    }
    if let Some(v) = req.can_focus_rel {
        am.can_focus_rel = Set(v);
    }
    if let Some(v) = req.can_focus_con {
        am.can_focus_con = Set(v);
    }
    if let Some(v) = req.min_focus_range {
        am.min_focus_range = Set(Some(v));
    }
    if let Some(v) = req.max_focus_range {
        am.max_focus_range = Set(Some(v));
    }
    if let Some(v) = req.min_focus_step {
        am.min_focus_step = Set(Some(v));
    }
    if let Some(v) = req.max_focus_step {
        am.max_focus_step = Set(Some(v));
    }
    if let Some(v) = req.has_focus_speed {
        am.has_focus_speed = Set(v);
    }
    if let Some(v) = req.min_focus_speed {
        am.min_focus_speed = Set(Some(v));
    }
    if let Some(v) = req.max_focus_speed {
        am.max_focus_speed = Set(Some(v));
    }
    if let Some(v) = req.can_iris {
        am.can_iris = Set(v);
    }
    if let Some(v) = req.can_auto_iris {
        am.can_auto_iris = Set(v);
    }
    if let Some(v) = req.can_iris_abs {
        am.can_iris_abs = Set(v);
    }
    if let Some(v) = req.can_iris_rel {
        am.can_iris_rel = Set(v);
    }
    if let Some(v) = req.can_iris_con {
        am.can_iris_con = Set(v);
    }
    if let Some(v) = req.min_iris_range {
        am.min_iris_range = Set(Some(v));
    }
    if let Some(v) = req.max_iris_range {
        am.max_iris_range = Set(Some(v));
    }
    if let Some(v) = req.min_iris_step {
        am.min_iris_step = Set(Some(v));
    }
    if let Some(v) = req.max_iris_step {
        am.max_iris_step = Set(Some(v));
    }
    if let Some(v) = req.has_iris_speed {
        am.has_iris_speed = Set(v);
    }
    if let Some(v) = req.min_iris_speed {
        am.min_iris_speed = Set(Some(v));
    }
    if let Some(v) = req.max_iris_speed {
        am.max_iris_speed = Set(Some(v));
    }
    if let Some(v) = req.can_gain {
        am.can_gain = Set(v);
    }
    if let Some(v) = req.can_auto_gain {
        am.can_auto_gain = Set(v);
    }
    if let Some(v) = req.can_gain_abs {
        am.can_gain_abs = Set(v);
    }
    if let Some(v) = req.can_gain_rel {
        am.can_gain_rel = Set(v);
    }
    if let Some(v) = req.can_gain_con {
        am.can_gain_con = Set(v);
    }
    if let Some(v) = req.min_gain_range {
        am.min_gain_range = Set(Some(v));
    }
    if let Some(v) = req.max_gain_range {
        am.max_gain_range = Set(Some(v));
    }
    if let Some(v) = req.min_gain_step {
        am.min_gain_step = Set(Some(v));
    }
    if let Some(v) = req.max_gain_step {
        am.max_gain_step = Set(Some(v));
    }
    if let Some(v) = req.has_gain_speed {
        am.has_gain_speed = Set(v);
    }
    if let Some(v) = req.min_gain_speed {
        am.min_gain_speed = Set(Some(v));
    }
    if let Some(v) = req.max_gain_speed {
        am.max_gain_speed = Set(Some(v));
    }
    if let Some(v) = req.can_white {
        am.can_white = Set(v);
    }
    if let Some(v) = req.can_auto_white {
        am.can_auto_white = Set(v);
    }
    if let Some(v) = req.can_white_abs {
        am.can_white_abs = Set(v);
    }
    if let Some(v) = req.can_white_rel {
        am.can_white_rel = Set(v);
    }
    if let Some(v) = req.can_white_con {
        am.can_white_con = Set(v);
    }
    if let Some(v) = req.min_white_range {
        am.min_white_range = Set(Some(v));
    }
    if let Some(v) = req.max_white_range {
        am.max_white_range = Set(Some(v));
    }
    if let Some(v) = req.min_white_step {
        am.min_white_step = Set(Some(v));
    }
    if let Some(v) = req.max_white_step {
        am.max_white_step = Set(Some(v));
    }
    if let Some(v) = req.has_white_speed {
        am.has_white_speed = Set(v);
    }
    if let Some(v) = req.min_white_speed {
        am.min_white_speed = Set(Some(v));
    }
    if let Some(v) = req.max_white_speed {
        am.max_white_speed = Set(Some(v));
    }
    if let Some(v) = req.has_presets {
        am.has_presets = Set(v);
    }
    if let Some(v) = req.num_presets {
        am.num_presets = Set(v);
    }
    if let Some(v) = req.has_home_preset {
        am.has_home_preset = Set(v);
    }
    if let Some(v) = req.can_set_presets {
        am.can_set_presets = Set(v);
    }
    if let Some(v) = req.can_move {
        am.can_move = Set(v);
    }
    if let Some(v) = req.can_move_diag {
        am.can_move_diag = Set(v);
    }
    if let Some(v) = req.can_move_map {
        am.can_move_map = Set(v);
    }
    if let Some(v) = req.can_move_abs {
        am.can_move_abs = Set(v);
    }
    if let Some(v) = req.can_move_rel {
        am.can_move_rel = Set(v);
    }
    if let Some(v) = req.can_move_con {
        am.can_move_con = Set(v);
    }
    if let Some(v) = req.can_pan {
        am.can_pan = Set(v);
    }
    if let Some(v) = req.min_pan_range {
        am.min_pan_range = Set(Some(v));
    }
    if let Some(v) = req.max_pan_range {
        am.max_pan_range = Set(Some(v));
    }
    if let Some(v) = req.min_pan_step {
        am.min_pan_step = Set(Some(v));
    }
    if let Some(v) = req.max_pan_step {
        am.max_pan_step = Set(Some(v));
    }
    if let Some(v) = req.has_pan_speed {
        am.has_pan_speed = Set(v);
    }
    if let Some(v) = req.min_pan_speed {
        am.min_pan_speed = Set(Some(v));
    }
    if let Some(v) = req.max_pan_speed {
        am.max_pan_speed = Set(Some(v));
    }
    if let Some(v) = req.has_turbo_pan {
        am.has_turbo_pan = Set(v);
    }
    if let Some(v) = req.turbo_pan_speed {
        am.turbo_pan_speed = Set(Some(v));
    }
    if let Some(v) = req.can_tilt {
        am.can_tilt = Set(v);
    }
    if let Some(v) = req.min_tilt_range {
        am.min_tilt_range = Set(Some(v));
    }
    if let Some(v) = req.max_tilt_range {
        am.max_tilt_range = Set(Some(v));
    }
    if let Some(v) = req.min_tilt_step {
        am.min_tilt_step = Set(Some(v));
    }
    if let Some(v) = req.max_tilt_step {
        am.max_tilt_step = Set(Some(v));
    }
    if let Some(v) = req.has_tilt_speed {
        am.has_tilt_speed = Set(v);
    }
    if let Some(v) = req.min_tilt_speed {
        am.min_tilt_speed = Set(Some(v));
    }
    if let Some(v) = req.max_tilt_speed {
        am.max_tilt_speed = Set(Some(v));
    }
    if let Some(v) = req.has_turbo_tilt {
        am.has_turbo_tilt = Set(v);
    }
    if let Some(v) = req.turbo_tilt_speed {
        am.turbo_tilt_speed = Set(Some(v));
    }
    if let Some(v) = req.can_auto_scan {
        am.can_auto_scan = Set(v);
    }
    if let Some(v) = req.num_scan_paths {
        am.num_scan_paths = Set(v);
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = Controls::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
