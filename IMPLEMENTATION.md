# IMPLEMENTATION.md

## Master Engineering Specification

**Project:** Spotlight for Windows

**Status:** Pre-Implementation

**Purpose:**
This document is the single source of truth for architecture,
implementation, performance, security, testing, packaging,
and production release.

An AI coding agent — Claude Code, Cursor, Codex, Gemini CLI, or any
equivalent — must read this document in full before writing a single
line of code. It defines every major technical decision, every
engineering constraint, and every acceptance criterion for the project.

No feature may be implemented in a way that violates the constraints
defined here. If a constraint must change, this document must be
updated first and reviewed before the change is applied.

---

## Table of Contents

1. [Product Vision](#1-product-vision)
2. [Core Requirements](#2-core-requirements)
3. [Architectural Principle](#3-architectural-principle)
4. [UI Lifecycle](#4-ui-lifecycle)
5. [Technology Stack](#5-technology-stack)
6. [Repository Structure](#6-repository-structure)
7. [Rust Architecture](#7-rust-architecture)
8. [Search Architecture](#8-search-architecture)
9. [Ranking Algorithm](#9-ranking-algorithm)
10. [Application Indexing](#10-application-indexing)
11. [File Indexing](#11-file-indexing)
12. [Incremental Indexing](#12-incremental-indexing)
13. [SQLite Design](#13-sqlite-design)
14. [IPC Architecture](#14-ipc-architecture)
15. [Keyboard System](#15-keyboard-system)
16. [Global Hotkey](#16-global-hotkey)
17. [Window Management](#17-window-management)
18. [UI/UX Direction](#18-uiux-direction)
19. [React Performance](#19-react-performance)
20. [Memory Optimization](#20-memory-optimization)
21. [CPU Optimization](#21-cpu-optimization)
22. [Offline-First](#22-offline-first)
23. [Calculator](#23-calculator)
24. [Command System](#24-command-system)
25. [Web Search](#25-web-search)
26. [History and Personalization](#26-history-and-personalization)
27. [Settings](#27-settings)
28. [First Run / Onboarding](#28-first-run--onboarding)
29. [System Tray](#29-system-tray)
30. [Startup With Windows](#30-startup-with-windows)
31. [Single Instance](#31-single-instance)
32. [Error Handling](#32-error-handling)
33. [Logging](#33-logging)
34. [Security](#34-security)
35. [Testing Strategy](#35-testing-strategy)
36. [Performance Benchmarking](#36-performance-benchmarking)
37. [Large Dataset Testing](#37-large-dataset-testing)
38. [Dependency Strategy](#38-dependency-strategy)
39. [Build Strategy](#39-build-strategy)
40. [Installer](#40-installer)
41. [Auto Updates](#41-auto-updates)
42. [Code Signing](#42-code-signing)
43. [CI/CD](#43-cicd)
44. [Versioning](#44-versioning)
45. [Git Strategy](#45-git-strategy)
46. [AI Coding Agent Instructions](#46-ai-coding-agent-instructions)
47. [Development Phases](#47-development-phases)
48. [Definition of Done](#48-definition-of-done)
49. [Feature Roadmap](#49-feature-roadmap)
50. [Product Differentiation](#50-product-differentiation)
51. [Pricing / Business Model](#51-pricing--business-model)
52. [Product Metrics](#52-product-metrics)
53. [Documentation](#53-documentation)
54. [README Requirements](#54-readme-requirements)
55. [Architecture Diagrams](#55-architecture-diagrams)
56. [Data Flow](#56-data-flow)
57. [Failure Scenarios](#57-failure-scenarios)
58. [Performance Budget](#58-performance-budget)
59. [Performance Regression Policy](#59-performance-regression-policy)
60. [Code Quality Rules](#60-code-quality-rules)
61. [AI Implementation Process](#61-ai-implementation-process)
62. [Avoid Overengineering](#62-avoid-overengineering)
63. [Production Quality](#63-production-quality)
64. [Final Principle](#64-final-principle)

---

## 1. Product Vision

Spotlight for Windows is a **Windows-first, keyboard-first, offline-first**
application launcher and search interface.

### How it works

1. The user installs the application.
2. The user completes a small first-run configuration wizard.
3. The user optionally opts in to starting with Windows.
4. A very lightweight Rust background process starts with Windows.
5. The user does **not** need to manually open the application.
6. Pressing **`Alt + Space`** opens the launcher instantly.
7. The user types a query.
8. The launcher searches local applications, files, folders, commands, and other providers.
9. Results appear immediately.
10. The user navigates using keyboard arrow keys.
11. Enter launches the selected result.
12. Escape hides the launcher.
13. The background core remains alive, idle, and extremely lightweight.

### Fundamental philosophy

> **Always ready. Never in the way.**

The launcher must feel:

| Quality | Meaning |
|---|---|
| **Instant** | Opens in under 150ms from hotkey press |
| **Minimal** | Only what is needed, nothing decorative |
| **Polished** | Every pixel and interaction is intentional |
| **Quiet** | Invisible until needed |
| **Private** | All data stays local |
| **Offline-first** | Works with no internet |
| **Keyboard-first** | Entire workflow achievable without a mouse |
| **Lightweight** | Nearly invisible CPU and RAM while idle |
| **Reliable** | Survives reboots, sleep, DPI changes, and multi-monitor configurations |

This product competes with **PowerToys Run**, **Flow Launcher**, **ueli**,
and **Wox** but differentiates itself through a relentlessly refined
Windows-native experience, extreme performance discipline, and a clean,
purposeful UI that feels like it belongs in Windows rather than running on top of it.

---

## 2. Core Requirements

These requirements are non-negotiable. No implementation decision may
violate them without explicit revision and approval of this document.

### Platform

- Windows 10 (1903+) and Windows 11
- No Mac, no Linux target in V1
- No cross-platform abstraction that degrades Windows behavior

### Technology

- **Backend/core:** Rust (stable toolchain)
- **Desktop shell:** Tauri 2
- **UI framework:** React 18+
- **UI language:** TypeScript (strict mode)
- **Database:** SQLite (via `rusqlite` with WAL mode)
- **Search:** Custom Rust search engine + SQLite FTS5

### Features (V1)

- Global hotkey (`Alt + Space`, configurable)
- System tray presence
- Windows startup integration
- Local application indexing
- Local file/folder indexing
- Incremental filesystem watching
- Usage-based ranking
- Keyboard navigation
- Calculator provider
- Application launching
- File launching
- Folder launching
- Command execution (safe, built-in only in V1)
- Optional web search provider
- Settings UI
- Onboarding flow
- Update mechanism
- Production-grade installer

### Performance requirements

All performance targets are **engineering goals** to be measured on real
builds. They are not marketing claims.

| Metric | Target | Measurement method |
|---|---|---|
| Idle CPU | < 0.1% average | Windows Task Manager / Process Hacker over 10 min |
| Idle RAM | < 50 MB | Private Working Set |
| Hotkey to visible UI | < 150 ms | Timestamp diff: hotkey event to first paint |
| First character search | < 50 ms | IPC round-trip timing |
| Full search response | < 100 ms (10k indexed items) | End-to-end timing |
| Cold startup readiness | < 2 s from Windows login | Measured from user session start |
| Indexing CPU (initial) | < 25% sustained | Process Monitor during initial scan |
| Indexing RAM (initial) | < 150 MB peak | Private Working Set |
| Bundle size | < 10 MB JS/CSS | Vite build output |
| Installer size | < 30 MB | Signed installer artifact |

Regression thresholds: Any metric exceeding 125% of its target triggers
mandatory review before the change may be merged. See Section 59.

---

## 3. Architectural Principle

The application is divided into two fundamentally distinct responsibilities.

### Background Core (Rust)

The Rust process is responsible for **everything that persists** when the
UI is not visible:

- Process lifecycle management
- Global hotkey registration and handling
- Application discovery and indexing
- File and folder indexing
- Filesystem watching (incremental updates)
- Search engine and query processing
- Result ranking
- SQLite database management
- Usage history
- Application/file launching
- Settings persistence
- Windows startup integration
- System tray management
- Update checking
- IPC API exposed to the UI layer

The Rust process must remain **alive and nearly invisible** while idle.
It must not poll unnecessarily, run background timers that consume CPU,
or maintain resources that are not actively needed.

### UI Layer (React/TypeScript)

The React/TypeScript UI is responsible for **only the visual layer**:

- Launcher window rendering
- Search input field
- Result list rendering
- Keyboard navigation state
- Visual animations and transitions
- Settings interface
- Onboarding screens
- Accessibility attributes

**Critical constraint:**
Search logic, indexing logic, ranking logic, history tracking, and
launching logic must **never** live in React. They belong in Rust.
React sends queries and receives result lists. It does not interpret them.

This boundary must be maintained rigorously. Any future AI agent or
developer that moves search logic into React is violating the architecture.

---

## 4. UI Lifecycle

```
Windows startup
      |
      v
Rust core starts (background process)
      |
      v
Initialize minimum services:
  - Open SQLite connection
  - Load settings
  - Register global hotkey
  - Start tray icon
  - Begin lazy background indexing if index is stale
      |
      v
IDLE (nearly zero CPU, ~30-50 MB RAM)
      |
      v [Alt + Space]
Tauri window -> show (if hidden) or create (if first open)
      |
      v
Window appears centered, focused
React mounts / becomes visible
Search input receives focus immediately
      |
      v
User types query
      |
      v
React debounces input (50-80 ms)
IPC -> Rust Search Engine
      |
      v
Rust: parse query -> providers -> ranking -> top N results
IPC -> React
      |
      v
React renders results (no full re-render of unchanged items)
      |
      v
User presses Enter -> launch -> window hides
User presses Escape -> window hides
      |
      v
IDLE (window is hidden, not destroyed)
Rust core remains alive
```

### Window lifecycle decision

The window should be **hidden, not destroyed**, between uses. Creating a
Tauri WebView from scratch on every hotkey press introduces unacceptable
latency on many systems.

However, the tradeoff is RAM while idle:
- A hidden WebView consumes additional memory.
- A destroyed WebView consumes none.

**Decision:** Keep the window hidden. Measure actual RAM impact. If idle
RAM exceeds the 50 MB budget including the hidden WebView, evaluate
whether partial resource release (suspending JS execution) is feasible
via Tauri APIs. Do not prematurely optimize this — measure first.

---

## 5. Technology Stack

### Desktop Shell: Tauri 2

**Justification:**
- Rust-native, correct integration surface
- Smallest possible binary footprint vs Electron
- Native WebView (WebView2 on Windows) — no bundled Chromium
- Strong IPC boundary between Rust and UI
- Active development, Windows-first in practice
- Supports transparent windows, system tray, global hotkey via plugins

**Key Tauri 2 crates and plugins:**
- `tauri` 2.x
- `tauri-plugin-global-shortcut`
- `tauri-plugin-single-instance`
- `tauri-plugin-updater`
- `tauri-plugin-shell` (restricted capabilities)
- `tauri-plugin-fs` (restricted capabilities)
- `tauri-plugin-notification` (optional)

### Core: Rust

- Stable toolchain (track latest stable)
- `tokio` for async runtime (single-threaded where sufficient, multi-threaded where needed)
- `rusqlite` for SQLite with bundled feature
- `serde` / `serde_json` for IPC serialization
- `notify` for filesystem watching
- `windows` crate for Win32 APIs where needed
- `tracing` / `tracing-subscriber` for structured logging
- `anyhow` for application-level error propagation
- `thiserror` for typed module-level errors

### UI: React + TypeScript

- React 18 with concurrent features
- TypeScript 5.x strict mode
- Vite as bundler
- CSS Modules or plain CSS (no Tailwind unless explicitly introduced)
- `zustand` for global UI state (minimal)
- No Redux, no MobX — state should be local where possible

### Database: SQLite

- WAL mode enabled at connection time
- `rusqlite` with `bundled` feature (no system SQLite dependency)
- FTS5 extension for full-text search
- Migrations managed in `src-tauri/migrations/`

### Search: Custom Rust engine

- SQLite FTS5 for text matching
- Custom scoring layer on top (not pure FTS5 rank)
- Fuzzy matching via Levenshtein distance for short queries
- Prefix matching as primary fast path
- Token matching for multi-word queries

### Windows integration

- Win32 APIs via the `windows` crate (not wrappers where precision matters)
- Shell link COM APIs for `.lnk` parsing
- `ReadDirectoryChangesW` or equivalent via the `notify` crate for filesystem events
- Registry access for startup key management
- DPI-aware window configuration
- Taskbar integration via tray

---

## 6. Repository Structure

```
spotlight-for-windows/
|
+-- src/                          # React / TypeScript UI
|   +-- components/               # Shared, reusable UI components
|   |   +-- ResultItem/
|   |   +-- SearchInput/
|   |   +-- ResultList/
|   |   +-- Icon/
|   +-- features/                 # Feature-specific UI modules
|   |   +-- launcher/             # Main launcher window UI
|   |   +-- settings/             # Settings UI
|   |   +-- onboarding/           # Onboarding wizard
|   +-- hooks/                    # Custom React hooks
|   |   +-- useSearch.ts
|   |   +-- useKeyboardNav.ts
|   |   +-- useSettings.ts
|   +-- lib/                      # Utilities, IPC wrappers, helpers
|   |   +-- ipc.ts                # Typed wrappers around Tauri IPC calls
|   |   +-- format.ts
|   |   +-- platform.ts
|   +-- stores/                   # Zustand stores
|   |   +-- launcherStore.ts
|   |   +-- settingsStore.ts
|   +-- types/                    # Shared TypeScript types
|   |   +-- results.ts
|   |   +-- settings.ts
|   |   +-- ipc.ts
|   +-- styles/                   # Global CSS, CSS variables, reset
|   |   +-- global.css
|   |   +-- variables.css
|   |   +-- animations.css
|   +-- App.tsx                   # Root component, route switching
|   +-- main.tsx                  # Entry point
|
+-- src-tauri/                    # Rust / Tauri backend
|   +-- src/
|   |   +-- core/                 # Application lifecycle and state
|   |   |   +-- app.rs
|   |   |   +-- lifecycle.rs
|   |   |   +-- process.rs
|   |   |   +-- state.rs
|   |   +-- hotkey/               # Global hotkey management
|   |   |   +-- manager.rs
|   |   |   +-- shortcuts.rs
|   |   +-- search/               # Search engine
|   |   |   +-- engine.rs
|   |   |   +-- parser.rs
|   |   |   +-- ranking.rs
|   |   |   +-- query.rs
|   |   |   +-- providers/
|   |   |       +-- mod.rs
|   |   |       +-- apps.rs
|   |   |       +-- files.rs
|   |   |       +-- folders.rs
|   |   |       +-- commands.rs
|   |   |       +-- calculator.rs
|   |   |       +-- web.rs
|   |   +-- indexer/              # Indexing subsystem
|   |   |   +-- manager.rs
|   |   |   +-- apps.rs
|   |   |   +-- files.rs
|   |   |   +-- folders.rs
|   |   |   +-- watcher.rs
|   |   +-- database/             # SQLite layer
|   |   |   +-- connection.rs
|   |   |   +-- migrations.rs
|   |   |   +-- apps.rs
|   |   |   +-- files.rs
|   |   |   +-- history.rs
|   |   |   +-- usage.rs
|   |   |   +-- settings.rs
|   |   +-- launcher/             # Result launching
|   |   |   +-- application.rs
|   |   |   +-- file.rs
|   |   |   +-- folder.rs
|   |   |   +-- command.rs
|   |   +-- windows/              # Windows-specific integration
|   |   |   +-- startup.rs
|   |   |   +-- window.rs
|   |   |   +-- focus.rs
|   |   |   +-- integration.rs
|   |   +-- tray/
|   |   |   +-- manager.rs
|   |   +-- settings/
|   |   |   +-- manager.rs
|   |   |   +-- models.rs
|   |   +-- commands.rs           # Tauri IPC command handlers
|   |   +-- error.rs              # Error types
|   |   +-- main.rs               # Entry point
|   |
|   +-- migrations/               # SQLite migration files (numbered)
|   |   +-- 0001_initial.sql
|   +-- capabilities/             # Tauri 2 capability definitions
|   |   +-- default.json
|   +-- icons/                    # App icons (various sizes)
|   +-- tauri.conf.json           # Tauri configuration
|
+-- tests/                        # Integration and E2E tests
|   +-- integration/
|   +-- e2e/
|
+-- benchmarks/                   # Performance benchmark suites
|   +-- search_bench.rs
|   +-- indexer_bench.rs
|
+-- scripts/                      # Build, release, utility scripts
|   +-- build.ps1
|   +-- sign.ps1
|   +-- release.ps1
|
+-- docs/                         # Engineering documentation
|   +-- architecture.md
|   +-- search.md
|   +-- decisions/                # Architecture Decision Records (ADRs)
|
+-- assets/                       # Static assets (screenshots, etc.)
|
+-- installer/                    # Installer configuration
|   +-- wix/                      # WiX Toolset config (or NSIS)
|
+-- .github/
|   +-- workflows/
|       +-- ci.yml
|       +-- release.yml
|
+-- .cargo/
|   +-- config.toml               # Cargo profile settings
|
+-- Cargo.toml                    # Workspace Cargo manifest
+-- package.json
+-- tsconfig.json
+-- vite.config.ts
+-- eslint.config.js
+-- .prettierrc
+-- README.md
+-- CONTRIBUTING.md
+-- SECURITY.md
+-- CHANGELOG.md
+-- LICENSE
+-- IMPLEMENTATION.md             # This file
```

### Directory responsibilities

| Directory | Purpose |
|---|---|
| `src/components/` | Reusable, stateless or minimally stateful UI primitives |
| `src/features/` | Self-contained feature modules with their own state and layout |
| `src/hooks/` | Custom hooks encapsulating IPC calls and UI behavior |
| `src/lib/` | Pure utilities, IPC typed wrappers, formatters |
| `src/stores/` | Global Zustand stores (keep minimal) |
| `src/types/` | Shared TypeScript type definitions matching Rust structs |
| `src/styles/` | Design tokens, CSS reset, global animation utilities |
| `src-tauri/src/core/` | Application startup, shutdown, and global state management |
| `src-tauri/src/hotkey/` | Global hotkey registration and change handling |
| `src-tauri/src/search/` | Query parsing, providers, ranking |
| `src-tauri/src/indexer/` | Background indexing, watcher integration |
| `src-tauri/src/database/` | All SQLite interaction (no raw SQL outside this module) |
| `src-tauri/src/launcher/` | Opening/executing results |
| `src-tauri/src/windows/` | Windows-native APIs: focus, DPI, startup |
| `src-tauri/src/tray/` | System tray icon and menu |
| `src-tauri/src/settings/` | Settings model, persistence, defaults |
| `migrations/` | Versioned SQL migration files, applied at startup |
| `benchmarks/` | Criterion-based Rust benchmarks |
| `scripts/` | PowerShell scripts for local build/sign/release |
| `installer/` | Installer toolchain configuration |
| `.github/workflows/` | CI and release pipelines |

---

## 7. Rust Architecture

### Overview

The Rust backend is a single process. It uses `tokio` as the async
runtime for I/O-bound work (database, filesystem watchers, IPC). CPU-bound
work (search, ranking, heavy indexing passes) runs on blocking thread pools
to avoid starving the async executor.

### State management

A single `AppState` struct is stored in Tauri's managed state:

```rust
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,       // SQLite connection
    pub settings: Arc<RwLock<Settings>>, // Runtime settings
    pub index_status: Arc<RwLock<IndexStatus>>,
    pub search_cancel: Arc<AtomicBool>,  // Cancellation flag
}
```

- `Arc<Mutex<Connection>>`: SQLite is single-writer. A Mutex is correct
  and sufficient. Do not use a connection pool for write operations.
- `Arc<RwLock<Settings>>`: Settings are read frequently, written rarely.
- All state is initialized in `main.rs` before the Tauri app starts.

### Module responsibilities

#### `core/`

- `app.rs`: Tauri app builder, plugin registration, state initialization
- `lifecycle.rs`: Startup sequence, graceful shutdown, signal handling
- `process.rs`: Single-instance check, mutex-based duplicate detection
- `state.rs`: `AppState` definition and initialization helpers

#### `hotkey/`

- `manager.rs`: Registers/unregisters shortcuts via `tauri-plugin-global-shortcut`. Handles conflict detection. Exposes change API.
- `shortcuts.rs`: Shortcut parsing, validation, default definitions

#### `search/`

- `engine.rs`: Top-level search coordinator. Accepts a `SearchQuery`, fans out to enabled providers, collects candidates, passes to ranker, returns top N.
- `parser.rs`: Tokenizes and interprets the raw query string. Detects special modes (calculator, command prefix, etc.).
- `ranking.rs`: Implements the scoring model defined in Section 9.
- `query.rs`: `SearchQuery` and `SearchResult` types used across the subsystem.
- `providers/apps.rs`: Queries the `applications` FTS5 table.
- `providers/files.rs`: Queries the `files` FTS5 table.
- `providers/folders.rs`: Queries the `folders` FTS5 table.
- `providers/commands.rs`: Matches against built-in command registry.
- `providers/calculator.rs`: Detects and evaluates mathematical expressions.
- `providers/web.rs`: Generates a web search result (URL only, no network call until user selects it).

#### `indexer/`

- `manager.rs`: Orchestrates initial indexing and incremental updates. Exposes pause/resume API to tray and settings.
- `apps.rs`: Discovers installed applications. See Section 10.
- `files.rs`: Scans user-selected directories. See Section 11.
- `folders.rs`: Indexes folder entries separately from files.
- `watcher.rs`: Wraps the `notify` crate. Converts filesystem events into debounced index update operations.

#### `database/`

- `connection.rs`: Opens the SQLite connection, enables WAL, sets pragmas, returns `Connection`.
- `migrations.rs`: Reads numbered SQL files from `migrations/`, applies unapplied ones in order.
- `apps.rs`: CRUD operations for the `applications` table.
- `files.rs`: CRUD operations for the `files` table.
- `history.rs`: Writes and reads launch history entries.
- `usage.rs`: Aggregates usage counts and recency for ranking.
- `settings.rs`: Reads/writes settings rows.

**Rule:** No SQL string literals may exist outside the `database/` module.

#### `launcher/`

- `application.rs`: Launches `.exe` or `.lnk` target with optional arguments.
- `file.rs`: Opens a file using `ShellExecuteW` (respects user's default app).
- `folder.rs`: Opens a folder in Explorer.
- `command.rs`: Executes a built-in safe command (e.g., lock screen).

**Rule:** No arbitrary shell string execution. Every launch path is validated.

#### `windows/`

- `startup.rs`: Reads/writes the `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` registry key.
- `window.rs`: Window show/hide, position, always-on-top, DPI-aware sizing.
- `focus.rs`: `SetForegroundWindow`, focus management after launch.
- `integration.rs`: DPI awareness declaration, monitor enumeration, multi-monitor position logic.

#### `tray/`

- `manager.rs`: Creates and manages the tray icon and context menu. Handles all menu item callbacks.

#### `settings/`

- `manager.rs`: Loads settings from DB on startup, merges with defaults, provides runtime access.
- `models.rs`: `Settings` struct with `serde` derives.

### Concurrency model

| Concern | Thread model |
|---|---|
| IPC handlers | `tokio` async tasks |
| SQLite writes | Single `Mutex<Connection>` (serialized) |
| Filesystem watcher | Dedicated `std::thread` (notify requirement) |
| Initial indexing | `tokio::task::spawn_blocking` |
| Search | `tokio::task::spawn_blocking` for DB queries |
| Hotkey callback | Tauri plugin's thread, posts message to app |

### Error handling

- Use `thiserror` to define typed errors per module: `DatabaseError`, `IndexerError`, `SearchError`, `HotkeyError`, `LauncherError`
- Use `anyhow::Result` at the IPC command handler boundary
- IPC command errors serialize to a consistent JSON error envelope:
  `{ "error": { "code": "DATABASE_UNAVAILABLE", "message": "..." } }`
- Never panic in production code paths. Use `expect` only for invariants that truly cannot fail.
- Log all errors with `tracing::error!` with context fields.

### Shutdown behavior

On application quit:
1. Unregister global hotkey
2. Signal watcher thread to stop
3. Drain pending index queue
4. Flush database WAL
5. Close SQLite connection
6. Remove tray icon
7. Exit process

Graceful shutdown must complete within 3 seconds. After 3 seconds, force exit.

---

## 8. Search Architecture

### Pipeline

```
User types query (React)
         |
         v
Debounce: 50-80 ms (React hook)
         |
         v
IPC: invoke("search", { query, timestamp })
         |
         v
Rust: commands::search()
         |
         v
SearchEngine::execute(query)
         |
         v
QueryParser::parse(raw)
  +-- Detect mode:
  |    - Empty -> return recent/frequent
  |    - "=" prefix -> calculator
  |    - ">" prefix -> command mode
  |    - URL-like -> web result
  |    - General -> multi-provider search
         |
         v
Fan out to enabled providers (parallel where safe):
  +-- AppsProvider     -> FTS5 query on `applications`
  +-- FilesProvider    -> FTS5 query on `files`
  +-- FoldersProvider  -> FTS5 query on `folders`
  +-- CommandsProvider -> in-memory match against built-ins
  +-- CalculatorProvider -> expression eval (no DB)
  +-- WebProvider      -> generate URL result (no network)
         |
         v
Collect candidates (Vec<SearchCandidate>)
         |
         v
Ranking::score_all(candidates, query, usage_map)
         |
         v
Sort descending by score
Take top N (configurable, default: 12)
         |
         v
IPC response: Vec<SearchResult>
         |
         v
React renders result list
```

### Debounce strategy

- Debounce of 50-80 ms applied in React before IPC call.
- On each new character, cancel the previous pending search and set a server-side cancellation flag.
- Searches must not queue up — if a new search arrives while one is running, cancel the in-flight search.

### Query behavior table

| Condition | Behavior |
|---|---|
| Empty query | Show recent launches (up to 8) |
| 1 character | Search apps only (fast path) |
| 2+ characters | Full multi-provider search |
| "= expr" | Calculator provider only |
| "> cmd" | Command provider only |
| Recognized URL | Web provider (no network) |

### Matching modes (applied in order, results merged)

1. **Exact match** — query exactly equals name (highest score)
2. **Prefix match** — name starts with query (high score)
3. **Token prefix match** — any significant token starts with query word
4. **FTS5 match** — SQLite FTS5 MATCH query
5. **Fuzzy match** — Levenshtein distance <= 2 for queries >= 4 chars (lower score)

### Result limits

- Maximum candidates from any single provider: 50
- Maximum results returned to UI: 12 (configurable up to 20)

### Cancellation

- A `CancellationToken` is passed into each search execution.
- Providers check the token before and during long DB queries.
- If cancelled, return `Err(SearchError::Cancelled)` — the IPC handler silently drops this result.

---

## 9. Ranking Algorithm

### Design goals

- **Deterministic:** Given the same candidates and usage data, always produces the same ranking.
- **Personalized:** Adapts over time to user behavior without cloud or AI.
- **Simple:** Must be auditable in plain code — no neural models, no opaque weights.
- **Explainable:** Score components can be logged for debugging.

### Scoring model

```
score = text_relevance_score
      + match_type_bonus
      + type_priority_bonus
      + usage_frequency_score
      + recency_score
```

#### `text_relevance_score` (0.0 to 1.0)

Normalized similarity between query and result name.

- Exact match: 1.0
- Prefix match: 0.85
- Token prefix match: 0.70
- FTS5 rank (normalized): 0.3 to 0.65
- Fuzzy match: 0.1 to 0.5 (scaled by edit distance)

#### `match_type_bonus` (0.0 to 0.3)

Additional flat bonus based on match quality:

- Exact: +0.3
- Prefix: +0.2
- Token prefix: +0.1
- Fuzzy only: +0.0

#### `type_priority_bonus` (0.0 to 0.25)

Applications receive a higher base priority than files:

- Application: +0.2
- Folder: +0.1
- File: +0.05
- Command: +0.15
- Calculator: +0.25 (always shown first when applicable)

#### `usage_frequency_score` (0.0 to 0.5)

```
usage_score = min(launch_count / 50.0, 1.0) * 0.5
```

This means a result launched 50+ times reaches maximum usage score.

#### `recency_score` (0.0 to 0.3)

```
hours_since_last_launch = (now - last_launched_at).as_secs() / 3600
recency_score = max(0.0, 0.3 - (hours_since_last_launch / 168.0) * 0.3)
```

168 hours = 1 week. A result launched 1 week ago has zero recency bonus.

### Tie breaking

If two results have equal scores (rare), break ties by:

1. Alphabetical order of display name
2. Result type order: Applications > Commands > Folders > Files

### Example: Personalization over time

User repeatedly searches "code" and launches VS Code:

- Initial state: VS Code has `launch_count=0`, `recency=0`
- After 5 launches: `usage_score ~= 0.05`, appears reliably first
- After 20 launches: `usage_score ~= 0.2`, VS Code will win even against closer text matches

The ranking does not require AI. It simply rewards demonstrated behavior.

---

## 10. Application Indexing

### Discovery sources

Applications are discovered from the following Windows locations:

| Source | Path | Priority |
|---|---|---|
| User Start Menu | `%APPDATA%\Microsoft\Windows\Start Menu\Programs\` | High |
| System Start Menu | `%ProgramData%\Microsoft\Windows\Start Menu\Programs\` | High |
| Desktop | `%USERPROFILE%\Desktop\` | Medium |
| Public Desktop | `%PUBLIC%\Desktop\` | Medium |

**V1 does not scan `%PROGRAMFILES%` or `%PROGRAMFILES(X86)%` directly.**

### Application record schema

```rust
pub struct ApplicationRecord {
    pub id: String,              // SHA256 of canonical path, hex-encoded
    pub display_name: String,    // Human-readable name
    pub exe_path: String,        // Absolute path to .exe
    pub shortcut_path: Option<String>, // Path to .lnk if discovered via shortcut
    pub arguments: Option<String>,
    pub icon_path: Option<String>,
    pub icon_index: i32,
    pub source: AppSource,       // StartMenuUser | StartMenuSystem | Desktop
    pub indexed_at: i64,         // Unix timestamp
    pub updated_at: i64,
}
```

### Icon handling

- **Do not** preload all application icons on startup.
- Icons are loaded on demand when a result is visible in the UI.
- Icons are cached in memory (bounded LRU, max 100 entries) after first load.
- The UI requests icons via a dedicated IPC call: `get_icon(id)` -> base64 PNG.

### Indexing frequency

- Full re-index of application sources: On startup + every 6 hours (configurable).
- Incremental: Whenever a `.lnk` file is created/modified/deleted in watched directories.
- Indexing runs on a background thread. UI is never blocked.

---

## 11. File Indexing

### Default indexed locations (V1)

| Location | Variable | Recursive |
|---|---|---|
| Desktop | `%USERPROFILE%\Desktop` | No (depth 1) |
| Documents | `%USERPROFILE%\Documents` | Yes (depth 4) |
| Downloads | `%USERPROFILE%\Downloads` | No (depth 1) |
| Pictures | `%USERPROFILE%\Pictures` | Yes (depth 3) |
| Videos | `%USERPROFILE%\Videos` | Yes (depth 3) |
| Music | `%USERPROFILE%\Music` | Yes (depth 3) |

User can add additional directories or remove defaults via Settings.

**The entire C:\ drive is never scanned by default.**

### File record schema

```rust
pub struct FileRecord {
    pub id: String,          // SHA256 of canonical path
    pub name: String,        // File name without directory
    pub display_name: String, // Name without extension (for display)
    pub extension: Option<String>,
    pub path: String,        // Absolute path
    pub parent_dir: String,  // Parent directory path
    pub size_bytes: u64,
    pub modified_at: i64,    // Unix timestamp
    pub indexed_at: i64,
    pub is_hidden: bool,
    pub is_system: bool,
}
```

### Ignored by default

- Hidden files (`FILE_ATTRIBUTE_HIDDEN`)
- System files (`FILE_ATTRIBUTE_SYSTEM`)
- Files matching patterns: `*.tmp`, `*.log`, `*.bak`, `desktop.ini`, `Thumbs.db`
- Directories: `node_modules`, `.git`, `__pycache__`, `$RECYCLE.BIN`, `System Volume Information`
- Symbolic links are followed only one level deep to prevent cycles

### Permission failures

- If a directory is inaccessible, log a warning, skip it, continue indexing.
- Do not crash or show an error to the user.
- Record inaccessible paths in a `skipped_paths` table for diagnostics.

---

## 12. Incremental Indexing

### Architecture

```
Filesystem event (ReadDirectoryChangesW via notify crate)
         |
         v
watcher.rs: Raw event received on watcher thread
         |
         v
Event normalization:
  - Deduplicate rapid repeat events (same path within 100ms)
  - Classify: Created | Modified | Deleted | Renamed
         |
         v
Debounced event queue (tokio channel, buffer: 500 events)
Flush batch every 500ms or when queue reaches 100 events
         |
         v
indexer/manager.rs: Process batch
  For each event:
  - Created -> insert/update record
  - Modified -> update metadata
  - Deleted -> remove record
  - Renamed (from->to) -> update path in record
         |
         v
SQLite transaction (batch commit all changes together)
         |
         v
Emit IPC event: "index_updated" (UI may refresh if launcher is open)
```

### Edge cases

| Scenario | Handling |
|---|---|
| Burst of events (e.g., extract archive) | Batch queue absorbs burst; process together |
| Rename pair (old + new) | `notify` delivers `Rename(from, to)` — update path atomically |
| File deleted before indexing | Catch `NotFound` error during stat; record deletion |
| Inaccessible path | Log warning; skip; do not fail batch |
| Database unavailable | Log error; discard batch; retry on next event |
| Watcher thread crash | Detected in `manager.rs`; restart watcher with backoff (1s, 5s, 30s) |
| Application shutdown | Signal watcher to stop; drain remaining queue; commit partial batch |

### Backpressure

The event queue has a maximum size of 500 events. If the queue is full,
oldest events are dropped (prioritize recency). This prevents unbounded
memory growth during very large directory operations.

---

## 13. SQLite Design

### Connection settings

Applied immediately after opening the connection:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA temp_store = MEMORY;
PRAGMA cache_size = -8000;   -- 8 MB page cache
PRAGMA mmap_size = 67108864; -- 64 MB memory-mapped I/O
```

### Schema

#### `applications`

```sql
CREATE TABLE applications (
    id           TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    exe_path     TEXT NOT NULL UNIQUE,
    shortcut_path TEXT,
    arguments    TEXT,
    icon_path    TEXT,
    icon_index   INTEGER NOT NULL DEFAULT 0,
    source       TEXT NOT NULL,
    indexed_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL
);

CREATE VIRTUAL TABLE applications_fts USING fts5(
    display_name,
    exe_path,
    content='applications',
    content_rowid='rowid'
);
```

#### `files`

```sql
CREATE TABLE files (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    display_name TEXT NOT NULL,
    extension    TEXT,
    path         TEXT NOT NULL UNIQUE,
    parent_dir   TEXT NOT NULL,
    size_bytes   INTEGER NOT NULL DEFAULT 0,
    modified_at  INTEGER NOT NULL,
    indexed_at   INTEGER NOT NULL,
    is_hidden    INTEGER NOT NULL DEFAULT 0,
    is_system    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_files_parent ON files(parent_dir);
CREATE INDEX idx_files_extension ON files(extension);

CREATE VIRTUAL TABLE files_fts USING fts5(
    name,
    display_name,
    path,
    content='files',
    content_rowid='rowid'
);
```

#### `folders`

```sql
CREATE TABLE folders (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    path         TEXT NOT NULL UNIQUE,
    parent_dir   TEXT NOT NULL,
    indexed_at   INTEGER NOT NULL
);

CREATE VIRTUAL TABLE folders_fts USING fts5(
    name,
    path,
    content='folders',
    content_rowid='rowid'
);
```

#### `usage`

```sql
CREATE TABLE usage (
    result_id        TEXT NOT NULL,
    result_type      TEXT NOT NULL,
    launch_count     INTEGER NOT NULL DEFAULT 0,
    last_launched_at INTEGER NOT NULL,
    PRIMARY KEY (result_id, result_type)
);
CREATE INDEX idx_usage_result ON usage(result_id);
```

#### `history`

```sql
CREATE TABLE history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    query        TEXT NOT NULL,
    result_id    TEXT NOT NULL,
    result_type  TEXT NOT NULL,
    result_name  TEXT NOT NULL,
    launched_at  INTEGER NOT NULL
);
CREATE INDEX idx_history_launched ON history(launched_at DESC);
CREATE INDEX idx_history_result ON history(result_id);
```

History table is capped at 10,000 rows. Oldest rows are deleted when limit is reached.

#### `settings`

```sql
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

#### `metadata`

```sql
CREATE TABLE metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Stores: schema_version, last_full_index_at, app_version, etc.
```

### Migrations

- Migration files: `migrations/0001_initial.sql`, `migrations/0002_add_history.sql`, etc.
- Loaded and applied at startup via `database/migrations.rs`.
- Each migration tracked in `metadata` by file name.
- Applied in numeric order; skipped if already applied.
- Migrations are never modified after release — only add new ones.

### Database location

```
%APPDATA%\SpotlightForWindows\spotlight.db
```

This path must never be hardcoded — always resolved programmatically.

---

## 14. IPC Architecture

### Principles

- All commands defined in `src-tauri/src/commands.rs` using `#[tauri::command]`.
- TypeScript wrappers live in `src/lib/ipc.ts` — React never calls `invoke` directly.
- Payloads are kept small. Never send the full file index over IPC.

### Commands (Rust, exposed to UI)

#### `search`

```typescript
// Request
{ query: string }

// Response
{
  results: SearchResult[];
  duration_ms: number;
}

// SearchResult
{
  id: string;
  result_type: 'app' | 'file' | 'folder' | 'command' | 'calculator' | 'web';
  display_name: string;
  subtitle: string;      // path, description, or expression
  score: number;
  icon_id: string | null;
}
```

#### `launch`

```typescript
// Request
{ id: string; result_type: string }
// Response
{ success: boolean; error?: string }
```

Side effects: Writes to `usage` and `history` tables.

#### `get_icon`

```typescript
// Request
{ id: string }
// Response
{ data: string | null }  // base64-encoded PNG
```

#### `hide_launcher`, `get_settings`, `update_settings`, `get_recent_results`, `get_index_status`, `rebuild_index`, `get_app_info`

All documented in full in `src-tauri/src/commands.rs`. Type definitions must
mirror the Rust structs exactly and be kept in `src/types/ipc.ts`.

### Events (Rust -> UI, unprompted)

| Event | Payload | Description |
|---|---|---|
| `index_updated` | `{ added: number; removed: number }` | Incremental index change |
| `index_progress` | `{ percent: number; phase: string }` | During initial indexing |
| `update_available` | `{ version: string; url: string }` | New version found |
| `settings_changed` | `Settings` | Settings updated from tray or elsewhere |

### Security boundaries

- The UI cannot call arbitrary shell commands via IPC.
- The UI cannot read arbitrary filesystem paths.
- Tauri capabilities (in `capabilities/default.json`) restrict which APIs are exposed.
- All inputs validated in Rust before use: path sanitization, length limits, type checking.

---

## 15. Keyboard System

### Core keyboard interactions

| Key | Action |
|---|---|
| `Alt + Space` | Open/close launcher (global hotkey) |
| `Arrow Down` | Move selection down |
| `Arrow Up` | Move selection up |
| `Enter` | Launch selected result |
| `Escape` | Hide launcher |
| `Tab` | Move to next result (same as Arrow Down) |
| `Shift + Tab` | Move to previous result |
| `Ctrl + L` | Clear search and focus input |
| `Ctrl + ,` | Open settings (when launcher is open) |
| `Ctrl + Backspace` | Delete word (in search input) |

### Navigation rules

- Selection wraps: Arrow Down from last result -> selects first result.
- Selection wraps: Arrow Up from first result -> selects last result.
- If no result is selected and user presses Enter, nothing happens.

### Shortcut customization

- The global hotkey is configurable in Settings.
- Supported modifiers: `Alt`, `Ctrl`, `Win`, `Shift` + any standard key.
- Validate that the new shortcut can be registered before saving.
- If conflict detected, notify user immediately.

---

## 16. Global Hotkey

### Implementation

- Implemented via `tauri-plugin-global-shortcut`.
- Registered during the startup sequence.
- Default: `Alt+Space`.

### Failure modes and recovery

| Scenario | Behavior |
|---|---|
| Shortcut already registered | Log warning, notify user via tray balloon |
| Registration fails (OS error) | Retry after 5 seconds, max 3 attempts; if all fail, notify user |
| User changes shortcut | Unregister old -> register new -> save only on success |
| Duplicate instance | Second instance signals first, exits immediately |

### Multi-instance prevention

`tauri-plugin-single-instance` must be registered first.
If a second instance starts:
1. Second instance sends signal to first.
2. First instance shows launcher window.
3. Second instance exits immediately.
4. Second instance must **not** start indexing or register hotkeys.

---

## 17. Window Management

### Window configuration (tauri.conf.json approximate)

```json
{
  "windows": [{
    "label": "launcher",
    "title": "Spotlight",
    "width": 640,
    "height": 480,
    "decorations": false,
    "transparent": true,
    "alwaysOnTop": true,
    "center": true,
    "visible": false,
    "skipTaskbar": true,
    "resizable": false
  }]
}
```

### Window behavior

- **Decorations:** None (custom UI with no title bar)
- **Transparency:** Enabled (for rounded corners and shadow effects)
- **Always on top:** Yes (while visible)
- **Skip taskbar:** Yes
- **Resizable:** No in V1

### Show/hide sequence

**Show:**
1. Recalculate position for current monitor/DPI
2. `window.show()`
3. `window.set_focus()`
4. Emit event to React to focus the search input

**Hide:**
1. Clear search query
2. `window.hide()`
3. Window remains in memory, hidden

### Multi-monitor support

- When `Alt + Space` is pressed, determine which monitor has the cursor.
- Position launcher centered on that monitor.
- Use `MonitorFromPoint` and `GetMonitorInfo` Win32 APIs.
- Recalculate on every show.

### DPI awareness

- Declare the process as `PerMonitorV2` DPI aware in the manifest.
- All pixel calculations must use logical pixels scaled by the monitor's DPI factor.

---

## 18. UI/UX Direction

### Design principles

The launcher UI should feel like it belongs in Windows, not like an
imported web app. The aesthetic is minimal, monochromatic, purposeful,
and fast-feeling.

### Color palette

```css
/* Dark mode (default) */
--bg-primary: #1a1a1a;
--bg-secondary: #242424;
--bg-hover: #2e2e2e;
--bg-selected: #383838;
--text-primary: #f0f0f0;
--text-secondary: #a0a0a0;
--text-muted: #606060;
--accent: #5b9cf6;
--accent-soft: #1e3a5f;
--border: rgba(255,255,255,0.08);
--shadow: rgba(0,0,0,0.5);

/* Light mode */
--bg-primary: #ffffff;
--bg-secondary: #f5f5f5;
--bg-hover: #ebebeb;
--bg-selected: #e0e0e0;
--text-primary: #1a1a1a;
--text-secondary: #555555;
--text-muted: #888888;
--accent: #2563eb;
--accent-soft: #dbeafe;
--border: rgba(0,0,0,0.08);
--shadow: rgba(0,0,0,0.15);
```

### Typography

- Font: `'Inter'` (Google Fonts), fallback to `'Segoe UI'`, then `system-ui`
- Search input: 18px, weight 400
- Result name: 14px, weight 500
- Result subtitle: 12px, weight 400, `--text-secondary`

### Layout

```
+---------------------------------------------+
|  [Search icon]  [Search input             ] |  <- 56px tall
+---------------------------------------------+
|  [Icon] Result Name          App           |  <- 48px per row
|          Subtitle                          |
|  [Icon] Result Name          File          |
|          C:\Users\...                      |
+---------------------------------------------+
```

- Window width: 640px
- Search bar height: 56px
- Result row height: 48px
- Max visible results: 8
- Window max height: ~450px
- Border radius: 12px
- Window shadow: subtle, 24px blur

### Animations

| Animation | Duration | Easing |
|---|---|---|
| Window appear | 120ms | ease-out |
| Window disappear | 100ms | ease-in |
| Result hover | 80ms | ease |
| Selection change | 60ms | ease |
| Icon load | 150ms | ease (fade in) |

**Rule:** No animation may delay user interaction.

### Empty state

When query is empty: Show recently launched items (up to 8).
When query returns no results: Show minimal "No results" message.
Do not show spinners or loading states.

### Forbidden design patterns

- Gradient backgrounds
- Card shadows on individual results
- Loading spinners during search
- Status bars
- Tabs in the launcher view
- Generic dashboard layouts

---

## 19. React Performance

### Rules

1. Search state is local to the launcher feature. Do not put `query` or `results` in a global store.
2. `ResultList` uses `React.memo`. Each `ResultItem` is memoized.
3. Stable keys: use `result.id` as key, never array index.
4. No virtualization in V1 unless measurement proves it is needed.
5. IPC calls are centralized in `src/lib/ipc.ts`. No `invoke` calls scattered in components.
6. Use event-driven pattern for search: input change -> debounce -> IPC call -> setState.
7. Settings UI is lazy-loaded: `React.lazy()` + `Suspense`.
8. No Redux. Zustand only for small shared state.
9. No unnecessary context. Only use React Context for theme and settings.

### Bundle size constraints

- Total JS bundle (gzipped): < 300 KB
- No UI framework beyond React
- CSS: < 50 KB gzipped
- Permitted dependencies: `react`, `react-dom`, `zustand`, `@tauri-apps/api`

Every new npm dependency must be justified before being added.

---

## 20. Memory Optimization

### Do

- Lazy-load icons (request on visibility)
- Use bounded in-memory caches (icon cache: max 100 entries, LRU eviction)
- Release search candidates after ranking
- Query only required columns from SQLite (never `SELECT *` in production)
- Limit search results to top N
- Batch filesystem events
- Profile memory on real builds with Windows Task Manager and Process Hacker

### Do Not

- Do not preload all application icons at startup
- Do not load the entire file index into a Rust `Vec` in memory
- Do not keep unlimited search history in memory
- Do not cache every search result indefinitely
- Do not create a new `Connection` per search (reuse the single connection)
- Do not retain the watcher event history beyond what is needed for deduplication

### Memory profiling methodology

1. Run release build (not debug)
2. Launch the application, complete onboarding, wait 5 minutes (idle)
3. Record Private Working Set: **idle baseline**
4. Open launcher, type queries, launch results, close launcher, wait 2 minutes
5. Record Private Working Set: **active baseline**
6. Trigger initial indexing of a large directory set
7. Record peak Private Working Set: **indexing peak**
8. All three values must be within targets defined in Section 2.

---

## 21. CPU Optimization

### Idle state

When the launcher is hidden, Rust must be doing nearly nothing:

- No polling loops
- No filesystem polling (use event-driven `notify`)
- No periodic search queries
- No continuous animations
- No network polling (check updates once at startup, then once per 24 hours)

### Indexing state

- Use `tokio::task::yield_now().await` inside long scanning loops
- Batch commits to SQLite (commit every 500 records)
- Set indexing thread priority to below-normal via Win32 `SetThreadPriority`

---

## 22. Offline-First

Core functionality must work with no internet connection.

| Feature | Offline? |
|---|---|
| App search | Always |
| File search | Always |
| Folder search | Always |
| Calculator | Always |
| Built-in commands | Always |
| History and ranking | Always |
| Settings | Always |
| Launching | Always |
| Indexing | Always |
| Web search provider | Requires internet; isolated, optional |
| Update checking | Requires internet; non-blocking, silent |

### Network failure handling

- Web search provider: If no internet, browser shows offline error. Acceptable.
- Update check: If unreachable, log debug, skip silently. Never show an error for a failed update check.
- Network timeout for update check: 5 seconds maximum. Never block startup.

---

## 23. Calculator

### Implementation

Use a safe, sandboxed expression evaluator. Recommended: `evalexpr` crate
(pure Rust, no arbitrary code execution) or a custom recursive descent parser.

**Do not use** `eval()` in JavaScript or any form of arbitrary code execution.

### Detection

The calculator activates when:
1. The query starts with `=` (explicit calculator mode)
2. The query matches a mathematical expression pattern

### Supported operations

| Category | Operations |
|---|---|
| Arithmetic | `+`, `-`, `*`, `/`, `^` (power), `%` (modulo) |
| Functions | `sqrt()`, `abs()`, `floor()`, `ceil()`, `round()` |
| Constants | `pi`, `e` |
| Percentage | `15%` -> `0.15` |

### Output format

- Integer results: displayed as integer (`42`)
- Float results: up to 10 significant digits, trailing zeros removed
- Division by zero: display "Division by zero"
- Invalid expression: no calculator result shown (fall through to regular search)

### Keyboard behavior

- Pressing Enter on a calculator result copies it to the clipboard.
- Display a brief "Copied!" confirmation in the result subtitle.

---

## 24. Command System

### V1 built-in commands

| Command query | Action | Confirmation required? |
|---|---|---|
| `lock` | Lock the Windows session | No |
| `sleep` | Put system to sleep | No |
| `shutdown` | Initiate system shutdown | Yes |
| `restart` | Initiate system restart | Yes |
| `logout` | Log out current user | Yes |
| `empty trash` | Empty Recycle Bin | Yes |

### Security model

- Commands are an immutable list in `src-tauri/src/search/providers/commands.rs`.
- No user-defined commands in V1.
- No shell string execution.
- Commands call specific Win32 APIs (`ExitWindowsEx`, `SetSuspendState`, etc.), not `cmd.exe`.

### Future expansion (V2+)

User-defined commands with shell execution are a future feature. When
implemented, they must have explicit opt-in, run with user's own permissions,
and audit log all executions.

---

## 25. Web Search

### Design

Web search is an **optional, isolated provider**. It is:
- Disabled by default (user must enable in Settings)
- The last result in any result list
- Never fetching data on every keystroke

### Supported search engines (V1)

- Google (default), Bing, DuckDuckGo, Custom URL template

### Behavior

- The result is generated locally from a URL template.
- Selecting and pressing Enter opens the URL in the default browser.
- No network call is made by the launcher itself.

---

## 26. History and Personalization

### What is stored

- In `history`: query string, result id, result type, result display name, launched_at timestamp
- In `usage`: result id, result type, launch_count, last_launched_at

### What is NOT stored

- Search queries that did not result in a launch
- Raw keystrokes
- Network activity

### Privacy rules

- All data stored locally in `%APPDATA%\SpotlightForWindows\spotlight.db`
- Nothing sent to any server
- No account required
- User can clear all history from Settings > Privacy > Clear History
- User can disable history entirely
- On uninstall, installer asks whether to delete user data

---

## 27. Settings

### Settings model

```rust
pub struct Settings {
    pub global_shortcut: String,
    pub start_with_windows: bool,
    pub minimize_on_close: bool,
    pub indexed_paths: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub index_hidden_files: bool,
    pub index_interval_hours: u32,
    pub max_results: u32,
    pub web_search_enabled: bool,
    pub web_search_engine: String,
    pub web_search_url_template: Option<String>,
    pub theme: Theme,              // Dark | Light | System
    pub history_enabled: bool,
    pub history_max_entries: u32,
    pub auto_check_updates: bool,
    pub update_channel: UpdateChannel, // Stable | Beta
    pub first_run_completed: bool,
    pub db_schema_version: u32,
}
```

### Settings UI sections

1. **General** — startup, hotkey
2. **Indexing** — paths, exclusions, rebuild index
3. **Search** — max results, web search
4. **Appearance** — theme
5. **Privacy** — history, clear history
6. **Updates** — auto-update toggle, channel
7. **About** — version, build, open log folder, reset settings

---

## 28. First Run / Onboarding

### Flow

```
[Welcome screen]
"Welcome to Spotlight for Windows"
[Get Started]
        |
        v
[Keyboard Shortcut]
"Your launcher shortcut is Alt + Space"
Option to change it. Live conflict detection.
[Continue]
        |
        v
[Start with Windows?]
Toggle: "Launch Spotlight at Windows startup (recommended)"
[Continue]
        |
        v
[Index your files]
Checkboxes: Desktop, Documents, Downloads, Pictures, Videos
[Start Indexing]
        |
        v
[Indexing begins in background]
"Your launcher is ready. Press Alt + Space anytime."
[Open Launcher]
```

### Design rules

- Maximum 5 steps.
- Every step must have a "Skip" option (except final confirmation).
- Defaults must be sensible.
- Onboarding must not block use of the launcher.
- First-run flag set in settings after step 1.

---

## 29. System Tray

### Icon

- 16x16 and 32x32 PNG variants
- Monochromatic (adapts to light/dark Windows taskbar)
- Search/magnifier motif

### Menu

```
Spotlight for Windows        [disabled header]
-----------------------------
> Open
Settings
Change Shortcut
-----------------------------
Pause / Resume Indexing
Rebuild Index
-----------------------------
Check for Updates
About
-----------------------------
Quit
```

### Rules

- Single left-click on tray icon: Show launcher window
- Right-click on tray icon: Show menu
- Tray icon always present when application is running
- No taskbar button

---

## 30. Startup With Windows

### Implementation

Enable startup via Windows Run registry key:

```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
Key: "SpotlightForWindows"
Value: "C:\Program Files\Spotlight for Windows\spotlight.exe" --startup
```

The `--startup` flag causes the application to start silently.

### Rules

- Startup is opt-in during onboarding (default: on).
- Toggling in Settings updates the registry key immediately.
- On uninstall, the startup key is removed.
- Duplicate startup entries are prevented.

---

## 31. Single Instance

### Implementation

Use `tauri-plugin-single-instance`.

On second-instance launch:
1. Plugin sends message to running instance.
2. Running instance receives message and shows launcher window.
3. Second instance exits immediately (exit code 0).

---

## 32. Error Handling

### Principles

- Never crash the application due to a recoverable error in one subsystem.
- Use structured error types with meaningful context.
- Log all errors with `tracing::error!` including relevant context fields.
- Never expose raw file paths or search data in user-visible error messages.

### Subsystem error boundaries

| Subsystem | On failure | User experience |
|---|---|---|
| SQLite unavailable | Attempt reconnect, enter degraded mode | Tray notification |
| Indexer crash | Restart indexer thread with backoff | Silent; indexing resumes |
| Hotkey registration failure | Log warning, notify user | Tray balloon |
| Window show failure | Log error, attempt again | User presses hotkey again |
| File launcher error | Log error | "Could not open file" in result |
| Watcher failure | Log error, restart watcher | Silent |
| Permission denied (indexing) | Log warning, skip directory | Silent |
| Update check failure | Log debug, skip | Silent |

---

## 33. Logging

### Development logging

- Level: `TRACE` and above
- Output: stderr + rolling file
- Format: structured with timestamps, module paths, line numbers

### Production logging

- Level: `INFO` and above by default
- Log rotation: max 5 files x 5 MB each
- Log location: `%APPDATA%\SpotlightForWindows\logs\`
- Format: structured text for human readability
- Sensitive data: file paths truncated to base name in `INFO` logs;
  full paths only at `DEBUG` level

### Implementation

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
```

---

## 34. Security

### Tauri capabilities

`capabilities/default.json` must follow the principle of least privilege.

Permitted:
- Shell: open URLs in browser only (no `execute`)
- FS: read user data directory only
- Clipboard: write only (for calculator copy)

Forbidden:
- Shell execute
- OS-level process spawning from JS
- Arbitrary filesystem read/write
- Network requests from JS layer

### IPC security

- All command inputs validated in Rust before use
- Path inputs: canonicalized, checked against allowed base directories
- Query inputs: length-limited (max 256 characters), sanitized
- Always use prepared statements via `rusqlite`'s parameter binding

### Application launching security

- Only launch paths that exist in the `applications` or `files` tables
- Validate that the path has not been tampered with
- Reject paths containing null bytes, path traversal (`..`), or unexpected UNC paths

### No administrator privileges

- The installer offers a per-user install option (no UAC required)
- The running application uses no elevated permissions
- Registry writes are to `HKCU` only (no `HKLM`)

---

## 35. Testing Strategy

### Rust unit tests

Location: `#[cfg(test)]` modules within each source file.

| Module | What to test |
|---|---|
| `search/parser.rs` | Query parsing, mode detection, edge cases |
| `search/ranking.rs` | Score components, tie-breaking, score ranges |
| `search/providers/calculator.rs` | Valid expressions, edge cases, division by zero |
| `database/migrations.rs` | Migration application, idempotency |
| `database/apps.rs` | CRUD operations, FTS5 match |
| `settings/models.rs` | Default values, serialization roundtrip |
| `launcher/application.rs` | Path validation, argument parsing |

### TypeScript unit tests

Use **Vitest** for unit tests. Location: `src/**/*.test.ts`.

| Module | What to test |
|---|---|
| `lib/ipc.ts` | Type correctness (mocked invoke) |
| `hooks/useKeyboardNav.ts` | Arrow key navigation, wrap-around |
| `hooks/useSearch.ts` | Debounce, empty query behavior |
| Result rendering | Correct display of name, subtitle, type |
| Settings form | Validation, save behavior |

### Integration tests

Location: `tests/integration/`.

Test Rust modules together using a real SQLite database (in-memory or temp file):
- Index test applications -> search -> verify results
- Update a file record -> verify FTS5 is updated
- Write settings -> restart settings manager -> verify settings loaded
- Insert usage data -> run ranking -> verify order

### End-to-end tests

Full application lifecycle tests on real Windows environment:
1. Install the application
2. Verify tray icon appears
3. Press Alt + Space (simulated) -> verify launcher appears
4. Type "notepad" -> verify Notepad appears in results
5. Press Enter -> verify Notepad launches
6. Return to launcher -> press Escape -> verify launcher hides
7. Verify Rust process still running

E2E framework: **Playwright** for the WebView UI portion plus
custom PowerShell scripts for OS-level interaction.

---

## 36. Performance Benchmarking

### Benchmark suite

Implemented using **Criterion.rs** in `benchmarks/`.

```rust
// benchmarks/search_bench.rs
fn bench_search(c: &mut Criterion) {
    let db = setup_test_db_with_10k_records();
    c.bench_function("search_10k_records", |b| {
        b.iter(|| search_engine.search("code"))
    });
}
```

### Metrics to benchmark

| Metric | Tool | How |
|---|---|---|
| Cold startup -> ready | Stopwatch in code | Log timestamp diff |
| Hotkey -> visible UI | Custom instrumentation | Timestamp: hotkey event -> window visible |
| Search latency | Criterion.rs | Isolated search bench with known dataset |
| DB query latency | Criterion.rs | Direct SQLite query bench |
| Initial indexing throughput | Instrumentation | Files/second logged during indexing |
| Idle CPU | Process Hacker | 10-min average on release build |
| Idle RAM | Process Hacker | Private Working Set, idle 5 min |
| Peak indexing RAM | Process Hacker | Peak Private Working Set during full index |

### Before/after rule

Every performance-relevant change must be accompanied by benchmark results
showing before and after. Never submit a performance change based on intuition alone.

---

## 37. Large Dataset Testing

### Test environments

| Environment | App count | File count | Expected behavior |
|---|---|---|---|
| Minimal | 50 | 1,000 | Search < 20ms |
| Typical | 200 | 10,000 | Search < 50ms |
| Heavy | 500 | 50,000 | Search < 100ms |
| Extreme | 1,000 | 100,000+ | Search < 200ms |

### Memory under large datasets

The file index must never be loaded into memory wholesale. At 100,000
files, only query results should be materialized in Rust memory.

### Indexing performance

At 50,000 files, initial indexing should complete within 60 seconds on
reference hardware with < 25% CPU and < 150 MB RAM peak.

---

## 38. Dependency Strategy

### Evaluation criteria for new dependencies

Before adding any dependency:

1. **Necessity:** Is there a standard library (`std`) solution?
2. **Maintenance:** Is the crate actively maintained? Last commit < 1 year ago?
3. **Binary size:** What does `cargo bloat` show for this dependency?
4. **Runtime cost:** Does it allocate excessively or start background threads?
5. **Attack surface:** Does it introduce security risks?
6. **License:** Is the license compatible (MIT/Apache-2.0 preferred)?

### Approved Rust dependencies (V1)

| Crate | Purpose |
|---|---|
| `tauri` 2.x | App shell |
| `tauri-plugin-global-shortcut` | Hotkey |
| `tauri-plugin-single-instance` | Single instance |
| `tauri-plugin-updater` | Updates |
| `rusqlite` (bundled) | SQLite |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `notify` | Filesystem watching |
| `windows` | Win32 APIs |
| `tracing` + `tracing-subscriber` + `tracing-appender` | Logging |
| `anyhow` | Error propagation |
| `thiserror` | Error types |

### Approved npm dependencies (V1)

| Package | Purpose |
|---|---|
| `react` / `react-dom` | UI framework |
| `@tauri-apps/api` | Tauri IPC |
| `zustand` | Minimal state |

---

## 39. Build Strategy

### Cargo release profile

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
strip = true
```

### Build pipeline

```
npm install + cargo fetch
        |
        v
TypeScript typecheck (tsc --noEmit)
        |
        v
ESLint + Prettier check
        |
        v
Rust clippy (--deny warnings in CI)
        |
        v
Rust fmt check
        |
        v
Unit tests (cargo test + vitest)
        |
        v
Integration tests
        |
        v
Release build (npm run tauri build)
        |
        v
Sign binary
        |
        v
Package installer
        |
        v
Sign installer
        |
        v
Upload artifacts
```

---

## 40. Installer

### Installer toolchain

Use **WiX Toolset** or **NSIS** (configured via Tauri's built-in bundler).
Prefer WiX for professional MSI output.

### Features

| Feature | Behavior |
|---|---|
| Install destination | `%PROGRAMFILES%\Spotlight for Windows\` (system) or per-user |
| Start Menu shortcut | Created during install |
| Desktop shortcut | Optional |
| Startup registry key | Set if user opts in |
| Silent install | Supported via `/S` flag |
| Upgrade | Detect existing version, preserve settings and database |

### Data directory (separate from install directory)

```
%APPDATA%\SpotlightForWindows\
+-- spotlight.db         <- User's index and settings
+-- logs\                <- Log files
+-- cache\               <- Icon cache (future)
```

The installer must **never** delete this directory automatically on uninstall.
During uninstall, ask: "Remove all search data and settings?" (default: No).

---

## 41. Auto Updates

### Update mechanism

Use `tauri-plugin-updater` with a self-hosted or GitHub Releases endpoint.

### Update flow

1. App starts -> check for updates (if `auto_check_updates = true`)
2. If new version found -> tray balloon: "Update available: v1.2.0. Click to install."
3. Download update in background (shows progress)
4. Verify signature
5. Prompt: "Restart now to apply update?" -> [Restart Now] [Later]
6. If "Restart Now": launch installer silently, exit current process

### Security

- All update downloads verified against cryptographic signature before application
- HTTPS only for update endpoint and download
- If signature verification fails: discard download, log error, do not apply

### Failure scenarios

| Scenario | Behavior |
|---|---|
| Update server unreachable | Log debug, skip silently |
| Download interrupted | Discard partial file, retry on next check |
| Signature verification failure | Discard download, log error, notify user |
| Installation failure | Log error, continue running current version |

---

## 42. Code Signing

### Certificate strategy

- Use an **Extended Validation (EV)** code signing certificate for production.
- OV certificate is acceptable for early beta.

### What to sign

1. `spotlight.exe` — main executable
2. The installer (.exe or .msi)
3. Any auto-update packages

### CI integration

- Certificate stored as encrypted CI secret
- Signing step runs only on `release` workflow
- Private key never stored in the repository

### Timestamping

Always use an RFC 3161 timestamp server.

---

## 43. CI/CD

### Pull Request workflow (`.github/workflows/ci.yml`)

Triggered on: `push` to any branch, `pull_request` to `main`.

```yaml
jobs:
  lint-and-typecheck:
    - npm ci
    - npx tsc --noEmit
    - npx eslint . --max-warnings 0
    - npx prettier --check .

  rust-checks:
    - cargo fmt -- --check
    - cargo clippy -- -D warnings
    - cargo test --all

  build:
    - npm run tauri build
```

### Release workflow (`.github/workflows/release.yml`)

Triggered on: push of tag matching `v*.*.*`.

```yaml
jobs:
  build-and-release:
    - Checkout + Install dependencies
    - Run full test suite
    - Build release binary
    - Sign binary (using CI secrets)
    - Build installer
    - Sign installer
    - Create GitHub Release + upload installer
    - Publish update JSON to update endpoint
```

### Required secrets

| Secret | Purpose |
|---|---|
| `CERT_FILE_BASE64` | Base64-encoded code signing certificate |
| `CERT_PASSWORD` | Certificate private key password |
| `UPDATE_ENDPOINT_TOKEN` | Authentication for pushing update JSON |
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri updater signature key |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Tauri updater key password |

---

## 44. Versioning

### Scheme

Semantic versioning: `MAJOR.MINOR.PATCH`

| Component | Meaning |
|---|---|
| `MAJOR` | Breaking changes to settings, schema, or installer that cannot auto-migrate |
| `MINOR` | New features, non-breaking changes |
| `PATCH` | Bug fixes, performance improvements, security patches |

### Version sources of truth

All three must be updated together before release:
- `Cargo.toml` (workspace root)
- `package.json`
- `tauri.conf.json` -> `version`

### Database migrations and versioning

If the database schema version is newer than the running application version,
the application refuses to start and shows an error: "Database created by a newer version of Spotlight. Please upgrade."

---

## 45. Git Strategy

### Branch model

| Branch | Purpose |
|---|---|
| `main` | Always deployable; requires passing CI |
| `develop` | Integration branch for in-progress features |
| `feature/*` | Individual feature branches |
| `fix/*` | Bug fix branches |
| `release/v*.*.*` | Release preparation |

### Commit conventions

Follow Conventional Commits (https://www.conventionalcommits.org/):

```
feat(search): add fuzzy matching for short queries
fix(indexer): prevent crash on inaccessible directory
perf(ranking): reduce allocations in score computation
docs(impl): update Section 9 with new scoring weights
test(search): add benchmark for 50k file dataset
```

### PR requirements

- All CI checks pass
- At least one reviewer approval
- No `TODO` comments without a linked issue
- CHANGELOG.md updated

---

## 46. AI Coding Agent Instructions

### Mandatory first steps

Every AI coding agent that opens this repository must:

1. **Read `IMPLEMENTATION.md` completely** before writing any code.
2. **Inspect the repository structure** to understand current state.
3. **Identify the relevant module** for the requested change.
4. **Understand the architecture boundaries** (especially the Rust/React split).
5. **Implement the smallest correct change** that fulfills the requirement.
6. **Run all tests** after implementation.
7. **Run lint, typecheck, and format checks.**
8. **Run the build** to verify nothing is broken.
9. **Check performance implications** if the change touches search, indexing, or IPC.
10. **Update `IMPLEMENTATION.md`** if any architectural decision has changed.

### Non-negotiable rules

| Rule | Reason |
|---|---|
| Never move search/indexing logic into React | Architecture boundary |
| Never perform full-disk scans during a search query | Performance requirement |
| Never add background polling where events can be used | CPU optimization |
| Never ignore Windows-specific behavior | Platform requirement |
| Never introduce network requirements into offline core features | Offline-first |
| Never silently weaken security boundaries | Security |
| Never add a dependency without justification | Dependency hygiene |
| Never skip tests for changes to ranking, search, or indexing | Correctness |
| Never optimize without measuring before and after | Performance discipline |

> **Do not implement a feature by violating the performance architecture.**

### When IMPLEMENTATION.md must be updated

- Any change to module structure
- Any change to the SQLite schema
- Any change to the IPC command/event list
- Any change to the ranking algorithm
- Any change to the security model
- Any addition of a new dependency
- Any change to performance targets
- Any new phase added to the roadmap

---

## 47. Development Phases

### Phase 0 - Project Bootstrap

**Objective:** Working development environment with correct toolchain.

**Tasks:**
- Initialize Tauri 2 project with React/TypeScript template
- Configure `tsconfig.json` (strict mode)
- Configure ESLint + Prettier
- Configure Rust workspace (`Cargo.toml`)
- Set up GitHub Actions CI workflow
- Verify `npm run tauri dev` works

**Acceptance criteria:**
- `npm run tauri dev` opens a basic Tauri window
- CI pipeline runs on push and passes
- TypeScript strict mode active
- Clippy runs with `--deny warnings`

---

### Phase 1 - Application Shell

**Objective:** Persistent background process with tray and single-instance enforcement.

**Tasks:**
- Implement `core/app.rs`, `core/lifecycle.rs`, `core/state.rs`
- Implement `tray/manager.rs` with basic menu
- Integrate `tauri-plugin-single-instance`
- Implement graceful shutdown sequence
- Implement startup argument handling (`--startup` flag)
- Window: basic frameless, transparent, hidden by default

**Acceptance criteria:**
- Application runs with a tray icon
- Right-click tray -> menu appears
- "Quit" exits the application
- Launching a second instance shows existing instead of starting new
- `--startup` flag starts with no visible window

---

### Phase 2 - Global Hotkey

**Objective:** `Alt + Space` toggles the launcher window.

**Tasks:**
- Implement `hotkey/manager.rs`
- Register `Alt + Space` via `tauri-plugin-global-shortcut`
- Show/hide the window on hotkey press
- Handle registration failure gracefully
- Implement hotkey conflict detection

**Acceptance criteria:**
- `Alt + Space` shows launcher window when hidden
- `Alt + Space` hides launcher window when shown
- Registration failure shows tray notification
- Hotkey works even when another application has focus

---

### Phase 3 - Launcher UI

**Objective:** Functional, keyboard-navigable search UI.

**Tasks:**
- Implement search input component
- Implement result list component
- Implement keyboard navigation (arrow keys, Enter, Escape)
- Implement result item component
- Apply color palette and typography from Section 18
- Implement window show/hide animation

**Acceptance criteria:**
- Search input receives focus when window opens
- Arrow keys navigate results
- Enter selects result (placeholder action)
- Escape hides window
- UI matches design spec from Section 18
- Animations complete in < 150ms

**Performance:** Verify React renders result list without unnecessary re-renders.

---

### Phase 4 - Application Indexing

**Objective:** Discover and index installed applications.

**Tasks:**
- Implement `database/connection.rs`, `database/migrations.rs`
- Create initial migration: `applications`, `applications_fts` tables
- Implement `indexer/apps.rs` (Start Menu, Desktop scanning)
- Implement `.lnk` parsing via COM APIs
- Implement `database/apps.rs` CRUD
- Store application records in SQLite
- Implement icon loading on demand

**Acceptance criteria:**
- All Start Menu applications discoverable and stored
- Icons load on demand without visible lag
- Indexing completes without blocking the UI

**Performance:** Indexing 200+ applications must complete in < 10 seconds.

---

### Phase 5 - File Indexing

**Objective:** Index files in default user directories.

**Tasks:**
- Implement `indexer/files.rs`, `indexer/folders.rs`
- Create `files`, `folders`, FTS5 tables in migration
- Implement configurable depth scanning
- Implement ignore rules
- Implement `indexer/watcher.rs` for incremental updates
- Implement progress reporting via IPC events

**Acceptance criteria:**
- Files in Desktop, Documents, Downloads appear in index
- Hidden/system files excluded by default
- Filesystem watcher correctly handles file changes
- Progress events reach the UI during initial indexing

**Performance:** 10,000 files indexed in < 15 seconds. RAM during indexing < 100 MB.

---

### Phase 6 - Search Engine

**Objective:** Fast, multi-provider search.

**Tasks:**
- Implement `search/engine.rs`, `search/parser.rs`
- Implement `search/providers/apps.rs`, `files.rs`, `folders.rs`
- Implement `search/query.rs`
- Expose `search` IPC command
- Implement empty-query behavior (recent items)
- Implement debounce in React hook

**Acceptance criteria:**
- Typing "notepad" returns Notepad in top 3 results
- Empty query returns up to 8 recent items
- Search results arrive within 100ms for 10k indexed items

**Performance benchmark:** Run `search_bench` Criterion benchmark. Record baseline.

---

### Phase 7 - Ranking

**Objective:** Intelligent, personalized result ranking.

**Tasks:**
- Implement `search/ranking.rs` with scoring model from Section 9
- Implement `database/usage.rs`, `database/history.rs`
- Record usage on every launch
- Expose `get_recent_results` IPC command
- Write unit tests for each score component

**Acceptance criteria:**
- After 5 launches of VS Code via "code" query, VS Code appears first
- Calculator result always appears first for math queries
- Unit tests for all score components pass

---

### Phase 8 - Launching

**Objective:** Open results correctly.

**Tasks:**
- Implement `launcher/application.rs`, `launcher/file.rs`, `launcher/folder.rs`
- Implement path validation before launch
- Hide launcher window before launching
- Record usage after successful launch
- Expose `launch` IPC command

**Acceptance criteria:**
- Enter on an application launches it
- Enter on a file opens it with the default app
- Enter on a folder opens Explorer
- Launcher hides cleanly before the launched app appears
- Invalid/deleted paths fail gracefully

---

### Phase 9 - Calculator

**Objective:** Safe local expression evaluation.

**Acceptance criteria:**
- `2 + 2` -> `4`
- `sqrt(144)` -> `12`
- Division by zero -> friendly message
- Enter copies result to clipboard

---

### Phase 10 - History

**Acceptance criteria:**
- Recent items shown on empty query
- History clears when user triggers "Clear History"
- Launch count correctly drives ranking

---

### Phase 11 - Settings

**Acceptance criteria:**
- All settings persist across restarts
- Theme toggle takes effect immediately
- Startup toggle updates registry key
- Indexed paths changes trigger re-index of new paths

---

### Phase 12 - Onboarding

**Acceptance criteria:**
- Fresh install shows onboarding
- Onboarding completes in < 2 minutes
- Subsequent launches skip onboarding
- All steps skippable

---

### Phase 13 - Performance Optimization

**Objective:** Meet all performance targets from Section 2.

**Tasks:**
- Profile idle CPU and RAM on release build
- Profile search latency with 10k/50k/100k records
- Optimize slow code paths identified in profiling
- Audit React renders with React DevTools Profiler
- Audit bundle size with `vite-bundle-visualizer`
- Verify debounce timing and cancellation of stale searches

**Acceptance criteria:**
- All metrics from Section 2 are met on reference hardware
- Benchmark results recorded in `benchmarks/results/`

---

### Phase 14 - Security Hardening

**Tasks:**
- Review and restrict Tauri capabilities to minimum
- Audit all IPC command input validation
- Audit all path operations for traversal risks
- Implement web search URL validation
- Review logging for sensitive data exposure
- Document security model in `SECURITY.md`

**Acceptance criteria:**
- Capability audit passes (no over-privileged APIs)
- All path inputs validated
- No sensitive data in `INFO`-level logs
- `SECURITY.md` complete

---

### Phase 15 - Testing

**Acceptance criteria:**
- Unit test coverage > 80% for `search/`, `ranking/`, `database/` modules
- All integration tests pass
- E2E core workflow test passes
- All failure scenarios tested

---

### Phase 16 - Installer

**Acceptance criteria:**
- Install completes without errors
- Uninstall removes all binaries
- Upgrade preserves settings and index
- Per-user install requires no UAC

---

### Phase 17 - Signing

**Acceptance criteria:**
- No SmartScreen warnings on signed installer
- `signtool verify /pa spotlight.exe` passes

---

### Phase 18 - Auto Update

**Acceptance criteria:**
- New version detected within 24 hours of release
- Signature verification prevents tampered updates
- User can defer updates
- Update failure does not crash the application

---

### Phase 19 - Production Release

**Acceptance criteria:**
- All items in Section 48 Definition of Done are checked
- Release build signed
- Installer tested on Windows 10 and Windows 11
- GitHub Release published
- Update endpoint live

---

## 48. Definition of Done

The project is **not complete** because it compiles. It is complete when
every item below is verified.

### Functionality

- [ ] `Alt + Space` reliably opens launcher on Windows 10 and 11
- [ ] Application search finds all Start Menu applications
- [ ] File search finds files in configured directories
- [ ] Calculator evaluates expressions correctly
- [ ] Built-in commands execute with required confirmation
- [ ] Results ranked by text relevance + usage + recency
- [ ] Repeated use personalizes ranking correctly
- [ ] Settings persist across restarts
- [ ] First-run onboarding completes successfully
- [ ] Incremental indexing keeps results current after file changes
- [ ] Web search result generated locally when enabled

### UX

- [ ] Full workflow operable without mouse
- [ ] Window appears centered on correct monitor
- [ ] Window appears within 150ms of hotkey press
- [ ] Focus goes to search input immediately
- [ ] Empty query shows recent items
- [ ] Escape hides window cleanly
- [ ] Theme works in dark, light, and system modes
- [ ] Result icons load without layout shift
- [ ] No visible layout jank during search

### Performance

- [ ] Idle CPU < 0.1% on reference hardware
- [ ] Idle RAM < 50 MB (Private Working Set)
- [ ] Hotkey to visible UI < 150ms
- [ ] Search response < 100ms at 10k records
- [ ] Initial indexing CPU < 25% sustained
- [ ] Peak indexing RAM < 150 MB

### Security

- [ ] Tauri capabilities restricted to minimum
- [ ] All IPC inputs validated in Rust
- [ ] No arbitrary shell execution via IPC
- [ ] No sensitive data in INFO-level logs
- [ ] Binary signed with valid certificate
- [ ] Installer signed
- [ ] Update signature verification implemented and tested

### Privacy

- [ ] All data stored locally only
- [ ] No network calls from core features
- [ ] History can be cleared
- [ ] History can be disabled
- [ ] Uninstaller offers data deletion

### Reliability

- [ ] Application survives Windows sleep/wake
- [ ] Application survives monitor disconnect/reconnect
- [ ] Application survives DPI change
- [ ] Application recovers from corrupted database
- [ ] Application recovers from watcher failure
- [ ] Second instance detected; first instance shown
- [ ] Graceful shutdown in < 3 seconds

### Testing

- [ ] Unit tests pass (Rust + TypeScript)
- [ ] Integration tests pass
- [ ] E2E core workflow test passes
- [ ] Large dataset test (50k files) passes
- [ ] All failure scenarios from Section 57 tested

### Installer

- [ ] Install completes without errors
- [ ] Uninstall removes all binaries
- [ ] Upgrade preserves data
- [ ] Per-user install available (no UAC)
- [ ] Startup entry created/removed correctly

### Documentation

- [ ] `README.md` complete
- [ ] `IMPLEMENTATION.md` up to date
- [ ] `CONTRIBUTING.md` complete
- [ ] `SECURITY.md` complete
- [ ] `CHANGELOG.md` updated for v1.0.0
- [ ] All IPC commands documented

### Release

- [ ] Version 1.0.0 tagged in Git
- [ ] GitHub Release published
- [ ] Signed installer attached to release
- [ ] Update endpoint live
- [ ] SHA-256 hash published

---

## 49. Feature Roadmap

### V1 (This specification)

- Application search and launching
- File and folder search and launching
- Calculator
- Safe built-in commands
- Usage-based personalization
- Keyboard navigation
- `Alt + Space` global hotkey (configurable)
- System tray
- Windows startup integration
- Settings UI
- Onboarding wizard
- Offline-first operation
- Optional web search provider
- Incremental filesystem indexing
- Production installer
- Signed binaries
- Auto updates

### V2 (Future — not in this specification)

| Feature | Notes |
|---|---|
| Clipboard history provider | Store and search clipboard items |
| Custom user-defined commands | With explicit security model |
| Plugin system | Simple, well-sandboxed |
| Application integrations | VS Code recent files, browser bookmarks |
| Workflow shortcuts | Multi-step macro sequences |
| Developer tools | Environment variables, port status |
| Advanced file preview | Quick look in launcher |
| AI-powered ranking | Local model only, opt-in |

### Explicit V1 exclusions

The following must **not** appear in V1:
- Cloud sync, accounts or login, cross-device features
- AI/ML inference infrastructure
- Analytics infrastructure
- Plugin marketplace
- Arbitrary shell command execution from search

---

## 50. Product Differentiation

Spotlight for Windows is **not** another app launcher.

It is a deliberate response to two dominant failure modes of existing launchers:

1. **Heavy launchers** that require cloud accounts, consume significant RAM, and feel like enterprise software.
2. **Stale open-source launchers** that are feature-rich but feel like developer tools rather than polished products.

### Differentiators

| Quality | Spotlight for Windows | Typical alternatives |
|---|---|---|
| **Idle RAM** | < 50 MB target | 100-300 MB common |
| **Idle CPU** | < 0.1% | Often 0.5-2% with polling |
| **Hotkey latency** | < 150ms | Often 200-500ms |
| **Network requirement** | None | Many require cloud account |
| **Privacy** | All local | Many upload queries |
| **UI philosophy** | Minimal, purposeful | Often feature-cluttered |
| **Search logic** | Rust, offline | Often Node.js with network fallback |
| **Windows integration** | Native Win32 APIs | Often abstracted away |
| **Installer** | Signed, professional | Often unsigned |

### The experience goal

> **A launcher that feels like part of Windows rather than another application running on Windows.**

---

## 51. Pricing / Business Model

This section defines strategic considerations. No pricing is committed at V1.

### Option A: Fully Free and Open Source

**Upside:** Maximum adoption, community growth, trust through transparency.
**Downside:** No direct revenue; sustainability depends on donations or future paid offering.

### Option B: Freemium (Recommended starting strategy)

**Free core:**
All V1 features — the core launcher must never be paywalled.

**Pro tier (V2+):**
- Advanced workflows
- Premium provider integrations (Notion, GitHub, Spotify)
- AI-assisted ranking (local model)
- Cloud sync / cross-device
- Team features

### Option C: One-Time Purchase

License: ~$19.99 one-time for Pro features.
Free tier: All V1 core features, forever.

### Option D: Subscription

Only justified if the product includes meaningful ongoing cloud infrastructure. Not appropriate for V1.

### Recommendation for V1

**Release V1 as free.** Focus on building an excellent product and user base.
Introduce Pro at V2 with genuinely valuable features.

### Payment infrastructure (when needed)

- Payment provider: Stripe or Paddle (Paddle handles VAT/tax automatically)
- License key delivery: email + in-app activation
- License validation: offline-capable (no server call required every launch)

### Free vs Pro boundary principle

The free tier must include everything a power user needs for the core keyboard-first workflow.
Pro must add **genuine value**, not simply paywalling existing functionality.

---

## 52. Product Metrics

Metrics must respect user privacy. The following are appropriate only with
explicit opt-in and aggregate processing (no PII).

**Preferred approach:** No analytics in V1. Instrument only crash reports
with explicit opt-in and no personally identifiable information.

### Metrics worth tracking (opt-in, aggregated, no PII)

| Metric | Why it matters |
|---|---|
| Daily active launcher opens | Core engagement |
| Searches per session | Indicates trust in search quality |
| Launches per session | Indicates workflow integration |
| Most common result types launched | Guides feature prioritization |
| Search-to-launch time (p50, p95) | UX quality signal |
| Crash rate | Reliability |
| Indexing failures | Reliability |

### Metrics explicitly excluded

- Query text, file paths, application names
- User identity or device fingerprint
- Network activity

---

## 53. Documentation

### Files and purposes

| File | Purpose |
|---|---|
| `README.md` | Product introduction, screenshots, quick start, features, philosophy |
| `IMPLEMENTATION.md` | This file. Master engineering specification. |
| `CONTRIBUTING.md` | How to contribute code, issues, and documentation |
| `SECURITY.md` | How to report vulnerabilities, security model overview |
| `CHANGELOG.md` | Version history, what changed, migration notes |
| `LICENSE` | License terms (MIT, Apache-2.0, or other chosen license) |
| `docs/architecture.md` | High-level architecture overview for contributors |
| `docs/search.md` | Detailed search engine and ranking explanation |
| `docs/decisions/` | Architecture Decision Records (ADRs) for major decisions |

### Architecture Decision Records

Every major non-obvious technical decision should be recorded as an ADR
in `docs/decisions/ADR-NNN-title.md`.

---

## 54. README Requirements

### Required sections

1. **Title and tagline** — what it is, in one sentence
2. **Screenshot** — launcher showing results, ideally animated GIF
3. **The pitch** — why this launcher is different (lightweight, private, instant)
4. **Features** — bulleted list of V1 capabilities
5. **Performance** — honest targets, not claims (link to benchmark methodology)
6. **Privacy** — explicitly state: local-only, no account, no telemetry
7. **Download** — link to latest release installer
8. **Usage** — "Press Alt + Space to open"
9. **Configuration** — brief settings overview
10. **Development** — prerequisites, build instructions
11. **Contributing** — link to CONTRIBUTING.md
12. **License** — one-line with link

### What the README must not include

- Performance numbers that cannot be verified
- Screenshots of features that don't exist yet
- A "coming soon" feature list longer than the actual feature list

---

## 55. Architecture Diagrams

### 1. Overall Architecture

```
+-----------------------------------------------------+
|                    Windows OS                       |
|                                                     |
|  +-------------------------------------------------+|
|  |          Spotlight for Windows                  ||
|  |                                                 ||
|  |  +---------------+    +------------------+     ||
|  |  |  Rust Core    |<-->|  React/TS UI     |     ||
|  |  |               |IPC |  (WebView2)      |     ||
|  |  | - Hotkey      |    | - Launcher UI    |     ||
|  |  | - Indexer     |    | - Settings       |     ||
|  |  | - Search      |    | - Onboarding     |     ||
|  |  | - Database    |    +------------------+     ||
|  |  | - Launcher    |                             ||
|  |  | - Tray        |                             ||
|  |  | - Settings    |                             ||
|  |  +------+--------+                             ||
|  |         |                                      ||
|  |  +------v--------+                             ||
|  |  |  SQLite DB    |                             ||
|  |  | spotlight.db  |                             ||
|  |  +---------------+                             ||
|  +-------------------------------------------------+|
|                                                     |
|  Win32 APIs: Hotkey, Shell, Registry, Explorer      |
+-----------------------------------------------------+
```

### 2. Process Lifecycle

```
Windows boots
      |
      v
HKCU\Run: spotlight.exe --startup
      |
      v
Process starts (Rust main())
      |
      v
Single-instance check
  |-> another instance? -> show existing -> exit
      | (first instance)
      v
Initialize AppState
  +-- Open SQLite
  +-- Run migrations
  +-- Load settings
  +-- Start logging
      |
      v
Register global hotkey
      |
      v
Start system tray
      |
      v
Start filesystem watcher
      |
      v
Schedule background indexing (if stale)
      |
      v
IDLE <--------------------------------------------------+
      |                                                  |
      | [Alt+Space]              [Quit]                  |
      v                             v                    |
Show window                   Graceful shutdown          |
Focus input                         |                    |
      |                       Unregister hotkey          |
      |                       Stop watcher               |
      | [User searches]       Flush DB + Exit            |
      v                                                  |
Search -> results -> render                              |
      |                                                  |
      | [Enter]           [Escape]                       |
      v                       v                          |
Launch result           Hide window ───────────────────> +
Hide window
Record usage
```

### 3. Search Pipeline

```
React: user types "code"
         |
         | (debounce 50ms)
         v
React: invoke("search", { query: "code" })
         |
         v
Rust: commands::search()
         |
         v
SearchEngine::execute("code")
         |
         +-- QueryParser: general query, 4 chars, no special prefix
         |
         +-- Parallel providers:
         |       +-- AppsProvider: FTS5 MATCH 'code*' -> [VSCode, ...]
         |       +-- FilesProvider: FTS5 MATCH 'code*' -> [code.txt, ...]
         |       +-- FoldersProvider: FTS5 MATCH 'code*' -> [code/, ...]
         |       +-- CommandsProvider: no match
         |       +-- CalculatorProvider: not math, skip
         |
         +-- Collect candidates (Vec<SearchCandidate>)
         |
         +-- Ranking::score_all() -> sorted by score descending
         |
         +-- Take top 12 -> Vec<SearchResult>
         |
         v
IPC response -> React
         |
         v
React: render result list (memoized items)
```

### 4. Indexing Pipeline

```
File created: C:\Users\User\Documents\report.docx
         |
         v
notify crate: Create event
         |
         v
watcher.rs: Normalize event
  +-- Is path in configured watch dirs? Yes
  +-- Is extension ignored? No
  +-- Is file hidden/system? No
         |
         v
Event queued (tokio channel)
         | (batch flush: 500ms or 100 events)
         v
indexer/manager.rs: Process batch
         |
         v
files.rs: stat file -> build FileRecord
         |
         v
database/files.rs: INSERT OR REPLACE into files table
                   FTS5 content table updated
         |
         v
Emit IPC event: "index_updated" { added: 1, removed: 0 }
```

### 5. Database Relationships

```
+------------------+     +------------------+
|   applications   |     |      files       |
+------------------+     +------------------+
| id (PK)          |     | id (PK)          |
| display_name     |     | name             |
| exe_path         |     | display_name     |
| shortcut_path    |     | extension        |
| icon_path        |     | path             |
| source           |     | parent_dir       |
| indexed_at       |     | modified_at      |
+--------+---------+     +--------+---------+
         |                        |
         +----------+-------------+
                    | result_id (logical FK)
                    v
          +---------+---------+
          |       usage       |
          +-------------------+
          | result_id         |
          | result_type       |
          | launch_count      |
          | last_launched_at  |
          +--------+----------+
                   |
                   | result_id
                   v
          +--------+----------+
          |      history      |
          +-------------------+
          | id (PK, auto)     |
          | query             |
          | result_id         |
          | result_type       |
          | result_name       |
          | launched_at       |
          +-------------------+

+-------------------+     +-------------------+
|     settings      |     |     metadata      |
+-------------------+     +-------------------+
| key (PK)          |     | key (PK)          |
| value (JSON)      |     | value             |
+-------------------+     +-------------------+
```

### 6. IPC Flow

```
React Component
      |
      | import { search } from '../lib/ipc'
      v
src/lib/ipc.ts
      |
      | invoke<SearchResponse>('search', { query })
      v
Tauri IPC bridge (WebView2 <-> Rust)
      |
      v
src-tauri/src/commands.rs
      | #[tauri::command] async fn search(...)
      v
SearchEngine::execute(query)
      |
      v
Vec<SearchResult>
      |
      v
Serialized to JSON via serde_json
      |
      v
Tauri IPC bridge
      |
      v
Deserialized in TypeScript as SearchResult[]
      |
      v
React: setState(results) -> re-render
```

### 7. Startup Flow

```
User logs into Windows
         |
         v
Windows reads HKCU\Run keys
         |
         v
spotlight.exe --startup
         |
      +--+-----------------------------+
      | Single instance check         |
      | (named mutex)                 |
      +--+-----------------------------+
         | (first instance)
         v
      +--+-----------------------------+
      | Initialize:                   |
      |  - DB connection              |
      |  - Settings load              |
      |  - Logging start              |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Register Alt+Space            |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Start tray icon               |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Start filesystem watcher      |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Schedule indexing?            |  <- Last indexed > 2h ago?
      | (background thread)           |
      +--+-----------------------------+
         |
         v
      IDLE (window hidden)
```

### 8. Release Pipeline

```
Developer: git tag v1.0.0
         |
         v
GitHub Actions: release.yml triggered
         |
         v
      +--+-----------------------------+
      | Checkout repository           |
      | Install dependencies          |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Run full test suite           |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Build release binary          |
      | (cargo build --release)       |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Sign spotlight.exe            |
      | (signtool + EV cert)          |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Build installer               |
      | (tauri bundle --target wix)   |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Sign installer                |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Create GitHub Release         |
      | Upload installer              |
      | Publish SHA-256 hash          |
      +--+-----------------------------+
         |
         v
      +--+-----------------------------+
      | Publish update JSON           |
      | to update endpoint            |
      +--+-----------------------------+
         |
         v
      Release live
```

---

## 56. Data Flow

### User interaction data flow

```
Physical keyboard
      | keystroke
      v
Windows keyboard subsystem
      | registered global hotkey
      v
Rust: tauri-plugin-global-shortcut callback
      |
      v
Rust: window.show() + window.set_focus()
      |
      v
WebView2 renders React app
      |
      v
React: search input receives focus
      | user types characters
      v
React: onChange event
      | debounce 50ms
      v
React: invoke('search', { query })
      | Tauri IPC (WebView2 -> Rust)
      v
Rust: search engine
      | SQLite FTS5 query
      v
SQLite: FTS5 index lookup
      | candidate rows
      v
Rust: ranking algorithm
      | sorted SearchResult[]
      v
Rust: serialize to JSON
      | Tauri IPC (Rust -> WebView2)
      v
React: setState(results)
      | memoized re-render
      v
WebView2: paint result list
      | user sees results
      v
User presses Enter
      |
      v
React: invoke('launch', { id, result_type })
      |
      v
Rust: window.hide()
Rust: launcher::launch(path)
Rust: database::usage::increment(id)
Rust: database::history::record(query, id)
      |
      v
Windows: application/file/folder opens
```

### Filesystem indexing data flow

```
File created/modified/deleted/renamed
      | (Windows ReadDirectoryChangesW)
      v
notify crate: raw filesystem event
      |
      v
watcher.rs: normalize + filter
  +-- Not in watched directories? -> discard
  +-- Hidden/system file? -> discard
  +-- Ignored extension? -> discard
      |
      v
tokio channel: event queued
      | (batch: 500ms timeout or 100 events)
      v
indexer/manager.rs: batch processing
  +-- Created/Modified -> stat file -> build FileRecord -> upsert
  +-- Deleted -> remove from files table
  +-- Renamed -> update path in files table
      | (single SQLite transaction for entire batch)
      v
SQLite: files table updated
SQLite: files_fts content table trigger updates FTS5
      |
      v
Rust: emit('index_updated', { added, removed })
```

---

## 57. Failure Scenarios

### Database unavailable

| Aspect | Behavior |
|---|---|
| **Detection** | rusqlite returns error on open or query |
| **Recovery** | Attempt reopen with exponential backoff (1s, 5s, 30s). After 3 attempts, enter degraded mode. |
| **User experience** | Tray balloon: "Search database unavailable." |
| **App continues?** | Yes |

### Corrupted database

| Aspect | Behavior |
|---|---|
| **Detection** | `PRAGMA integrity_check` returns errors at startup |
| **Recovery** | Rename corrupted file with `.corrupt` suffix, create fresh DB, re-index |
| **User experience** | Tray balloon: "Search index rebuilt due to data error." |
| **App continues?** | Yes, after rebuild |

### Permission denied (indexing)

| Aspect | Behavior |
|---|---|
| **Detection** | `std::io::ErrorKind::PermissionDenied` |
| **Recovery** | Skip directory, continue indexing |
| **User experience** | Silent |
| **App continues?** | Yes |

### Shortcut conflict

| Aspect | Behavior |
|---|---|
| **Detection** | `tauri-plugin-global-shortcut` registration returns error |
| **Recovery** | Retain previous shortcut; notify user |
| **User experience** | Tray balloon: "Spotlight hotkey is in use. Change it in Settings." |
| **App continues?** | Yes |

### Application deleted after indexing

| Aspect | Behavior |
|---|---|
| **Detection** | Watcher event, or launch failure |
| **Recovery** | Remove from index on watcher event or on next re-index |
| **User experience** | "Application not found" in result subtitle if user tries to launch |
| **App continues?** | Yes |

### Indexing interrupted

| Aspect | Behavior |
|---|---|
| **Detection** | Partial indexing state detected at next startup |
| **Recovery** | Resume from beginning on next startup |
| **User experience** | Silent |
| **App continues?** | Yes |

### Windows sleep

| Aspect | Behavior |
|---|---|
| **Detection** | WM_POWERBROADCAST message (PBT_APMSUSPEND) |
| **Recovery** | On wake: re-register hotkey, verify watcher active |
| **User experience** | Seamless; hotkey works after wake |
| **App continues?** | Yes |

### Monitor disconnected

| Aspect | Behavior |
|---|---|
| **Detection** | On next window show, `MonitorFromPoint` returns different monitor |
| **Recovery** | Recalculate position using current monitor at show time |
| **User experience** | Launcher appears on primary monitor correctly |
| **App continues?** | Yes |

### DPI changed

| Aspect | Behavior |
|---|---|
| **Detection** | WM_DPICHANGED or monitor DPI re-query |
| **Recovery** | Recalculate window size and position on next show |
| **App continues?** | Yes |

### Network unavailable

| Aspect | Behavior |
|---|---|
| **Detection** | Update check returns network error |
| **Recovery** | Skip silently. Web search generates local URL; browser handles offline. |
| **User experience** | Core search completely unaffected. |
| **App continues?** | Yes, fully |

### Updater failure

| Aspect | Behavior |
|---|---|
| **Detection** | Download error or signature mismatch |
| **Recovery** | Discard partial download; log error; retry on next check |
| **User experience** | Silent failure; user can manually check via tray menu |
| **App continues?** | Yes |

### Duplicate process

| Aspect | Behavior |
|---|---|
| **Detection** | `tauri-plugin-single-instance` detects existing instance |
| **Recovery** | Signal first instance to show launcher; second instance exits |
| **User experience** | Launcher window appears; nothing unexpected |
| **App continues?** | Yes (first instance) |

### Invalid query

| Aspect | Behavior |
|---|---|
| **Detection** | Input validation in Rust `search` command handler |
| **Recovery** | Return empty results; do not crash |
| **User experience** | No results shown |
| **App continues?** | Yes |

### Malformed calculator expression

| Aspect | Behavior |
|---|---|
| **Detection** | Expression evaluator returns parse error |
| **Recovery** | Fall through to regular search; do not show calculator result |
| **User experience** | Regular search results shown |
| **App continues?** | Yes |

### Watcher thread failure

| Aspect | Behavior |
|---|---|
| **Detection** | Watcher thread send handle returns error |
| **Recovery** | Restart watcher with exponential backoff (1s, 5s, 30s) |
| **User experience** | Incremental indexing paused temporarily |
| **App continues?** | Yes |

---

## 58. Performance Budget

These are design-time budgets and measurement targets, not marketing claims.

| Budget area | Target | How measured |
|---|---|---|
| **Startup -> hotkey registered** | < 2,000ms from process start | Log timestamp diff |
| **Startup -> first search ready** | < 2,500ms from process start | Hotkey ready + DB open |
| **Hotkey -> visible window** | < 150ms | Timestamp diff: hotkey event to WM_PAINT |
| **Window focus -> first character** | < 16ms (one frame) | Input latency measurement |
| **Search response (10k items)** | < 100ms total | IPC round-trip timing |
| **Search response (50k items)** | < 200ms total | IPC round-trip timing |
| **IPC serialization overhead** | < 5ms | Measured in isolation |
| **SQLite FTS5 query** | < 30ms at 50k records | Criterion benchmark |
| **Ranking computation** | < 10ms for 100 candidates | Criterion benchmark |
| **Initial indexing (10k files)** | < 20s | Timed in integration test |
| **Initial indexing (50k files)** | < 60s | Timed in integration test |
| **Incremental update latency** | < 1,000ms (event to index) | Measured via test |
| **Idle CPU** | < 0.1% average over 10 min | Windows Process Monitor |
| **Idle RAM** | < 50 MB Private Working Set | Windows Task Manager |
| **Active RAM (searching)** | < 80 MB Private Working Set | Task Manager during use |
| **Peak indexing RAM** | < 150 MB Private Working Set | Task Manager during indexing |
| **Peak indexing CPU** | < 25% sustained | Process Monitor during indexing |
| **JS bundle (gzipped)** | < 300 KB | Vite build output |
| **Installer size** | < 30 MB | Final installer artifact |
| **Binary size (.exe)** | < 15 MB | Release build artifact |

---

## 59. Performance Regression Policy

A performance regression is defined as any measurable increase in a
budget metric beyond the regression threshold.

### Regression thresholds

| Metric | Regression threshold |
|---|---|
| Startup time | > 125% of target |
| Hotkey latency | > 125% of target |
| Search latency | > 125% of target |
| Idle CPU | > 150% of target |
| Idle RAM | > 125% of target |
| Bundle size | > 110% of target |
| Installer size | > 110% of target |

### Regression process

1. Regression detected (CI benchmark or manual check)
2. PR author notified — PR cannot merge until resolved
3. Author must provide before/after benchmark output
4. If regression is unavoidable, document the new baseline and update targets in this document
5. Reviewers must explicitly approve the new baseline

### Policy statement

**Performance is a feature.** Every change that touches search, indexing,
IPC, or rendering must be accompanied by evidence that it does not regress performance.

---

## 60. Code Quality Rules

### Rust

- All code formatted with `rustfmt` (enforced in CI)
- All warnings treated as errors in CI: `cargo clippy -- -D warnings`
- No `#[allow(dead_code)]` without documented reason
- No `unwrap()` in non-test code without documented invariant
- No `panic!` in non-test code
- No `todo!()` or `unimplemented!()` in merged code
- All public functions have doc comments (`///`)
- Module files should generally be < 300 lines
- No raw SQL outside `src-tauri/src/database/`

### TypeScript

- `"strict": true` in `tsconfig.json`
- ESLint with `@typescript-eslint/strict` rules
- Prettier for formatting (enforced in CI)
- No `any` types without documented justification
- No `@ts-ignore` without documented justification
- No `console.log` in production code
- All exported functions have JSDoc comments
- All IPC types mirror the Rust types exactly

### General

- Meaningful, descriptive names (no single-letter variables outside loop indices)
- Comments explain **why**, not **what**
- Small, focused functions (do one thing)
- No functions longer than 50 lines without documented justification
- Tests required for: ranking algorithm, calculator parser, IPC commands, keyboard navigation

---

## 61. AI Implementation Process

Every AI coding agent must follow this process:

```
Step 1: Read IMPLEMENTATION.md
        Do not skip. Every section is relevant to some change.
        |
        v
Step 2: Inspect repository
        Understand current file structure and implementations.
        |
        v
Step 3: Understand current architecture
        Trace the relevant code path end-to-end.
        Identify which modules will be affected.
        |
        v
Step 4: Identify the relevant module
        Map the feature to its home directory (Section 6 + 7).
        |
        v
Step 5: Plan the change
        Write a brief plan before writing code.
        Identify all files that will be modified.
        Identify any new dependencies needed (justify each).
        |
        v
Step 6: Implement the smallest correct change
        Do not refactor unrelated code.
        Do not change architecture without updating this document first.
        |
        v
Step 7: Run tests
        cargo test --all
        npx vitest run
        All must pass.
        |
        v
Step 8: Run lint and typecheck
        cargo clippy -- -D warnings
        cargo fmt -- --check
        npx tsc --noEmit
        npx eslint . --max-warnings 0
        All must pass.
        |
        v
Step 9: Build
        npm run tauri build
        Must succeed.
        |
        v
Step 10: Performance check (if relevant)
         If the change touches search, indexing, ranking, or IPC:
         Run benchmarks before and after.
         Verify no regressions.
        |
        v
Step 11: Security review (if relevant)
         If the change touches IPC commands, launching, or settings:
         Verify input validation is complete.
         Verify no new shell execution paths.
        |
        v
Step 12: Update documentation
         Update IMPLEMENTATION.md if architecture changed.
         Add/update doc comments.
         Update CHANGELOG.md.
        |
        v
Step 13: Report exactly what changed
         List every file modified.
         Explain every decision.
         Flag any deviations from this specification.
```

---

## 62. Avoid Overengineering

Spotlight for Windows is a desktop application, not a platform.

### V1 must NOT include

| Anti-pattern | Why not |
|---|---|
| Microservices architecture | Single process is correct for a launcher |
| Cloud backend | Offline-first; no cloud in V1 |
| Message queue (Kafka, RabbitMQ) | Overkill; use tokio channels |
| Multiple databases | SQLite is sufficient for this scale |
| Plugin system | Out of scope for V1; adds complexity and attack surface |
| AI/ML inference | Not needed for excellent local ranking |
| Analytics infrastructure | Not in V1; use opt-in crash reports only |
| gRPC or complex IPC protocols | Tauri IPC is sufficient |
| Dependency injection framework | Not idiomatic Rust; use plain structs and Arc |

### Simplicity tests

Before implementing any design pattern, ask:

1. Can this be done with a plain function?
2. Can this be done without a new file?
3. Can this be done with existing data structures?
4. Does the abstraction make the code easier to understand or harder?
5. Will a future developer reading this code understand it without reading the commit message?

If the answer to 5 is "no," simplify the implementation.

---

## 63. Production Quality

Shipping a launcher that merely compiles and works on a developer machine is not production quality.

Production quality for Spotlight for Windows means:

| Quality area | Standard |
|---|---|
| **Graceful failures** | Every subsystem fails without crashing the whole app |
| **Installer** | Signed, silent install capable, upgrade preserves data |
| **Uninstaller** | Removes binaries; asks before deleting user data |
| **Updater** | Signed updates, user-controlled, survives download failure |
| **Startup reliability** | Works after reboot, sleep, and update |
| **Resource consumption** | Within budgets defined in Section 58 |
| **Indexing robustness** | Handles permission errors, missing directories, large trees |
| **Keyboard UX** | 100% operable without mouse |
| **Accessibility** | Screen reader accessible search input and results |
| **Windows compatibility** | Tested on clean Windows 10 and Windows 11 VMs |
| **Logging** | Useful diagnostic information without sensitive data |
| **Documentation** | README, CONTRIBUTING, SECURITY, CHANGELOG complete |
| **Tests** | Unit, integration, and E2E passing |

The application must be ready for a real user to install on their primary
workstation and rely on it every day.

---

## 64. Final Principle

```
Always ready.
Never in the way.

Fast enough to feel instant.
Light enough to forget it is running.
Private enough to trust.
Simple enough to understand.
Powerful enough to become part of the daily workflow.
```

Every implementation decision in this project should be evaluated against
this principle.

If a feature makes the launcher slower, heavier, more intrusive, less
private, harder to understand, or less reliable — it does not belong in
Spotlight for Windows.

The goal is not to add more. The goal is to build something that
disappears into the workflow so completely that losing it would feel like
losing a native Windows feature.

---

*This document is the master engineering contract for Spotlight for Windows.*
*It must be kept current. It must be read before any code is written.*
*It is the single source of truth.*

**Last updated:** 2026-08-26
**Status:** Pre-Implementation
**Version:** 1.0.0 specification
