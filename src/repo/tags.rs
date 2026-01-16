use std::collections::HashMap;

use sea_orm::*;

use crate::dto::request::tags::{CreateTagRequest, UpdateTagRequest};
use crate::entity::events;
use crate::entity::events_tags;
use crate::entity::tags::{ActiveModel, Column, Entity as Tags, Model as TagModel};
use crate::error::AppResult;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<TagModel>> {
    Ok(Tags::find().all(db).await?)
}

pub async fn find_all_paginated(
    db: &DatabaseConnection,
    page: u64,
    page_size: u64,
) -> AppResult<(Vec<TagModel>, u64)> {
    let paginator = Tags::find()
        .order_by_asc(Column::Id)
        .paginate(db, page_size);

    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page.saturating_sub(1)).await?;

    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u64) -> AppResult<Option<TagModel>> {
    Ok(Tags::find_by_id(id).one(db).await?)
}

pub async fn find_by_name(db: &DatabaseConnection, name: &str) -> AppResult<Option<TagModel>> {
    Ok(Tags::find().filter(Column::Name.eq(name)).one(db).await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateTagRequest) -> AppResult<TagModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        create_date: Set(req.create_date),
        ..Default::default()
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u64,
    req: &UpdateTagRequest,
) -> AppResult<Option<TagModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = &req.name {
        am.name = Set(v.clone());
    }
    if let Some(v) = req.create_date {
        am.create_date = Set(Some(v));
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u64) -> AppResult<bool> {
    let res = Tags::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

/// Get event counts for all tags
pub async fn get_event_counts(db: &DatabaseConnection) -> AppResult<HashMap<u64, u64>> {
    // Fetch all event-tag associations and count by tag_id
    let associations = events_tags::Entity::find().all(db).await?;

    let mut counts: HashMap<u64, u64> = HashMap::new();
    for assoc in associations {
        *counts.entry(assoc.tag_id).or_insert(0) += 1;
    }

    Ok(counts)
}

/// Get paginated events for a specific tag
pub async fn find_events_for_tag(
    db: &DatabaseConnection,
    tag_id: u64,
    page: u64,
    page_size: u64,
) -> AppResult<(Vec<events::Model>, u64)> {
    // Get event IDs associated with this tag
    let associations = events_tags::Entity::find()
        .filter(events_tags::Column::TagId.eq(tag_id))
        .all(db)
        .await?;

    let event_ids: Vec<u64> = associations.iter().map(|a| a.event_id).collect();
    let total = event_ids.len() as u64;

    if event_ids.is_empty() {
        return Ok((vec![], 0));
    }

    // Paginate and fetch events
    let paginator = events::Entity::find()
        .filter(events::Column::Id.is_in(event_ids))
        .order_by_desc(events::Column::StartDateTime)
        .paginate(db, page_size);

    let events_list = paginator.fetch_page(page).await?;

    Ok((events_list, total))
}
