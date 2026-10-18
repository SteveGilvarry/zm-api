//! Requests for ZoneMinder 1.39.30's MonitorActions (#179).

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// When an action fires. zmc fires all but `Manual`, which a client runs
/// with `POST /monitors/{id}/actions/{action_id}/run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum TriggerOn {
    EventStart,
    EventEnd,
    Alarm,
    AlarmEnd,
    Manual,
}

impl TriggerOn {
    pub const ALL: [Self; 5] = [
        Self::EventStart,
        Self::EventEnd,
        Self::Alarm,
        Self::AlarmEnd,
        Self::Manual,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::EventStart => "EventStart",
            Self::EventEnd => "EventEnd",
            Self::Alarm => "Alarm",
            Self::AlarmEnd => "AlarmEnd",
            Self::Manual => "Manual",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

/// What the action does to the target monitor's device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum ActionType {
    LightOn,
    LightOff,
    IndicatorLightOn,
    IndicatorLightOff,
    AudioPlay,
    AudioStop,
}

impl ActionType {
    pub const ALL: [Self; 6] = [
        Self::LightOn,
        Self::LightOff,
        Self::IndicatorLightOn,
        Self::IndicatorLightOff,
        Self::AudioPlay,
        Self::AudioStop,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::LightOn => "LightOn",
            Self::LightOff => "LightOff",
            Self::IndicatorLightOn => "IndicatorLightOn",
            Self::IndicatorLightOff => "IndicatorLightOff",
            Self::AudioPlay => "AudioPlay",
            Self::AudioStop => "AudioStop",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateMonitorActionRequest {
    /// Defaults to `EventStart`, as in ZoneMinder.
    #[garde(skip)]
    pub trigger_on: Option<TriggerOn>,
    #[garde(skip)]
    pub action_type: ActionType,
    /// The monitor whose device is driven; often not this one.
    #[garde(skip)]
    pub target_monitor_id: u32,
    /// The sound to play, for `AudioPlay`; within the target control's
    /// `min_audio_file..=max_audio_file`.
    #[garde(skip)]
    pub audio_file: Option<u32>,
    #[garde(inner(length(chars, max = 64)))]
    pub label: Option<String>,
    /// Defaults to true.
    #[garde(skip)]
    pub enabled: Option<bool>,
    #[garde(skip)]
    pub sequence: Option<u16>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateMonitorActionRequest {
    #[garde(skip)]
    pub trigger_on: Option<TriggerOn>,
    #[garde(skip)]
    pub action_type: Option<ActionType>,
    #[garde(skip)]
    pub target_monitor_id: Option<u32>,
    /// `null` clears it; omit to leave it unchanged.
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<u32>)]
    #[garde(skip)]
    pub audio_file: Option<Option<u32>>,
    #[garde(inner(length(chars, max = 64)))]
    pub label: Option<String>,
    #[garde(skip)]
    pub enabled: Option<bool>,
    #[garde(skip)]
    pub sequence: Option<u16>,
}
