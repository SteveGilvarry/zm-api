#!/usr/bin/env python3
"""Compare the current OpenAPI spec against the committed baseline.

Exists because `GET /me` changed shape from `UserResponse` to `MeResponse`
without a changelog entry, and the first anyone knew was ~45 dashboard e2e
timeouts hunting for controls that no longer rendered (GH #62). The spec is
generated from the code, so a change of that kind is mechanically detectable
in the pull request that makes it.

Additive changes pass with a notice — new endpoints and new optional fields
break nobody. Changes that can break a client fail:

  * a path or operation that existed is gone
  * the schema behind an existing 2xx response was replaced
  * a required field was removed from a response
  * an enum lost a value
  * a request gained a required field

Usage:
    check-openapi-compat.py <baseline.json> <current.json>

Refresh the baseline with `zm-api --openapi > openapi.json` once the change is
intended and written down.
"""

from __future__ import annotations

import json
import sys

METHODS = ("get", "put", "post", "delete", "patch", "head", "options", "trace")


def load(path: str) -> dict:
    try:
        with open(path, encoding="utf-8") as fh:
            return json.load(fh)
    except FileNotFoundError:
        sys.exit(f"error: {path} not found")
    except json.JSONDecodeError as exc:
        sys.exit(f"error: {path} is not valid JSON: {exc}")


def operations(spec: dict) -> dict[tuple[str, str], dict]:
    out = {}
    for path, item in (spec.get("paths") or {}).items():
        for method in METHODS:
            if method in item:
                out[(path, method)] = item[method]
    return out


def success_schema(op: dict) -> str | None:
    """The schema behind an operation's first 2xx JSON response."""
    for code, resp in (op.get("responses") or {}).items():
        if not str(code).startswith("2"):
            continue
        content = (resp or {}).get("content") or {}
        for media in content.values():
            schema = (media or {}).get("schema") or {}
            if "$ref" in schema:
                return schema["$ref"]
            # Inline arrays are common enough to be worth naming precisely.
            items = schema.get("items") or {}
            if schema.get("type") == "array" and "$ref" in items:
                return f"array<{items['$ref']}>"
            if schema:
                return json.dumps(schema, sort_keys=True)[:120]
    return None


def schemas(spec: dict) -> dict:
    return ((spec.get("components") or {}).get("schemas")) or {}


def _refs(node) -> set[str]:
    """Every `#/components/schemas/X` name reachable from a JSON subtree."""
    found: set[str] = set()
    if isinstance(node, dict):
        ref = node.get("$ref")
        if isinstance(ref, str) and ref.startswith("#/components/schemas/"):
            found.add(ref.rsplit("/", 1)[-1])
        for value in node.values():
            found |= _refs(value)
    elif isinstance(node, list):
        for value in node:
            found |= _refs(value)
    return found


def schema_roles(spec: dict) -> tuple[set[str], set[str]]:
    """Split schemas into those used for requests and those used for responses.

    The direction decides what counts as breaking. Gaining a required field is
    fine on a response — the client simply receives more — but breaks a request,
    because the server now demands something the client does not send. Losing a
    guaranteed field is the mirror image.

    Resolved transitively: a schema nested inside a request body inherits the
    request role.
    """
    request_seeds: set[str] = set()
    response_seeds: set[str] = set()
    for op in operations(spec).values():
        request_seeds |= _refs(op.get("requestBody") or {})
        response_seeds |= _refs(op.get("responses") or {})

    defs = schemas(spec)

    def close(seeds: set[str]) -> set[str]:
        seen, queue = set(), list(seeds)
        while queue:
            name = queue.pop()
            if name in seen or name not in defs:
                continue
            seen.add(name)
            queue.extend(_refs(defs[name]))
        return seen

    return close(request_seeds), close(response_seeds)


def compare(old: dict, new: dict) -> tuple[list[str], list[str]]:
    breaking: list[str] = []
    additive: list[str] = []

    old_ops, new_ops = operations(old), operations(new)

    for key in sorted(old_ops.keys() - new_ops.keys()):
        breaking.append(f"removed operation: {key[1].upper()} {key[0]}")
    for key in sorted(new_ops.keys() - old_ops.keys()):
        additive.append(f"new operation: {key[1].upper()} {key[0]}")

    for key in sorted(old_ops.keys() & new_ops.keys()):
        before, after = success_schema(old_ops[key]), success_schema(new_ops[key])
        if before != after:
            breaking.append(
                f"response shape changed: {key[1].upper()} {key[0]}\n"
                f"      was: {before}\n"
                f"      now: {after}"
            )

    old_schemas, new_schemas = schemas(old), schemas(new)

    for name in sorted(old_schemas.keys() - new_schemas.keys()):
        breaking.append(f"removed schema: {name}")
    for name in sorted(new_schemas.keys() - old_schemas.keys()):
        additive.append(f"new schema: {name}")

    in_requests, in_responses = schema_roles(new)

    for name in sorted(old_schemas.keys() & new_schemas.keys()):
        before, after = old_schemas[name], new_schemas[name]

        gone = set(before.get("required") or []) - set(after.get("required") or [])
        if gone and name in in_responses:
            breaking.append(
                f"{name}: response field(s) no longer guaranteed: {', '.join(sorted(gone))}"
            )
        elif gone:
            additive.append(
                f"{name}: request field(s) no longer required: {', '.join(sorted(gone))}"
            )

        old_enum, new_enum = before.get("enum"), after.get("enum")
        if old_enum and new_enum:
            dropped = set(old_enum) - set(new_enum)
            gained = set(new_enum) - set(old_enum)
            if dropped:
                breaking.append(
                    f"{name}: enum value(s) removed: {', '.join(sorted(map(str, dropped)))}"
                )
            elif gained:
                additive.append(
                    f"{name}: enum value(s) added: {', '.join(sorted(map(str, gained)))}"
                )

        old_props = set((before.get("properties") or {}).keys())
        new_props = set((after.get("properties") or {}).keys())
        for prop in sorted(new_props - old_props):
            required_now = prop in (after.get("required") or [])
            if required_now and name in in_requests:
                breaking.append(
                    f"{name}: request gained a required field: {prop} "
                    f"(existing clients do not send it)"
                )
            elif required_now:
                additive.append(f"{name}: new response field: {prop}")
            else:
                additive.append(f"{name}: new optional field: {prop}")

    return breaking, additive


def main() -> int:
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    baseline, current = load(sys.argv[1]), load(sys.argv[2])

    breaking, additive = compare(baseline, current)

    if not breaking and not additive:
        print("OpenAPI spec matches the committed baseline.")
        return 0

    if additive:
        print(f"Additive changes ({len(additive)}) — these break no client:")
        for line in additive:
            print(f"  + {line}")
        print()

    if not breaking:
        print("No breaking changes. Refresh the baseline when convenient:")
        print("    cargo run --bin zm-api -- --openapi > openapi.json")
        return 0

    print(f"BREAKING changes ({len(breaking)}):")
    for line in breaking:
        print(f"  ! {line}")
    print()
    print("These can break a deployed client. Before merging:")
    print("  1. Confirm the change is intended.")
    print("  2. Add a CHANGELOG entry under '### Changed', marked BREAKING.")
    print("  3. Refresh the baseline:")
    print("       cargo run --bin zm-api -- --openapi > openapi.json")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
