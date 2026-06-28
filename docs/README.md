# zm-api Plans & Docs Index

The single entry point for all design notes, implementation plans, and operator
guides. Every plan doc gets a status line so a reader can tell at a glance
whether it's active, mostly-done with follow-ups, or superseded.

**Status meanings**

| Tag | Meaning |
|---|---|
| Active | Currently being built or about to be |
| Done — verify | Code is in, manual/browser verification still pending |
| Done — follow-ups | Code shipped; specific items listed as remaining work |
| Reference | Operator/dev guide, not a plan |
| Superseded | Kept for history; live work lives elsewhere |

Last verified: **2026-06-28** (cross-checked against the code state on
`master @ 766c1a7`). Numbers behind each status reflect that snapshot — re-verify
before relying on any specific phase verdict.

---

## Active plans

| Doc | Status | One-line scope |
|---|---|---|
| [STORAGE_PLAN.md](STORAGE_PLAN.md) | Active (design) | Per-Storage retention with declared capacity, per-Monitor quotas, soft-delete + reap, cold-tier handoff (S3 etc.). Replaces `PurgeWhenFull` and `Update DiskSpace`. |
| [FILTERS_PLAN.md](FILTERS_PLAN.md) | Active (design) | Retire user-facing Filters. Saved searches → bulk event ops; notifications → rules engine; media → job queue; `AutoExecuteCmd` → webhooks. |
| [MONITOR_EVENTS_TASKS.md](MONITOR_EVENTS_TASKS.md) | Active | Capture-fault + state-change SSE on `GET /api/v3/monitors/{id}/events`. zmc-side (Phase 1) is the gating dependency; Phase 2-3 mostly land via the zm-next protocol code already in tree. |
| [HEVC_WEBRTC_TASKS.md](HEVC_WEBRTC_TASKS.md) | Done — verify | Phase 1 server correctness shipped. Phase 2 (Safari verification) + Phase 3 (Swift/Kotlin apps) pending. |
| [AUDIO_TASKS.md](AUDIO_TASKS.md) | Done — verify | Phases 1-3 (HLS AAC, WebRTC G.711, AAC→Opus) shipped. Real-camera browser verification pending; Phase 4 stretch items open. |
| [MOTION_SYNOPSIS_API_SPEC.md](MOTION_SYNOPSIS_API_SPEC.md) | Active (P1-P2 in tree) | Synopsis service exists at `src/service/synopsis/{compositor,optimiser,render}.rs`; P3 video render + P4 range/overview/retention still open. |
| [NL_EVENT_SEARCH_PLAN.md](NL_EVENT_SEARCH_PLAN.md) | Done — follow-ups | Vertical slice shipped on MariaDB 11.8 native VECTOR. Open: stand up local inference servers; sqlite-vec floor; response caching/ETag; image-embed. |
| [ONVIF_TASKS.md](ONVIF_TASKS.md) | Done — follow-ups | Phases 1-4 shipped. Open: conformance vectors, CI feature-matrix, deferred LOW parser items, Phase 5 live event push. |
| [ZMNEXT_TASKS.md](ZMNEXT_TASKS.md) | Done — coord pending | Tasks 1-5 landed (EVENT 0x06, ingest, daemon spawn, pipeline JSON, `UseZmNext` graceful flag). Waiting on: ZoneMinder fork's `Monitors.UseZmNext` migration; zm-next `store` plugin handshake. |
| [PTZ_TASKS.md](PTZ_TASKS.md) | Phase 0 done; later phases mostly superseded | Phase 0 Perl bridge complete. Phase 1 (native ONVIF PTZ) is delivered by ONVIF_TASKS, not here. Phases 2-5 (Dahua/HikVision/Reolink, serial, presets/tours, deprecation) still open. |
| [REVIEW_FIXES_PLAN.md](REVIEW_FIXES_PLAN.md) | Done — follow-ups | Phases 1-4 mostly shipped (password hash, ACL, status codes, daemon-id unification, transactional `apply_state`, spawn_blocking shm). Open: 3.2 idle HLS reaping, 4.4 bounded frames, 5.3 percent-encoded DB URL, 5.4 hand-rolled percent_decode replacement, 5.5 utoipa security annotations. |

## Reference docs (not plans)

| Doc | Scope |
|---|---|
| [deployment.md](deployment.md) | Packaging, profiles, passive/active mode, distro matrix, release flow. |
| [tls.md](tls.md) | Built-in TLS + ACMEv2 config and external certbot/lego flows. |

## Archive (kept for history)

| Doc | Why superseded |
|---|---|
| [archive/RTSP_STREAMING_PLAN.md](archive/RTSP_STREAMING_PLAN.md) | Pre-implementation design from before native HLS, WebRTC live, and the stream-socket source landed. Useful as historical context only — live plans are in HEVC_WEBRTC_TASKS, AUDIO_TASKS, MONITOR_EVENTS_TASKS. |

---

## Working on this docs tree

- One file per concern. Don't merge plans; cross-link instead.
- When a plan ships in code, change its status here and add a follow-ups list to
  the doc itself — don't archive working plans.
- When a plan is fully done with nothing outstanding, move it to `archive/` with
  a one-line note at the top saying which commit / release shipped it.
- Verify status claims by reading code, not by re-reading the plan. The
  "Last verified" date here is the floor of those claims.
