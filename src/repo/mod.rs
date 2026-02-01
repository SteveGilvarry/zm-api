use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, PrimaryKeyTrait, QueryFilter, Value,
};

pub mod config;
pub mod control_presets;
pub mod controls;
pub mod devices;
pub mod event_data;
pub mod event_summaries;
pub mod events;
pub mod events_tags;
pub mod filters;
pub mod frames;
pub mod groups;
pub mod groups_monitors;
pub mod groups_permissions;
pub mod logs;
pub mod manufacturers;
pub mod models;
pub mod monitor_presets;
pub mod monitor_status;
pub mod monitors;
pub mod monitors_permissions;
pub mod montage_layouts;
pub mod object_types;
pub mod ptz;
pub mod reports;
pub mod server_stats;
pub mod servers;
pub mod sessions;
pub mod snapshots;
pub mod snapshots_events;
pub mod states;
pub mod stats;
pub mod storage;
pub mod tags;
pub mod triggers_x10;
pub mod user_preferences;
pub mod users;
pub mod zone_presets;
pub mod zones;

// Create a new entity record
pub async fn create<T>(entity: T::ActiveModel, db: &DatabaseConnection) -> Result<T::Model, DbErr>
where
    T: EntityTrait,
    T::Model: IntoActiveModel<T::ActiveModel>, // Ensure the model can convert to ActiveModel
    T::ActiveModel: Send,                      // Ensure the ActiveModel can be sent between threads
{
    entity.insert(db).await
}

// Find an entity by primary key (supports both i32 and Uuid or other primary key types)
pub async fn find_by_id<T, PK>(id: PK, db: &DatabaseConnection) -> Result<Option<T::Model>, DbErr>
where
    T: EntityTrait,
    PK: Into<<T::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    T::Model: Send, // Ensure the model can be sent between threads
{
    T::find_by_id(id.into()).one(db).await
}

// Update an existing entity record
pub async fn update<T>(entity: T::ActiveModel, db: &DatabaseConnection) -> Result<T::Model, DbErr>
where
    T: EntityTrait,
    T::Model: IntoActiveModel<T::ActiveModel>, // Ensure the model can convert to ActiveModel
    T::ActiveModel: Send,                      // Ensure the ActiveModel can be sent between threads
{
    entity.update(db).await
}

// Delete an entity by primary key
pub async fn delete_by_id<T, PK>(id: PK, db: &DatabaseConnection) -> Result<(), DbErr>
where
    T: EntityTrait,
    PK: Into<<T::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    T::Model: Send, // Ensure the model can be sent between threads
{
    T::delete_by_id(id.into()).exec(db).await?;
    Ok(())
}

// Find an entity by any column
// Find an entity by any column
pub async fn find_by_column<T, C>(
    db: &DatabaseConnection,
    column: C,
    value: Value,
) -> Result<Option<T::Model>, DbErr>
where
    T: EntityTrait,
    C: ColumnTrait, // ColumnTrait provides comparison capabilities
    T::Model: Send, // Ensure the model can be sent between threads
{
    let condition = Condition::all().add(column.eq(value)); // The eq method handles the type comparison
    T::find().filter(condition).one(db).await
}
