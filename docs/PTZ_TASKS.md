# PTZ Control System - Implementation Tasks

This document tracks implementation tasks for the PTZ Control System.
See the full plan context in the original design document.

## Status Legend
- `[ ]` - Not started
- `[~]` - In progress
- `[x]` - Complete
- `[!]` - Blocked

---

## Phase 0: Perl Bridge (Immediate API Functionality) ✅ COMPLETE

### 0.1 Core Types and Traits
- [x] **0.1.1** Create `src/ptz/mod.rs` module structure
- [x] **0.1.2** Define `PtzControl` trait with all standard PTZ methods
  - `move_up`, `move_down`, `move_left`, `move_right`
  - `move_up_left`, `move_up_right`, `move_down_left`, `move_down_right`
  - `move_stop`
  - `zoom_in`, `zoom_out`, `zoom_stop`
  - `focus_near`, `focus_far`, `focus_stop`, `focus_auto`
  - `iris_open`, `iris_close`, `iris_stop`, `iris_auto`
  - `goto_preset`, `set_preset`, `clear_preset`
  - `move_absolute`, `move_relative`
- [x] **0.1.3** Define `PtzCapabilities` struct matching Controls table columns
  - `can_move`, `can_move_diagonally`, `can_move_continuous`, `can_move_relative`, `can_move_absolute`
  - `can_zoom`, `can_zoom_continuous`, `can_zoom_relative`, `can_zoom_absolute`
  - `can_focus`, `can_focus_continuous`, `can_focus_relative`, `can_focus_absolute`, `can_auto_focus`
  - `can_iris`, `can_iris_continuous`, `can_iris_relative`, `can_iris_absolute`, `can_auto_iris`
  - `has_presets`, `num_presets`, `has_home_preset`
  - `can_white_balance`, `can_auto_white_balance`
  - `can_gain`, `can_auto_gain`
  - Speed limits: `min_*_speed`, `max_*_speed` for each axis
- [x] **0.1.4** Define `PtzError` enum for error handling
  - `CameraOffline`, `AuthenticationFailed`, `CommandNotSupported`
  - `InvalidParameter`, `CommandTimeout`, `ProtocolError`
  - `PerlBridgeError`, `MonitorNotFound`, `NoControlConfigured`
- [x] **0.1.5** Define parameter structs
  - `MoveParams` (speed, duration, auto_stop)
  - `AbsolutePosition` (pan, tilt, zoom)
  - `RelativePosition` (pan_delta, tilt_delta, zoom_delta)
  - `PresetParams` (preset_id, name)

### 0.2 Perl Proxy Bridge
- [x] **0.2.1** Create `PerlControlProxy` struct implementing `PtzControl` trait
- [x] **0.2.2** Implement zmcontrol.pl execution via `tokio::process::Command`
- [x] **0.2.3** Map Rust method calls to zmcontrol.pl command-line arguments
  - Example: `move_con_up()` → `zmcontrol.pl --id {id} --command moveConUp --tiltspeed {speed}`
- [x] **0.2.4** Parse zmcontrol.pl stdout/stderr for success/error detection
- [x] **0.2.5** Handle process spawning, timeout, and cleanup
- [x] **0.2.6** Implement auto-start of zmcontrol.pl daemon if not running

### 0.3 Factory and Registry Pattern
- [x] **0.3.1** Create `PtzControlFactory` trait for creating control instances
- [x] **0.3.2** Create `PerlControlFactory` implementing factory trait
- [x] **0.3.3** Create `PtzRegistry` for managing protocol implementations
  - Register factories by protocol name
  - Lookup factory by protocol name
  - List available protocols
- [x] **0.3.4** Create `PtzManager` for caching control instances per monitor
  - Get or create control instance for monitor ID
  - Cache instances with configurable TTL
  - Handle monitor config changes (invalidate cache)

### 0.4 Database Integration
- [x] **0.4.1** Create repo function to load `Controls` table by ID
- [x] **0.4.2** Map Controls columns to `PtzCapabilities` struct
- [x] **0.4.3** Create repo function to get Monitor's control configuration
  - Join Monitor → ControlId → Controls
- [x] **0.4.4** Parse `ControlAddress` field (supports multiple formats)
  - `user:pass@host:port`
  - `https://host`
  - `/dev/ttyUSB0` (serial)
- [x] **0.4.5** Extract credentials from Monitor.User, Monitor.Pass, or ControlAddress

### 0.5 DTOs
- [x] **0.5.1** Create `src/dto/request/ptz.rs`
  - `PtzMoveRequest` (direction, speed, duration)
  - `PtzZoomRequest` (direction, speed)
  - `PtzPresetRequest` (preset_id, name)
  - `PtzCommandRequest` (command_name, params as JSON)
- [x] **0.5.2** Create `src/dto/response/ptz.rs`
  - `PtzStatusResponse` (capabilities, current_position, implementation_type)
  - `PtzCommandResponse` (success, message)
  - `PtzProtocolListResponse` (protocols with native/perl flags)
  - `PtzCapabilitiesResponse` (full capabilities struct)

### 0.6 API Handlers
- [x] **0.6.1** Create `src/handlers/ptz.rs`
- [x] **0.6.2** Implement movement handlers
  - `POST /api/ptz/monitors/{id}/move/up`
  - `POST /api/ptz/monitors/{id}/move/down`
  - `POST /api/ptz/monitors/{id}/move/left`
  - `POST /api/ptz/monitors/{id}/move/right`
  - `POST /api/ptz/monitors/{id}/move/up-left`
  - `POST /api/ptz/monitors/{id}/move/up-right`
  - `POST /api/ptz/monitors/{id}/move/down-left`
  - `POST /api/ptz/monitors/{id}/move/down-right`
  - `POST /api/ptz/monitors/{id}/move/stop`
- [x] **0.6.3** Implement zoom handlers
  - `POST /api/ptz/monitors/{id}/zoom/in`
  - `POST /api/ptz/monitors/{id}/zoom/out`
  - `POST /api/ptz/monitors/{id}/zoom/stop`
- [x] **0.6.4** Implement focus handlers
  - `POST /api/ptz/monitors/{id}/focus/near`
  - `POST /api/ptz/monitors/{id}/focus/far`
  - `POST /api/ptz/monitors/{id}/focus/auto`
- [x] **0.6.5** Implement preset handlers
  - `GET /api/ptz/monitors/{id}/presets`
  - `POST /api/ptz/monitors/{id}/presets/{preset_id}/goto`
  - `POST /api/ptz/monitors/{id}/presets/{preset_id}/set`
  - `DELETE /api/ptz/monitors/{id}/presets/{preset_id}`
- [ ] **0.6.6** Implement generic command handler (deferred to Phase 2)
  - `POST /api/ptz/monitors/{id}/command` (accepts command name + params)
- [x] **0.6.7** Implement status/info handlers
  - `GET /api/ptz/monitors/{id}/status`
  - `GET /api/ptz/monitors/{id}/capabilities`
  - `GET /api/ptz/protocols`

### 0.7 Service Layer
- [x] **0.7.1** Create `src/service/ptz.rs`
- [x] **0.7.2** Implement `PtzService` with methods for each operation
- [x] **0.7.3** Add validation logic (check capabilities before executing)
- [x] **0.7.4** Add logging for all PTZ operations

### 0.8 Routes
- [x] **0.8.1** Create `src/routes/ptz.rs`
- [x] **0.8.2** Wire all PTZ routes
- [x] **0.8.3** Register PTZ router in `src/routes/mod.rs`

### 0.9 Testing
- [x] **0.9.1** Unit tests for `PtzCapabilities` parsing from DB
- [x] **0.9.2** Unit tests for `ControlAddress` parsing
- [ ] **0.9.3** Integration tests for API endpoints (mock Perl bridge) - deferred
- [ ] **0.9.4** Manual test with real camera via Perl bridge - requires camera

---

## Phase 1: Native ONVIF Implementation

### 1.1 ONVIF Protocol Implementation
- [ ] **1.1.1** Add dependencies: `reqwest`, `quick-xml`, `sha1`, `base64`
- [ ] **1.1.2** Create `src/ptz/protocols/mod.rs` module structure
- [ ] **1.1.3** Create `src/ptz/protocols/onvif.rs`
- [ ] **1.1.4** Implement ONVIF device discovery (GetCapabilities)
- [ ] **1.1.5** Implement WS-Security UsernameToken authentication
  - Nonce generation
  - Password digest: Base64(SHA1(nonce + created + password))
- [ ] **1.1.6** Implement ONVIF PTZ methods
  - `ContinuousMove` (pan, tilt, zoom velocity)
  - `Stop` (with optional pan/tilt/zoom flags)
  - `AbsoluteMove` (position + speed)
  - `RelativeMove` (translation + speed)
  - `GotoPreset` (profile token, preset token)
  - `SetPreset` (profile token, preset name)
  - `RemovePreset` (profile token, preset token)
  - `GetPresets` (profile token)
  - `GetStatus` (profile token)
- [ ] **1.1.7** Parse ONVIF SOAP responses
- [ ] **1.1.8** Handle ONVIF error responses
- [ ] **1.1.9** Support configurable profile token

### 1.2 Factory Integration
- [ ] **1.2.1** Create `OnvifControlFactory` implementing `PtzControlFactory`
- [ ] **1.2.2** Register "onvif" protocol in `PtzRegistry`
- [ ] **1.2.3** Update registry to prefer native Rust over Perl for "onvif"

### 1.3 Testing
- [ ] **1.3.1** Unit tests with mocked SOAP responses
- [ ] **1.3.2** Integration tests with ONVIF camera simulator
- [ ] **1.3.3** Manual test with real ONVIF camera
- [ ] **1.3.4** Performance comparison: Rust vs Perl implementation

---

## Phase 2: Common Brand Protocols

### 2.1 Protocol Audit
- [ ] **2.1.1** Analyze ZoneMinder Perl modules for usage patterns
- [ ] **2.1.2** Survey users for most common camera brands
- [ ] **2.1.3** Prioritize implementation order

### 2.2 Dahua Protocol
- [ ] **2.2.1** Create `src/ptz/protocols/dahua.rs`
- [ ] **2.2.2** Implement HTTP API (`/cgi-bin/ptz.cgi`)
- [ ] **2.2.3** Implement Digest authentication
- [ ] **2.2.4** Support multi-channel devices
- [ ] **2.2.5** Register in factory/registry
- [ ] **2.2.6** Tests with real/simulated camera

### 2.3 HikVision Protocol
- [ ] **2.3.1** Create `src/ptz/protocols/hikvision.rs`
- [ ] **2.3.2** Implement ISAPI HTTP protocol
- [ ] **2.3.3** Determine if ONVIF fallback is sufficient for most HikVision
- [ ] **2.3.4** Register in factory/registry
- [ ] **2.3.5** Tests with real/simulated camera

### 2.4 Reolink Protocol
- [ ] **2.4.1** Create `src/ptz/protocols/reolink.rs`
- [ ] **2.4.2** Consolidate Reolink.pm and Reolink_HTTP.pm approaches
- [ ] **2.4.3** Implement HTTP API
- [ ] **2.4.4** Register in factory/registry
- [ ] **2.4.5** Tests with real/simulated camera

### 2.5 Amcrest Protocol
- [ ] **2.5.1** Create `src/ptz/protocols/amcrest.rs`
- [ ] **2.5.2** Implement HTTP API (similar to Dahua)
- [ ] **2.5.3** Register in factory/registry
- [ ] **2.5.4** Tests with real/simulated camera

---

## Phase 3: Serial Protocols

### 3.1 Serial Port Infrastructure
- [ ] **3.1.1** Add `tokio-serial` dependency
- [ ] **3.1.2** Create serial device configuration parsing
- [ ] **3.1.3** Handle baud rate, parity, stop bits, flow control
- [ ] **3.1.4** Parse `ControlDevice` field from database

### 3.2 Pelco-D Protocol
- [ ] **3.2.1** Create `src/ptz/protocols/pelco_d.rs`
- [ ] **3.2.2** Implement packet construction (sync, address, cmd1, cmd2, data1, data2, checksum)
- [ ] **3.2.3** Implement all Pelco-D commands
- [ ] **3.2.4** Register in factory/registry
- [ ] **3.2.5** Tests with RS-485 adapter

### 3.3 Pelco-P Protocol
- [ ] **3.3.1** Create `src/ptz/protocols/pelco_p.rs`
- [ ] **3.3.2** Implement Pelco-P packet format (differs from Pelco-D)
- [ ] **3.3.3** Register in factory/registry
- [ ] **3.3.4** Tests

### 3.4 Visca Protocol
- [ ] **3.4.1** Create `src/ptz/protocols/visca.rs`
- [ ] **3.4.2** Implement Sony Visca packet format
- [ ] **3.4.3** Handle Visca daisy-chain addressing
- [ ] **3.4.4** Register in factory/registry
- [ ] **3.4.5** Tests

---

## Phase 4: Advanced Features

### 4.1 Preset Management
- [ ] **4.1.1** Create `PtzPresets` table migration (if not exists)
- [ ] **4.1.2** Create preset CRUD repository functions
- [ ] **4.1.3** API endpoints for preset management
- [ ] **4.1.4** Optional: Capture thumbnail at preset position

### 4.2 PTZ Tours/Patrols
- [ ] **4.2.1** Create `PtzTours` and `PtzTourSteps` table migrations
- [ ] **4.2.2** Define tour as sequence of presets with dwell times
- [ ] **4.2.3** Background task service for tour execution
- [ ] **4.2.4** API endpoints: start/stop/pause tours
- [ ] **4.2.5** Tour CRUD operations

### 4.3 Multi-User Coordination
- [ ] **4.3.1** Track active controller per camera (in-memory or DB)
- [ ] **4.3.2** Implement control lock/unlock mechanism
- [ ] **4.3.3** Prevent conflicting commands from multiple users
- [ ] **4.3.4** Optional: Command queue with priority

### 4.4 Rate Limiting
- [ ] **4.4.1** Per-camera command throttling
- [ ] **4.4.2** Configurable limits based on camera capabilities
- [ ] **4.4.3** Reject commands if previous hasn't completed

### 4.5 Position Tracking
- [ ] **4.5.1** Query camera for current position (if supported)
- [ ] **4.5.2** Cache position in memory
- [ ] **4.5.3** API endpoint for current position
- [ ] **4.5.4** Periodic position polling (configurable)

---

## Phase 5: Deprecation & Cleanup

### 5.1 Migration Warnings
- [ ] **5.1.1** Log warnings when Perl fallback is used
- [ ] **5.1.2** Add UI notification for Perl fallback usage
- [ ] **5.1.3** Suggest ONVIF upgrade where applicable

### 5.2 Database Updates
- [ ] **5.2.1** Add `Controls.RustImplementation` column (nullable)
- [ ] **5.2.2** Add `Controls.IsLegacy` boolean flag
- [ ] **5.2.3** Add `Controls.DeprecationWarning` text field

### 5.3 Optional Perl Disable
- [ ] **5.3.1** Configuration flag to disable Perl fallback
- [ ] **5.3.2** Clear error message guiding to native implementation

### 5.4 Documentation
- [ ] **5.4.1** Camera compatibility matrix
- [ ] **5.4.2** Migration guide for Perl → Rust protocols
- [ ] **5.4.3** Troubleshooting guide

---

## Dependencies to Add

```toml
# Cargo.toml additions for PTZ support
tokio = { version = "1", features = ["process"] }  # For Perl bridge
reqwest = { version = "0.12", features = ["json"] }  # For HTTP protocols
quick-xml = "0.36"  # For ONVIF SOAP parsing
sha1 = "0.10"  # For ONVIF auth
base64 = "0.22"  # For ONVIF auth
tokio-serial = "5.4"  # For serial protocols (Phase 3)
```

---

## Notes

- Each phase delivers independent value and can be released incrementally
- Phase 0 provides immediate functionality via Perl bridge
- Native implementations replace Perl one protocol at a time
- Serial protocols (Phase 3) can be deferred if no immediate need
- Advanced features (Phase 4) are optional enhancements
