//! Shared shape for the `Controls` seed rows several upstream updates add:
//! `INSERT ... SELECT ... WHERE NOT EXISTS (SELECT 1 FROM Controls WHERE Name = ?)`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

/// Insert a Controls row keyed by `Name` unless one exists. `cols` are the
/// non-Name columns and their integer values, in upstream's order.
pub(super) async fn add_control(
    m: &SchemaManager<'_>,
    name: &str,
    control_type: &str,
    protocol: &str,
    cols: &[(&str, i32)],
) -> Result<bool, DbErr> {
    add_control_keyed(m, "Name", name, name, control_type, protocol, cols).await
}

pub(super) async fn add_control_keyed(
    m: &SchemaManager<'_>,
    key_col: &str,
    key: &str,
    name: &str,
    control_type: &str,
    protocol: &str,
    cols: &[(&str, i32)],
) -> Result<bool, DbErr> {
    let mut columns = vec![c("Name"), c("Type"), c("Protocol")];
    let mut values: Vec<SimpleExpr> = vec![
        name.into(),
        enum_val("controls_type", control_type),
        protocol.into(),
    ];
    for (col, v) in cols {
        columns.push(c(col));
        values.push((*v).into());
    }
    insert_if_absent(
        m,
        probe_eq("Controls", key_col, key),
        Query::insert()
            .into_table(t("Controls"))
            .columns(columns)
            .values_panic(values)
            .to_owned(),
    )
    .await
}
