//! Responses for ZoneMinder 1.39.30's MonitorActions (#179).

use serde::Serialize;
use utoipa::ToSchema;

use crate::dto::request::monitor_actions::{ActionType, TriggerOn};
use crate::entity::monitor_actions::Model;

#[derive(Debug, Serialize, ToSchema)]
pub struct MonitorActionResponse {
    pub id: u32,
    pub monitor_id: u32,
    /// `null` for a value this build doesn't know (a newer ZoneMinder).
    pub trigger_on: Option<TriggerOn>,
    pub action_type: Option<ActionType>,
    pub target_monitor_id: u32,
    pub audio_file: Option<u32>,
    pub label: String,
    pub enabled: bool,
    pub sequence: u16,
}

impl From<&Model> for MonitorActionResponse {
    fn from(m: &Model) -> Self {
        Self {
            id: m.id,
            monitor_id: m.monitor_id,
            trigger_on: TriggerOn::parse(&m.trigger_on),
            action_type: ActionType::parse(&m.action_type),
            target_monitor_id: m.target_monitor_id,
            audio_file: m.audio_file,
            label: m.label.clone(),
            enabled: m.enabled != 0,
            sequence: m.sequence,
        }
    }
}

/// A monitor an action can drive, and what it can do (ZoneMinder's
/// `MonitorAction::targetCandidates`).
#[derive(Debug, Serialize, ToSchema)]
pub struct ActionTargetResponse {
    pub id: u32,
    pub name: String,
    /// `Camera` or `Speaker`.
    pub device_class: String,
    /// The action types its control can perform.
    pub types: Vec<ActionType>,
    pub min_audio_file: Option<u32>,
    pub max_audio_file: Option<u32>,
}
