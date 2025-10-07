// app_entity_impl
use crate::entity::{users, config}; // Import your generated entities
use crate::error::ResourceType;
use crate::entity::AppEntity;

impl AppEntity for users::Model {
    const RESOURCE: ResourceType = ResourceType::User;
}

impl AppEntity for config::Model {
    const RESOURCE: ResourceType = ResourceType::Config;
}
