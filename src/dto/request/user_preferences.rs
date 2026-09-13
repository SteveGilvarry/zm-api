use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateUserPreferenceRequest {
    #[garde(skip)]
    pub user_id: u32,
    /// Required since ZoneMinder 1.39.19 (`User_Preferences.Name` is NOT NULL
    /// and unique per user).
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdateUserPreferenceRequest {
    #[garde(skip)]
    pub user_id: Option<u32>,
    #[garde(inner(length(chars, max = 64)))]
    pub name: Option<String>,
    #[garde(skip)]
    pub value: Option<String>,
}
