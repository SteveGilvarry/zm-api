# API reference

Every endpoint is generated from the running server's own OpenAPI 3.1 document,
so it cannot drift from the code. The same document is served live at
`/api-docs/openapi.json` on any zm-api instance and is attached to every GitHub
release.

<p class="zm-cta" style="justify-content:flex-start;margin:1.5rem 0;">
<a class="primary" href="../api/index.html" target="_blank" rel="noopener">Open the full reference ↗</a>
<a href="../openapi.json" download>Download openapi.json</a>
</p>

The explorer below is the same page, embedded. It reads better full-screen.

<iframe class="zm-api-frame" src="../api/index.html" title="zm-api OpenAPI reference" loading="lazy"></iframe>

## Using the spec directly

`openapi.json` is a plain OpenAPI 3.1 document — feed it to any generator:

```bash
# From a running instance
curl -s localhost:8080/api-docs/openapi.json > openapi.json

# Or from the binary, without starting a server
zm-api --openapi > openapi.json
```

Because `--openapi` needs no database and no configuration, it is also the way
to diff the API surface between two releases:

```bash
diff <(zm-api-3.0.0-alpha.1 --openapi | jq -S .) \
     <(zm-api-3.0.0-alpha.2 --openapi | jq -S .)
```

## Conventions

Details are on their own pages, but in short:

- Everything lives under `/api/v3`.
- Authentication is `Authorization: Bearer <jwt>`; see [Authentication](../guide/authentication.md).
- Reads need `View` on the relevant feature, writes need `Edit`; see [Permissions](../guide/permissions.md).
- List endpoints take `page` and `page_size` and return `{ items, total, per_page, current_page, last_page }`.
- Errors return `{ kind, error_message, code, details }` — `kind` is a stable
  string such as `INVALID_INPUT_ERROR` or `NOT_FOUND_ERROR`, and `details`
  names the offending fields.
