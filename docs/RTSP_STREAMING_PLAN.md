# RTSP Streaming API Architecture Plan

## Executive Summary

This document outlines a comprehensive plan to expose RTSP camera feeds through zm-api, implement native streaming protocols (WebRTC, WebSocket/MSE, HLS), and propose improvements to ZoneMinder to replace the legacy `zms` streaming service.

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Proposed Architecture](#2-proposed-architecture)
3. [Phase 1: go2rtc Integration Enhancement](#3-phase-1-go2rtc-integration-enhancement)
4. [Phase 2: Native WebRTC Implementation](#4-phase-2-native-webrtc-implementation)
5. [Phase 3: Native HLS Streaming](#5-phase-3-native-hls-streaming)
6. [Phase 4: WebSocket/MSE Enhancements](#6-phase-4-websocketmse-enhancements)
7. [Phase 5: RTSP Proxy Server](#7-phase-5-rtsp-proxy-server)
8. [ZoneMinder Improvements](#8-zoneminder-improvements)
9. [API Specification](#9-api-specification)
10. [Configuration Design](#10-configuration-design)
11. [Security Considerations](#11-security-considerations)
12. [Migration Strategy](#12-migration-strategy)

---

## 0. ZoneMinder Internals Analysis

### 0.1 ZoneMinder Capture Architecture (zmc)

Based on analysis of the [ZoneMinder source code](https://github.com/ZoneMinder/zoneminder/blob/master/src/zm_ffmpeg_camera.cpp), the capture daemon (`zmc`) uses the following architecture:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            ZoneMinder Capture (zmc)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────┐     ┌──────────────────┐     ┌────────────────────┐   │
│  │  FFmpeg Camera  │     │   PacketQueue    │     │   Shared Memory    │   │
│  │                 │────▶│   (deque-based)  │────▶│   (image frames)   │   │
│  │ - RTSP Client   │     │                  │     │                    │   │
│  │ - av_read_frame │     │ - ZMPacket ptrs  │     │ - Decoded frames   │   │
│  │ - Codec decode  │     │ - Multi-iterator │     │ - Event buffers    │   │
│  └─────────────────┘     │ - Condition vars │     └────────┬───────────┘   │
│                          └────────┬─────────┘              │               │
│                                   │                        │               │
│                          ┌────────▼─────────┐              │               │
│                          │    FIFO Writer   │              │               │
│                          │  (Named Pipes)   │              ▼               │
│                          │                  │     ┌────────────────────┐   │
│                          │ - H.264 packets  │     │   Analysis (zma)   │   │
│                          │ - H.265 packets  │     │   Motion detect    │   │
│                          │ - AAC audio      │     │   Object detect    │   │
│                          └────────┬─────────┘     └────────────────────┘   │
│                                   │                                         │
└───────────────────────────────────┼─────────────────────────────────────────┘
                                    │
                                    ▼
                    ┌───────────────────────────────┐
                    │     ZoneMinder RTSP Server    │
                    │         (xop library)         │
                    │                               │
                    │  - Reads from FIFOs           │
                    │  - H264/H265ZoneMinderFifo    │
                    │  - Multi-client distribution  │
                    │  - Dynamic monitor mgmt       │
                    └───────────────────────────────┘
```

### 0.2 Key Integration Points for zm-api

#### 0.2.1 PacketQueue (Recommended for Low Latency)

**Location**: `zm_packetqueue.cpp`

The PacketQueue provides thread-safe access to raw FFmpeg packets before decoding:

```cpp
// ZoneMinder PacketQueue key features:
class PacketQueue {
    std::deque<std::shared_ptr<ZMPacket>> pktQueue;  // Packet buffer
    std::mutex mutex;                                  // Thread safety
    std::condition_variable condition;                 // Wait/notify
    std::list<packetqueue_iterator*> iterators;       // Multi-consumer
};
```

**Benefits for zm-api**:
- Access raw H.264/H.265 NAL units directly
- No duplicate RTSP connection to camera
- Same packets used by ZoneMinder analysis
- Iterator-based: multiple consumers without copies

**Integration approach**:
```rust
// Future: ZoneMinder PacketQueue reader
pub struct ZmPacketQueueReader {
    shm_key: i32,
    iterator_ptr: *mut c_void,
    // Shared memory access to ZM's PacketQueue
}
```

#### 0.2.2 FIFO Interface (Recommended for Simplicity)

**Location**: `zm_rtsp_server.cpp`

ZoneMinder's RTSP server reads from FIFOs (named pipes) that zmc writes to:

```
/var/cache/zoneminder/events/{monitor_id}/.video_fifo      # H.264/H.265
/var/cache/zoneminder/events/{monitor_id}/.audio_fifo      # AAC/G.711
```

**Benefits for zm-api**:
- Simple UNIX pipe reader - no FFI needed
- Already packetized for streaming
- Used by ZoneMinder's own RTSP server
- Low latency access to encoded packets

**Integration approach**:
```rust
// FIFO-based packet reader
pub struct ZmFifoReader {
    video_fifo: std::fs::File,
    audio_fifo: Option<std::fs::File>,
    monitor_id: u32,
}

impl ZmFifoReader {
    pub fn read_packet(&mut self) -> Result<ZmPacket, Error> {
        // Read from FIFO, parse NAL units
    }
}
```

#### 0.2.3 Direct RTSP (Current Approach)

Connect directly to camera RTSP, independent of ZoneMinder:

**Benefits**:
- Works when ZoneMinder is down
- Can access different quality streams
- No ZoneMinder dependency

**Drawbacks**:
- Duplicate connection to camera
- No access to ZoneMinder's processing
- Some cameras limit concurrent connections

### 0.3 Recommended Integration Strategy

```
┌────────────────────────────────────────────────────────────────────┐
│                        zm-api Streaming                             │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Source Priority (configurable):                                   │
│                                                                     │
│  1. FIFO Reader (preferred)                                        │
│     └─ Reads from ZoneMinder's FIFOs                               │
│     └─ Same stream as ZM, low overhead                             │
│     └─ Requires ZoneMinder running                                 │
│                                                                     │
│  2. PacketQueue Reader (advanced)                                  │
│     └─ Direct shared memory access                                 │
│     └─ Lowest latency possible                                     │
│     └─ Requires FFI to ZoneMinder                                  │
│                                                                     │
│  3. Direct RTSP (fallback)                                         │
│     └─ Independent camera connection                               │
│     └─ Works without ZoneMinder                                    │
│     └─ Used for go2rtc integration                                 │
│                                                                     │
│  4. go2rtc Proxy (compatibility)                                   │
│     └─ External service handles streaming                          │
│     └─ Already implemented                                         │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘
```

---

## 1. Current State Analysis

### 1.1 Existing Streaming Capabilities

The zm-api currently implements three streaming mechanisms:

| Protocol | Status | Implementation | Plugin Dependency |
|----------|--------|----------------|-------------------|
| **WebRTC** | Functional | TCP signaling to C++ plugin (127.0.0.1:9050) | Yes - external plugin |
| **MSE/fMP4** | Functional | TCP socket to plugin (127.0.0.1:9051) | Yes - external plugin |
| **go2rtc Proxy** | Functional | HTTP proxy to go2rtc service | Yes - go2rtc service |

### 1.2 Current Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Applications                       │
│              (Web Browsers, Mobile Apps, Desktop)                │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                      zm-api (Axum Server)                        │
│                          Port 8080                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │   WebRTC     │  │     MSE      │  │    go2rtc Proxy      │  │
│  │  Signaling   │  │   Handler    │  │      Handler         │  │
│  │   Handler    │  │              │  │                      │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                      │               │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌──────────▼───────────┐  │
│  │   WebRTC     │  │     MSE      │  │      HTTP Client     │  │
│  │   Client     │  │   Manager    │  │                      │  │
│  │  (TCP:9050)  │  │  (TCP:9051)  │  │                      │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
└─────────┼─────────────────┼─────────────────────┼───────────────┘
          │                 │                      │
          ▼                 ▼                      ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────────┐
│  WebRTC Plugin  │ │   MSE Plugin    │ │     go2rtc Service      │
│  (C++ Binary)   │ │   (C++ Binary)  │ │   (External Process)    │
│   127.0.0.1:    │ │   127.0.0.1:    │ │   192.168.0.35:1984     │
│      9050       │ │      9051       │ │                         │
└─────────────────┘ └─────────────────┘ └────────────┬────────────┘
                                                      │
                                                      ▼
                                        ┌─────────────────────────┐
                                        │    RTSP Camera Feeds    │
                                        │  rtsp://user:pass@host  │
                                        └─────────────────────────┘
```

### 1.3 Current Limitations

1. **Hardcoded Configuration**: Plugin addresses and go2rtc URL are hardcoded
2. **External Dependencies**: Heavy reliance on external plugins for core streaming
3. **No Native HLS**: HLS only available through go2rtc proxy
4. **Limited RTSP Access**: No direct RTSP stream exposure
5. **Inconsistent APIs**: Different patterns across streaming endpoints
6. **No Stream Transcoding**: Cannot adjust quality/resolution natively
7. **Missing Authentication on Streams**: Tokens not validated on media segments

---

## 2. Proposed Architecture

### 2.1 Target Architecture

```
┌───────────────────────────────────────────────────────────────────────────┐
│                           Client Applications                              │
│                    (Browsers, Mobile, Desktop, TVs)                        │
└───────────────────────────────────────────────────────────────────────────┘
                                      │
        ┌─────────────────────────────┼─────────────────────────────┐
        │                             │                             │
        ▼                             ▼                             ▼
┌───────────────┐           ┌─────────────────┐           ┌─────────────────┐
│    WebRTC     │           │   WebSocket     │           │      HLS        │
│   (P2P/TURN)  │           │   (fMP4/MSE)    │           │   (.m3u8)       │
└───────┬───────┘           └────────┬────────┘           └────────┬────────┘
        │                            │                             │
        └────────────────────────────┼─────────────────────────────┘
                                     │
                                     ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                           zm-api Streaming Core                            │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                      Unified Stream Manager                          │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │  │
│  │  │   WebRTC    │  │     MSE     │  │     HLS     │  │   RTSP     │  │  │
│  │  │  Signaling  │  │   Server    │  │   Server    │  │   Proxy    │  │  │
│  │  │   Server    │  │             │  │             │  │            │  │  │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └─────┬──────┘  │  │
│  │         │                │                │               │         │  │
│  │         └────────────────┴────────────────┴───────────────┘         │  │
│  │                                 │                                    │  │
│  │                    ┌────────────▼────────────┐                      │  │
│  │                    │    Stream Orchestrator   │                      │  │
│  │                    │   (Transcode/Mux/Demux)  │                      │  │
│  │                    └────────────┬────────────┘                      │  │
│  └─────────────────────────────────┼───────────────────────────────────┘  │
│                                    │                                       │
│  ┌─────────────────────────────────▼───────────────────────────────────┐  │
│  │                        Stream Source Manager                         │  │
│  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐   │  │
│  │  │  RTSP Client  │  │   go2rtc      │  │  ZoneMinder Shared    │   │  │
│  │  │   (Native)    │  │  Integration  │  │  Memory Interface     │   │  │
│  │  └───────┬───────┘  └───────┬───────┘  └───────────┬───────────┘   │  │
│  └──────────┼──────────────────┼──────────────────────┼────────────────┘  │
└─────────────┼──────────────────┼──────────────────────┼────────────────────┘
              │                  │                      │
              ▼                  ▼                      ▼
    ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
    │  RTSP Cameras   │  │     go2rtc      │  │   ZoneMinder    │
    │                 │  │    Service      │  │   (zmc/zma)     │
    └─────────────────┘  └─────────────────┘  └─────────────────┘
```

### 2.2 Core Components

| Component | Purpose | Technology |
|-----------|---------|------------|
| **Unified Stream Manager** | Centralized stream lifecycle management | Rust + Tokio |
| **Stream Orchestrator** | Transcode, mux, quality adaptation | FFmpeg/GStreamer bindings |
| **WebRTC Server** | Native WebRTC with TURN/STUN | webrtc-rs or Pion bindings |
| **HLS Server** | Native HLS segment generation | Custom fMP4 segmenter |
| **RTSP Proxy** | Authenticated RTSP re-streaming | Custom RTSP server |
| **Stream Source Manager** | Unified access to RTSP/go2rtc/ZM sources | Abstraction layer |

---

## 3. Phase 1: go2rtc Integration Enhancement

### 3.0 ZoneMinder's go2rtc Integration Analysis

Based on analysis of ZoneMinder's [zm_monitor_go2rtc.cpp](https://github.com/ZoneMinder/zoneminder/blob/master/src/zm_monitor_go2rtc.cpp) and [go2rtc's video-rtc.js](https://github.com/AlexxIT/go2rtc/blob/master/www/video-rtc.js):

#### 3.0.1 ZoneMinder Go2RTCManager Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ZoneMinder go2rtc Integration                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Monitor (C++)                                                               │
│  └── Go2RTCManager                                                           │
│       ├── Go2RTC_endpoint    (from ZM_GO2RTC_PATH config)                   │
│       ├── rtsp_path          (primary camera RTSP URL)                       │
│       ├── rtsp_second_path   (secondary stream URL)                         │
│       └── rtsp_restream_path (ZM's own RTSP server URL)                     │
│                                                                              │
│  Registration Flow (libcurl HTTP):                                          │
│                                                                              │
│  1. add_to_Go2RTC():                                                        │
│     PUT /api/streams?src={rtsp_url}&name={monitor_id}_0                     │
│     PUT /api/streams?src={rtsp_url}&name={monitor_id}_1  (if secondary)     │
│     PUT /api/streams?src={rtsp_url}&name={monitor_id}_2  (if RTSP enabled)  │
│                                                                              │
│  2. check_Go2RTC():                                                         │
│     GET /api/streams?src={monitor_id}                                       │
│                                                                              │
│  3. remove_from_Go2RTC():                                                   │
│     DELETE /api/streams?src={monitor_id}                                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 3.0.2 go2rtc WebRTC/MSE Protocol

go2rtc uses a WebSocket-based signaling protocol with automatic format negotiation:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        go2rtc Streaming Protocol                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Browser                              go2rtc Server                          │
│     │                                      │                                 │
│     │──── WebSocket Connect ──────────────▶│                                 │
│     │      ws://host:1984/api/ws?src=zm5   │                                 │
│     │                                      │                                 │
│     │◀─── {type:"stream", streams:[...]} ──│  Available formats              │
│     │                                      │  (webrtc/mse/hls/mjpeg)         │
│     │                                      │                                 │
│  WebRTC Mode:                              │                                 │
│     │──── {type:"webrtc/offer", sdp:...} ─▶│                                 │
│     │◀─── {type:"webrtc/answer", sdp:...}──│                                 │
│     │◀──▶ {type:"webrtc/candidate"} ──────▶│  ICE Trickle                    │
│     │                                      │                                 │
│  MSE Mode:                                 │                                 │
│     │◀─── Codec info + binary segments ────│  MediaSource API                │
│     │                                      │                                 │
│  ICE Servers (default):                    │                                 │
│     • stun:stun.cloudflare.com:3478        │                                 │
│     • stun:stun.l.google.com:19302         │                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 3.0.3 ZoneMinder Web UI Integration

Player options available in ZoneMinder watch page:
- `go2rtc` - Auto-select best format
- `go2rtc_webrtc` - Force WebRTC
- `go2rtc_mse` - Force Media Source Extensions
- `go2rtc_hls` - Force HLS

The UI embeds go2rtc's `video-stream.js` component which handles all protocol negotiation.

#### 3.0.4 Key Learnings for zm-api

| ZoneMinder Approach | zm-api Improvement |
|---------------------|-------------------|
| libcurl for HTTP calls | Use `reqwest` with async/retry |
| Hardcoded port 1984 | Configurable endpoint |
| 3 channels per monitor | Dynamic channel management |
| No health monitoring | Periodic health checks |
| Credentials in URL | Token-based auth option |

#### 3.0.5 ⚠️ CRITICAL: go2rtc Security Gaps

**go2rtc has NO built-in per-stream authentication.** This is a major security concern:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     go2rtc Security Vulnerabilities                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  1. NO PER-STREAM AUTH                                                      │
│     • Anyone who reaches go2rtc can access ANY registered stream            │
│     • Stream names (zm1, zm2, etc.) are easily guessable                    │
│     • Basic auth applies to entire API, not individual streams              │
│                                                                              │
│  2. LOCALHOST BYPASS                                                        │
│     • Requests from localhost SKIP authentication entirely                  │
│     • Even with basic auth configured, local access is unrestricted         │
│                                                                              │
│  3. CREDENTIAL EXPOSURE                                                     │
│     • Camera RTSP credentials embedded in registration URLs                 │
│     • go2rtc stores these in plaintext in go2rtc.yaml                       │
│     • API exposes stream sources: GET /api/streams shows credentials        │
│                                                                              │
│  4. CVE-2024-29192 (CVSS 8.8)                                               │
│     • CSRF vulnerability in /api/config endpoint                            │
│     • Can lead to arbitrary command execution via exec source               │
│     • Fixed in go2rtc > 1.8.5                                               │
│                                                                              │
│  5. NO ENCRYPTION                                                           │
│     • WebSocket (ws://) and HTTP used by default                            │
│     • Requires external reverse proxy for HTTPS/WSS                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Current ZoneMinder + go2rtc Attack Scenario:**

```
Attacker                                    go2rtc                  Cameras
   │                                           │                        │
   │── Scan network, find port 1984 ──────────▶│                        │
   │                                           │                        │
   │── GET /api/streams ──────────────────────▶│                        │
   │◀── {streams: {zm1: {src: "rtsp://        │                        │
   │     admin:password@192.168.1.50:554"}}}   │  CREDENTIALS EXPOSED!  │
   │                                           │                        │
   │── ws://host:1984/api/ws?src=zm1 ─────────▶│                        │
   │◀── Live video stream ────────────────────│◀───────────────────────│
   │                                           │                        │
   │   FULL ACCESS TO ALL CAMERAS              │                        │
   │   NO AUTHENTICATION REQUIRED              │                        │
```

**zm-api Solution: Authenticated Streaming Proxy**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      zm-api Secure Streaming Architecture                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Client                      zm-api                        go2rtc           │
│     │                           │                             │             │
│     │── JWT Token ─────────────▶│                             │             │
│     │                           │── Validate token            │             │
│     │                           │── Check camera permissions  │             │
│     │                           │── Generate short-lived      │             │
│     │                           │   stream token              │             │
│     │                           │                             │             │
│     │◀── Proxied WebSocket ─────│── Internal request ────────▶│             │
│     │    (wss://zm-api/...)     │   (localhost bypass OK)     │             │
│     │                           │                             │             │
│     │   BENEFITS:               │                             │             │
│     │   ✓ JWT auth required     │   go2rtc bound to          │             │
│     │   ✓ Per-camera RBAC       │   127.0.0.1 only           │             │
│     │   ✓ Audit logging         │   (not exposed externally)  │             │
│     │   ✓ HTTPS/WSS             │                             │             │
│     │   ✓ Credentials hidden    │                             │             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Recommended go2rtc Security Configuration:**

```yaml
# go2rtc.yaml - SECURE configuration
api:
  listen: "127.0.0.1:1984"     # CRITICAL: localhost only!
  origin: ""                    # Disable CORS

rtsp:
  listen: "127.0.0.1:8554"     # RTSP also localhost only

# Do NOT expose these ports externally
# All external access goes through zm-api proxy
```

### 3.1 Objectives

- Move go2rtc configuration from hardcoded to config file
- Add proper error handling and retry logic
- Implement stream health monitoring
- Add automatic stream registration on monitor creation
- **Mirror ZoneMinder's multi-channel registration** (primary, secondary, restream)

### 3.2 Configuration Schema

```toml
# settings/base.toml
[streaming]
enabled = true
default_provider = "go2rtc"  # "go2rtc" | "native" | "hybrid"

[streaming.go2rtc]
enabled = true
base_url = "http://localhost:1984"
api_path = "/api"
timeout_seconds = 30
retry_attempts = 3
health_check_interval_seconds = 60

# Auto-registration settings
auto_register = true
register_on_startup = false
```

### 3.3 New Endpoints

```
POST   /api/v3/streaming/providers                 # List available providers
GET    /api/v3/streaming/providers/{provider}/status  # Provider health status

PUT    /api/v3/monitors/{id}/stream/register       # Register with streaming provider
DELETE /api/v3/monitors/{id}/stream/unregister     # Unregister stream
GET    /api/v3/monitors/{id}/stream/endpoints      # Get all streaming URLs
```

### 3.4 Implementation Tasks

1. Create `StreamingConfig` struct in `src/configure/`
2. Create `Go2RtcClient` service with proper error handling
3. Implement health check background task
4. Add automatic registration hooks for monitor CRUD
5. Update existing handlers to use configuration

### 3.5 Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `src/configure/streaming.rs` | Create | Streaming configuration structs |
| `src/client/go2rtc.rs` | Create | go2rtc HTTP client with retry logic |
| `src/service/streaming.rs` | Create | Unified streaming service |
| `src/handlers/streaming.rs` | Modify | Use new service layer |
| `settings/base.toml` | Modify | Add streaming configuration |

---

## 4. Phase 2: Native WebRTC Implementation

### 4.1 Objectives

- Implement native WebRTC without external C++ plugin
- Support ICE/STUN/TURN for NAT traversal
- Provide fallback to go2rtc when needed
- Handle multiple simultaneous viewers

### 4.2 Technology Choice

**Recommended: `webrtc-rs`** - Pure Rust WebRTC implementation

```toml
# Cargo.toml additions
webrtc = "0.10"
webrtc-media = "0.7"
webrtc-util = "0.8"
```

**Alternative: `str0m`** - Lightweight WebRTC for servers

### 4.3 Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     WebRTC Service                            │
├──────────────────────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │
│  │   Signaling    │  │  Connection    │  │    Media       │ │
│  │    Server      │  │    Manager     │  │   Pipeline     │ │
│  │                │  │                │  │                │ │
│  │ - SDP Exchange │  │ - ICE Handling │  │ - RTP Packetize│ │
│  │ - ICE Trickle  │  │ - DTLS Setup   │  │ - H264 Payload │ │
│  │ - Session Mgmt │  │ - SCTP (data)  │  │ - Opus Audio   │ │
│  └────────────────┘  └────────────────┘  └────────────────┘ │
│                              │                               │
│                    ┌─────────▼─────────┐                    │
│                    │   RTSP Demuxer    │                    │
│                    │  (H264/H265/AAC)  │                    │
│                    └───────────────────┘                    │
└──────────────────────────────────────────────────────────────┘
```

### 4.4 Signaling Protocol (WebSocket)

```typescript
// Client → Server
interface SignalingMessage {
  type: "offer" | "answer" | "ice-candidate" | "hangup";
  camera_id: number;
  session_id: string;
  payload: SDPOffer | SDPAnswer | IceCandidate;
}

// Server → Client
interface SignalingResponse {
  type: "offer" | "answer" | "ice-candidate" | "error" | "connected";
  camera_id: number;
  session_id: string;
  payload: any;
}
```

### 4.5 Endpoints

```
# WebSocket signaling
WS  /api/v3/webrtc/{camera_id}/signaling

# REST fallback signaling (current style, maintained for compatibility)
POST /api/v3/webrtc/{camera_id}/offer           # Client sends offer
POST /api/v3/webrtc/{camera_id}/answer          # Server receives answer
POST /api/v3/webrtc/{camera_id}/ice-candidate   # ICE candidate exchange
GET  /api/v3/webrtc/{camera_id}/stats           # Connection stats
DELETE /api/v3/webrtc/{camera_id}/session       # End session

# Configuration
GET  /api/v3/webrtc/config                      # ICE servers, codecs
```

### 4.6 TURN/STUN Configuration

```toml
[streaming.webrtc]
enabled = true
mode = "native"  # "native" | "plugin" | "go2rtc"

# ICE Configuration
stun_servers = [
    "stun:stun.l.google.com:19302",
    "stun:stun1.l.google.com:19302"
]

[streaming.webrtc.turn]
enabled = false
server = "turn:your-turn-server.com:3478"
username = ""
password = ""
realm = ""
```

### 4.7 Implementation Tasks

1. Add webrtc-rs dependencies to Cargo.toml
2. Create `WebRtcEngine` struct for managing connections
3. Implement RTSP to RTP pipeline
4. Create WebSocket signaling handler
5. Implement session management with cleanup
6. Add connection statistics endpoints
7. Write integration tests

### 4.8 Files to Create

| File | Description |
|------|-------------|
| `src/streaming/mod.rs` | Streaming module root |
| `src/streaming/webrtc/mod.rs` | WebRTC module |
| `src/streaming/webrtc/engine.rs` | WebRTC engine core |
| `src/streaming/webrtc/signaling.rs` | Signaling protocol |
| `src/streaming/webrtc/session.rs` | Session management |
| `src/streaming/webrtc/pipeline.rs` | Media pipeline |
| `src/handlers/webrtc_native.rs` | New WebRTC handlers |

---

## 5. Phase 3: Native HLS Streaming

### 5.1 Objectives

- Generate HLS playlists and segments natively
- Support multiple quality levels (ABR)
- Low-latency HLS (LL-HLS) for live viewing
- DVR functionality with segment retention

### 5.2 HLS Specification

```
Stream Format:
  - Container: fMP4 (Fragmented MP4)
  - Video Codec: H.264/H.265 passthrough
  - Audio Codec: AAC passthrough
  - Segment Duration: 2-6 seconds (configurable)
  - Playlist Type: EVENT (live) or VOD (recordings)

Low-Latency HLS:
  - Partial Segments: 200-500ms
  - Preload Hints: Next partial segment
  - Blocking Playlist Reload: Client waits for updates
```

### 5.3 Directory Structure

```
/var/cache/zm-api/hls/
├── {camera_id}/
│   ├── master.m3u8              # Master playlist (ABR)
│   ├── stream_720p.m3u8         # Variant playlist
│   ├── stream_1080p.m3u8        # Variant playlist
│   ├── init.mp4                 # Initialization segment
│   ├── segment_00001.m4s        # Media segment
│   ├── segment_00002.m4s
│   └── ...
```

### 5.4 Endpoints

```
# Playlist endpoints
GET /api/v3/hls/{camera_id}/master.m3u8          # Master playlist
GET /api/v3/hls/{camera_id}/{quality}.m3u8       # Variant playlist
GET /api/v3/hls/{camera_id}/init.mp4             # Init segment
GET /api/v3/hls/{camera_id}/{segment}.m4s        # Media segment

# LL-HLS endpoints
GET /api/v3/hls/{camera_id}/{quality}.m3u8?_HLS_msn={n}&_HLS_part={p}  # Blocking reload
GET /api/v3/hls/{camera_id}/{segment}.m4s?_HLS_part={p}               # Partial segment

# Control endpoints
POST   /api/v3/hls/{camera_id}/start             # Start HLS generation
DELETE /api/v3/hls/{camera_id}/stop              # Stop HLS generation
GET    /api/v3/hls/{camera_id}/stats             # Stream statistics
```

### 5.5 Configuration

```toml
[streaming.hls]
enabled = true
segment_duration_seconds = 4
playlist_size = 6           # Number of segments in playlist
ll_hls_enabled = true       # Low-latency HLS
partial_segment_ms = 300    # Partial segment duration

# Quality variants
[[streaming.hls.variants]]
name = "1080p"
width = 1920
height = 1080
bitrate = 5000000
passthrough = true          # No transcode

[[streaming.hls.variants]]
name = "720p"
width = 1280
height = 720
bitrate = 2500000
passthrough = false         # Transcode

[[streaming.hls.variants]]
name = "480p"
width = 854
height = 480
bitrate = 1000000
passthrough = false

# Storage
[streaming.hls.storage]
path = "/var/cache/zm-api/hls"
retention_minutes = 30      # How long to keep segments
cleanup_interval_seconds = 60
```

### 5.6 Implementation Tasks

1. Create HLS segment generator (fMP4 muxer)
2. Implement playlist generator (m3u8)
3. Create segment storage manager with cleanup
4. Implement LL-HLS partial segments
5. Add ABR quality switching logic
6. Create HTTP handlers with proper caching headers

### 5.7 Files to Create

| File | Description |
|------|-------------|
| `src/streaming/hls/mod.rs` | HLS module |
| `src/streaming/hls/segmenter.rs` | fMP4 segment generation |
| `src/streaming/hls/playlist.rs` | m3u8 playlist generation |
| `src/streaming/hls/storage.rs` | Segment storage and cleanup |
| `src/streaming/hls/ll_hls.rs` | Low-latency HLS support |
| `src/handlers/hls.rs` | HLS HTTP handlers |
| `src/routes/hls.rs` | HLS routes |

---

## 6. Phase 4: WebSocket/MSE Enhancements

### 6.1 Current Implementation Review

Current MSE implementation (`src/mse_client.rs`, `src/handlers/mse.rs`):
- ✅ fMP4 segment delivery via WebSocket
- ✅ Segment buffering (300 segments)
- ✅ Broadcast to multiple clients
- ❌ Depends on external MSE plugin
- ❌ No quality adaptation
- ❌ No native segment generation

### 6.2 Enhancement Objectives

- Remove dependency on external MSE plugin
- Implement native RTSP to fMP4 pipeline
- Add adaptive bitrate support
- Improve buffering and latency

### 6.3 Native fMP4 Pipeline

```
RTSP Source → Demuxer → Depacketizer → fMP4 Muxer → Segment Buffer → WebSocket
                           │
                           ▼
                   ┌───────────────┐
                   │   Transcoder  │ (optional)
                   │   (FFmpeg)    │
                   └───────────────┘
```

### 6.4 Enhanced WebSocket Protocol

```typescript
// Server → Client messages
interface SegmentMessage {
  type: "segment";
  camera_id: number;
  sequence: number;
  timestamp: number;
  duration_ms: number;
  is_init: boolean;
  quality: string;           // NEW: "1080p", "720p", etc.
  data: string;              // base64 encoded fMP4
}

interface QualityChangeMessage {
  type: "quality_change";
  available: string[];       // NEW: available qualities
  current: string;
  bandwidth_estimate: number;
}

// Client → Server messages
interface QualityRequest {
  type: "set_quality";
  quality: string | "auto";
}
```

### 6.5 Implementation Tasks

1. Create native RTSP demuxer (using `retina` crate)
2. Implement fMP4 muxer (using `mp4` crate)
3. Add quality adaptation based on client feedback
4. Migrate from plugin socket to native implementation
5. Keep plugin support as fallback option

### 6.6 Dependencies

```toml
retina = "0.4"          # RTSP client and demuxer
mp4 = "0.14"            # MP4/fMP4 muxer
h264-reader = "0.7"     # H.264 NAL parsing
```

---

## 7. Phase 5: RTSP Proxy Server

### 7.1 Objectives

- Expose authenticated RTSP streams to clients
- Hide camera credentials from end users
- Add stream access control and logging
- Support re-streaming for load distribution

### 7.2 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      RTSP Proxy Server                       │
│                        (Port 5554)                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Client Request:                                             │
│  rtsp://zm-api:5554/cameras/5?token=eyJhbG...               │
│                    │                                         │
│                    ▼                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │               Authentication Layer                    │   │
│  │   - Validate JWT token                               │   │
│  │   - Check camera access permissions                  │   │
│  │   - Rate limiting                                    │   │
│  └──────────────────────────────────────────────────────┘   │
│                    │                                         │
│                    ▼                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                Session Manager                        │   │
│  │   - Map client sessions to camera streams            │   │
│  │   - Handle RTSP commands (DESCRIBE, SETUP, PLAY)     │   │
│  │   - Manage RTP/RTCP ports                            │   │
│  └──────────────────────────────────────────────────────┘   │
│                    │                                         │
│                    ▼                                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │               Stream Multiplexer                      │   │
│  │   - Single connection to camera RTSP                 │   │
│  │   - Distribute to multiple clients                   │   │
│  │   - RTP packet forwarding                            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 7.3 RTSP URL Format

```
# Authenticated RTSP access via zm-api proxy
rtsp://zm-api-server:5554/cameras/{camera_id}?token={jwt_token}

# Alternative: Basic auth with API credentials
rtsp://username:api_key@zm-api-server:5554/cameras/{camera_id}
```

### 7.4 Endpoints

```
# RTSP Protocol (Port 5554)
DESCRIBE rtsp://host:5554/cameras/{id}
SETUP    rtsp://host:5554/cameras/{id}/trackID=1
PLAY     rtsp://host:5554/cameras/{id}
TEARDOWN rtsp://host:5554/cameras/{id}

# REST Management (Port 8080)
GET    /api/v3/rtsp/sessions                  # List active RTSP sessions
DELETE /api/v3/rtsp/sessions/{session_id}     # Kill session
GET    /api/v3/rtsp/cameras/{id}/sdp          # Get SDP for camera
POST   /api/v3/rtsp/cameras/{id}/announce     # For RTSP publishing (future)
```

### 7.5 Configuration

```toml
[streaming.rtsp_proxy]
enabled = true
bind_address = "0.0.0.0"
port = 5554
rtp_port_range = [20000, 30000]
max_sessions = 100
session_timeout_seconds = 300

# Authentication
auth_method = "token"  # "token" | "basic" | "digest"
require_https = false  # Require HTTPS for token in query

# Transport
transport = "udp"      # "udp" | "tcp" | "auto"
```

### 7.6 Implementation with `retina`

```toml
retina = { version = "0.4", features = ["server"] }  # If server feature exists
# Or custom RTSP server implementation
```

### 7.7 Files to Create

| File | Description |
|------|-------------|
| `src/streaming/rtsp/mod.rs` | RTSP proxy module |
| `src/streaming/rtsp/server.rs` | RTSP server implementation |
| `src/streaming/rtsp/session.rs` | Session management |
| `src/streaming/rtsp/auth.rs` | Token/credential validation |
| `src/streaming/rtsp/multiplexer.rs` | Stream multiplexing |

---

## 8. ZoneMinder Improvements

### 8.1 Current ZoneMinder Streaming Architecture

ZoneMinder currently has **three streaming paths**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Current ZoneMinder Streaming Options                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  1. zms (CGI Binary) - LEGACY                                               │
│     ├── Spawns new process per request                                      │
│     ├── Reads from shared memory (decoded frames)                           │
│     ├── MJPEG output only                                                   │
│     └── Relies on PHP session for auth                                      │
│                                                                              │
│  2. ZoneMinder RTSP Server - MODERN (xop library)                           │
│     ├── Reads from FIFOs (H.264/H.265 encoded packets)                      │
│     ├── Multi-client distribution                                           │
│     ├── Low latency passthrough                                             │
│     └── Port range: ZM_MIN_RTSP_PORT                                        │
│                                                                              │
│  3. go2rtc Integration - EXTERNAL                                           │
│     ├── Separate process/container                                          │
│     ├── WebRTC, HLS, MJPEG outputs                                          │
│     └── Connects to camera RTSP directly                                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.2 zms Limitations

The `zms` (ZoneMinder Streaming) CGI binary has critical limitations:

| Issue | Impact | zm-api Solution |
|-------|--------|-----------------|
| CGI-based | New process per request, high overhead | Persistent async server |
| MJPEG only | No modern protocol support | WebRTC, HLS, MSE |
| Shared memory coupling | Tight coupling to zmc/zma | FIFO-based or direct RTSP |
| No authentication | Relies on PHP session | JWT-based auth |
| Single-threaded | Cannot utilize modern hardware | Tokio async runtime |
| Decodes frames | Unnecessary transcode for passthrough | H.264/H.265 passthrough |

### 8.3 Proposed FIFO-Based Integration

The most efficient zm-api integration leverages ZoneMinder's existing FIFO infrastructure:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Proposed: zm-api FIFO Integration                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Camera ──RTSP──▶ zmc ──▶ PacketQueue ──▶ FIFO ──┬──▶ ZM RTSP Server       │
│                              │                    │                          │
│                              │                    └──▶ zm-api FIFO Reader    │
│                              │                              │                │
│                              ▼                              ▼                │
│                      Shared Memory              ┌─────────────────────┐     │
│                      (decoded frames)           │   zm-api Streaming  │     │
│                              │                  │                     │     │
│                              ▼                  │  ┌───────────────┐  │     │
│                      zms (MJPEG) ◄──DEPRECATED  │  │ WebRTC Server │  │     │
│                                                 │  ├───────────────┤  │     │
│                                                 │  │  HLS Server   │  │     │
│                                                 │  ├───────────────┤  │     │
│                                                 │  │  MSE/WS Server│  │     │
│                                                 │  └───────────────┘  │     │
│                                                 └─────────────────────┘     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### FIFO Reader Implementation

```rust
// src/streaming/source/fifo.rs
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

pub struct ZmFifoSource {
    monitor_id: u32,
    video_path: PathBuf,
    audio_path: Option<PathBuf>,
    codec: VideoCodec,
}

impl ZmFifoSource {
    pub fn new(monitor_id: u32, zm_cache_path: &Path) -> Self {
        let base = zm_cache_path.join("events").join(monitor_id.to_string());
        Self {
            monitor_id,
            video_path: base.join(".video_fifo"),
            audio_path: Some(base.join(".audio_fifo")),
            codec: VideoCodec::H264, // Detected from FIFO data
        }
    }

    pub async fn read_packet(&mut self) -> Result<EncodedPacket, Error> {
        // Read NAL unit length prefix + data from FIFO
        // Parse H.264/H.265 NAL units
        // Return packetized data ready for WebRTC/HLS/MSE
    }
}
```

### 8.4 Proposed Replacement Strategy

```
                    CURRENT                          PROPOSED

┌─────────────┐                          ┌─────────────────────────────┐
│   Browser   │                          │         Browser             │
└──────┬──────┘                          └──────────────┬──────────────┘
       │                                                │
       ▼                                                ▼
┌─────────────┐                          ┌─────────────────────────────┐
│   Apache    │                          │           nginx             │
│  /cgi-bin/  │                          │   (reverse proxy only)      │
└──────┬──────┘                          └──────────────┬──────────────┘
       │                                                │
       ▼                                                ▼
┌─────────────┐                          ┌─────────────────────────────┐
│     zms     │◄──── REPLACE ────────────│          zm-api             │
│  (CGI/C++)  │                          │     Streaming Core          │
└──────┬──────┘                          │                             │
       │                                 │  • WebRTC                   │
       ▼                                 │  • HLS                      │
┌─────────────┐                          │  • MSE/WebSocket            │
│  Shared     │                          │  • RTSP Proxy               │
│  Memory     │                          └──────────────┬──────────────┘
│  (zmc/zma)  │                                         │
└─────────────┘                          ┌──────────────┴──────────────┐
                                         │                             │
                                         ▼                             ▼
                                  ┌─────────────┐             ┌─────────────┐
                                  │   Direct    │             │  Shared     │
                                  │   RTSP      │             │  Memory     │
                                  │   (new)     │             │  (legacy)   │
                                  └─────────────┘             └─────────────┘
```

### 8.3 ZoneMinder Integration Points

#### 8.3.1 Direct RTSP Access (Recommended)

zm-api connects directly to camera RTSP streams, bypassing zmc/zma for live viewing:

```
Advantages:
  ✓ Lower latency (no shared memory hop)
  ✓ Multiple quality streams from camera
  ✓ Independent of ZoneMinder processing load
  ✓ Can stream while ZoneMinder is down

Disadvantages:
  ✗ Separate connection to camera (2 instead of 1)
  ✗ No access to ZoneMinder's processed stream
```

#### 8.3.2 Shared Memory Interface (Compatibility)

zm-api reads from ZoneMinder's shared memory for processed frames:

```rust
// Future: ZoneMinder shared memory reader
pub struct ZmSharedMemory {
    monitor_id: u32,
    shm_key: i32,
    image_buffer: *mut u8,
    // ...
}
```

#### 8.3.3 Database Integration

Monitor configuration from ZoneMinder database:

```sql
-- Monitor RTSP details
SELECT Id, Name, Protocol, Method, Host, Port, Path, User, Pass,
       Width, Height, Colours, Capturing
FROM Monitors
WHERE Capturing != 'None';
```

### 8.4 Proposed ZoneMinder Changes

#### 8.4.1 New Configuration Options

```php
// New ZoneMinder config options
ZM_STREAMING_PROVIDER = 'zm-api'  // 'zms' | 'zm-api' | 'go2rtc'
ZM_API_STREAMING_URL = 'http://localhost:8080'
ZM_API_STREAMING_WS = 'ws://localhost:8080'
```

#### 8.4.2 Web Interface Changes

```javascript
// zmNinja / ZoneMinder Web - streaming URL construction
function getStreamUrl(monitorId, type) {
  if (ZM_STREAMING_PROVIDER === 'zm-api') {
    switch(type) {
      case 'webrtc':
        return `${ZM_API_STREAMING_WS}/api/v3/webrtc/${monitorId}/signaling`;
      case 'hls':
        return `${ZM_API_STREAMING_URL}/api/v3/hls/${monitorId}/master.m3u8`;
      case 'mse':
        return `${ZM_API_STREAMING_WS}/api/v3/mse/streams/${monitorId}/live`;
      default:
        return `${ZM_API_STREAMING_URL}/api/v3/streams/${monitorId}`;
    }
  }
  // Legacy zms fallback
  return `/cgi-bin/zms?monitor=${monitorId}&mode=jpeg`;
}
```

#### 8.4.3 Deprecation Path

```
Phase 1: Add zm-api streaming as option alongside zms
Phase 2: Make zm-api the default, zms available as legacy
Phase 3: Remove zms, full zm-api streaming
```

### 8.5 Proposed ZoneMinder Project Changes

To fully realize zm-api streaming integration, we propose the following changes to the ZoneMinder project:

#### 8.5.1 FIFO Protocol Documentation

**Issue**: FIFO format is undocumented, making external integration difficult.

**Proposal**: Document the FIFO wire protocol in ZoneMinder's developer docs:

```
FIFO Wire Format (proposed documentation):

Video FIFO (.video_fifo):
┌─────────────────────────────────────────────────────┐
│ Header (8 bytes)                                    │
├─────────────────────────────────────────────────────┤
│ nal_unit_length: u32 (big-endian)                   │
│ timestamp_us: u32 (presentation timestamp)          │
├─────────────────────────────────────────────────────┤
│ NAL Unit Data (nal_unit_length bytes)               │
│ - H.264: 0x00 0x00 0x00 0x01 + NAL                  │
│ - H.265: 0x00 0x00 0x00 0x01 + NAL                  │
└─────────────────────────────────────────────────────┘

Audio FIFO (.audio_fifo):
┌─────────────────────────────────────────────────────┐
│ Header (8 bytes)                                    │
├─────────────────────────────────────────────────────┤
│ frame_length: u32 (big-endian)                      │
│ timestamp_us: u32 (presentation timestamp)          │
├─────────────────────────────────────────────────────┤
│ Audio Frame Data (frame_length bytes)               │
│ - AAC: ADTS frame                                   │
│ - G.711: Raw PCM samples                            │
└─────────────────────────────────────────────────────┘
```

#### 8.5.2 Configurable FIFO Paths

**Issue**: FIFO paths are hardcoded in zmc.

**Proposal**: Add ZoneMinder config options:

```php
// Options -> Paths
ZM_PATH_FIFOS = '/var/cache/zoneminder/fifos'  // Dedicated FIFO directory
ZM_FIFO_VIDEO_SUFFIX = '.video_fifo'
ZM_FIFO_AUDIO_SUFFIX = '.audio_fifo'
```

#### 8.5.3 zm-api Streaming Provider Option

**Issue**: ZoneMinder web UI only supports zms for live streaming.

**Proposal**: Add streaming provider configuration:

```php
// Options -> Network
ZM_STREAMING_PROVIDER = 'zm-api'  // Options: 'zms', 'zm-api', 'rtsp'
ZM_API_STREAM_URL = 'http://localhost:8080'
ZM_API_WS_URL = 'ws://localhost:8080'
```

**Web UI Changes** (web/skins/classic/views/watch.php):

```php
function getStreamUrl($monitor_id, $type = 'auto') {
    global $ZM_STREAMING_PROVIDER, $ZM_API_STREAM_URL;

    if ($ZM_STREAMING_PROVIDER === 'zm-api') {
        switch ($type) {
            case 'webrtc':
                return "{$ZM_API_WS_URL}/api/v3/webrtc/{$monitor_id}/signaling";
            case 'hls':
                return "{$ZM_API_STREAM_URL}/api/v3/hls/{$monitor_id}/master.m3u8";
            case 'mse':
                return "{$ZM_API_WS_URL}/api/v3/mse/streams/{$monitor_id}/live";
            default:
                return "{$ZM_API_STREAM_URL}/api/v3/monitors/{$monitor_id}/stream";
        }
    }

    // Legacy zms fallback
    return "/cgi-bin/zms?monitor={$monitor_id}&mode=jpeg";
}
```

#### 8.5.4 FIFO Availability API

**Issue**: No way for external services to know when FIFOs are ready.

**Proposal**: Add API endpoint to check FIFO status:

```
GET /api/monitors/{id}/fifo/status

Response:
{
  "monitor_id": 5,
  "video_fifo": {
    "path": "/var/cache/zoneminder/fifos/5/.video_fifo",
    "exists": true,
    "has_writer": true,
    "codec": "h264"
  },
  "audio_fifo": {
    "path": "/var/cache/zoneminder/fifos/5/.audio_fifo",
    "exists": true,
    "has_writer": true,
    "codec": "aac"
  }
}
```

#### 8.5.5 Deprecation of zms

**Proposal**: Gradual deprecation path for zms:

| Version | Status |
|---------|--------|
| ZM 1.38 | Add zm-api as optional streaming provider |
| ZM 1.39 | Make zm-api default, zms available with warning |
| ZM 1.40 | zms deprecated, minimal maintenance |
| ZM 2.0  | Remove zms entirely |

#### 8.5.6 PacketQueue IPC Interface (Advanced)

**Issue**: Direct PacketQueue access requires embedding in ZoneMinder process.

**Proposal**: Add Unix domain socket interface to PacketQueue:

```cpp
// zm_packet_server.cpp - New component
class PacketServer {
    void start(const std::string& socket_path);
    void handle_client(int client_fd);
    // Sends packets to connected clients via Unix socket
    // Clients can request specific monitor_id streams
};
```

This would allow zm-api to receive packets with even lower latency than FIFOs.

### 8.6 Benefits Summary

| Improvement | Benefit |
|-------------|---------|
| **Performance** | Async Rust vs fork-per-request C++ |
| **Scalability** | Handle 1000s of concurrent viewers |
| **Protocols** | WebRTC, HLS, MSE vs MJPEG only |
| **Latency** | Sub-second with WebRTC/LL-HLS |
| **Mobile** | Native protocol support |
| **Security** | JWT auth, RBAC, audit logging |
| **Maintainability** | Modern codebase, better testing |
| **Integration** | FIFO-based, no duplicate camera connections |

---

## 9. API Specification

### 9.1 Unified Streaming API

```yaml
openapi: 3.0.0
info:
  title: ZM-API Streaming
  version: 3.1.0

paths:
  # Stream Management
  /api/v3/monitors/{id}/stream:
    get:
      summary: Get streaming endpoints for monitor
      responses:
        200:
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/StreamEndpoints'

    post:
      summary: Start streaming for monitor
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/StreamRequest'

    delete:
      summary: Stop streaming for monitor

  # WebRTC
  /api/v3/webrtc/{camera_id}/signaling:
    get:
      summary: WebSocket signaling endpoint

  # HLS
  /api/v3/hls/{camera_id}/master.m3u8:
    get:
      summary: HLS master playlist

  # MSE
  /api/v3/mse/streams/{camera_id}/live:
    get:
      summary: WebSocket for live fMP4 segments

components:
  schemas:
    StreamEndpoints:
      type: object
      properties:
        monitor_id:
          type: integer
        webrtc:
          type: object
          properties:
            signaling_url: string
            stun_servers: array
        hls:
          type: object
          properties:
            master_playlist: string
            variants: array
        mse:
          type: object
          properties:
            websocket_url: string
            init_segment_url: string
        rtsp:
          type: object
          properties:
            url: string

    StreamRequest:
      type: object
      properties:
        protocols:
          type: array
          items:
            enum: [webrtc, hls, mse, rtsp]
        quality:
          enum: [auto, 1080p, 720p, 480p]
        low_latency:
          type: boolean
```

### 9.2 Response Examples

```json
// GET /api/v3/monitors/5/stream
{
  "monitor_id": 5,
  "monitor_name": "Front Door",
  "status": "streaming",
  "protocols": {
    "webrtc": {
      "signaling_url": "wss://api.example.com/api/v3/webrtc/5/signaling",
      "ice_servers": [
        {"urls": "stun:stun.l.google.com:19302"}
      ],
      "supported": true
    },
    "hls": {
      "master_playlist": "https://api.example.com/api/v3/hls/5/master.m3u8",
      "variants": ["1080p", "720p", "480p"],
      "low_latency": true,
      "supported": true
    },
    "mse": {
      "websocket_url": "wss://api.example.com/api/v3/mse/streams/5/live",
      "init_segment": "https://api.example.com/api/v3/mse/streams/5/init.mp4",
      "supported": true
    },
    "rtsp": {
      "url": "rtsp://api.example.com:5554/cameras/5?token={jwt}",
      "transport": "tcp",
      "supported": true
    },
    "go2rtc": {
      "webrtc": "http://192.168.0.35:1984/webrtc.html?src=zm5",
      "hls": "http://192.168.0.35:1984/api/stream.m3u8?src=zm5",
      "supported": true
    }
  },
  "source": {
    "type": "rtsp",
    "resolution": "1920x1080",
    "codec": "h264",
    "fps": 30
  }
}
```

---

## 10. Configuration Design

### 10.1 Complete Configuration Schema

```toml
# settings/base.toml - Streaming section

[streaming]
enabled = true
default_protocol = "auto"  # auto | webrtc | hls | mse

# Source configuration
[streaming.source]
priority = ["fifo", "rtsp", "go2rtc"]  # Source preference order
prefer_direct_rtsp = true              # Connect to camera directly
fallback_to_go2rtc = true              # Use go2rtc if direct fails
cache_sdp_seconds = 300                # SDP caching duration

# ZoneMinder FIFO integration (preferred source)
[streaming.zoneminder]
enabled = true
fifo_base_path = "/var/cache/zoneminder/events"  # or "/var/cache/zoneminder/fifos"
video_fifo_suffix = ".video_fifo"
audio_fifo_suffix = ".audio_fifo"
fifo_read_timeout_ms = 5000
reconnect_delay_ms = 1000

# go2rtc integration
[streaming.go2rtc]
enabled = true
base_url = "http://localhost:1984"
timeout_seconds = 30
auto_register = true
health_check_interval = 60

# Native WebRTC
[streaming.webrtc]
enabled = true
mode = "native"                # native | plugin | go2rtc
max_connections = 500
connection_timeout_seconds = 60
stun_servers = ["stun:stun.l.google.com:19302"]

[streaming.webrtc.turn]
enabled = false
server = ""
username = ""
password = ""

# Native HLS
[streaming.hls]
enabled = true
segment_duration = 4
playlist_size = 6
ll_hls_enabled = true
partial_segment_ms = 300

[streaming.hls.storage]
path = "/var/cache/zm-api/hls"
retention_minutes = 30

[[streaming.hls.variants]]
name = "1080p"
width = 1920
height = 1080
bitrate = 5000000
passthrough = true

[[streaming.hls.variants]]
name = "720p"
width = 1280
height = 720
bitrate = 2500000

# Native MSE/WebSocket
[streaming.mse]
enabled = true
mode = "native"                # native | plugin
max_buffer_segments = 300
segment_duration_ms = 500

# RTSP Proxy
[streaming.rtsp_proxy]
enabled = false
port = 5554
rtp_port_range = [20000, 30000]
max_sessions = 100
transport = "tcp"
```

### 10.2 Environment Variable Overrides

```bash
# All streaming config can be overridden via environment
APP_STREAMING__ENABLED=true
APP_STREAMING__GO2RTC__BASE_URL=http://go2rtc:1984
APP_STREAMING__WEBRTC__MODE=native
APP_STREAMING__HLS__STORAGE__PATH=/data/hls
```

---

## 11. Security Considerations

### 11.1 Authentication Flow

```
┌──────────┐     ┌──────────┐     ┌──────────────┐     ┌─────────┐
│  Client  │────▶│  Verify  │────▶│   Streaming  │────▶│  Camera │
│          │     │   JWT    │     │    Service   │     │  RTSP   │
└──────────┘     └──────────┘     └──────────────┘     └─────────┘
      │                │
      │    Token in:   │
      │    - Header    │
      │    - Query     │ (for m3u8/m4s/ws)
      │    - Cookie    │
      └────────────────┘
```

### 11.2 Security Measures

| Layer | Protection |
|-------|------------|
| **Transport** | HTTPS/WSS required for production |
| **Authentication** | JWT tokens with expiry |
| **Authorization** | Per-camera access control |
| **Rate Limiting** | Per-user connection limits |
| **Credential Hiding** | Camera passwords never exposed |
| **Audit Logging** | All stream access logged |
| **Token Rotation** | Stream tokens with short expiry |

### 11.3 Token for Media Segments

```rust
// Generate short-lived token for HLS segments
pub fn generate_segment_token(user_id: i32, camera_id: i32, expires_in: Duration) -> String {
    // Short-lived token (5 minutes) for segment access
    jwt::encode(SegmentClaims {
        user_id,
        camera_id,
        exp: Utc::now() + expires_in,
    })
}

// Validate in segment handler
pub async fn get_segment(
    Query(params): Query<SegmentParams>,
    Path((camera_id, segment)): Path<(i32, String)>,
) -> Result<Response, AppError> {
    validate_segment_token(&params.token, camera_id)?;
    // Serve segment...
}
```

---

## 12. Migration Strategy

### 12.1 Phase Rollout

```
Phase 1 (Current + 2 weeks): go2rtc Enhancement
  └─ Configuration externalization
  └─ Error handling improvements
  └─ Health monitoring

Phase 2 (Phase 1 + 4 weeks): Native WebRTC
  └─ webrtc-rs integration
  └─ Remove plugin dependency
  └─ WebSocket signaling

Phase 3 (Phase 2 + 4 weeks): Native HLS
  └─ Segment generation
  └─ LL-HLS support
  └─ ABR variants

Phase 4 (Phase 3 + 2 weeks): MSE Enhancement
  └─ Native RTSP demuxing
  └─ Remove MSE plugin dependency

Phase 5 (Phase 4 + 4 weeks): RTSP Proxy
  └─ RTSP server implementation
  └─ Authentication integration

Phase 6 (Ongoing): ZoneMinder Integration
  └─ UI changes
  └─ zms deprecation
  └─ Documentation
```

### 12.2 Backward Compatibility

```
Existing Endpoints (Maintained):
  - PUT/GET/DELETE /api/v3/streams/{id}     → go2rtc proxy
  - GET /api/v3/mse/streams/{id}/*          → MSE streaming
  - */webrtc/* signaling endpoints          → WebRTC plugin

New Endpoints (Added):
  - /api/v3/monitors/{id}/stream            → Unified streaming
  - /api/v3/webrtc/{id}/signaling           → Native WebRTC WS
  - /api/v3/hls/{id}/*                      → Native HLS
  - rtsp://host:5554/cameras/{id}           → RTSP proxy

Plugin Fallback:
  - If native fails, fall back to plugin
  - Configuration controls preference
```

### 12.3 Testing Strategy

```
Unit Tests:
  - Segment generation
  - Playlist formatting
  - Token validation

Integration Tests:
  - RTSP source connectivity
  - WebRTC signaling flow
  - HLS playback in browser
  - Multi-client streaming

Load Tests:
  - 100 concurrent WebRTC connections
  - 500 concurrent HLS viewers
  - Memory and CPU profiling

Compatibility Tests:
  - Safari (HLS native)
  - Chrome/Firefox (MSE/WebRTC)
  - VLC (RTSP/HLS)
  - Mobile browsers
```

---

## Appendix A: Crate Dependencies

```toml
# Core streaming dependencies to add to Cargo.toml

# WebRTC
webrtc = "0.10"
webrtc-media = "0.7"

# RTSP Client
retina = "0.4"

# Media containers
mp4 = "0.14"
h264-reader = "0.7"

# Optional: FFmpeg bindings for transcoding
ffmpeg-next = "6.0"  # Or use GStreamer
```

---

## Appendix B: Reference Projects

| Project | Use Case | URL |
|---------|----------|-----|
| **go2rtc** | Multi-protocol streaming | github.com/AlexxIT/go2rtc |
| **mediamtx** | RTSP server | github.com/bluenviron/mediamtx |
| **webrtc-rs** | Rust WebRTC | github.com/webrtc-rs/webrtc |
| **retina** | RTSP client | github.com/scottlamb/retina |
| **Janus** | WebRTC gateway | github.com/meetecho/janus-gateway |

---

## Appendix C: Success Metrics

| Metric | Target |
|--------|--------|
| WebRTC connection setup | < 2 seconds |
| HLS startup time | < 3 seconds |
| Live latency (WebRTC) | < 500ms |
| Live latency (LL-HLS) | < 2 seconds |
| Concurrent viewers per camera | 100+ |
| Total concurrent connections | 1000+ |
| CPU usage per stream (passthrough) | < 5% |
| Memory per connection | < 10MB |

---

*Document Version: 1.0*
*Last Updated: January 2026*
*Author: zm-api Development Team*
