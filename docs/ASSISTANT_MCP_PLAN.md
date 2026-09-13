# Assistant and MCP plan

Status: **Plan, not built.** Written 2026-09-13.

zm-api gets one Rust tool registry and two ways in:

- `POST /api/v3/chat/completion`: a server-side agent loop against an
  OpenAI-compatible chat model, streamed as NDJSON.
- `/mcp`: a Model Context Protocol server (the `rmcp` crate) exposing the same
  tools to external MCP hosts (Claude Desktop, IDEs, Home Assistant's MCP
  client).

Both call the same tool code, which calls service functions directly (not HTTP)
and enforces the same JWT, RBAC and per-monitor ACL as the REST API. Tools are
read-only by default. Write tools exist only when config turns them on, only for
admins, and only after an explicit approval.

## What we're copying and what we aren't

**Frigate 0.18** (`frigate/api/chat.py`, `frigate/genai/prompts.py`, tag
`v0.18.0`):

- Keep: OpenAI function-calling tool definitions; a server-side loop capped by
  `max_tool_iterations` (default 5, client may ask for 1–10); NDJSON streaming
  with typed lines; the client replays the returned `messages` chain next turn
  (keeps the prompt prefix stable for caching); images never go in tool
  messages but are re-injected as a user message with `image_url` parts, and
  only when the model supports vision; per-camera access checks inside each
  tool.
- Don't copy: Frigate has no approval step. `set_camera_state` just checks the
  `remote-role` header and tells the model "admin required". It also sends
  images to any vision-capable provider, cloud included, with no policy switch.
  Both are fixed below.

**zmNinjaNg** (`app/src/lib/assistant/`):

- Keep: read-only enforced by structure (the tool type can't express a
  mutation) plus a refusal for known action names; the two code-decidable
  grounding checks in `grounding.ts` (answer denies data the tools found; answer
  echoes raw tool JSON) with one corrective retry, then a fallback to the tool's
  own summary; history bounded by message count, characters and turns.
- Different here: zmNinjaNg's loop runs on the client and calls REST endpoints,
  and it never sends images to a model. zm-api runs the loop server-side, where
  the tools can use service functions, the stream socket and the VLM directly.
  zmNinjaNg can keep its client loop and call the MCP endpoint or the same REST
  API; nothing here forces it to switch.

## Tool registry

`src/service/assistant/tools/`. One trait, one registry built at startup from
config:

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;   // JSON Schema
    fn access(&self) -> ToolAccess;               // Read { feature } | Write { feature }
    async fn call(&self, ctx: &ToolCtx, args: serde_json::Value) -> Result<ToolOutput, ToolError>;
}

pub struct ToolCtx<'a> {
    pub state: &'a AppState,
    pub claims: &'a UserClaims,      // uid, permissions
    pub scope: &'a MonitorScope,     // resolved once per request
}

pub struct ToolOutput {
    pub json: serde_json::Value,     // what the model sees
    pub summary: String,             // one line for the UI and the grounding fallback
    pub images: Vec<ToolImage>,      // never inlined into json; see "Images" below
    pub cited_event_ids: Vec<u64>,   // ids the model may legitimately mention
}
```

Before `call` runs, the dispatcher checks `access` against the caller's
`UserPermissions` (same `Feature`/`Level` as `util::authz`). Every tool that
takes a monitor id checks `scope.allows(id, Level::View)` and reports a hidden
monitor as not found, as the REST handlers do. Arguments are deserialized into
a typed struct with `garde` limits (lengths, `limit <= 50`, time windows <= 31
days), so the model can't ask for an unbounded scan.

### Read-only tools

| Tool | Parameters | Backed by | Feature |
|---|---|---|---|
| `list_monitors` | `group_id?` | `service::monitor::list_all` | Monitors:View |
| `count_events` | `monitor_id?`, `after`, `before`, `object?` | `service::events::get_event_counts` / `get_event_counts_by_monitor`; with `object`, a filtered `store.count()` in `service::search` | Events:View |
| `search_events` | `query?`, `monitor_id?`, `after?`, `before?`, `object?`, `limit` | `SearchService::search` (hybrid ANN + FTS + rerank) when `query` is set, else `service::events::list` | Events:View |
| `find_similar` | `event_id`, `limit` | `SearchService::similar` | Events:View |
| `get_event` | `event_id` | `service::events::get_by_id` plus tags, notes (zm-next description), cause and top detections | Events:View |
| `get_event_image` | `event_id` | the image `GET /api/v3/events/{id}/thumbnail` serves (`handlers::events_playback::get_event_thumbnail`), moved into a service fn; alarm-frame and frame-by-id selection later | Events:View |
| `get_live_context` | `monitor_id`, `with_image?`, `describe?` | `service::zmnext::ondemand::snapshot` (fresh JPEG), the latest detection from the open or last event, and `ondemand::describe` when `describe` is set; for non-zm-next monitors, `SnapshotService::get_snapshot` and no describe | Stream:View |
| `recap` | `after`, `before`, `monitor_id?` | new `service::events::recap`: events in the window grouped by monitor, each with start/end, cause, notes, peak score and top objects; the model writes the prose from that | Events:View |

`recap` stays deterministic on purpose. The model summarises rows it was given;
it never gets to pick which rows exist. Time words ("last night", "since I
left") are resolved by the model into `after`/`before`, and the tool echoes the
resolved window back so the answer can state it.

### Write tools (later, off by default)

Candidates: `set_monitor_function` (Monitor/Record/None), `trigger_alarm`,
`set_event_archived`, `tag_event`. They're listed so the gate gets designed
now, not so they get built in phase 1.

A write tool is registered only when **all** of these hold:

1. `[assistant].write_tools = true` (config, default false);
2. the tool's own entry in `[assistant].enabled_write_tools`;
3. the caller has the tool's `Feature` at `Edit` **and** `System:Edit`.

Otherwise it's absent from the tool list sent to the model. If a model names it
anyway, the dispatcher returns the fixed refusal (zmNinjaNg's
`WITHHELD_TOOL_REFUSAL` idea).

## Chat endpoint

`POST /api/v3/chat/completion`, JWT, `Feature::Events` at View to reach it at
all. Individual tools check their own features.

Request:

```json
{
  "messages": [{"role": "user", "content": "Was anyone at the front door last night?"}],
  "stream": true,
  "max_tool_iterations": 5,
  "approvals": [{"tool_call_id": "call_abc", "approval_token": "..."}]
}
```

Loop (in `src/service/assistant/loop.rs`):

1. Truncate history (last 3 turns, 40 messages, 12 000 characters, as in
   zmNinjaNg) and prepend the system prompt, which states the current time and
   the caller's timezone, the monitors they can see (names only), and the
   grounding rules.
2. Call the model with the caller's allowed tools.
3. If it returns tool calls: run read tools concurrently (each with a timeout:
   `get_live_context` uses the command timeouts from `[zmnext.commands]`,
   everything else 10 s); append `role: tool` messages carrying `json` only.
   Handle images as below. Go round again.
4. If a call is a write tool with no matching approval, stop the loop and emit
   `approval_required` (below). The client asks the user and re-POSTs the same
   `messages` plus `approvals`.
5. With no tool calls: run the grounding checks, retry once with a correction
   if they fail, otherwise fall back to the concatenated tool summaries. Emit
   the answer.
6. At the iteration cap, emit a fixed "stopped after N tool rounds" message with
   `finish_reason: "length"`.

The model client extends `service::search::provider` (already OpenAI-compatible
over `reqwest`) with tool calling and streaming. Config:

```toml
[assistant]
enabled = false
chat_url = "http://127.0.0.1:8080/v1/chat/completions"
model = ""
api_key = ""                 # optional; sent as Bearer
vision = "auto"              # auto | on | off
image_policy = "local_only"  # local_only | allow_remote | never
max_tool_iterations = 5
write_tools = false
enabled_write_tools = []
```

### Streaming format

`Content-Type: application/x-ndjson`, one JSON object per line. Same shapes as
Frigate where they overlap, so a client can target both:

```text
{"type":"messages","messages":[...]}                  full chain; first line and after each tool round
{"type":"tool_call","id":"call_1","name":"search_events","arguments":{...}}
{"type":"tool_result","id":"call_1","summary":"3 events on Front Door","images":["/api/v3/events/812/thumbnail"]}
{"type":"content","delta":"Yes, twice: "}
{"type":"reasoning","delta":"..."}                     only for models that stream reasoning
{"type":"approval_required","id":"call_2","name":"set_monitor_function","arguments":{...},"approval_token":"...","expires_at":"..."}
{"type":"error","error":"..."}
{"type":"done","finish_reason":"stop","tool_iterations":2}
```

`tool_result` gives the UI a summary and zm-api image URLs (which the client
fetches with its own token). It never carries tool JSON or image bytes.
`stream:false` returns one JSON body with `message`, `finish_reason`,
`tool_iterations`, `tool_calls` and `messages`.

### Approval gate

Stateless, so it survives a restart and needs no session table:

- `approval_token` = HMAC-SHA256 over `(uid, tool_call_id, tool name, canonical
  JSON of arguments, expiry)`, keyed with the JWT signing secret, 5-minute
  expiry.
- On the follow-up POST the server recomputes the HMAC over the tool call it
  finds in `messages`. If the client (or a prompt-injected model on replay)
  changed the arguments, the token doesn't match and the call is refused.
- Approval covers that single call. A second write in the same turn needs its
  own approval.
- Every executed write tool logs an audit line: user, tool, arguments, result.

## MCP server

`/mcp`, Streamable HTTP transport, using `rmcp`'s server and axum integration.
Needs a spike to confirm the current `rmcp` API and how it mounts into an axum
0.8 `Router` (check with Context7 before building).

- **Auth:** `Authorization: Bearer <zm-api JWT>`, the same access token as the
  REST API, checked by the existing revocation-aware middleware. Each MCP
  session resolves `UserClaims` and `MonitorScope` once, then re-resolves the
  scope per call so permission changes apply immediately. No anonymous MCP.
- **Tools:** the registry, filtered by the caller's permissions exactly as for
  chat. Each tool's JSON Schema becomes its MCP `inputSchema`. Annotations:
  `readOnlyHint: true` on read tools, `destructiveHint` and
  `readOnlyHint: false` on write tools.
- **Results:** `json` goes out as text content (and `structuredContent` where
  the client supports it); images go out as MCP `ImageContent` (base64 JPEG),
  subject to the image policy below, since the MCP host decides which model
  sees them.
- **Writes over MCP:** MCP hosts show their own confirmation for tool calls,
  but zm-api can't verify that happened. So the server-side conditions still
  apply (config, per-tool enable, admin), and `[assistant].mcp_write_tools`
  (default false) must also be on. The HMAC approval flow doesn't apply to MCP.
- **Resources (later):** `zm://monitors/{id}/snapshot`, `zm://events/{id}` as
  read-only resources if hosts find them useful. Not phase 1.

## Images and cloud models

This is where privacy is decided, so the rules are explicit:

1. **Tool messages carry text only.** An image a tool produced is held aside
   and, when allowed, attached to a following `user` message as an `image_url`
   data URI, as Frigate does. That's what OpenAI-compatible APIs accept.
2. **`vision`:** `off` never attaches images. `auto` attaches only when the
   model reports image input (llama.cpp and vLLM `/v1/models` metadata) or the
   config says `on`.
3. **`image_policy`**, checked against the host in `chat_url`:
   - `local_only` (default): attach only when `chat_url` resolves to loopback,
     RFC 1918, or link-local. For a remote model, use the zm-next VLM instead:
     `get_live_context` calls `describe_now` and returns the description as text,
     and `get_event_image` returns the event's stored description (notes). The
     frame never leaves the site; the cloud model gets words.
   - `allow_remote`: attach for any host. The operator has opted in; the
     response carries a `"remote_images": true` flag so a UI can show it.
   - `never`: text only, everywhere.
4. **Before attaching:** downscale to at most 1024 px on the long side,
   re-encode JPEG (which drops EXIF/GPS), attach at most 4 images per turn, and
   log each attachment (event or monitor id, never the bytes).
5. **MCP** follows the same policy. zm-api can't see which model an MCP host
   uses, so `local_only` treats every MCP client as remote unless
   `[assistant].mcp_images = true`.

## Grounding checks

Server-side, adapted from zmNinjaNg's `grounding.ts`:

- **Denies the data:** the answer says nothing was found while a tool this turn
  returned matches.
- **Echoes tool output:** the answer is raw tool JSON instead of prose.
- **Invented ids** (new): every event id in the answer must appear in some
  tool's `cited_event_ids` this turn. Unknown ids are removed from the answer
  and the removal is logged.

On failure: one corrective retry, then the fallback of joined tool summaries.

## Phases

1. Tool trait, registry and the eight read tools, with unit tests per tool
   against the mock DB and ACL tests (restricted user can't see a hidden
   monitor through any tool). No LLM yet: expose `POST /api/v3/chat/tools/{name}`
   (admin-only, debugging) so the tools can be exercised directly.
2. Chat loop, non-streaming first, against a scripted fake OpenAI server in
   tests (tool call → result → answer; iteration cap; grounding retry), then
   NDJSON streaming.
3. Image policy and vision handling, with tests for each policy and host type.
4. MCP endpoint over the same registry; an `rmcp` client test that lists tools
   and calls `list_monitors` with and without a token.
5. Write tools, approval tokens and audit, behind the config switches.

## Open questions

- Which local model to default docs to for tool calling: needs a short bake-off
  on the MariaDB-backed search data (Qwen3 and Llama 3.x instruct sizes that fit
  alongside the VLM).
- Whether `recap` should also use zm-next's `llm_event_review` summaries once
  those land in the DB, instead of per-event notes.
- Rate limiting per user for `get_live_context` with `describe`, since each call
  costs a VLM inference.
