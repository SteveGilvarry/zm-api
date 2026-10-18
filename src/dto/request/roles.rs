use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::users::UserPermissionsInput;

/// Create a role: a named permission set users hold through `role_id`.
/// Omitted permission levels default to `None` (a role grants only what it
/// names).
#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateRoleRequest {
    #[garde(length(chars, min = 1, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
    #[serde(flatten)]
    #[garde(skip)]
    pub permissions: UserPermissionsInput,
}

/// Partial update of a role.
#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateRoleRequest {
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub name: Option<String>,
    /// `null` clears it; omit to leave it unchanged.
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(skip)]
    pub description: Option<Option<String>>,
    #[serde(flatten)]
    #[garde(skip)]
    pub permissions: UserPermissionsInput,
}
