# Multi-Display & Multi-Device Workspace Architecture

Copyright (c) 2026 Timothy Charles Holborn. All rights reserved.

## 1. Vision

A user may work across multiple machines (desktop, laptop, mobile, tablet)
and multiple displays on a single machine. The workspace is **distributed**:
some containers live on one device, others on another. A phone can serve as
a remote control surface; a tablet can display a control panel; a desktop
runs the heavy compute containers.

The user has a **unified cryptographic identity chain** (DID + key chain)
that authenticates all devices. Devices discover each other through the
socially defined networking layer. Workspace state syncs through
cryptographically signed deltas — no central server required.

### 1.1 Use Cases

| Scenario | Description |
|----------|-------------|
| **Multi-monitor desktop** | User has 2-3 displays. Manifold canvas spans all monitors. Containers are placed on specific displays. |
| **Desktop + laptop** | User starts work on desktop, continues on laptop. Workspace state syncs via crypto chain. |
| **Phone as remote** | Phone displays a compact control surface (play/pause, navigate manifolds, trigger commands). |
| **Tablet as control surface** | Tablet displays dock panels (inspector, property sheet, toolbox) while desktop shows canvas. |
| **Distributed compute** | Heavy NLP processing runs on desktop; results stream to laptop for review. |
| **Mobile quick check** | User opens mobile to check status, approve a workflow, or sign a transaction. |

## 2. Architecture

### 2.1 Device Graph

Each device has a `DeviceProfile`:

```rust
struct DeviceProfile {
    device_id: String,          // did:qualia:device:<uuid>
    device_type: DeviceType,    // Desktop, Laptop, Tablet, Phone, Watch
    label: String,              // "Tim's Desktop", "Pixel 9"
    capabilities: DeviceCaps,   // screen, compute, input, sensors
    displays: Vec<DisplayInfo>, // physical displays
    crypto_key_id: String,      // key in the unified chain
    last_seen: i64,             // unix timestamp
    status: DeviceStatus,       // Online, Offline, Paired, Suspended
}
```

Devices form a **device graph** — a P2P mesh authenticated by the user's
crypto chain. Each device holds a signed device manifest listing all known
devices. New devices are paired by scanning a QR code or entering a
one-time code displayed on an already-paired device.

### 2.2 Display Layout

Each device reports its physical displays:

```rust
struct DisplayInfo {
    display_id: String,
    label: String,              // "Display 1", "Left Monitor"
    width_px: u32,
    height_px: u32,
    scale_factor: f32,          // HiDPI
    is_primary: bool,
    position: (i32, i32),       // virtual desktop position
}
```

The manifold canvas can span multiple displays. Each container has an
optional `display_id` field — if set, the container is rendered on that
display. The workspace manager computes the virtual coordinate space
spanning all displays.

### 2.3 Workspace State & Sync

```rust
struct WorkspaceState {
    workspace_id: String,
    owner_did: String,          // did:qualia:timothy_charles_holborn
    manifolds: Vec<ManifoldSeed>,
    active_manifold: String,
    container_overrides: Vec<ContainerOverride>,
    device_assignments: Vec<DeviceAssignment>,
    version: u64,               // monotonic version counter
    last_modified: i64,
    signature: String,          // crypto chain signature
}
```

**Sync protocol:**
1. Each device maintains a local copy of `WorkspaceState`.
2. When the user modifies the workspace (move container, add container,
   switch manifold), a **delta** is created and signed with the device's
   crypto key.
3. Deltas are broadcast to all online devices via the P2P mesh.
4. Receiving devices verify the signature, apply the delta, and update
   their local workspace.
5. Conflict resolution: last-writer-wins by version number. If two devices
   modify the same container simultaneously, the higher version wins and
   the loser is notified.

### 2.4 Device Roles

Devices can be assigned **roles** that determine what they display:

| Role | Description | Typical device |
|------|-------------|----------------|
| **Primary** | Full manifold canvas + all containers | Desktop |
| **Secondary** | Extended canvas — additional containers on a second display | Desktop monitor 2 |
| **Remote** | Compact control surface — manifold switcher, command triggers, status | Phone |
| **Control Surface** | Dock panels only — inspector, property sheet, toolbox | Tablet |
| **Compute** | Headless — runs NLP/processing, streams results | Desktop (background) |
| **Display Only** | Read-only canvas view — presentations, monitoring | TV, projector |

### 2.5 Container Placement

Containers can be assigned to specific devices:

```rust
struct DeviceAssignment {
    container_id: String,
    device_id: String,
    role: DeviceRole,
    display_id: Option<String>,  // specific display on the device
}
```

When a device receives a container assignment, it renders that container
in its local viewport. If the device is offline, the container is queued
and rendered when it comes online.

### 2.6 Cryptographic Identity Chain

The user's crypto chain is the root of trust for the device graph:

1. User has a master DID: `did:qualia:timothy_charles_holborn`
2. Each device has a device DID: `did:qualia:device:<uuid>`
3. The master DID signs a **device attestation** for each paired device.
4. Device attestations are stored in the crypto chain and can be verified
   by any paired device.
5. Workspace state changes are signed by the device's key and chained to
   the user's master DID.

No central server. No cloud account. The crypto chain IS the account.

## 3. Implementation Plan

### 3.1 P0 — Core Data Structures + Device Manifold

- `DeviceProfile`, `DeviceType`, `DeviceCaps`, `DisplayInfo` structs
- `WorkspaceState` with serde serialization
- `DeviceAssignment` and `DeviceRole` enums
- `DeviceGraph` for P2P device mesh
- New **Devices** manifold with containers:
  - `device_manager` — list paired devices, pair new device, revoke device
  - `display_layout` — visual display arrangement, assign containers to displays
  - `workspace_sync` — sync status, version history, conflict resolution
  - `remote_control` — compact control surface for phone/remote devices
  - `device_role_assigner` — assign roles to devices

### 3.2 P1 — Live Sync + P2P Transport

- WebRTC data channels for device-to-device communication
- Delta-based sync protocol with CRDT fallback
- QR code pairing flow
- Crypto chain signing and verification
- Offline queue and replay

### 3.3 P2 — Advanced

- Container migration (drag container from one device to another)
- Compute delegation (send NLP task to desktop, receive results on laptop)
- Multi-user collaboration (multiple crypto chains, shared workspaces)
- Ambient display mode (always-on status, notifications)

## 4. File Structure

```
src/tool_chest/core/
    device.rs          — DeviceProfile, DeviceType, DeviceCaps, DisplayInfo
    workspace.rs       — WorkspaceState, ContainerOverride, DeviceAssignment
    device_graph.rs    — DeviceGraph, pairing, discovery
    sync.rs            — Delta, SyncProtocol, conflict resolution

src/tool_chest/manifolds/
    devices.rs         — Devices manifold seed

src/browser/device_views/
    mod.rs
    device_manager.rs       — paired devices list + pairing UI
    display_layout.rs       — multi-monitor display arrangement
    workspace_sync.rs       — sync status + version history
    remote_control.rs       — compact remote control surface
    device_role_assigner.rs — role assignment per device
```

## 5. Integration Points

- **Crypto chain:** `did:qualia:` identity + key management (QualiaDB)
- **Social networking layer:** device discovery via social graph
- **ManifoldSeed:** already serde-serializable — workspace state extends it
- **Container system:** containers gain optional `display_id` + `device_id`
- **Intent bus:** device-to-device commands via intent dispatch
- **Provenance:** all workspace changes are provenance-tracked

## 6. Honesty Labels

All P0 containers use `present` honesty — mock data, no live sync.
P1 will upgrade to `partial` with WebRTC transport.
P2 will upgrade to `live` with full crypto chain verification.
