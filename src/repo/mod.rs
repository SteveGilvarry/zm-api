use sea_orm::{ColumnTrait, EntityTrait, Condition, QueryFilter, DbErr, DatabaseConnection, ActiveModelTrait, IntoActiveModel, PrimaryKeyTrait, Value};

pub mod user;
pub mod message;
pub mod monitors;
pub mod config;
pub mod events;

pub use message::*;
pub use monitors::*;
pub use config::*;
pub use events::*;

// Create a new entity record
pub async fn create<T>(entity: T::ActiveModel, db: &DatabaseConnection) -> Result<T::Model, DbErr>
where
    T: EntityTrait,
    T::Model: IntoActiveModel<T::ActiveModel>, // Ensure the model can convert to ActiveModel
    T::ActiveModel: Send, // Ensure the ActiveModel can be sent between threads
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
    T::ActiveModel: Send, // Ensure the ActiveModel can be sent between threads
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