//! Properties the served OpenAPI document must hold (GH #32).
//!
//! These are the ones a generated client trips over. They run without a
//! database because the spec is built from the code, not from data.

use std::collections::{BTreeMap, BTreeSet};

use utoipa::OpenApi;

fn spec() -> serde_json::Value {
    serde_json::to_value(zm_api::handlers::openapi::ApiDoc::openapi()).expect("serialise")
}

fn operations(spec: &serde_json::Value) -> Vec<(String, String, serde_json::Value)> {
    let mut out = Vec::new();
    for (path, item) in spec["paths"].as_object().expect("paths") {
        for (method, op) in item.as_object().expect("path item") {
            if op.get("operationId").is_some() {
                out.push((method.to_uppercase(), path.clone(), op.clone()));
            }
        }
    }
    out
}

/// Every route that is genuinely reachable without a token.
///
/// Deliberately the same set as `rbac_completeness.rs`'s `PUBLIC`, because a
/// route being unauthenticated in the spec and unauthenticated in the router
/// are two claims that must not drift apart. If one list grows, the other has
/// to as well — and a reviewer should notice.
const PUBLIC: &[&str] = &[
    "/api/v3/auth/login",
    "/api/v3/auth/refresh",
    "/api/v3/host/getVersion",
    "/api/v3/server/health_check",
];

/// `operationId` is what every code generator names its method after, so a
/// duplicate silently overwrites one of the two.
#[test]
fn operation_ids_are_unique() {
    let spec = spec();
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (method, path, op) in operations(&spec) {
        let id = op["operationId"].as_str().expect("string id").to_string();
        seen.entry(id).or_default().push(format!("{method} {path}"));
    }
    let dupes: Vec<_> = seen.iter().filter(|(_, v)| v.len() > 1).collect();
    assert!(
        dupes.is_empty(),
        "duplicate operationIds break client generation — one method silently \
         replaces the other:\n{dupes:#?}"
    );
}

/// An operation with no `security` reads as "no token required". Saying that
/// about a route that enforces auth sends a generated client into a 401 it was
/// told not to expect.
#[test]
fn only_genuinely_public_routes_omit_security() {
    let spec = spec();
    let public: BTreeSet<&str> = PUBLIC.iter().copied().collect();

    let undeclared: Vec<String> = operations(&spec)
        .into_iter()
        .filter(|(_, path, op)| op.get("security").is_none() && !public.contains(path.as_str()))
        .map(|(method, path, _)| format!("{method} {path}"))
        .collect();

    assert!(
        undeclared.is_empty(),
        "these enforce authentication but the spec does not say so — add \
         security((\"jwt\" = [])) or add them to PUBLIC with a reason:\n  {}",
        undeclared.join("\n  ")
    );
}

/// The reverse: a route listed as public must not also declare security, or the
/// list is lying.
#[test]
fn the_public_list_has_no_stale_or_contradictory_entries() {
    let spec = spec();
    let ops = operations(&spec);

    for path in PUBLIC {
        let matching: Vec<_> = ops.iter().filter(|(_, p, _)| p == path).collect();
        assert!(
            !matching.is_empty(),
            "PUBLIC names {path}, which the spec does not serve — remove it or \
             fix the path"
        );
        for (method, _, op) in matching {
            assert!(
                op.get("security").is_none(),
                "{method} {path} is listed as public but declares security"
            );
        }
    }
}

/// A `date-time` with no offset is not RFC 3339, and generators produce a
/// parser that rejects the values this API actually sends.
#[test]
fn datetime_examples_carry_an_offset() {
    let spec = spec();
    let Some(schemas) = spec["components"]["schemas"].as_object() else {
        return;
    };

    let mut naive = Vec::new();
    for (name, schema) in schemas {
        if schema.get("format").and_then(|f| f.as_str()) != Some("date-time") {
            continue;
        }
        if let Some(example) = schema.get("example").and_then(|e| e.as_str()) {
            let has_offset =
                example.ends_with('Z') || example.rfind(['+', '-']).is_some_and(|i| i > 10);
            if !has_offset {
                naive.push(format!("{name}: {example}"));
            }
        }
    }
    assert!(
        naive.is_empty(),
        "a date-time example without a timezone offset is not RFC 3339:\n  {}",
        naive.join("\n  ")
    );
}
