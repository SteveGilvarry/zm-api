//! Mapping from the whitelisted [`FilterField`] vocabulary to (a) ZoneMinder's
//! attribute name, (b) the real DB column, and (c) the value type expected.
//!
//! This is the single source of truth that keeps the SQLi-safe AST tied to real
//! columns — the same closed-vocabulary-to-`Column` pattern used by
//! `EventSortField`/`get_sort_column` in `repo/events.rs`.

use crate::dto::request::filter_ast::FilterField;
use crate::entity::{events, monitors};

/// The DB column a [`FilterField`] resolves to. `Monitor` columns require a join
/// from `Events.MonitorId` when building a native query.
#[derive(Debug, Clone, Copy)]
pub enum FieldColumn {
    Event(events::Column),
    Monitor(monitors::Column),
}

/// The value type a field accepts, used to validate and to coerce the JSON
/// `value` into a correctly typed bound parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Int,
    Decimal,
    Str,
    DateTime,
    /// Stored as a 0/1 tinyint.
    Bool,
}

/// Everything the translator and the query-builder need about one field.
#[derive(Debug, Clone, Copy)]
pub struct FieldMeta {
    /// ZoneMinder's attribute name, emitted into the flat `Query_json`.
    pub zm_attr: &'static str,
    pub kind: ValueKind,
    pub column: FieldColumn,
}

/// Resolve a [`FilterField`] to its metadata. Exhaustive — adding a field is a
/// compile error until it is mapped here.
pub fn meta(field: FilterField) -> FieldMeta {
    use events::Column as E;
    use FieldColumn::{Event, Monitor};
    use FilterField as F;
    use ValueKind::{Bool, DateTime, Decimal, Int, Str};

    let (zm_attr, kind, column) = match field {
        F::Id => ("Id", Int, Event(E::Id)),
        F::MonitorId => ("MonitorId", Int, Event(E::MonitorId)),
        F::Name => ("Name", Str, Event(E::Name)),
        F::Cause => ("Cause", Str, Event(E::Cause)),
        F::Notes => ("Notes", Str, Event(E::Notes)),
        F::StartTime => ("StartDateTime", DateTime, Event(E::StartDateTime)),
        F::EndTime => ("EndDateTime", DateTime, Event(E::EndDateTime)),
        F::Length => ("Length", Decimal, Event(E::Length)),
        F::Frames => ("Frames", Int, Event(E::Frames)),
        F::AlarmFrames => ("AlarmFrames", Int, Event(E::AlarmFrames)),
        F::TotScore => ("TotScore", Int, Event(E::TotScore)),
        F::AvgScore => ("AvgScore", Int, Event(E::AvgScore)),
        F::MaxScore => ("MaxScore", Int, Event(E::MaxScore)),
        F::Archived => ("Archived", Bool, Event(E::Archived)),
        F::Videoed => ("Videoed", Bool, Event(E::Videoed)),
        F::Uploaded => ("Uploaded", Bool, Event(E::Uploaded)),
        F::Emailed => ("Emailed", Bool, Event(E::Emailed)),
        F::Messaged => ("Messaged", Bool, Event(E::Messaged)),
        F::Executed => ("Executed", Bool, Event(E::Executed)),
        F::Locked => ("Locked", Bool, Event(E::Locked)),
        F::StateId => ("StateId", Int, Event(E::StateId)),
        F::StorageId => ("StorageId", Int, Event(E::StorageId)),
        F::DiskSpace => ("DiskSpace", Int, Event(E::DiskSpace)),
        F::Width => ("Width", Int, Event(E::Width)),
        F::Height => ("Height", Int, Event(E::Height)),
        F::MonitorName => ("MonitorName", Str, Monitor(monitors::Column::Name)),
    };
    FieldMeta {
        zm_attr,
        kind,
        column,
    }
}

/// Reverse lookup: ZoneMinder attribute name -> [`FilterField`]. Used when
/// parsing a stored flat `Query_json` back into the AST. Returns `None` for
/// attributes outside our vocabulary.
pub fn from_zm_attr(attr: &str) -> Option<FilterField> {
    use FilterField as F;
    Some(match attr {
        "Id" => F::Id,
        "MonitorId" => F::MonitorId,
        "Name" => F::Name,
        "Cause" => F::Cause,
        "Notes" => F::Notes,
        "StartDateTime" => F::StartTime,
        "EndDateTime" => F::EndTime,
        "Length" => F::Length,
        "Frames" => F::Frames,
        "AlarmFrames" => F::AlarmFrames,
        "TotScore" => F::TotScore,
        "AvgScore" => F::AvgScore,
        "MaxScore" => F::MaxScore,
        "Archived" => F::Archived,
        "Videoed" => F::Videoed,
        "Uploaded" => F::Uploaded,
        "Emailed" => F::Emailed,
        "Messaged" => F::Messaged,
        "Executed" => F::Executed,
        "Locked" => F::Locked,
        "StateId" => F::StateId,
        "StorageId" => F::StorageId,
        "DiskSpace" => F::DiskSpace,
        "Width" => F::Width,
        "Height" => F::Height,
        "MonitorName" => F::MonitorName,
        _ => return None,
    })
}
