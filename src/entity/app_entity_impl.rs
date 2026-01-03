// app_entity_impl
use crate::entity::AppEntity;
use crate::entity::{config, users}; // Import your generated entities
use crate::error::ResourceType;

impl AppEntity for users::Model {
    const RESOURCE: ResourceType = ResourceType::User;
}

impl AppEntity for config::Model {
    const RESOURCE: ResourceType = ResourceType::Config;
}
