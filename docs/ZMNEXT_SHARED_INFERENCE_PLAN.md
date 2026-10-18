# zm-next Shared Inference Plan

Collapse the per-monitor GPU-inference footprint from **O(N) CUDA contexts + ORT
sessions** (one per camera) into **O(GPU)** by sharing inference across monitors.

> Status (2026-09-18): **Phase 1 built in zm-next** (`zm-infer` daemon, remote
> branch in `decode_detect`'s CUDA backend, protocol tests); zm-api's side was
> already in place, and it ran end to end under zm-api on 2026-09-18. Measured:
> host memory −43% at 3 cameras; **VRAM unchanged** (see "Phase 1 as built"). Phase 0 found
> EP tuning ineffective and a worker memory leak instead (fixed 2026-06-27).

## Review, 2026-09-18

The per-GPU daemon is still the right design. What changed since June:

- **The projections held up.** Measured on the RTX 5070 Ti box, 3 replayed cameras
  (zm-next `docs/Performance.md`): a worker with its own session costs ~1.1 GB host
  PSS and ~0.73 GB VRAM; a worker doing NVDEC decode with *no* session costs ~0.2 GB
  host and ~0.33 GB VRAM. So after = one session + ~0.2 GB host / ~0.33 GB VRAM per
  camera (the table below guessed 0.4 / 0.3).
- **The motion gate cuts the load far below what this plan sized for.** The plan
  assumed every frame goes to the daemon (20 cameras × 10 fps × 4.9 MB). Measured
  gating was 97%, 89% and 0% of frames on three cameras; the June engine benchmark
  ran ~1,250 inferences/s. Throughput hardening (per-connection CUDA streams,
  double-buffered batch, batch linger) is therefore **out of Phase 1**; the safety
  items (deadlines, exit on an engine error, degrade-never-allocate) stay.
- **`roi_motion` stays as configured on the remote path.** An earlier version of
  this plan said to force it off. `pipeline.rs::inject_shared_inference` keeps it,
  correctly: the gate runs on the worker and only moving regions are sent.
- **The control-protocol work applies to this socket too**: kernel peer credentials
  decide who may register or submit, the daemon sends a hello first, and a worker
  reports the daemon being unreachable as `worker_degraded` (EVENT `0x0403`).
- The shared-memory ring mentioned below (`zm_shmring`) was removed from zm-next on
  2026-09-15; it never crossed processes.
- Still open: a CPU session in the worker as the daemon-down fallback. At 97%
  gating a camera needs well under one inference per second, so it would cost
  almost nothing and no VRAM; a camera whose gate stays open would cost ~25% of a
  core. Not built; today a down daemon means dropped detections.

## Phase 1 as built (zm-next)

- `zm-infer --gpu <n> --socket <path> [--allow-uid <uid>]... [--max-batch 8]
  [--max-wait-us 0]`. Exit codes as zm-core: 0 stopped, 64 usage, 69 socket
  unusable, 70 internal. **Any inference exception exits 70** after answering the
  request (CUDA errors are sticky); zm-api restarts it and workers drop detections
  meanwhile.
- Wire (`plugins/detect_onnx/infer_remote.hpp`): the stream-socket 24-byte header
  with socket-local types `0x20` InferHello, `0x21` Register `{model_path, net}`,
  `0x22` RegisterAck `{ok, model_id | error}`, `0x23` Submit (letterbox, confidence,
  class allow-list, fp32 CHW tensor), `0x24` Result (boxes, or an error string),
  `0x25`/`0x26` Ping/Pong `{clients, models, requests, errors}` for an app-level
  health probe. One session per `(model_path, net)`.
- Auth: only the daemon's own uid, plus `--allow-uid`, may register or submit;
  others get `control_peer:false` and are closed before the model loader is reached.
- Worker: `decode_detect` with `hw: cuda` reads `infer_endpoint` (and `gpu_id`,
  informational). It then loads **no** ONNX Runtime session: preprocess on the GPU,
  one device-to-host copy into pinned memory, Submit. Request deadline 1 s
  (30 s for Register, which can include the model load). On failure the detection
  is dropped, `worker_degraded {component:"zm-infer", degraded:true}` is published
  once, the connection retries with 1 s → 30 s backoff, and recovery publishes
  `degraded:false`. There is no local-session fallback, by design.
- **Measured** (RTX 5070 Ti, 3 replayed cameras, back to back with local sessions;
  zm-next `docs/Performance.md`):

  | | Local sessions | Shared (`zm-infer`) |
  |---|---|---|
  | Host PSS | 3,197 MB | 1,809 MB (workers 708 + daemon 1,101) |
  | Per camera | ~1.07 GB | ~0.23 GB, plus ~1.1 GB once |
  | VRAM over baseline | +2.15 GB | +2.10 GB (workers 0.49–0.63 GB each + daemon 0.5 GB) |
  | CPU (of one core) | 5.4% | 6.8% |

  The host-memory win is as projected. **The VRAM projection below was wrong**: a
  session-less worker keeps ~0.56 GB (CUDA context, NVDEC surfaces, kernels), not
  0.3 GB, so VRAM saves only ~0.17 GB per camera past the third. Cameras per GPU is
  now bounded by that per-worker floor, not by ORT sessions. Detections are
  identical: from the same point in the clip loop, all three cameras' counters
  (gated / inferred / detections) matched the local run exactly. Killing the daemon
  under a live worker: worker kept running, one `worker_degraded`, self-recovered.
- Not done: `detect_onnx` ignores `infer_endpoint` with a warning (it isn't in the
  default pipeline); no per-connection CUDA streams; no blue/green model swap.
- zm-api: nothing new needed to start it. Two follow-ups: `zm-infer` must be
  found next to `zm-core` when `[zmnext.worker].binary` is an absolute path, and
  the Ping/Pong probe can replace the CPU-time `appears_hung` exemption.

## ⚠️ Measured result (2026-06-27) — pivot

Phase 0 EP tuning (HEURISTIC conv search + kSameAsRequested arena) was shipped and
measured against a 14-min warmup window. **It did not change the trajectory.** Worker
RSS climbs **linearly and unbounded** (~40 MB/min on 3 of 4 workers; 1.3 → 1.9 GB
and still rising; total RSS 5.0 → 6.5 GB over 14 min), the same curve as the untuned
run. This refutes the original "cuDNN EXHAUSTIVE warmup creep" hypothesis — a warmup
cost plateaus; this is a **leak**.

Evidence (per-PID `smaps_rollup`): the growth is entirely **anonymous private-dirty**
memory in 64 MB allocator chunks — NOT the ORT/CUDA arena, NOT shmem (the `zm_shmring`
is constant at 122 MB), NOT library code (`Shared_Clean` constant ~256 MB). Leads:
**Monitor-2 stays flat (~1.19 GB) while identical Monitors 1 & 3 climb in lockstep to
1857 MB**; and the shared stdout is flooded with repeated FFmpeg `mov,mp4` demuxer
probe warnings at ever-new heap addresses (repeated `AVFormatContext` creation —
classic leak vector; source not yet attributed to worker vs zm-api playback).

**Implication for this plan:** the leak must be found/fixed *regardless* of the
daemon. The Phase-1 daemon does not fix a per-worker decode/record leak, and would
*concentrate* an inference-path leak across all cameras. Leak hunt precedes Phase 1.
The N→1 session collapse (the daemon's baseline-footprint win) remains valid for
scaling, but is no longer the most urgent memory problem.

### Root cause FOUND & FIXED (2026-06-27)

A live `bpftrace` trace of large `mmap`s on the running workers (passwordless sudo;
`zm-core` has debug symbols) pinned every 64/128 MB allocation to one call stack:
`alloc_new_heap → __libc_malloc → std::vector::vector → StreamManager::
process_and_publish_frame → capture_loop`. Source confirmed the bug in zm-next
`plugins/capture_rtsp_multi/stream_manager.cpp`: the capture loop's **video branch
never called `av_packet_unref`** on the packet from `av_read_frame`, while the audio
(`publish_audio_packet`) and discard branches did. Every video frame leaked one
compressed `AVPacket` buffer. This explains all three constraints: ~40 MB/min
(fps × packet size), the 64 MB glibc-arena chunks (leaked `av_malloc` buffers piling
up in the capture thread's arena), and mon-2 being "flat" (a lower-bitrate/fps stream
leaking ~3 MB/min vs ~40 for the identical mon-1/mon-3).

**Fix:** added `av_packet_unref(state->packet)` to the video branch (also covers the
early-return paths inside `process_and_publish_frame`, since the unref runs in the
caller). Tool note: `bpftrace`/`perf` beat valgrind here — they attach to the live
realtime worker with ~zero overhead; valgrind can't attach and its 20–50× slowdown
would stop an RTSP worker from reproducing the leak.

**VERIFIED FIXED.** Post-fix plateau measurement (`scratchpad/verify-fix.sh`, 18 min):
the two lower-res workers were flat throughout; the two 4K workers ramped ~120 MB
(warmup) and plateaued at ~1.38 GB. A 90 s `bpftrace` mmap trace on the running
workers then caught **zero** large allocations (vs a steady stream pre-fix). Leak rate
went from a constant unbounded ~38 MB/min (→2.4 GB+ and climbing) to **0 — bounded,
stable ~5.1 GB total for 4 cameras**. The workflow independently corroborated the root
cause and supplied the DB discriminator (mon-2 = the only 720p stream → lowest bitrate
→ slowest leak; the leak rate == camera bitrate). Independently confirmed by:
bpftrace stack + source asymmetry + DB bitrate analysis + post-fix plateau/trace.

## Problem (measured)

`zm-next` runs **one `zm-core` process per monitor**. Each process builds its own
CUDA context + ONNX Runtime CUDA arena for the `detect_onnx` plugin:

- ~1.2–1.5 GB **host RSS** per monitor (measured: 4 monitors ≈ 6 GB).
  - ~1.2 GB anonymous host-side ORT/CUDA arena
  - ~200 MB resident CUDA/cuDNN/onnxruntime library code
- ~0.7 GB **VRAM** per monitor (the ORT CUDA session).
- RSS **keeps creeping for minutes** after start because the CUDA EP runs with
  default `OrtCUDAProviderOptions{}`: `cudnn_conv_algo_search = EXHAUSTIVE`
  (JIT-compiles + reserves large conv workspaces as new input shapes appear) and
  `arena_extend_strategy = kNextPowerOfTwo` (doubling/overshoot).

This scales strictly linearly and does not survive 20+ cameras — VRAM is the first
wall (N separate ORT sessions become infeasible past ~8 cameras on a 16 GB card).

## Decision record (2026-06-27)

Chosen direction after a multi-agent design + adversarial-critique pass (4 designs,
4 lenses each). Winner: **per-GPU `zm-infer` daemon, host-serialized tensors over a
Unix domain socket**. Key decisions made by Steve:

| Decision | Choice | Rationale |
|---|---|---|
| **Architecture** | Per-GPU `zm-infer` daemon (out-of-process) | Keeps capture/decode/record per-monitor with OS-enforced isolation; only inference shares a context. Recording reliability > detection latency for a security product. |
| **Daemon-loss fallback** | **Degrade: drop detections + reconnect with backoff** | Avoids the thundering-herd OOM where all N cameras re-allocate ORT arenas at once. Recording always continues. Per-instance fallback only behind a global GPU-session admission cap. |
| **Phase-1 data plane** | **UDS-serialized fp32** (~4.9 MB/frame) | ~8% of PCIe Gen3 x16 at 20 cam × 10 fps — the bandwidth saving from shmem is irrelevant, and the socket is self-cleaning (no slot-ownership / use-after-free hazards). Shmem/CUDA-IPC deferred to Phase 2 only if measured. |

### Rejected alternatives

- **In-process "fat" `zm-core`** (M monitors in one process reusing the existing
  `InferenceEngine`): rejected — widens the fault domain to capture+record (one
  CUDA fault kills M cameras' recording), and requires de-globalizing
  `EventBus`/`gHost`, where a routing bug could silently deliver camera A's
  detections to camera B's recordings.
- **Shmem-slab daemon**: same as the chosen daemon but with a shared-memory data
  plane — carries a cross-process slot use-after-free on the request-timeout path,
  for a bandwidth saving that doesn't matter. Kept as a deferred Phase-2 option
  behind the same protocol.

## Honesty caveats (must be measured, not assumed)

- The robust, **verified** win is **N→1 ORT sessions** — workers stop loading
  ONNX Runtime/cuDNN entirely. This holds independent of any tuning.
- The ~1.2 GB "host arena" attribution is **partly mis-diagnosed**:
  `gpu_mem_limit` / `arena_extend_strategy` act on **device** memory, so they may
  not shrink host RSS as much as hoped. **Phase 0 is also the measurement** that
  tells us how much of the 1.2 GB is host vs device before we commit to Phase-1
  numbers. Validate with `smaps_rollup` **Pss** (RSS double-counts shared `.so`).
- VRAM after = `O(1)_inference + O(N)_decode` (each worker keeps a CUDA
  primary-context floor + NVDEC DPB), **not** "constant in N".
- Numbers assume **detect-only, one model**. ReID/face = a second session per
  worker and re-inflate per-worker cost unless also remoted.

## Memory projection (detect-only, one model)

| | N=4 | N=12 | N=24 |
|---|---|---|---|
| Host RSS before (~1.5 GB/mon) | ~6 GB *(measured)* | ~18 GB | ~36 GB |
| Host RSS after (~1.5 GB daemon + ~0.4 GB/worker) | ~3.1 GB | ~6.3 GB | ~11 GB |
| VRAM before (~0.7 GB/mon) | ~2.8 GB | ~8.4 GB | ~16.8 GB |
| VRAM after (~1.5 GB arena + ~0.3 GB/worker) | ~2.7 GB | ~5.1 GB | ~8.7 GB |

Host slope drops from ~1.5 GB/camera → ~0.4 GB/camera; the decisive change is VRAM
going O(N)→O(1).

## Target architecture (Phase 1)

```
   GPU 0   ┌────────────────────────────────────────────┐
           │ zm-infer --gpu 0  (THE single CUDA context  │
  UDS      │ + 1 ORT session per (model,net), leaked)    │
 gpu0.sock─┼▶ accept loop → per-(model,net) dispatcher:  │
 (control  │   linger ≤maxWait → assemble [N,3,640,640]  │
  +4.9MB   │   → ONE IoBinding Run → NMS → RESULT(boxes) │
  tensor)  └──▲──────────────────────────────┬───────────┘
   SUBMIT{seq,tensor,src_wh,lb}│             │RESULT{seq,boxes}
   ┌──────────┐ ┌──────────┐         ┌──────────┐
   │ zm-core1 │ │ zm-core2 │  ...    │ zm-coreK │  ONE PROC / MONITOR
   │ decode   │ │ decode   │         │ decode   │  (bare CUDA ctx for
   │ preproc  │ │ preproc  │         │ preproc  │   NVDEC only, no ORT)
   │ →D2H     │ │ →D2H     │         │ →D2H     │
   └────┬─────┘ └────┬─────┘         └────┬─────┘
        │ EVENT 0x0301 json_detail on stream_{id}.sock — UNCHANGED
        ▼            ▼                    ▼
   ┌──────────────────────────────────────────────────┐
   │ zm-api DaemonManager spawns BOTH workers AND the  │
   │ per-GPU zm-infer; StreamSocketReader ingest as-is │
   └──────────────────────────────────────────────────┘
```

- Single CUDA context + ORT session live **only** in `zm-infer`, in the leaked
  `InferenceEngine::get` static map, re-keyed `(model_path, net, device_id)`.
- Workers construct **no** ORT session; they keep a bare CUDA context for NVDEC +
  the preprocess kernel, do one D2H into a pinned buffer, and ship 4.9 MB over UDS.
- Detections leave on the **unchanged** stream-socket `EVENT 0x0301` path — zm-api
  ingest (`src/streaming/source/protocol.rs`) is byte-for-byte untouched.

## Phased rollout

### Phase 0 — tune in place (days) — IN PROGRESS

Pure runway; no topology change; strictly still O(N). Doubles as the
host-vs-device measurement.

- **zm-next** (`plugins/detect_onnx/`): tune the CUDA EP at both sites that create
  it — the per-instance path (`detect_onnx.cpp`) and the shared engine
  (`detect_engine.cpp`). Defaults applied: `cudnn_conv_algo_search = HEURISTIC`,
  `arena_extend_strategy = kSameAsRequested`. Each overridable per-process via env
  so we can A/B old-vs-tuned on one binary:
  - `ZM_ORT_CUDNN_ALGO` = `exhaustive` | `heuristic` (default) | `default`
  - `ZM_ORT_ARENA_STRATEGY` = `power2` | `requested` (default)
  - `ZM_ORT_GPU_MEM_LIMIT_MB` = `<N>` (0 / default = unlimited)
- **zm-api** (`src/daemon/manager.rs`): set `CUDA_MODULE_LOADING=LAZY` on the
  spawned worker env (no-op/default on CUDA ≥ 12.2, explicit for older toolkits).
- **Measure**: compare `smaps_rollup` **Pss** + steady-state RSS over a warmup
  window, tuned vs. the measured ~1.5 GB baseline. This tells us how much of the
  1.2 GB is host vs device before committing Phase-1 numbers.
- Reversibility: env flag / two-line revert.

### Phase 1 — UDS `zm-infer` daemon (weeks) — TARGET

The real O(N)→O(GPU) collapse. Ship **with** the hardening below in-scope:

- New `zm-infer` per-GPU daemon (in zm-next), UDS server; lifts `InferenceEngine`.
- `detect_onnx`/`decode_detect` remote branch + `RemoteBackend` (covers both the
  inline `detect_onnx` branch and the `decode_detect` `HwBackend` seam).
- Wire protocol: `REGISTER/REGISTER_ACK/SUBMIT/RESULT/DEREGISTER/PING` (reuse the
  24-byte LE header + TLV; carry `epoch`/`abi_version`).
- zm-api: `DaemonManager` infer category (`zminfer_daemon_id(gpu)`), monitor-less
  reconcile (run if any zm-next monitor on that GPU is enabled), start-before-
  workers ordering, `CUDA_VISIBLE_DEVICES=<gpu>` per daemon, exempt from the
  `appears_hung` CPU-time heuristic (use app-level PING/STATS instead).
- pipeline.rs: inject `infer_endpoint`/`gpu_id` into detect cfg at compose time,
  never persisted; `roi_motion` stays as configured (see Review).
- **Required hardening**: ~~per-connection CUDA streams + stream-scoped sync +
  double-buffered `d_batch_`~~ (deferred 2026-09-18: gated load is a small
  fraction of one stream's capacity; revisit if `zm-infer` ever queues);
  per-request deadlines; try/catch around `Run` (sticky CUDA
  error → exit for clean restart); degrade-don't-allocate fallback behind a global
  GPU-session admission cap + re-attach loop; dynamic-batch ONNX preflight (fail
  closed to per-instance if fixed-batch).
- Reversibility: `ShareInference.enabled=false` → no daemon, workers use today's
  per-instance path on restart. Per-monitor opt-out via `UseZmNext` / stored graph.
- A/B: tag some monitors `instance`, some `daemon`, from the same DB; compare
  RSS/VRAM/latency/drop-rate side by side.

### Phase 2 — same-GPU zero-copy (optional, only if measured)

Behind the same protocol: `cudaHostRegister` pinned slab or opportunistic CUDA-IPC
for same-GPU pairs; and/or an async submit/result detect stage to densify batching
if throughput (not memory) is the limiter. Only if PCIe bites (>30 fps × 30+ cams).

## Open risks to resolve before Phase 1 coding

1. **Host-arena attribution** — measure which of {ORT CPU arena, device BFC arena,
   cuDNN host module state, pinned staging} dominates the 1.2 GB. Bank the N→1 win.
2. **Thundering-herd fallback** — handled by decision above (degrade + admission cap).
3. **Gray failure** (alive-but-wedged daemon) — per-request deadline + try/catch.
4. **`appears_hung` mismatch** — exempt the `zm-infer` category; app-level probe.
5. **Null-stream serialization** — per-connection streams required, not optional.
6. **Dynamic-batch ONNX** — preflight-validate `yolo26n.onnx`; fail closed.
7. **`InferenceEngine::get` keyed on model-path only** — re-key `(model,net,device)`.
8. **NVDEC concurrent-session cap** per GPU — hardware ceiling on cameras/GPU.
9. **ReID/face scope** — stays per-worker unless also remoted; gate the claim.
10. **Upgrades** — one session/GPU ⇒ model swap is a GPU-wide detection-dark window;
    mitigate with pre-warm + blue/green socket (`zm_infer_gpu{N}_{epoch}.sock`).

## Key files

**zm-next** — `plugins/detect_onnx/detect_engine.cpp` (EP opts, leaked singleton,
dispatcher), `plugins/detect_onnx/detect_onnx.cpp` (per-instance CUDA EP ~L202,
fallback, engine branch, host CHW path ~L558), `plugins/detect_onnx/detect_engine.hpp`
(shared tuning helper), `plugins/detect_onnx/detect_postprocess.hpp` (tensor
contract), `plugins/detect_onnx/hw_backend.hpp` (`make_backend` seam).

**zm-api** — `src/daemon/manager.rs` (`spawn_daemon`, `start_daemon_with_stdin`,
`reconcile_monitors`, id helpers), `src/daemon/process.rs` (`appears_hung`),
`src/service/zmnext/pipeline.rs` (detect cfg emission, `compose_pipeline`),
`src/service/zmnext/graph.rs` (`KNOWN_KINDS`/`FORBIDDEN_CFG_KEYS`),
`src/streaming/source/protocol.rs` (unchanged ingest).
