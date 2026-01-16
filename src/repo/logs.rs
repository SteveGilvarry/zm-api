use crate::entity::logs::{Column as LogColumn, Entity as Logs, Model as LogModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection, limit: u64) -> AppResult<Vec<LogModel>> {
    Ok(Logs::find()
        .order_by_desc(LogColumn::Id)
        .limit(limit)
        .all(db)
        .await?)
}

pub async fn find_all_paginated(
    db: &DatabaseConnection,
    page: u64,
    page_size: u64,
) -> AppResult<(Vec<LogModel>, u64)> {
    let paginator = Logs::find()
        .order_by_desc(LogColumn::Id)
        .paginate(db, page_size);

    let total = paginator.num_items().await?;
    let items = paginator.fetch_page(page.saturating_sub(1)).await?;

    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<LogModel>> {
    Ok(Logs::find_by_id(id).one(db).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn mk(id: u32, message: &str) -> LogModel {
        LogModel {
            id,
            time_key: Decimal::new(0, 0),
            component: "zmdc".into(),
            server_id: None,
            pid: None,
            level: 1,
            code: "A01".into(),
            message: message.into(),
            file: None,
            line: None,
        }
    }

    #[tokio::test]
    async fn test_find_all_returns_limited_rows() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<LogModel, _, _>(vec![vec![mk(1, "a"), mk(2, "b")]])
            .into_connection();

        let rows = find_all(&db, 2).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, 1);
    }

    #[tokio::test]
    async fn test_find_by_id_some_and_none() {
        let db_some = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<LogModel, _, _>(vec![vec![mk(9, "x")]])
            .into_connection();
        assert_eq!(find_by_id(&db_some, 9).await.unwrap().unwrap().id, 9);

        let empty: Vec<LogModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<LogModel, _, _>(vec![empty])
            .into_connection();
        assert!(find_by_id(&db_none, 9).await.unwrap().is_none());
    }
}
