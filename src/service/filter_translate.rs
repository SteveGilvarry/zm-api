//! Translate the typed filter AST to/from ZoneMinder's flat `Query_json`.
//!
//! ZoneMinder stores a filter as a flat array of `terms`, where the UI's nested
//! brackets are encoded as per-term `obr`/`cbr` bracket *counts* and `cnj`
//! conjunctions. `zmfilter.pl` and the ZM web UI read that format, so the API
//! keeps the AST as its contract but persists ZM's flat form.
//!
//! - [`to_zm_query_json`] flattens the AST tree into ZM terms (computing the
//!   bracket counts and conjunctions from the nesting).
//! - [`from_zm_query_json`] parses a stored flat form back into the AST for
//!   display, applying SQL precedence (AND binds tighter than OR).
//! - [`validate`] checks that each condition's value matches its field type and
//!   operator before anything is persisted or executed.

use serde::Serialize;
use serde_json::{json, Value};

use crate::dto::request::events::SortDirection;
use crate::dto::request::filter_ast::{FilterExpr, FilterField, FilterOp, FilterQuery, MatchOp};
use crate::error::{AppError, AppResult};

use super::filter_field::{from_zm_attr, meta, ValueKind};

fn bad(msg: impl Into<String>) -> AppError {
    AppError::BadRequestError(msg.into())
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validate that every condition's value is well-typed for its field and
/// operator. Rejects empty groups and type mismatches with a 400.
pub fn validate(query: &FilterQuery) -> AppResult<()> {
    validate_expr(&query.predicate)
}

fn validate_expr(expr: &FilterExpr) -> AppResult<()> {
    match expr {
        FilterExpr::Group { rules, .. } => {
            if rules.is_empty() {
                return Err(bad("filter group must contain at least one rule"));
            }
            for rule in rules {
                validate_expr(rule)?;
            }
            Ok(())
        }
        FilterExpr::Condition { field, op, value } => validate_condition(*field, *op, value),
    }
}

fn validate_condition(field: FilterField, op: FilterOp, value: &Option<Value>) -> AppResult<()> {
    let kind = meta(field).kind;
    match op {
        FilterOp::IsNull | FilterOp::IsNotNull => {
            if value.is_some() {
                return Err(bad("is_null/is_not_null take no value"));
            }
            Ok(())
        }
        FilterOp::In | FilterOp::NotIn => {
            let arr = value
                .as_ref()
                .and_then(Value::as_array)
                .ok_or_else(|| bad("in/not_in require an array value"))?;
            if arr.is_empty() {
                return Err(bad("in/not_in require a non-empty array"));
            }
            for v in arr {
                check_scalar(v, kind)?;
            }
            Ok(())
        }
        FilterOp::Like | FilterOp::NotLike | FilterOp::Regexp | FilterOp::NotRegexp => {
            // Pattern operators always compare as text.
            let v = value
                .as_ref()
                .ok_or_else(|| bad("operator requires a value"))?;
            if !v.is_string() {
                return Err(bad("like/regexp operators require a string value"));
            }
            Ok(())
        }
        _ => {
            let v = value
                .as_ref()
                .ok_or_else(|| bad("operator requires a value"))?;
            check_scalar(v, kind)
        }
    }
}

fn check_scalar(v: &Value, kind: ValueKind) -> AppResult<()> {
    let ok = match kind {
        ValueKind::Int => v.is_i64() || v.is_u64(),
        ValueKind::Decimal => v.is_number(),
        ValueKind::Str | ValueKind::DateTime => v.is_string(),
        ValueKind::Bool => v.is_boolean() || matches!(v.as_u64(), Some(0) | Some(1)),
    };
    if ok {
        Ok(())
    } else {
        Err(bad(format!("value {v} does not match the field type")))
    }
}

// ---------------------------------------------------------------------------
// AST -> ZM flat Query_json
// ---------------------------------------------------------------------------

/// A single ZoneMinder filter term.
#[derive(Debug, Serialize)]
struct ZmTerm {
    #[serde(skip_serializing_if = "Option::is_none")]
    cnj: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    obr: Option<u32>,
    attr: String,
    op: String,
    val: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cbr: Option<u32>,
}

#[derive(Default)]
struct Flattener {
    terms: Vec<ZmTerm>,
    pending_obr: u32,
    pending_cnj: Option<&'static str>,
}

/// Translate a validated [`FilterQuery`] to ZoneMinder's flat `Query_json`
/// string. Call [`validate`] first.
pub fn to_zm_query_json(query: &FilterQuery) -> AppResult<String> {
    validate(query)?;

    let mut f = Flattener::default();
    flatten(&query.predicate, &mut f)?;

    let mut root = serde_json::Map::new();
    root.insert("terms".into(), serde_json::to_value(&f.terms)?);

    if let Some(sort) = &query.sort {
        let attr = meta(sort.field).zm_attr;
        root.insert("sort_field".into(), json!(attr));
        let asc = matches!(sort.dir, SortDirection::Asc);
        root.insert("sort_asc".into(), json!(if asc { "1" } else { "0" }));
    }
    if let Some(limit) = query.limit {
        root.insert("limit".into(), json!(limit.to_string()));
    }

    Ok(serde_json::to_string(&Value::Object(root))?)
}

fn flatten(expr: &FilterExpr, f: &mut Flattener) -> AppResult<()> {
    match expr {
        FilterExpr::Condition { field, op, value } => {
            let m = meta(*field);
            f.terms.push(ZmTerm {
                cnj: f.pending_cnj.take(),
                obr: nonzero(std::mem::take(&mut f.pending_obr)),
                attr: m.zm_attr.to_string(),
                op: zm_op(*op).to_string(),
                val: zm_val(*op, value, m.kind),
                cbr: None,
            });
            Ok(())
        }
        FilterExpr::Group { match_op, rules } => {
            // This group adds one opening bracket before its first leaf.
            f.pending_obr += 1;
            let before = f.terms.len();
            for (i, rule) in rules.iter().enumerate() {
                if i > 0 {
                    f.pending_cnj = Some(conj(*match_op));
                }
                flatten(rule, f)?;
            }
            // Close the bracket on the last leaf emitted within this group.
            if f.terms.len() > before {
                let last = f.terms.last_mut().expect("group emitted a term");
                last.cbr = Some(last.cbr.unwrap_or(0) + 1);
            } else {
                // Empty group (validate() rejects these, but stay consistent).
                f.pending_obr = f.pending_obr.saturating_sub(1);
            }
            Ok(())
        }
    }
}

fn nonzero(n: u32) -> Option<u32> {
    (n > 0).then_some(n)
}

fn conj(m: MatchOp) -> &'static str {
    match m {
        MatchOp::All => "and",
        MatchOp::Any => "or",
    }
}

fn zm_op(op: FilterOp) -> &'static str {
    match op {
        FilterOp::Eq => "=",
        FilterOp::Ne => "!=",
        FilterOp::Gt => ">",
        FilterOp::Gte => ">=",
        FilterOp::Lt => "<",
        FilterOp::Lte => "<=",
        FilterOp::Like => "LIKE",
        FilterOp::NotLike => "NOT LIKE",
        FilterOp::In => "=[]",
        FilterOp::NotIn => "!=[]",
        FilterOp::IsNull => "IS",
        FilterOp::IsNotNull => "IS NOT",
        FilterOp::Regexp => "=~",
        FilterOp::NotRegexp => "!~",
    }
}

fn zm_val(op: FilterOp, value: &Option<Value>, kind: ValueKind) -> String {
    match op {
        FilterOp::IsNull | FilterOp::IsNotNull => "NULL".to_string(),
        FilterOp::In | FilterOp::NotIn => value
            .as_ref()
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|v| scalar_to_string(v, kind))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default(),
        _ => value
            .as_ref()
            .map(|v| scalar_to_string(v, kind))
            .unwrap_or_default(),
    }
}

fn scalar_to_string(v: &Value, _kind: ValueKind) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// ZM flat Query_json -> AST
// ---------------------------------------------------------------------------

/// Parse a stored ZoneMinder flat `Query_json` back into the AST for display.
///
/// Returns `Ok(None)` when the JSON has no usable terms (e.g. `{}` or a legacy
/// filter we don't model), and `Err` only on malformed/unrepresentable input.
pub fn from_zm_query_json(query_json: &str) -> AppResult<Option<FilterQuery>> {
    let trimmed = query_json.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let root: Value = serde_json::from_str(trimmed)
        .map_err(|e| bad(format!("stored query_json invalid: {e}")))?;
    let Some(terms) = root.get("terms").and_then(Value::as_array) else {
        return Ok(None);
    };
    let mut parsed_terms: Vec<ParsedTerm> = Vec::new();
    for term in terms {
        // `Ok(None)` skips a non-condition row; `Err` (unrepresentable attr/op)
        // bubbles up and the caller falls back to no AST.
        if let Some(p) = parse_term(term)? {
            parsed_terms.push(p);
        }
    }
    if parsed_terms.is_empty() {
        return Ok(None);
    }

    let tokens = to_tokens(&parsed_terms);
    let predicate = Parser::new(tokens).parse()?;

    let sort = root
        .get("sort_field")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .and_then(from_zm_attr)
        .map(|field| {
            let asc = root.get("sort_asc").and_then(zm_truthy).unwrap_or(false);
            crate::dto::request::filter_ast::FilterSort {
                field,
                dir: if asc {
                    SortDirection::Asc
                } else {
                    SortDirection::Desc
                },
            }
        });
    let limit = root.get("limit").and_then(|v| match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.trim().parse::<u64>().ok(),
        _ => None,
    });

    Ok(Some(FilterQuery {
        predicate,
        sort,
        limit,
    }))
}

fn zm_truthy(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::Number(n) => n.as_u64().map(|n| n != 0),
        Value::String(s) => Some(matches!(s.trim(), "1" | "true" | "True")),
        _ => None,
    }
}

struct ParsedTerm {
    cnj: Option<MatchOp>,
    obr: u32,
    cbr: u32,
    field: FilterField,
    op: FilterOp,
    value: Option<Value>,
}

/// Parse one stored ZM term. `Ok(None)` means "skip this row" (e.g. a bare
/// conjunction placeholder with no `attr`); `Err` means the term references an
/// attribute/operator outside our vocabulary, so the AST can't be shown.
fn parse_term(term: &Value) -> AppResult<Option<ParsedTerm>> {
    let Some(obj) = term.as_object() else {
        return Ok(None);
    };
    // Some ZM rows carry no attr (e.g. a bare conjunction placeholder); skip.
    let Some(attr) = obj.get("attr").and_then(Value::as_str) else {
        return Ok(None);
    };
    let field =
        from_zm_attr(attr).ok_or_else(|| bad(format!("unsupported filter attribute {attr:?}")))?;
    let op_sym = obj.get("op").and_then(Value::as_str).unwrap_or("=");
    let val = obj.get("val");
    let (op, value) = from_zm_op(op_sym, val, meta(field).kind)
        .ok_or_else(|| bad(format!("unsupported filter operator {op_sym:?}")))?;
    let count = |k: &str| -> u32 {
        match obj.get(k) {
            Some(Value::Number(n)) => n.as_u64().unwrap_or(0) as u32,
            Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
            _ => 0,
        }
    };
    let cnj = obj
        .get("cnj")
        .and_then(Value::as_str)
        .map(|s| match s.to_lowercase().as_str() {
            "or" => MatchOp::Any,
            _ => MatchOp::All,
        });
    Ok(Some(ParsedTerm {
        cnj,
        obr: count("obr"),
        cbr: count("cbr"),
        field,
        op,
        value,
    }))
}

fn from_zm_op(
    sym: &str,
    val: Option<&Value>,
    kind: ValueKind,
) -> Option<(FilterOp, Option<Value>)> {
    let to_typed = |v: Option<&Value>| -> Option<Value> { v.map(|v| zm_str_to_value(v, kind)) };
    let split_set = |v: Option<&Value>| -> Value {
        let s = v.and_then(Value::as_str).unwrap_or("");
        Value::Array(
            s.split(',')
                .filter(|p| !p.is_empty())
                .map(|p| zm_str_to_value(&Value::String(p.trim().to_string()), kind))
                .collect(),
        )
    };
    Some(match sym {
        "=" => (FilterOp::Eq, to_typed(val)),
        "!=" => (FilterOp::Ne, to_typed(val)),
        ">" => (FilterOp::Gt, to_typed(val)),
        ">=" => (FilterOp::Gte, to_typed(val)),
        "<" => (FilterOp::Lt, to_typed(val)),
        "<=" => (FilterOp::Lte, to_typed(val)),
        "LIKE" => (FilterOp::Like, to_typed(val)),
        "NOT LIKE" => (FilterOp::NotLike, to_typed(val)),
        "=[]" => (FilterOp::In, Some(split_set(val))),
        "!=[]" | "!~[]" => (FilterOp::NotIn, Some(split_set(val))),
        "=~" => (FilterOp::Regexp, to_typed(val)),
        "!~" => (FilterOp::NotRegexp, to_typed(val)),
        "IS" => (FilterOp::IsNull, None),
        "IS NOT" => (FilterOp::IsNotNull, None),
        _ => return None,
    })
}

fn zm_str_to_value(v: &Value, kind: ValueKind) -> Value {
    let s = match v {
        Value::String(s) => s.clone(),
        other => return other.clone(),
    };
    match kind {
        ValueKind::Int => s
            .parse::<i64>()
            .map(Value::from)
            .unwrap_or(Value::String(s)),
        ValueKind::Decimal => s
            .parse::<f64>()
            .map(Value::from)
            .unwrap_or(Value::String(s)),
        ValueKind::Bool => match s.as_str() {
            "1" | "true" => Value::Bool(true),
            "0" | "false" => Value::Bool(false),
            _ => Value::String(s),
        },
        ValueKind::Str | ValueKind::DateTime => Value::String(s),
    }
}

// --- token stream + precedence parser (AND binds tighter than OR) -----------

enum Token {
    Open,
    Close,
    And,
    Or,
    Leaf(FilterField, FilterOp, Option<Value>),
}

fn to_tokens(terms: &[ParsedTerm]) -> Vec<Token> {
    let mut out = Vec::new();
    for (i, t) in terms.iter().enumerate() {
        if i > 0 {
            out.push(match t.cnj.unwrap_or(MatchOp::All) {
                MatchOp::All => Token::And,
                MatchOp::Any => Token::Or,
            });
        }
        for _ in 0..t.obr {
            out.push(Token::Open);
        }
        out.push(Token::Leaf(t.field, t.op, t.value.clone()));
        for _ in 0..t.cbr {
            out.push(Token::Close);
        }
    }
    out
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse(mut self) -> AppResult<FilterExpr> {
        let expr = self.parse_or()?;
        if self.pos != self.tokens.len() {
            return Err(bad("malformed stored filter (unbalanced brackets)"));
        }
        Ok(expr)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn parse_or(&mut self) -> AppResult<FilterExpr> {
        let mut children = vec![self.parse_and()?];
        while matches!(self.peek(), Some(Token::Or)) {
            self.pos += 1;
            children.push(self.parse_and()?);
        }
        Ok(group_or_single(MatchOp::Any, children))
    }

    fn parse_and(&mut self) -> AppResult<FilterExpr> {
        let mut children = vec![self.parse_primary()?];
        while matches!(self.peek(), Some(Token::And)) {
            self.pos += 1;
            children.push(self.parse_primary()?);
        }
        Ok(group_or_single(MatchOp::All, children))
    }

    fn parse_primary(&mut self) -> AppResult<FilterExpr> {
        match self.tokens.get(self.pos) {
            Some(Token::Open) => {
                self.pos += 1;
                let inner = self.parse_or()?;
                match self.tokens.get(self.pos) {
                    Some(Token::Close) => {
                        self.pos += 1;
                        Ok(inner)
                    }
                    _ => Err(bad("malformed stored filter (missing close bracket)")),
                }
            }
            Some(Token::Leaf(field, op, value)) => {
                let leaf = FilterExpr::Condition {
                    field: *field,
                    op: *op,
                    value: value.clone(),
                };
                self.pos += 1;
                Ok(leaf)
            }
            _ => Err(bad("malformed stored filter (expected a condition)")),
        }
    }
}

fn group_or_single(match_op: MatchOp, mut children: Vec<FilterExpr>) -> FilterExpr {
    if children.len() == 1 {
        children.pop().expect("one child")
    } else {
        FilterExpr::Group {
            match_op,
            rules: children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::request::filter_ast::{FilterSort, MatchOp};

    fn cond(field: FilterField, op: FilterOp, value: Value) -> FilterExpr {
        FilterExpr::Condition {
            field,
            op,
            value: Some(value),
        }
    }

    #[test]
    fn flattens_nested_and_or_with_brackets() {
        // monitor_id = 5 AND ( name LIKE '%f%' OR max_score >= 80 )
        let q = FilterQuery {
            predicate: FilterExpr::Group {
                match_op: MatchOp::All,
                rules: vec![
                    cond(FilterField::MonitorId, FilterOp::Eq, json!(5)),
                    FilterExpr::Group {
                        match_op: MatchOp::Any,
                        rules: vec![
                            cond(FilterField::Name, FilterOp::Like, json!("%f%")),
                            cond(FilterField::MaxScore, FilterOp::Gte, json!(80)),
                        ],
                    },
                ],
            },
            sort: Some(FilterSort {
                field: FilterField::StartTime,
                dir: SortDirection::Desc,
            }),
            limit: Some(100),
        };
        let json = to_zm_query_json(&q).expect("translate");
        let v: Value = serde_json::from_str(&json).unwrap();
        let terms = v["terms"].as_array().unwrap();
        assert_eq!(terms.len(), 3);
        // Outer "(" + inner "(" land on the first/second leaves; both ")" on the last.
        assert_eq!(terms[0]["attr"], "MonitorId");
        assert_eq!(terms[0]["obr"], 1);
        assert_eq!(terms[1]["cnj"], "and");
        assert_eq!(terms[1]["attr"], "Name");
        assert_eq!(terms[1]["obr"], 1);
        assert_eq!(terms[2]["cnj"], "or");
        assert_eq!(terms[2]["cbr"], 2);
        assert_eq!(v["sort_field"], "StartDateTime");
        assert_eq!(v["limit"], "100");
    }

    #[test]
    fn round_trips_through_zm_format() {
        let q = FilterQuery {
            predicate: FilterExpr::Group {
                match_op: MatchOp::All,
                rules: vec![
                    cond(FilterField::MonitorId, FilterOp::Eq, json!(5)),
                    FilterExpr::Group {
                        match_op: MatchOp::Any,
                        rules: vec![
                            cond(FilterField::Name, FilterOp::Like, json!("%f%")),
                            cond(FilterField::MaxScore, FilterOp::Gte, json!(80)),
                        ],
                    },
                ],
            },
            sort: None,
            limit: None,
        };
        let json = to_zm_query_json(&q).unwrap();
        let back = from_zm_query_json(&json).unwrap().expect("some");
        // Re-flatten and compare term structure for semantic equality.
        let again = to_zm_query_json(&back).unwrap();
        assert_eq!(json, again, "round-trip should be stable");
    }

    #[test]
    fn rejects_type_mismatch_and_empty_group() {
        let bad_type = FilterQuery {
            predicate: cond(FilterField::MaxScore, FilterOp::Eq, json!("not a number")),
            sort: None,
            limit: None,
        };
        assert!(to_zm_query_json(&bad_type).is_err());

        let empty = FilterQuery {
            predicate: FilterExpr::Group {
                match_op: MatchOp::All,
                rules: vec![],
            },
            sort: None,
            limit: None,
        };
        assert!(to_zm_query_json(&empty).is_err());
    }

    #[test]
    fn empty_or_unmodelled_json_yields_none() {
        assert!(from_zm_query_json("{}").unwrap().is_none());
        assert!(from_zm_query_json(r#"{"filter":"all"}"#).unwrap().is_none());
    }
}
