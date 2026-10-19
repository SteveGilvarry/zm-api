//! Requests for ZoneMinder 1.39's Notifications, push token registrations
//! (#175).

use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PushPlatform {
    Android,
    Ios,
    Web,
}

impl PushPlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Android => "android",
            Self::Ios => "ios",
            Self::Web => "web",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [Self::Android, Self::Ios, Self::Web]
            .into_iter()
            .find(|p| p.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PushState {
    Enabled,
    Disabled,
}

impl PushState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [Self::Enabled, Self::Disabled]
            .into_iter()
            .find(|p| p.as_str() == s)
    }
}

/// Register a push token. A token already registered is updated in place,
/// as ZoneMinder's `notifications.json` does.
#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateNotificationRequest {
    #[garde(length(chars, min = 1, max = 512))]
    pub token: String,
    #[garde(skip)]
    pub platform: PushPlatform,
    /// Comma-separated monitor ids to notify for; empty or absent for all.
    #[garde(skip)]
    pub monitor_list: Option<String>,
    /// Minimum seconds between pushes.
    #[garde(skip)]
    pub interval: Option<u32>,
    #[garde(skip)]
    pub push_state: Option<PushState>,
    #[garde(inner(length(chars, max = 32)))]
    pub app_version: Option<String>,
    #[garde(inner(length(chars, max = 128)))]
    pub profile: Option<String>,
    #[garde(skip)]
    pub badge_count: Option<i32>,
    /// RFC 3339.
    #[schema(value_type = Option<String>, format = DateTime)]
    #[garde(skip)]
    pub last_notified_at: Option<DateTime<Utc>>,
    /// The owner. Only a System editor may set it; otherwise it is the caller.
    #[garde(skip)]
    pub user_id: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateNotificationRequest {
    #[garde(inner(length(chars, min = 1, max = 512)))]
    pub token: Option<String>,
    #[garde(skip)]
    pub platform: Option<PushPlatform>,
    /// `null` clears it; omit to leave it unchanged.
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(skip)]
    pub monitor_list: Option<Option<String>>,
    #[garde(skip)]
    pub interval: Option<u32>,
    #[garde(skip)]
    pub push_state: Option<PushState>,
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(inner(inner(length(chars, max = 32))))]
    pub app_version: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(inner(inner(length(chars, max = 128))))]
    pub profile: Option<Option<String>>,
    #[garde(skip)]
    pub badge_count: Option<i32>,
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    #[garde(skip)]
    pub last_notified_at: Option<Option<DateTime<Utc>>>,
    /// System editors only.
    #[garde(skip)]
    pub user_id: Option<u32>,
}
