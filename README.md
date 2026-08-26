# Spotlight for Windows

> **A Windows-first, keyboard-first, offline-first application launcher and search interface.**

[![CI](https://github.com/spotlight-windows/spotlight/actions/workflows/ci.yml/badge.svg)](https://github.com/spotlight-windows/spotlight/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## The Pitch

Existing Windows launchers are often heavy, require cloud accounts, poll in the background, or feel like clunky developer tools.

**Spotlight for Windows** is engineered with extreme performance discipline:

- **Instant:** Opens in under 150ms from hotkey press.
- **Lightweight:** < 50 MB idle RAM, < 0.1% idle CPU.
- **Private & Local:** 100% offline-first; all data stays on your machine.
- **Native:** Windows-first design using native Win32 APIs and SQLite FTS5.

---

## Features

- **Global Hotkey:** Press `Alt + Space` (configurable) to toggle anywhere.
- **Comprehensive Application Discovery:** Automatically discovers applications from:
  - User and System Start Menu shortcuts
  - User and Public Desktop shortcuts
  - User-installed applications (`%LOCALAPPDATA%\Programs`)
  - Microsoft Store / WindowsApps execution aliases
  - Windows Registry App Paths (`HKLM` / `HKCU`)
  - Standard Windows system utilities (Notepad, Calculator, Paint, Task Manager, PowerShell, Command Prompt, Registry Editor, etc.)
- **Local File & Folder Search:** High-performance full-text and partial search across user folders (Desktop, Documents, Downloads, Pictures, Videos, and Music).
- **Native Launching:**
  - Files (`.txt`, `.pdf`, documents, media) open directly in their Windows-associated default application.
  - Folders open directly in File Explorer.
  - Applications launch with full argument and shortcut support.
- **Real-Time Filesystem Watching:** Incremental updates via Windows directory change notifications.
- **Smart Personalized Ranking:** Rewards frequently and recently launched results deterministically.
- **Built-in Calculator:** Instant math evaluation (e.g. `= 25 * 4`, `sqrt(144)`).
- **Safe System Commands:** Quick system operations (`> lock`, `> task manager`, etc.).
- **System Tray Presence:** Lightweight background process with quick preferences access.
- **Multi-Monitor Aware:** Centers automatically on the monitor currently containing your mouse cursor.

---

## Performance Targets

All performance metrics are engineering design budgets:

| Metric                          | Target                      |
| ------------------------------- | --------------------------- |
| **Hotkey to visible UI**        | < 150 ms                    |
| **Search response (10k items)** | < 100 ms                    |
| **Idle CPU**                    | < 0.1% average              |
| **Idle RAM**                    | < 50 MB                     |
| **Frontend Bundle Size**        | ~52 KB gzipped JS, 2 KB CSS |

---

## Privacy Guarantee

- **100% Local Storage:** SQLite database stored in `%APPDATA%\SpotlightForWindows\spotlight.db`.
- **No Cloud Accounts:** No login, no telemetry, no tracking.
- **Zero Network Dependency for Core Search:** Fully functional without an internet connection.

---

## Usage

1. Press **`Alt + Space`** to open the launcher.
2. Type your query (e.g. `notepad`, `antigravity`, `notes.txt`, `= 50 * 12`, `> lock`).
3. Navigate results using **`Up` / `Down`** arrow keys or **`Tab`**.
4. Press **`Enter`** (or click a result) to launch the app, open the file, or explore the folder.
5. Press **`Escape`** to dismiss.
6. Press **`Ctrl + ,`** to open Preferences.

---

## Development

### Prerequisites

- Node.js (v18+)
- Rust & Cargo (stable toolchain)
- Visual Studio C++ Build Tools

### Getting Started

```powershell
# Clone the repository
git clone https://github.com/spotlight-windows/spotlight.git
cd "spotlight for windows"

# Install frontend dependencies
npm install

# Run frontend development server
npm run dev

# Run full Tauri desktop application
npm run tauri dev
```

### Verification & Testing

```powershell
# Run frontend checks
npm run typecheck    # Strict TypeScript checks
npm run lint         # ESLint checks
npm run format:check # Prettier code style checks
npm run build        # Production bundle build

# Run backend Rust tests
cd src-tauri
cargo test
```

---

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on code style, conventional commit rules, and PR guidelines.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

# Qevik
