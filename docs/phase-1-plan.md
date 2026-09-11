# Python Hunter — Phase 1 Plan

Date: 2026-09-09
Phase: 1 — Core Scanner Shell

## Goals
- Rust backend: process discovery, basic kill/approve, blacklist
- Tauri v2 shell: dark tactical UI, process table, status panel
- SQLite: local history of kills/approvals
- MSI-ready build path

## Deliverables
1. `src-tauri/src/scanner.rs` — process enumeration + kill helpers
2. `src-tauri/src/db.rs` — SQLite schema + CRUD for process history
3. `src-tauri/src/blacklist.rs` — blacklist rules engine
4. `src/` — dark tactical UI shell with process table + actions
5. `tauri.conf.json` — MSI build config

## Rust data model
- `Process { pid, name, memory_mb, cmdline, status, timestamp }`
- `BlacklistEntry { name, created_at, reason }`
- `KillRecord { pid, name, memory_mb, method, timestamp }`

## UI layout
- Top: global status + detector toggle
- Left: process table sorted by memory
- Right: detail panel + action buttons
- Bottom: log/output panel

## Definition of done
- App builds to MSI
- Scans running processes on launch
- Shows top memory consumers
- Can approve/kill/blacklist entries
- History persists in SQLite
- UI renders correctly on 1080p+

## Not in scope for Phase 1
- Python script inspection
- Batch policies
- Remote rules
- Licensing/activation
- Auto-update
