# Python Hunter — About

## Product

**Python Hunter** is a native Windows desktop application built by **BrownMediaGroup LLC** — a family-owned business operated by Joe Brown (founder), his son Alex Brown (repairs, teardown, on-camera work), and his daughter Lily Brown (bookkeeping, scheduling, CapCut/video production).

### Purpose

Python Hunter scans your Windows system for running processes, with special focus on Python executables and high-memory consumers. It gives power users, developers, and IT teams precise control to inspect, approve, blacklist, or terminate processes — all offline, with no cloud dependency.

### Open-Core Model

| Tier | Features | License |
|---|---|---|
| **Core** (free) | Process scanning, Python detection, kill, blacklist, history, CSV export, rules import/export | Open-source (MIT-style) |
| **Pro** (paid) | Deep script inspection (AST, modules), batch policies, remote rules, PDF reporting, licensing | Commercial |

### Tech Stack

- **Tauri v2** desktop shell (Rust backend)
- **sysinfo** crate for cross-platform process discovery
- **rusqlite** for local SQLite history (kill records, blacklist, settings)
- **Vite** + vanilla JavaScript frontend (dark tactical UI)
- **WiX/MSI** packaging for Windows deployment

### Brand

Python Hunter inherits the BrownMediaGroup dark tactical aesthetic:
- **Palette:** `#0a0a0a` (black), `#1a1a1a` (dark gray), `#ffffff` (white), `#39FF14` (lime accent), `#FF6B00` (orange), `#ff3333` (red)
- **Typography:** Headings in Montserrat/Bebas Neue, body in Inter, code in JetBrains Mono
- **Design philosophy:** Minimal, functional, keyboard-friendly. No chrome bloat.

### Development Team

| Role | Person |
|---|---|
| **Direction & testing** | Joe Brown |
| **Ops, planning, architecture, docs** | Newton (AI Operations Lead) |
| **Primary builder** | Angel (Rust + Tauri) |

### Status

Phase 1 complete — scanner shell with process table, Python detection, kill, blacklist, and CSV export. Building MSI-ready packaging path.

### Links

- **GitHub:** [RiseUpGit/python-hunter-v1.0-free](https://github.com/RiseUpGit/python-hunter-v1.0-free)
- **Local path:** `D:/BrownMediaGroup-Dev/python-hunter/`
- **Brand assets:** `F:/obsidian/black beauty/BrownMediaGroup/Brand Assets.md`
