# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-08-26

### Added

- **Core:** Tauri 2 background desktop launcher shell with single-instance enforcement.
- **Hotkey:** Global `Alt + Space` registration with active cursor monitor positioning.
- **Indexing:** Start Menu, Desktop, and user folder indexing with real-time `notify` filesystem watching.
- **Database:** SQLite with WAL mode, FTS5 sync triggers, and usage/history tracking.
- **Search Engine:** Multi-provider coordinator (Apps, Files, Folders, Commands, Calculator, Web).
- **Ranking:** Deterministic scoring model combining text relevance, match quality, type priority, frequency, and recency.
- **UI:** Minimalist Windows-native UI with dark/light themes, Inter font, and keyboard navigation.
- **Settings & Onboarding:** Lazy-loaded Preferences screen and first-run welcome wizard.
