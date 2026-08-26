# System Architecture

Spotlight for Windows is split into two strict layers:

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

## Boundaries

- **Rust Backend:** Responsible for OS integration, global shortcuts, database management, file indexing, filesystem watching, query processing, and process launching.
- **React Frontend:** Purely responsible for rendering the UI view, debouncing inputs (60ms), and transmitting keyboard interactions to the Rust core via typed IPC invocations.
