//! Typed, nested filter-query AST — the SQLi-safe contract for building a
//! filter's `WHERE`/sort/limit.
//!
//! A filter is modelled as a recursive expression tree instead of ZoneMinder's
//! flat `terms` + `obr`/`cbr` bracket-count array. A [`FilterExpr::Group`] is a
//! pair of brackets (its `match` is the AND/OR joining its children); a
//! [`FilterExpr::Condition`] is a single `field op value`. Because `field` and
//! `op` are closed enums (never free text) and `value` is always carried as
//! data (bound as a parameter downstream), no client input can reach SQL as an
//! identifier or operator — SQL injection is impossible by construction.
//!
//! The service layer translates this tree to ZoneMinder's flat `Query_json`
//! (so `zmfilter.pl` and the ZM web UI keep working) and can also compile it to
//! a parameterised `sea_orm` query for `/preview`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::dto::request::events::SortDirection;

/// A complete structured filter query: a root predicate plus optional sort and
/// row limit.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterQuery {
    /// Root predicate. Usually a [`FilterExpr::Group`].
    #[serde(rename = "where")]
    pub predicate: FilterExpr,
    /// Optional sort column + direction.
    #[serde(default)]
    pub sort: Option<FilterSort>,
    /// Optional row limit applied by the downstream executor.
    #[serde(default)]
    pub limit: Option<u64>,
}

/// One node of the filter tree: either a bracketed group of sub-expressions
/// joined by AND/OR, or a single leaf condition.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum FilterExpr {
    /// A bracketed group — `match` joins its `rules` with AND (`all`) or OR
    /// (`any`). Nesting groups gives arbitrarily deep brackets.
    Group {
        #[serde(rename = "match")]
        match_op: MatchOp,
        // Break utoipa's schema-generation recursion on this self-referential
        // type (otherwise the OpenAPI builder overflows the stack).
        #[schema(no_recursion)]
        rules: Vec<FilterExpr>,
    },
    /// A leaf condition: `field op value`.
    Condition {
        field: FilterField,
        op: FilterOp,
        /// The comparison value. Omitted for `is_null`/`is_not_null`; an array
        /// for `in`/`not_in`; otherwise a scalar. Always treated as data.
        #[serde(default)]
        value: Option<Value>,
    },
}

/// How a [`FilterExpr::Group`] joins its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchOp {
    /// Logical AND.
    All,
    /// Logical OR.
    Any,
}

/// Comparison operators a condition may use. A closed set mapped to fixed SQL
/// operators downstream — never interpolated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Regexp,
    NotRegexp,
}

/// Whitelisted attributes a filter may reference. Each maps to a real column
/// (and a ZoneMinder attribute name) in `service::filter_field`. Most are
/// `Events` columns; a few are joined `Monitors`/`Storage` attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterField {
    Id,
    MonitorId,
    Name,
    Cause,
    Notes,
    StartTime,
    EndTime,
    Length,
    Frames,
    AlarmFrames,
    TotScore,
    AvgScore,
    MaxScore,
    Archived,
    Videoed,
    Uploaded,
    Emailed,
    Messaged,
    Executed,
    Locked,
    StateId,
    StorageId,
    DiskSpace,
    Width,
    Height,
    /// Joined from `Monitors.Name` via `Events.MonitorId`.
    MonitorName,
}

/// Sort specification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct FilterSort {
    pub field: FilterField,
    #[serde(default)]
    pub dir: SortDirection,
}
