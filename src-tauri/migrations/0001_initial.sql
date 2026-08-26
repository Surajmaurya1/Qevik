-- Applications Table
CREATE TABLE IF NOT EXISTS applications (
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

CREATE VIRTUAL TABLE IF NOT EXISTS applications_fts USING fts5(
    display_name,
    exe_path,
    content='applications',
    content_rowid='rowid'
);

-- Triggers to keep FTS5 synchronized with applications table
CREATE TRIGGER IF NOT EXISTS applications_ai AFTER INSERT ON applications BEGIN
  INSERT INTO applications_fts(rowid, display_name, exe_path) VALUES (new.rowid, new.display_name, new.exe_path);
END;

CREATE TRIGGER IF NOT EXISTS applications_ad AFTER DELETE ON applications BEGIN
  INSERT INTO applications_fts(applications_fts, rowid, display_name, exe_path) VALUES('delete', old.rowid, old.display_name, old.exe_path);
END;

CREATE TRIGGER IF NOT EXISTS applications_au AFTER UPDATE ON applications BEGIN
  INSERT INTO applications_fts(applications_fts, rowid, display_name, exe_path) VALUES('delete', old.rowid, old.display_name, old.exe_path);
  INSERT INTO applications_fts(rowid, display_name, exe_path) VALUES (new.rowid, new.display_name, new.exe_path);
END;

-- Files Table
CREATE TABLE IF NOT EXISTS files (
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

CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent_dir);
CREATE INDEX IF NOT EXISTS idx_files_extension ON files(extension);

CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    name,
    display_name,
    path,
    content='files',
    content_rowid='rowid'
);

-- Triggers to keep FTS5 synchronized with files table
CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
  INSERT INTO files_fts(rowid, name, display_name, path) VALUES (new.rowid, new.name, new.display_name, new.path);
END;

CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
  INSERT INTO files_fts(files_fts, rowid, name, display_name, path) VALUES('delete', old.rowid, old.name, old.display_name, old.path);
END;

CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
  INSERT INTO files_fts(files_fts, rowid, name, display_name, path) VALUES('delete', old.rowid, old.name, old.display_name, old.path);
  INSERT INTO files_fts(rowid, name, display_name, path) VALUES (new.rowid, new.name, new.display_name, new.path);
END;

-- Folders Table
CREATE TABLE IF NOT EXISTS folders (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    path         TEXT NOT NULL UNIQUE,
    parent_dir   TEXT NOT NULL,
    indexed_at   INTEGER NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS folders_fts USING fts5(
    name,
    path,
    content='folders',
    content_rowid='rowid'
);

-- Triggers to keep FTS5 synchronized with folders table
CREATE TRIGGER IF NOT EXISTS folders_ai AFTER INSERT ON folders BEGIN
  INSERT INTO folders_fts(rowid, name, path) VALUES (new.rowid, new.name, new.path);
END;

CREATE TRIGGER IF NOT EXISTS folders_ad AFTER DELETE ON folders BEGIN
  INSERT INTO folders_fts(folders_fts, rowid, name, path) VALUES('delete', old.rowid, old.name, old.path);
END;

CREATE TRIGGER IF NOT EXISTS folders_au AFTER UPDATE ON folders BEGIN
  INSERT INTO folders_fts(folders_fts, rowid, name, path) VALUES('delete', old.rowid, old.name, old.path);
  INSERT INTO folders_fts(rowid, name, path) VALUES (new.rowid, new.name, new.path);
END;

-- Usage Frequency & Recency Table
CREATE TABLE IF NOT EXISTS usage (
    result_id        TEXT NOT NULL,
    result_type      TEXT NOT NULL,
    launch_count     INTEGER NOT NULL DEFAULT 0,
    last_launched_at INTEGER NOT NULL,
    PRIMARY KEY (result_id, result_type)
);

CREATE INDEX IF NOT EXISTS idx_usage_result ON usage(result_id);

-- History Table
CREATE TABLE IF NOT EXISTS history (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    query        TEXT NOT NULL,
    result_id    TEXT NOT NULL,
    result_type  TEXT NOT NULL,
    result_name  TEXT NOT NULL,
    launched_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_history_launched ON history(launched_at DESC);
CREATE INDEX IF NOT EXISTS idx_history_result ON history(result_id);

-- Settings Table
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Metadata Table
CREATE TABLE IF NOT EXISTS metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
