# Python Hunter — Project Plan

**Project:** Python Hunter  
**Stack:** Tauri v2 + Rust backend + SQLite + dark tactical web frontend  
**Model:** Open-core commercial  
**Repo:** `D:/BrownMediaGroup-Dev/python-hunter/`  
**Date:** 2026-09-09

## What this is
A standalone Windows process scanner focused on Python executables and memory hogs. It scans the PC, identifies running Python processes and other high-memory consumers, and gives the user control to inspect, approve, blacklist, or terminate them. Core is free; Pro adds deeper inspection, batch policies, and remote rules.

## Core vs Pro split
- **Core (free):** Rust-only scanning, basic kill/approve, process list, blacklist
- **Pro (paid):** Python/script inspection, batch policies, remote rules, reporting, team policies

## Tech stack
- Tauri v2 desktop shell
- Rust backend for process discovery and control
- SQLite for local history/state
- Dark tactical UI styled after Mission Control/KITT
- MSI packaging via WiX/MSI workflow

## Commercial framing
- Open-core: core functionality free, pro features paid
- Target users: power users, developers, IT shops managing Python-heavy environments
- Differentiator: focused Python awareness + tactical dark UI + offline-first local tooling

## Development phases
1. **Phase 1:** Rust process scanner + basic Tauri shell + core UI
2. **Phase 2:** Blacklist/approve/kill workflow + CSV/log export
3. **Phase 3:** Pro feature gates + Python script inspection
4. **Phase 4:** Batch policies + remote rules + commercial packaging
5. **Phase 5:** Marketing site, docs, and release

## Team
- Joe: direction, testing, commercial validation
- Newton: ops, planning, architecture, docs
- Angel: primary builder

## Next actions
1. Create full architecture diagram
2. Define Rust data model for processes/history/rules
3. Create Tauri project scaffold
4. Build core scanner backend first
