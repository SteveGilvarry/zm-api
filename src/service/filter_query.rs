//! Validation for a filter's `Query_json`.
//!
//! ## Why this exists (stored / second-order SQL injection)
//!
//! `zm_api` never executes a filter's `Query_json` itself — it only stores it.
//! But ZoneMinder's `zmfilter.pl` (via `ZoneMinder::Filter`) reads that JSON
//! back and **builds a SQL `WHERE`/`ORDER BY`/`LIMIT` clause from it**, then
//! runs it. The pieces of a filter term that become SQL *identifiers* and
//! *operators* — `attr`, `op`, `cnj`, the bracket counts, `sort_field`,
//! `limit` — are not quoted as data downstream, so a crafted value there is a
//! classic stored SQL-injection vector: write it through our API, and the
//! Perl executor turns it into live SQL.
//!
//! Our API is the injection *sink*, so it is the right place to refuse the
//! payload. We don't try to reproduce ZoneMinder's full filter grammar; we
//! enforce that every field that lands in a SQL identifier/operator position
//! is either a bare identifier (`^[A-Za-z][A-Za-z0-9_]*$` — no quotes, spaces,
//! semicolons, comment markers or parentheses) or a member of a fixed operator
//! whitelist. `val` is left unconstrained: it is real, arbitrary filter data
//! that ZoneMinder binds/quotes as a value.

use crate::error::{AppError, AppResult};
use serde_json::Value;

/// Operators ZoneMinder accepts in a filter term. Matched exactly, so the
/// bracketed/`=~` forms are safe — they map to fixed SQL constructs downstream
/// rather than being interpolated as identifiers.
const ALLOWED_OPS: &[&str] = &[
    "=", "!=", ">=", ">", "<=", "<", "=~", "!~", "=[]", "!=[]", "!~[]", "IS", "IS NOT", "LIKE",
    "NOT LIKE", "IN", "NOT IN",
];

/// A bare SQL identifier: starts with a letter, then letters/digits/underscore.
/// Deliberately excludes every SQL metacharacter (quote, backtick, space,
/// `;`, `-`, `/`, `(`, `,`, …) so it can never break out of its position.
fn is_safe_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn bad(msg: impl Into<String>) -> AppError {
    AppError::BadRequestError(msg.into())
}

/// Accept a JSON value that is either a non-negative integer or a string of
/// digits, within a sane bound. Used for `obr`/`cbr` bracket counts and
/// `limit`, which ZoneMinder concatenates into SQL.
fn is_count(v: &Value, max: u64) -> bool {
    match v {
        Value::Number(n) => n.as_u64().map(|n| n <= max).unwrap_or(false),
        Value::String(s) => s.parse::<u64>().map(|n| n <= max).unwrap_or(false),
        _ => false,
    }
}

/// Validate a filter's `query_json` before it is persisted.
///
/// Empty input and `{}` are accepted (a filter with no terms). Anything that
/// could inject SQL through `zmfilter.pl`'s clause builder is rejected with a
/// `400`.
pub fn validate_query_json(query_json: &str) -> AppResult<()> {
    let trimmed = query_json.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let root: Value = serde_json::from_str(trimmed)
        .map_err(|e| bad(format!("query_json is not valid JSON: {e}")))?;
    let obj = root
        .as_object()
        .ok_or_else(|| bad("query_json must be a JSON object"))?;

    if let Some(terms) = obj.get("terms") {
        let terms = terms
            .as_array()
            .ok_or_else(|| bad("query_json.terms must be an array"))?;
        for (i, term) in terms.iter().enumerate() {
            let term = term
                .as_object()
                .ok_or_else(|| bad(format!("query_json.terms[{i}] must be an object")))?;

            if let Some(attr) = term.get("attr") {
                let attr = attr
                    .as_str()
                    .ok_or_else(|| bad(format!("query_json.terms[{i}].attr must be a string")))?;
                if !is_safe_identifier(attr) {
                    return Err(bad(format!(
                        "query_json.terms[{i}].attr {attr:?} is not a valid filter attribute"
                    )));
                }
            }

            if let Some(op) = term.get("op") {
                let op = op
                    .as_str()
                    .ok_or_else(|| bad(format!("query_json.terms[{i}].op must be a string")))?;
                if !ALLOWED_OPS.contains(&op) {
                    return Err(bad(format!(
                        "query_json.terms[{i}].op {op:?} is not an allowed operator"
                    )));
                }
            }

            if let Some(cnj) = term.get("cnj") {
                let cnj = cnj
                    .as_str()
                    .ok_or_else(|| bad(format!("query_json.terms[{i}].cnj must be a string")))?;
                if !matches!(cnj.to_lowercase().as_str(), "and" | "or") {
                    return Err(bad(format!(
                        "query_json.terms[{i}].cnj {cnj:?} must be \"and\" or \"or\""
                    )));
                }
            }

            for bracket in ["obr", "cbr"] {
                if let Some(v) = term.get(bracket) {
                    if !is_count(v, 64) {
                        return Err(bad(format!(
                            "query_json.terms[{i}].{bracket} must be a small non-negative integer"
                        )));
                    }
                }
            }
            // `val` is intentionally unvalidated: it is filter data that
            // ZoneMinder binds/quotes as a value, not a SQL identifier.
        }
    }

    // Sort column and limit also land in the generated SQL.
    for sort_key in ["sort_field", "sort_column", "sort_by"] {
        if let Some(v) = obj.get(sort_key) {
            if let Some(s) = v.as_str() {
                if !s.is_empty() && !is_safe_identifier(s) {
                    return Err(bad(format!(
                        "query_json.{sort_key} {s:?} is not a valid column"
                    )));
                }
            } else if !v.is_null() {
                return Err(bad(format!("query_json.{sort_key} must be a string")));
            }
        }
    }

    if let Some(limit) = obj.get("limit") {
        // An empty string means "no limit" in ZoneMinder's UI.
        let empty = matches!(limit, Value::String(s) if s.trim().is_empty());
        if !empty && !limit.is_null() && !is_count(limit, 1_000_000_000) {
            return Err(bad("query_json.limit must be a non-negative integer"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_empty_and_object() {
        assert!(validate_query_json("").is_ok());
        assert!(validate_query_json("{}").is_ok());
    }

    #[test]
    fn accepts_a_realistic_filter() {
        let q = r#"{
            "terms": [
                {"attr":"Monitor","op":"=","val":"5","obr":"1"},
                {"cnj":"and","attr":"StartDateTime","op":">=","val":"-1 day","cbr":"1"},
                {"cnj":"or","attr":"Name","op":"LIKE","val":"%front%"}
            ],
            "sort_field":"StartDateTime",
            "sort_asc":"0",
            "limit":"100"
        }"#;
        assert!(validate_query_json(q).is_ok(), "valid filter should pass");
    }

    #[test]
    fn rejects_non_json() {
        assert!(validate_query_json("not json").is_err());
        assert!(validate_query_json("[1,2,3]").is_err());
    }

    #[test]
    fn rejects_injection_in_attr() {
        let q = r#"{"terms":[{"attr":"Id; DROP TABLE Events; --","op":"=","val":"1"}]}"#;
        assert!(validate_query_json(q).is_err());
        let q2 = r#"{"terms":[{"attr":"Id`,(SELECT 1)","op":"=","val":"1"}]}"#;
        assert!(validate_query_json(q2).is_err());
    }

    #[test]
    fn rejects_unknown_operator() {
        let q = r#"{"terms":[{"attr":"Id","op":"= 1 OR 1=1 --","val":"1"}]}"#;
        assert!(validate_query_json(q).is_err());
    }

    #[test]
    fn rejects_injection_in_sort_and_limit() {
        let q = r#"{"sort_field":"StartDateTime; DROP TABLE Events"}"#;
        assert!(validate_query_json(q).is_err());
        let q2 = r#"{"limit":"1; DROP TABLE Events"}"#;
        assert!(validate_query_json(q2).is_err());
    }

    #[test]
    fn rejects_bad_conjunction_and_brackets() {
        assert!(validate_query_json(r#"{"terms":[{"cnj":"and 1=1"}]}"#).is_err());
        assert!(validate_query_json(r#"{"terms":[{"obr":"1)) OR 1=1"}]}"#).is_err());
    }
}
