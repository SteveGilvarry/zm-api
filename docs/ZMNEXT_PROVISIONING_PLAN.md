# zm-next Provisioning & Configuration — Implementation Plan

Goal: let an operator take a ZoneMinder monitor, mark it **zm-next**, and configure
the **full plugin pipeline** (detect → track → analytics → describe → audio →
outputs) with arbitrary topology — plus discover cameras over ONVIF and onboard
them as monitors — **without** turning the config store into a second source of
truth for what ZoneMinder already owns.

> Status (2026-06-27): design agreed in session; credentials-over-stdin already
> landed (see below). This doc is the roadmap for the remaining work.

## Architectural principle — a *layered* config model

ZoneMinder's `Monitors`/`Zones` tables stay **authoritative** for everything they
already model. zm-api owns a **new, additive** store for the part ZoneMinder has
no schema for: the zm-next **processing plugin graph**.

| Concern | Owner | Delivery to worker |
|---|---|---|
| Capture URL, credentials, transport | `Monitors.Path`/`User`/`Pass` | re-derived every spawn; creds split out (`split_url_credentials`) and piped over stdin — **never persisted** |
| Recording mode (`store`) | `Monitors.Function` → `StoreMode` | injected at compose time |
| Zones (detection include/exclude, privacy) | `Monitors` → `Zones` (with `Type`) | injected at compose time (see Phase 1/5) |
| **Processing plugin graph** (decode→detect→track→analytics→describe→audio→outputs + per-plugin cfg) | **new zm-api `monitor_pipeline` table** | the stored doc *is* the graph |

At spawn, `generate_pipeline` becomes a **compose** step: build the capture node
from `Monitors`, graft the stored processing graph, inject the `store` node, the
`zones` node, and (if needed) the `privacy_mask`/`encode` nodes — secrets and
capture topology are always re-derived, so the stored doc is *only* the net-new
plugin config. This is why it is **not** a competing source of truth: ZM-owned
fields are never copied into it, so there is nothing to drift.

## Locked decisions

1. **Free plugin graph + validation** (not typed-overrides). The stored doc is the
   `{id,kind,cfg,children}` processing graph; zm-api validates it is well-formed
   with known plugin `kind`s; zm-next remains the deep per-plugin validator.
2. **In-memory delivery (DONE).** Pipeline JSON is piped to the worker over stdin
   (`zm-core --pipeline -`); nothing on disk; crash-restart re-pipes from the
   cached `ManagedProcess.stdin_payload`. Credentials ride as separate
   `username`/`password` fields; the capture plugin builds a transient
   credentialed URL only at `avformat_open_input` and redacts logs.
3. **Secrets never at rest.** The stored graph excludes capture creds; they are
   always injected from `Monitors` at spawn.
4. **Zones stay ZM-authoritative**, injected at compose time — not editable inside
   the free graph (for now).
5. **Privacy:** honor it wherever we are already decoding/encoding (detection,
   live, and a transcode recording path); passthrough recording cannot mask. The
   **correct long-term play is on-camera privacy masks set via ONVIF** — on the
   plan as Phase 6.
6. **Reversibility preserved.** `UseZmNext=0` falls back to legacy zmc/zma; the
   stored doc goes dormant, not deleted.

## Current state (what already exists)

- **Monitor CRUD: already implemented.** `POST /api/v3/monitors` (create),
  `PATCH`/`DELETE /{id}`, state/alarm control; `service::monitor::{create,update,
  delete}`; `CreateMonitorRequest` (128 fields, **no `Default`** — relevant to
  Phase 4). Adding monitors via the API is done.
- **ONVIF discovery: implemented, discover-only.** `POST /api/v3/discovery/probe`
  (WS-Discovery → `CameraCandidate`s) and `/inspect` (per-device → `InspectResult`
  with resolved `rtsp://` `stream_uri` per profile), with WS-Security auth and
  SSRF guards. Nothing creates a Monitor from a discovered device.
- **On-the-fly pipeline generation** from `Monitors`+`Zones` at spawn
  (`pipeline::generate_pipeline`); no stored pipeline table anywhere (checked
  migrations, entities, git history, stash).
- **Credentials-over-stdin: DONE & verified** (both repos) — the layered delivery
  in decision #2 above.

## Key technical findings (shape the work)

- **Detection zones are currently dead, and zone handling is topology-dependent.**
  `decode_detect`/`detect_onnx` read only `roi_motion`/`class_filter` — they ignore
  an inline `zones` array, so today's `detect_cfg["zones"]` does nothing. Spatial
  zone logic in zm-next lives in **two different consumers depending on pipeline
  shape**: (a) the **motion path** — the `zones` plugin (parses ZM `Type`,
  Boost.Geometry R-tree) feeds `motion_pixel_diff` (zone-aware); (b) the **object
  path** — `analytics_rules` (intrusion = object-in-polygon, linecross, loiter) on
  tracker output. So zones can't be a standalone translation; they must be
  **injected into whichever zone-consumer the composed graph contains** — which is
  why zones live inside the config-model work (Phase 2), not a separate phase.
- **Privacy masking needs decoded frames.** `privacy_mask` is a PROCESS plugin on
  decoded RGB/GRAY frames (black/blur/pixelate). The current `store` records the
  **compressed passthrough** stream (hangs off `capture`), so it never sees masked
  frames → privacy can't reach a passthrough recording. Masked recording requires
  a transcode path `capture→decode→privacy_mask→encode_ffmpeg→store`
  (`encode_ffmpeg` exists).
- **`CreateMonitorRequest` has 128 fields, no `Default`.** Onboarding from ONVIF
  needs a construction story (add `Default`/builder, or a dedicated onboard DTO
  that fills a defaulted `ActiveModel`).
- **`migrate_database` is not wired at startup** (`src/client/database.rs:59`,
  only called from tests). Any new owned table needs this wired (or applied via
  packaging) before prod can rely on it.

## ZoneMinder zone `Type` → zm-next mapping

ZoneMinder's zone types are tuned for its **pixel-difference motion** detector and
its alarm-combination algebra (`zm_zone.h:60`, `docs/userguide/definezone.rst`).
zm-next does **object detection (YOLO)**, where the only concept that ports
cleanly is a spatial **region of interest**. So this is an explicit
*reinterpretation*, not a faithful port — the cross-zone logic largely evaporates.

| ZM `Type` | ZoneMinder meaning (pixel-motion) | zm-next (object detection) |
|---|---|---|
| `Active` | default; motion here triggers an alarm (can initiate) | **include** ROI |
| `Inclusive` | alarms only if an Active zone *also* alarmed (supporting, for jittery areas) | **include** ROI (cross-zone dependency dropped) |
| `Exclusive` | alarms only if *no* Active zone alarmed (small special events) | **include** ROI (cross-zone condition dropped) |
| `Preclusive` | if triggered, **vetoes the whole frame's alarm** — a shortcut to suppress global lighting/large-scale changes | largely **N/A** (YOLO isn't fooled by lighting); drop, or optionally reinterpret as an **ignore/exclude** ROI |
| `Inactive` | disabled | dropped |
| `Privacy` | blank pixels | `privacy_mask` polygon (Phase 5) |

Practical model for zm-next: **include ROIs** (Active/Inclusive/Exclusive),
optional **exclude/ignore ROIs**, and **privacy masks** — the alarm-combination
subtleties of ZM's motion world do not carry over.

## Phased plan

### Phase 1 — Pipeline-config store (the free graph) — the centerpiece
- New zm-api-owned table `monitor_pipeline` (`monitor_id` PK, `graph_json`,
  `version`, timestamps; logical FK to `Monitors`). Migration follows the
  `event_synopsis` pattern (portable DDL, offline-tested, hand-written entity —
  **do not** widen the generated `monitors` entity). Wire `migrate_database` at
  startup.
- `repo::monitor_pipeline` + `service` for CRUD; **validation** (well-formed tree,
  known `kind`s, no capture/creds nodes).
- Refactor `generate_pipeline` into **compose**: capture (from `Monitors`) +
  stored processing graph + injected `store` node. When a monitor has a stored
  graph, use it; else fall back to today's default generator.
- **Zones inside compose:** inject `Monitors.Zones` into whichever zone-consumer
  the graph contains — the `zones` plugin (motion path) and/or `analytics_rules`
  intrusion polygons (object path) — mapping ZM `Type` per the table above. Zones
  stay ZM-authoritative; the graph just references where they apply.
- API: `GET/PUT /api/v3/monitors/{id}/pipeline` (read/replace the graph),
  validated; reload re-pipes over stdin (existing path).

### Phase 2 — "Make this monitor zm-next"
Endpoint that (a) sets `UseZmNext=1` and (b) materializes a default
processing-graph doc (from a template merged with `[zmnext.pipeline]` defaults).
Reversible (clear flag → dormant doc). Reconcile/restart picks it up.

### Phase 3 — ONVIF discover → onboard
- Resolve the `CreateMonitorRequest` construction (add `Default` or a focused
  `from_onvif`/onboard DTO).
- `POST /api/v3/discovery/onboard` (or `/monitors/from-onvif`): inspect → map
  chosen `InspectProfile.stream_uri` + identity + creds → create monitor (reuse
  `service::monitor::create`), set ONVIF columns. Reuse SSRF gate + ACL.
- Add an `already_monitor`/`monitor_id` cross-reference to probe/inspect results
  (match xaddr host / `onvif_url` against existing monitors) so UI shows new vs.
  onboarded. Thread `ProbeRequest.timeout_ms` (currently ignored).

### Phase 4 — Privacy zones — **implemented (zm-api side)**
`Privacy` zones are surfaced (`privacy_regions`) instead of dropped. When a
monitor has any, the generator switches from the passthrough default to a
**transcode topology** — `capture → decode_ffmpeg → privacy_mask → { detect_onnx,
encode_ffmpeg → store }` — so the mask reaches detection, live, AND the stored
clip (passthrough recording can't mask, since `store` records the compressed
capture). `compose` injects `Privacy` regions into any `privacy_mask` node in a
stored graph (symmetric with detection-zone injection). Unit-tested at the
generated-JSON shape level.

> **Open verification:** the transcode chain needs end-to-end testing against a
> camera with `Privacy` zones — specifically that zm-next's `encode_ffmpeg → store`
> records the re-encoded masked stream correctly. The cfg keys used
> (`privacy_mask.regions`, `encode_ffmpeg.codec`, `decode_ffmpeg.output_format`)
> are confirmed against the zm-next plugins, but the wired chain is unproven here.

### Phase 5 — On-camera privacy / settings via ONVIF — **researched conclusion**
The correct long-term home for privacy is the camera, but **on-camera privacy
masks are not standardized in ONVIF core** (Media/Media2 has no privacy-mask
service). Vendors expose them via the Imaging-service extensions or proprietary
APIs, so a single generic implementation is not feasible — it must be done
**per-vendor** when needed. What *is* standardized and would be the real ONVIF
"push settings to the camera" foundation: `Imaging::SetImagingSettings`
(brightness/contrast/focus/…) and `Media::SetVideoEncoderConfiguration`
(resolution/bitrate/codec/fps) — both additive write methods on the existing
`src/onvif` client. Recommendation: implement those standardized writes when a
concrete need arises, and add vendor-specific privacy-mask drivers behind a small
trait keyed off `Monitors.Manufacturer`. Deliberately **not** shipping a generic
privacy-mask SOAP call that would silently fail on most cameras.

## Cross-cutting

- **Security:** onboard + pipeline-edit endpoints keep `Feature::Monitors` RBAC +
  unrestricted-scope gate (minting/reconfiguring monitors is admin-equivalent).
  No secrets in the stored graph; no creds in logs (done) or argv.
- **Validation/testing:** per-phase unit tests; pipeline-graph validator tests;
  offline migration DDL test; live end-to-end against monitor 3.
- **Migration runner:** wire `migrate_database` into startup (or document
  packaging application) as part of Phase 2.

## Open questions

- Default template(s) for "make it zm-next" — single sensible default, or a small
  catalog (detect-only, alert-cascade, synopsis)?
- Pipeline-graph validation depth in zm-api vs. deferring to zm-next at spawn.
- Whether to ever allow zones/capture-substream editing inside the graph (kept ZM-
  authoritative for now).
