# ONVIF Client Subsystem — Implementation Plan

> **Status (2026-06-28):** Done — follow-ups. Phases 1-4 shipped (foundation,
> service clients, integration with PTZ/discovery/event listener, adversarial
> verification + fixes). Open: conformance vectors (11 sample-response
> fixtures), CI feature-matrix per profile, deferred LOW parser items
> (CDATA-spanning text, `xml:lang` Reason), Phase 5 live event push to API
> clients (SSE/webhook).

Goal: make zm_api a **client-only ONVIF NVR** — discover cameras, retrieve media
profiles/stream URIs, control PTZ natively, and consume device events — by adding
a reusable ONVIF client subsystem and wiring it into the existing monitor, PTZ,
streaming, and event layers.

> Scope decision (2026-06): **client-only** (ZM consumes ONVIF cameras). Exposing
> ZM *as* an ONVIF server/Profile-G service to upstream VMS is explicitly out of
> scope for this plan.

## Architectural principle

ONVIF is **not** "another PTZ protocol" — it spans five WSDL services (Device,
Media, PTZ, Events, plus WS-Discovery). It therefore lives in its **own
top-level module** `src/onvif/` as a reusable client/transport library, and each
consuming subsystem holds a thin adapter:

| ONVIF service | Consumer in zm_api |
|---|---|
| WS-Discovery (UDP 239.255.255.250:3702) | `src/service/discovery.rs` + handler/route |
| Device (GetCapabilities, GetDeviceInformation) | discovery + monitor create |
| Media (GetProfiles, GetStreamUri → RTSP) | feeds existing Retina `MonitorSource` |
| PTZ (ContinuousMove, …) | `src/ptz/protocols/onvif.rs` impl `PtzControl` |
| Events (PullPoint subscription) | `src/daemon/onvif_event_listener.rs` → `Events` |

The existing PTZ trait/registry/handlers/routes are **unchanged** — the ONVIF PTZ
adapter is registered as a native factory in `state.rs`, exactly as
`docs/PTZ_TASKS.md` already anticipated.

## What already exists (no work needed)

- **Schema**: `onvif_url`, `onvif_username`, `onvif_password`, `onvif_events_path`,
  `onvif_event_listener`, `onvif_alarm_text` are already on the `Monitor` entity.
- **SSRF guard**: `is_safe_onvif_url()` in `src/dto/request/monitor.rs`.
- **Media**: `GetStreamUri` yields an RTSP URI that drops into the existing
  Retina-based `MonitorSource` — no streaming work.
- **Background tasks**: `DaemonManager` + `util::task::join_all` supervise loops.

## Module layout (new)

```
src/onvif/
  mod.rs          re-exports; declares the service submodules
  error.rs        OnvifError / OnvifResult
  types.rs        shared types (Credentials, service endpoints)
  transport.rs    SOAP-over-HTTP via reqwest + quick-xml envelope build/parse
  security.rs     WS-Security UsernameToken digest (base64(sha1(nonce+created+pass)))
  discovery.rs    WS-Discovery Probe → ProbeMatch over UDP multicast
  device.rs       GetDeviceInformation, GetCapabilities
  media.rs        GetProfiles, GetStreamUri
  ptz.rs          PTZ ops (ContinuousMove, AbsoluteMove, Stop, GetStatus)
  events.rs       CreatePullPointSubscription, PullMessages, Renew, Unsubscribe

src/ptz/protocols/onvif.rs        OnvifControl: impl PtzControl (delegates to onvif::ptz)
src/service/discovery.rs          probe orchestration → CameraCandidate
src/dto/request/discovery.rs      ProbeRequest, InspectRequest
src/dto/response/discovery.rs     CameraCandidate, InspectResult
src/handlers/discovery.rs         POST /api/v3/discovery/probe, /inspect
src/routes/discovery.rs           route registration
src/daemon/onvif_event_listener.rs  per-monitor PullPoint loop → Events table
```

Shared-file edits: `src/lib.rs` (`pub mod onvif;`), `src/routes/mod.rs`
(register discovery routes), `src/server/state.rs` (register `OnvifControl`
factory; spawn event listeners), `Cargo.toml` (`quick-xml`, `sha1`).

## Correctness anchors (defend against "tests agree with the bug")

- **WS-Security digest** is verified against the OASIS published known-answer
  test vector (fixed nonce/created/password → fixed digest), not agent-invented
  expectations.
- **SOAP/XML parsers** are tested against representative ONVIF response fixtures,
  with cases for namespace-prefix variance, missing optional fields, and
  multi-camera ProbeMatch batches.
- **SSRF**: `inspect` targets pass `is_safe_onvif_url()` and are restricted to
  addresses surfaced by a probe / RFC1918 ranges.

> Field caveat: fixture-green ≠ ONVIF-compliant. Real-camera interop testing is
> required before claiming compliance; this plan delivers a complete, unit-tested
> client ready for that testing.

## Phases

- **Phase 1 — Foundation.** ✅ `error`, `types`, `transport`, `security` + module
  stubs. WS-Security known-answer test (OASIS oracle). Committed `d72114f`.
- **Phase 2 — Service clients.** ✅ `device`, `media`, `ptz`, `events`,
  `discovery` — each a self-contained module + fixture tests.
- **Phase 3 — Integration.** ✅ `ptz/protocols/onvif.rs`, `service/discovery.rs`,
  DTOs, handler, route, `daemon/onvif_event_listener.rs`; wired into `lib.rs`,
  `routes/mod.rs`, `state.rs` (event-listener spawn at startup), OpenAPI.
  Green: fmt/clippy/686 tests/release. Committed `6e763e9`.
- **Phase 3.5 — Feature gating.** ✅ Per-capability Cargo features
  (`onvif-core/device/media/ptz/events/discovery`) + profile umbrellas
  (`onvif-profile-s/t/g/m`), `default = ["onvif"]`. Verified: ONVIF fully off
  (`--no-default-features`), each capability standalone, and all-on all compile.
  OpenAPI discovery fragment (`DiscoveryApiDoc`) merged at runtime under
  `onvif-discovery` since utoipa can't `#[cfg]` macro entries.
- **Phase 4 — Adversarial verification.** ✅ (review) 6-agent verify workflow.
  WS-Security digest verified correct vs OASIS (no defects). Fixed:
  - **WS-Addressing** now emitted (`wsa:To`/`Action`/`MessageID`/`ReplyTo`) on
    every call — required by the Events service (the `WSA_NS` removal during
    integration *was* a regression). `src/onvif/transport.rs`.
  - **SOAP fault Subcode** extracted + preferred (deepest wins), so
    `ter:NotAuthorized` is surfaced and auth faults map to 401 regardless of
    Reason language. `transport.rs`.
  - **HTTP 401/403** with no SOAP body → `OnvifError::Auth` (was 503).
  - **Media2 classification**: ver10 vs ver20 media namespaces disambiguated
    (the `/media2/wsdl` branch was dead code). `src/onvif/device.rs`.
  - **IPv6 SSRF gate**: typed `url::Host` match so IPv6 camera literals aren't
    universally rejected; SSRF docstring corrected (no probe-allowlist exists).
    `src/service/discovery.rs`.
  - **Renew timing**: `tokio::select!` races the pull vs a renew deadline so
    renew can't be starved by the long-poll. `src/daemon/onvif_event_listener.rs`.
  - Deferred LOW: Reason multi-lang preference; URI split across CDATA fragments.
  - Conformance test vectors (11 recommended) not yet folded in — see below.
- **Phase 5 (later) — Live event push.** SSE/webhook from the event listener to
  API clients (no push mechanism exists today). Out of the initial build.

### Follow-ups / known gaps

- **Conformance vectors:** the Phase 4 conformance agent produced 11
  spec-cited sample-response vectors (GetDeviceInformation, GetCapabilities/
  GetServices, GetProfiles, GetStreamUri, PTZ moves/GetStatus, PullMessages,
  ProbeMatches) to replace hand-built fixtures. Fold into `src/onvif/*` tests.
- **Deferred LOW parser items:** accumulate text across CDATA/comment fragments
  in `media.rs::parse_media_uri`; prefer `xml:lang="en"` Reason text in the SOAP
  fault parser.
- **CI matrix:** add `cargo check --no-default-features` and a per-profile
  build (`--features onvif-profile-s|g|m`) to `.github/workflows/test.yml` so no
  feature combo bit-rots. Not yet wired.
- **Richer discovery candidates:** the agent also produced a `host`/`port`/
  `already_monitor` projection (removed as a duplicate `dto/response/discovery`).
  Folding `already_monitor` (cross-reference existing monitors) into
  `service::discovery::CameraCandidate` is a worthwhile enhancement.
- **media2/imaging/recording/replay/analytics** features are declared in the
  profile umbrellas' intent but not yet implemented (Profile T/G/M are partial).

## Feature gating (ONVIF capabilities → Cargo features)

ONVIF support is exposed as **gated Cargo features**, one per ONVIF Device Test
Specification category, composed into profile umbrellas. We only advertise a
capability whose conformance-derived test suite is green. `transport`,
`security`, `types`, `error` are the always-on base (any service needs them);
each service module is `#[cfg(feature = …)] pub mod …;` in `src/onvif/mod.rs`.

Granular features (↔ test-spec categories):

```toml
[features]
onvif            = ["onvif-discovery", "onvif-device"]      # base client
onvif-discovery  = []   # WS-Discovery (Device Feature Discovery)
onvif-device     = []   # Base Device Operations / Device Mgmt
onvif-media      = ["onvif-device"]   # Media Configuration + Real-Time Streaming (Profile S)
onvif-media2     = ["onvif-device"]   # Media2 Configuration (Profile T)
onvif-ptz        = ["onvif-device"]   # PTZ Device Test
onvif-events     = ["onvif-device"]   # Event Handling (PullPoint)
onvif-imaging    = ["onvif-device"]   # Imaging
onvif-recording  = ["onvif-device"]   # Recording Search/Control (Profile G)
onvif-replay     = ["onvif-recording"]# Replay Control (Profile G)
onvif-analytics  = ["onvif-device"]   # Analytics / metadata (Profile M)

# Profile umbrellas (what a deployment actually targets)
onvif-profile-s  = ["onvif-media", "onvif-ptz", "onvif-events", "onvif-discovery"]
onvif-profile-t  = ["onvif-profile-s", "onvif-media2", "onvif-imaging"]
onvif-profile-g  = ["onvif-device", "onvif-recording", "onvif-replay"]
onvif-profile-m  = ["onvif-device", "onvif-events", "onvif-analytics"]
```

Out of scope (access control): Profiles A/C/D. The PTZ adapter, discovery
service, and event daemon are each compiled only when their feature is on, and
the route/state wiring is `#[cfg]`-gated to match. CI builds the umbrella
feature sets so no combination bit-rots.

## Conformance-driven tests

The reference for what each parser/builder must handle is ONVIF's **Device Test
Specifications** (released twice yearly; current **v25.12 / v25.06**), per
category: Base Device Test, Media/Media2 Configuration, Real-Time Streaming, PTZ
Device Test, Event Handling, Imaging, Recording/Replay. Approach:

1. Pull the relevant Device Test Specification PDFs (and the Profile Feature
   Overview) into `docs/onvif-specs/` (gitignored if large) as the source of
   mandatory/conditional fields and request/response expectations.
2. Derive unit-test fixtures + assertions from the spec test procedures — which
   elements are mandatory, expected fault behaviour, field cardinality — rather
   than agent-invented XML. WS-Security digest stays anchored to the OASIS
   known-answer vector.
3. A feature is only enabled-by-default / advertised once its conformance-derived
   suite passes.

> The official ONVIF Test Tool (a Windows GUI driving a live device) and formal
> certification are out of scope here — the written test specs are our oracle,
> and real-device interop remains a manual follow-up before claiming compliance.

## Build approach

Phases 1–4 are executed by a multi-agent **workflow**
(`.claude` Workflow). Phase 1 lands and commits the foundation so the parallel
Phase 2 agents (worktree-isolated) compile against the *real* committed source,
eliminating contract drift. Final integration gate + release build are confirmed
in the main session.
