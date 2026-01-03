use crate::dto::request::sessions::{CreateSessionRequest, UpdateSessionRequest};
use crate::entity::sessions::{ActiveModel, Entity as Sessions, Model as SessionModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<SessionModel>> {
    Ok(Sessions::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: &str) -> AppResult<Option<SessionModel>> {
    Ok(Sessions::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateSessionRequest,
) -> AppResult<SessionModel> {
    let am = ActiveModel {
        id: Set(req.id.clone()),
        access: Set(req.access),
        data: Set(req.data.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: &str,
    req: &UpdateSessionRequest,
) -> AppResult<Option<SessionModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = req.access {
        am.access = Set(Some(v));
    }
    if let Some(v) = &req.data {
        am.data = Set(Some(v.clone()));
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: &str) -> AppResult<bool> {
    let res = Sessions::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
