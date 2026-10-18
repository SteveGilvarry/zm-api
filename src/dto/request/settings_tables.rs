//! Requests for ZoneMinder 1.39's EncoderTemplates and Menu_Items (#144).

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateEncoderTemplateRequest {
    /// The encoder the parameters are for, e.g. `libx264`.
    #[garde(length(chars, min = 1, max = 32))]
    pub encoder: String,
    #[garde(length(chars, min = 1, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
    /// `key=value` lines handed to the encoder.
    #[garde(skip)]
    pub params: String,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateEncoderTemplateRequest {
    #[garde(inner(length(chars, min = 1, max = 32)))]
    pub encoder: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub name: Option<String>,
    /// `null` clears it; omit to leave it unchanged.
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(skip)]
    pub description: Option<Option<String>>,
    #[garde(skip)]
    pub params: Option<String>,
}

fn valid_icon_type(v: &Option<String>, _: &()) -> garde::Result {
    match v.as_deref() {
        None | Some("material" | "fontawesome" | "image" | "none") => Ok(()),
        Some(_) => Err(garde::Error::new(
            "must be `material`, `fontawesome`, `image` or `none`",
        )),
    }
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateMenuItemRequest {
    /// The navbar entry this row configures, e.g. `Console`.
    #[garde(length(chars, min = 1, max = 32))]
    pub menu_key: String,
    #[garde(skip)]
    pub enabled: Option<bool>,
    #[garde(inner(length(chars, max = 64)))]
    pub label: Option<String>,
    #[garde(skip)]
    pub sort_order: Option<i16>,
    #[garde(inner(length(chars, max = 128)))]
    pub icon: Option<String>,
    #[garde(custom(valid_icon_type))]
    pub icon_type: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    pub link: Option<String>,
}

/// Partial update. For `label`, `icon` and `link`, `null` clears the value.
#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateMenuItemRequest {
    #[garde(skip)]
    pub enabled: Option<bool>,
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(skip)]
    pub label: Option<Option<String>>,
    #[garde(skip)]
    pub sort_order: Option<i16>,
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(skip)]
    pub icon: Option<Option<String>>,
    #[garde(custom(valid_icon_type))]
    pub icon_type: Option<String>,
    #[serde(default, deserialize_with = "crate::dto::serde_helpers::double_option")]
    #[schema(value_type = Option<String>)]
    #[garde(skip)]
    pub link: Option<Option<String>>,
}
