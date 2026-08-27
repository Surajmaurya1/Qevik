# Qevik

A fast, keyboard-first desktop launcher and file search tool for Windows 10 and 11.

[![CI](https://github.com/Surajmaurya1/Qevik/actions/workflows/ci.yml/badge.svg)](https://github.com/Surajmaurya1/Qevik/actions/workflows/ci.yml)
[![Release](https://github.com/Surajmaurya1/Qevik/actions/workflows/release.yml/badge.svg)](https://github.com/Surajmaurya1/Qevik/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Overview

Windows search is often slow, indexes unwanted directories, or sends search queries to web engines by default.

**Qevik** is a lightweight desktop utility designed to open applications, locate local files, calculate expressions, and run system commands instantly from a single keyboard shortcut. It runs locally on your machine, stores its index in a local SQLite database, and requires no online accounts or external services.

---

## Features

- **Global Hotkey Access**: Open and dismiss the launcher from anywhere using `Alt + Space` (customizable).
- **Application Search**: Automatically finds software from:
  - Start Menu shortcuts (User & System)
  - Desktop shortcuts
  - User program folders (`%LOCALAPPDATA%\Programs`)
  - Windows execution aliases (`wt.exe`, `winget.exe`, etc.)
  - Standard Windows system tools (`Notepad`, `Calculator`, `Paint`, `Task Manager`, `cmd`, `PowerShell`)
  - Registry App Paths (`HKLM` and `HKCU`)
- **Local File & Folder Search**: Full-text searching across common user libraries (Desktop, Documents, Downloads, Pictures, Videos, Music) using SQLite FTS5. Files open in their default Windows apps; folders open directly in File Explorer.
- **Built-in Calculator**: Type math expressions directly in the search box (e.g. `2 + 2`, `(45 * 12) + 180`, `25 * 4`). Pressing `Enter` copies the result to the clipboard.
- **System Commands**: Run Windows system actions quickly using the `>` prefix:
  - `> lock` — Lock your Windows workstation
  - `> task manager` — Launch Windows Task Manager
  - `> recycle bin` — Open the Recycle Bin in File Explorer
  - `> settings` — Open Windows System Settings
- **Search Ranking & History**: Automatically ranks frequently and recently opened apps and files higher over time. Empty queries display your recent launches.
- **Multi-Monitor Awareness**: Automatically centers the search bar on whichever monitor your mouse cursor is currently active on.
- **System Tray Integration**: Runs quietly in the background with a system tray menu to toggle visibility, open settings, or exit.

---

## How It Works

1. Press **`Alt + Space`** to open the search bar.
2. Start typing to filter applications, files, system actions, or math calculations.
3. Use the **`Up`** and **`Down`** arrow keys (or **`Tab`** / **`Shift + Tab`**) to navigate results.
4. Press **`Enter`** to launch the selected item or copy the calculation result.
5. Press **`Escape`** or click outside to dismiss the window.

### Keyboard Shortcuts

| Shortcut                  | Action                                   |
| :------------------------ | :--------------------------------------- |
| **`Alt + Space`**         | Toggle launcher visibility               |
| **`Enter`**               | Open selected result / execute command   |
| **`Up` / `Down`**         | Move selection up / down                 |
| **`Tab` / `Shift + Tab`** | Cycle forward / backward through results |
| **`Escape`**              | Close / hide launcher                    |
| **`Ctrl + L`**            | Clear search input                       |
| **`Ctrl + ,`**            | Open Preferences / Settings              |

---

## Installation

### Pre-Built Binaries

Download the latest version from the [**Releases Page**](https://github.com/Surajmaurya1/Qevik/releases/latest):

| Package                | Format        | Description                                                                  |
| :--------------------- | :------------ | :--------------------------------------------------------------------------- |
| **Standard Installer** | `.exe` (NSIS) | Windows setup installer with Start Menu and Desktop shortcuts (Recommended). |
| **MSI Package**        | `.msi` (WiX)  | Windows Installer package for system-wide deployment.                        |
| **Portable Binary**    | `.exe`        | Standalone executable that runs directly without installation.               |

---

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or higher)
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (stable toolchain)
- Visual Studio C++ Build Tools (with the Windows 10/11 SDK component)

### Running Locally

```powershell
# 1. Clone the repository
git clone https://github.com/Surajmaurya1/Qevik.git
cd Qevik

# 2. Install dependencies
npm install

# 3. Start development mode
npm run tauri dev
```

### Verification Suite

Run frontend and backend checks locally:

```powershell
# Frontend linting and typechecking
npm run typecheck
npm run lint
npm run format:check

# Backend tests and lints
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

---

## Build

To build the production executable and installer packages:

```powershell
npm run tauri build
```

The output files are generated in:

- **NSIS Installer**: `target/release/bundle/nsis/Spotlight for Windows_1.0.0_x64-setup.exe`
- **MSI Installer**: `target/release/bundle/msi/Spotlight for Windows_1.0.0_x64_en-US.msi`
- **Standalone Binary**: `target/release/spotlight-for-windows.exe`

---

## Project Structure

```text
├── .github/workflows/    # CI/CD and release automation
├── src/                  # Frontend UI (React + TypeScript + Vite)
│   ├── components/       # SearchInput, ResultList, ResultItem, Icon
│   ├── features/         # Launcher, Settings, Onboarding screens
│   ├── lib/              # Tauri IPC bridge client
│   ├── stores/           # UI state management
│   └── styles/           # Design system tokens and styling
└── src-tauri/            # Backend (Rust + Tauri v2)
    ├── migrations/       # SQLite schema and FTS5 triggers
    └── src/
        ├── commands.rs   # Tauri invoke command handlers
        ├── core/         # Application setup and state
        ├── database/     # SQLite connection and search index models
        ├── hotkey/       # Global shortcut registration
        ├── indexer/      # Windows app discovery and file scanner
        ├── search/       # Query parser, providers, and ranking engine
        ├── tray/         # System tray setup and context menu
        └── windows/      # Multi-monitor positioning and focus handling
```

---

## Tech Stack

- **Backend**: Rust, [Tauri v2](https://v2.tauri.app/), native Win32 APIs (`windows-rs`).
- **Database**: SQLite 3 with FTS5 virtual tables (`rusqlite`).
- **Frontend**: React 18, TypeScript, Vite, [Zustand](https://github.com/pmndrs/zustand), [Lucide React](https://lucide.dev/).
- **Evaluation**: `evalexpr` for arithmetic parsing.

---

## Roadmap

- [ ] Custom plugin and extension system
- [ ] File content search integration
- [ ] Configurable search folders in settings
- [ ] Custom themes and accent colors

---

## Contributing

Contributions are welcome.

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/my-feature`).
3. Commit your changes (`git commit -m "feat: add my feature"`).
4. Run `npm run typecheck`, `npm run lint`, and `cargo test --all` to make sure checks pass.
5. Push to the branch (`git push origin feature/my-feature`).
6. Open a Pull Request.

---

## License

This project is licensed under the [MIT License](LICENSE).
