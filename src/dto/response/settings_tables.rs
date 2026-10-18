//! Responses for ZoneMinder 1.39's EncoderTemplates and Menu_Items (#144).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::{encoder_templates, menu_items};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EncoderTemplateResponse {
    pub id: u32,
    pub encoder: String,
    pub name: String,
    pub description: Option<String>,
    pub params: String,
}

impl From<&encoder_templates::Model> for EncoderTemplateResponse {
    fn from(m: &encoder_templates::Model) -> Self {
        Self {
            id: m.id,
            encoder: m.encoder.clone(),
            name: m.name.clone(),
            description: m.description.clone(),
            params: m.params.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MenuItemResponse {
    pub id: u32,
    pub menu_key: String,
    pub enabled: bool,
    /// Label to show instead of the entry's default one.
    pub label: Option<String>,
    pub sort_order: i16,
    pub icon: Option<String>,
    /// `material`, `fontawesome`, `image` or `none`.
    pub icon_type: String,
    pub link: Option<String>,
}

impl From<&menu_items::Model> for MenuItemResponse {
    fn from(m: &menu_items::Model) -> Self {
        Self {
            id: m.id,
            menu_key: m.menu_key.clone(),
            enabled: m.enabled != 0,
            label: m.label.clone(),
            sort_order: m.sort_order,
            icon: m.icon.clone(),
            icon_type: m.icon_type.clone(),
            link: m.link.clone(),
        }
    }
}
