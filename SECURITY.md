# Security Policy

## Reporting Security Vulnerabilities

If you discover a security vulnerability in Spotlight for Windows, please report it responsibly:

1. **Do NOT open a public GitHub issue.**
2. Email your findings to `security@spotlightforwindows.local` or submit a private security advisory on GitHub.
3. Include detailed steps to reproduce the issue, proof-of-concept payload, and the affected operating system version.
4. We aim to acknowledge receipt of vulnerability reports within 48 hours.

---

## Security Model & Principles

Spotlight for Windows is designed around strict local-first, offline-first security guarantees:

### 1. No Arbitrary Shell Execution

- The frontend UI layer cannot send arbitrary shell commands to be executed.
- Result launching uses parameterized Win32 `ShellExecuteW` APIs with validated file paths.
- Built-in commands are registered in a closed, immutable in-memory catalog.

### 2. Strict IPC Boundary

- Tauri 2 capability files (`capabilities/default.json`) strictly limit frontend permissions.
- All IPC arguments are strongly typed, sanitized, and validated in Rust before processing.

### 3. Local-Only Data Storage

- SQLite database (`spotlight.db`) resides in the user's local `%APPDATA%` directory.
- No launch queries, search terms, or indexed filenames are ever transmitted over the network.
- Web search provider generates local browser search URLs only; no network requests are made by the background process.

### 4. Sandboxed Expression Evaluation

- The calculator provider uses the pure-Rust `evalexpr` evaluator with no access to system commands, IO, or arbitrary code execution.

### 5. Filesystem Traversal Protections

- File indexers validate directory paths and follow symlinks only one level deep to prevent infinite loop cycles or directory escape vulnerabilities.
