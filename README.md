# Python Hunter

**A standalone Windows process scanner for Python-heavy environments.**

> **Built by BrownMediaGroup LLC** — family-owned and operated (Joe Brown, founder; Alex Brown, son — repairs and on-camera; Lily Brown, daughter — bookkeeping and video production). Part of our home-lab tooling suite that ships on our workstations alongside BrainStorm/Sam and Mission Control.

---

## What This Is

Python Hunter is a native Windows desktop app (Tauri v2 + Rust) that scans running processes, highlights Python executables and memory hogs, and gives you precise control to inspect, approve, blacklist, or terminate them — all offline, no cloud sync, no telemetry.

### Core (free, open-core)
- Real-time process enumeration sorted by memory usage
- Python process detection with script path, working directory, venv status, and Python version
- One-click kill or detailed inspection panel
- Blacklist: auto-kill rule for process names
- Kill history with CSV export
- Rules import/export (JSON)
- Dark tactical UI styled after Mission Control and KITT

### Pro (paid, future)
- Deep Python script inspection (AST analysis, imported modules, loaded libraries)
- Batch kill policies (threshold-based, pattern-based)
- Remote rules engine (team-shared blacklists)
- PDF/HTML reporting
- Licensing and activation system

---

## 🎯 Upcoming Updates (in development)

**Approved design slate — no dates set yet, built iteratively with our community:**

1. 📈 **Live Memory Spike Alerts** — notification + in-app banner the moment a process jumps past a threshold; catch the RAM hog before it becomes a problem
2. 📊 **Process Delta Timeline** — per-process memory sparklines (last 5m/15m/1h) + "by growth" sorting to spot leaks and runaway processes at a glance
3. 🔍 **Python Deep-Peek Panel** — expanded inspection: script path, venv, Python version, safety classification ("runs from venv ✅", "system Python ⚠️"), open-in-editor/terminal
4. ⚡ **One-Click Kill Presets** — save named batch rules ("kill all venv python", "everything over 300MB") and apply in one click
5. 🗂️ **Kill Explorer + Reports** — filtered kill history by day/process/method, "RAM reclaimed" stats, PDF report export

*Core stays free and fully featured; Pro adds the deeper inspection, presets, and reporting power.*

---

## Repo Structure

```
python-hunter/
├── src/                    # Frontend (Vite + vanilla JS)
│   ├── index.html          # Tab-based UI shell
│   ├── app.js             # Tauri invoke bridge + process table logic
│   ├── styles.css         # Dark tactical theme (#0a0a0a / #39FF14 palette)
│   └── icons/             # App icons
├── src-tauri/              # Tauri v2 backend (Rust)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── lib.rs          # Tauri builder, invoke handlers, command registry
│   │   ├── db.rs           # SQLite schema + KillRecord/BlacklistEntry/Setting structs
│   │   └── blacklist.rs    # Blacklist rules engine
│   ├── capabilities/       # Tauri v2 capability JSON (MSI-ready)
│   ├── Cargo.toml         # Rust deps: tauri, sysinfo, rusqlite, ureq
│   └── tauri.conf.json    # Build config
├── docs/                   # Plans and reference
│   ├── plan.md            # Full project plan (open-core model)
│   ├── phase-1-plan.md    # Phase 1 scope: scanner shell + core UI
│   └── memory_killer_win11.py  # Origin script (psutil-based, reference)
├── package.json
├── README.md
└── index.html
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| **Desktop shell** | Tauri v2.11.3 |
| **Backend** | Rust (sysinfo, rusqlite, ureq) |
| **Frontend** | Vanilla JavaScript (no frameworks) |
| **Packaging** | MSI via WiX toolset |
| **Theme** | Dark tactical — BrownMediaGroup palette (`#0a0a0a`, `#39FF14`, `#FF6B00`, `#ff3333`) |
| **Fonts** | JetBrains Mono / Fira Code (headings), Inter (body) |

---

## Development

### Prerequisites
- Rust 1.77.2+
- Node.js 18+
- Tauri CLI (`cargo install create-tauri-app`)
- WiX 3.5+ (for MSI builds)

### Local dev
```bash
npm install
npm run dev
# In another terminal:
cargo tauri dev
```

### Production build (MSI)
```bash
npm run build && cargo tauri build -b msi
```

The `build_token` is stamped in `build_version.txt` to prevent watchdog flap during rebuilds.

---

## Brand

Python Hunter is part of the **BrownMediaGroup** product family. Brand palette and typography specs are in our [Brand Assets](https://ppcrepair.com) documentation.

- **Black:** `#0a0a0a` (backgrounds)
- **Dark Gray:** `#1a1a1a` (cards, overlays)
- **Lime:** `#39FF14` (accents, CTAs)
- **Orange:** `#FF6B00` (secondary alerts)
- **Red:** `#ff3333` (errors, live indicators)

---

## License

**Open-core:** Core functionality is free and open-source. Pro features will be commercially licensed.

## Links

- **Website:** [ppcrepair.com/tools/python-hunter](https://ppcrepair.com/tools/python-hunter)
- **Pro version:** [ppcrepair.com/tools/python-hunter-pro](https://ppcrepair.com/tools/python-hunter-pro)
- **BrownMediaGroup:** [ppcrepair.com](https://ppcrepair.com)

© 2026 BrownMediaGroup LLC — all rights reserved.
