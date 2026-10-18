use crate::entity::logs::{self, Entity as Logs, Model as LogModel};
use crate::error::AppResult;
use sea_orm::*;

/// Query options for filtering logs.
#[derive(Debug, Default, Clone)]
pub struct LogQueryOptions {
    /// Exact `Component` match against any of these.
    pub components: Option<Vec<String>>,
    /// Exact `Level` match against any of these.
    pub levels: Option<Vec<i8>>,
    /// Only these row ids.
    pub ids: Option<Vec<u32>>,
    /// Severity threshold: keep rows at least this severe. On ZoneMinder's
    /// inverted scale that means `Level <= min_level`.
    pub min_level: Option<i8>,
    /// Case-insensitive substring match on the message.
    pub search: Option<String>,
    /// Unix-time (seconds) lower/upper bounds on `TimeKey`.
    pub start: Option<f64>,
    pub end: Option<f64>,
    /// Column to order by; `None` is `TimeKey`.
    pub sort_by: Option<logs::Column>,
    /// Descending when true (the default), ascending when false.
    pub sort_desc: bool,
    pub server_id: Option<u32>,
}

impl LogQueryOptions {
    /// The `sea_query` condition for every filter on these options. Shared by
    /// the list select and the clear-logs delete so they always match the same
    /// rows.
    fn condition(&self) -> Condition {
        use rust_decimal::prelude::FromPrimitive;
        use rust_decimal::Decimal;

        let mut cond = Condition::all();
        if let Some(ref components) = self.components {
            cond = cond.add(logs::Column::Component.is_in(components.iter().cloned()));
        }
        if let Some(ref levels) = self.levels {
            cond = cond.add(logs::Column::Level.is_in(levels.iter().copied()));
        }
        if let Some(ref ids) = self.ids {
            cond = cond.add(logs::Column::Id.is_in(ids.iter().copied()));
        }
        if let Some(min_level) = self.min_level {
            // Inverted scale: "this severe or worse" == Level <= threshold.
            cond = cond.add(logs::Column::Level.lte(min_level));
        }
        if let Some(ref search) = self.search {
            cond = cond.add(logs::Column::Message.contains(search.clone()));
        }
        if let Some(start) = self.start {
            if let Some(d) = Decimal::from_f64(start) {
                cond = cond.add(logs::Column::TimeKey.gte(d));
            }
        }
        if let Some(end) = self.end {
            if let Some(d) = Decimal::from_f64(end) {
                cond = cond.add(logs::Column::TimeKey.lte(d));
            }
        }
        if let Some(server_id) = self.server_id {
            cond = cond.add(logs::Column::ServerId.eq(server_id));
        }
        cond
    }
}

/// Find logs with pagination and filtering.
pub async fn find_with_options(
    db: &DatabaseConnection,
    options: LogQueryOptions,
    page: u64,
    page_size: u64,
) -> AppResult<(Vec<LogModel>, u64)> {
    let order = if options.sort_desc {
        Order::Desc
    } else {
        Order::Asc
    };
    let column = options.sort_by.unwrap_or(logs::Column::TimeKey);
    // Id breaks ties so pages don't shuffle when many rows share a value.
    let query = Logs::find()
        .filter(options.condition())
        .order_by(column, order.clone())
        .order_by(logs::Column::Id, order);

    let paginator = query.paginate(db, page_size);
    let total = paginator.num_items().await?;
    let logs = paginator.fetch_page(page).await?;

    Ok((logs, total))
}

/// Delete logs matching the filters; returns the number of rows removed.
/// Pagination/sort fields on `options` are ignored.
pub async fn delete_with_options(
    db: &DatabaseConnection,
    options: LogQueryOptions,
) -> AppResult<u64> {
    let res = Logs::delete_many()
        .filter(options.condition())
        .exec(db)
        .await?;
    Ok(res.rows_affected)
}

/// Row count of the whole table, ignoring every filter.
pub async fn count_all(db: &DatabaseConnection) -> AppResult<u64> {
    Ok(Logs::find().count(db).await?)
}

/// Every distinct `Component`, sorted.
pub async fn distinct_components(db: &DatabaseConnection) -> AppResult<Vec<String>> {
    Ok(Logs::find()
        .select_only()
        .column(logs::Column::Component)
        .distinct()
        .order_by_asc(logs::Column::Component)
        .into_tuple::<String>()
        .all(db)
        .await?)
}

/// Find all logs with a limit (legacy helper)
pub async fn find_all(db: &DatabaseConnection, limit: u64) -> AppResult<Vec<LogModel>> {
    Ok(Logs::find()
        .order_by_desc(logs::Column::TimeKey)
        .limit(limit)
        .all(db)
        .await?)
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
