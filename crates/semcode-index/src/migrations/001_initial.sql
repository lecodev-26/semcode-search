-- ============================================================
-- semcode-search — Migración 001: Esquema inicial
-- ============================================================
-- Tablas: schema_version, repositories, files, symbols,
--         chunks, embeddings, fts_chunks, query_cache, stats.
-- ============================================================

-- ---------- METADATA ----------
CREATE TABLE schema_version (
    version     INTEGER NOT NULL PRIMARY KEY,
    applied_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE repositories (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    path            TEXT    NOT NULL UNIQUE,
    name            TEXT    NOT NULL,
    indexed_at      TEXT    NOT NULL,
    file_count      INTEGER NOT NULL DEFAULT 0,
    symbol_count    INTEGER NOT NULL DEFAULT 0,
    chunk_count     INTEGER NOT NULL DEFAULT 0,
    total_size      INTEGER NOT NULL DEFAULT 0,
    schema_version  INTEGER NOT NULL DEFAULT 1
);

-- ---------- FILES ----------
CREATE TABLE files (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id       INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    path          TEXT    NOT NULL,
    language      TEXT    NOT NULL,
    hash          BLOB    NOT NULL,
    size          INTEGER NOT NULL,
    lines         INTEGER NOT NULL,
    modified_at   TEXT    NOT NULL,
    indexed_at    TEXT    NOT NULL,
    symbol_count  INTEGER NOT NULL DEFAULT 0,
    chunk_count   INTEGER NOT NULL DEFAULT 0,
    is_binary     INTEGER NOT NULL DEFAULT 0,
    is_generated  INTEGER NOT NULL DEFAULT 0,
    UNIQUE(repo_id, path)
);

CREATE INDEX idx_files_repo     ON files(repo_id);
CREATE INDEX idx_files_language ON files(repo_id, language);
CREATE INDEX idx_files_hash     ON files(hash);

-- ---------- SYMBOLS ----------
CREATE TABLE symbols (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id         INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    parent_id       INTEGER REFERENCES symbols(id) ON DELETE CASCADE,
    name            TEXT    NOT NULL,
    qualified_name  TEXT    NOT NULL,
    kind            TEXT    NOT NULL,
    language        TEXT    NOT NULL,
    start_line      INTEGER NOT NULL,
    end_line        INTEGER NOT NULL,
    start_byte      INTEGER NOT NULL,
    end_byte        INTEGER NOT NULL,
    signature       TEXT,
    doc_comment     TEXT,
    visibility      TEXT    NOT NULL DEFAULT 'unknown',
    is_async        INTEGER NOT NULL DEFAULT 0,
    is_unsafe       INTEGER NOT NULL DEFAULT 0,
    is_exported     INTEGER NOT NULL DEFAULT 0,
    complexity      INTEGER
);

CREATE INDEX idx_symbols_file      ON symbols(file_id);
CREATE INDEX idx_symbols_parent    ON symbols(parent_id);
CREATE INDEX idx_symbols_name      ON symbols(name);
CREATE INDEX idx_symbols_kind      ON symbols(kind);
CREATE INDEX idx_symbols_qualified ON symbols(qualified_name);
CREATE INDEX idx_symbols_lang_kind ON symbols(language, kind);

-- ---------- CHUNKS ----------
CREATE TABLE chunks (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id        INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    symbol_id      INTEGER REFERENCES symbols(id) ON DELETE SET NULL,
    kind           TEXT    NOT NULL,
    content        TEXT    NOT NULL,
    start_line     INTEGER NOT NULL,
    end_line       INTEGER NOT NULL,
    start_byte     INTEGER NOT NULL,
    end_byte       INTEGER NOT NULL,
    token_count    INTEGER NOT NULL,
    hash           BLOB    NOT NULL,
    prev_chunk_id  INTEGER REFERENCES chunks(id) ON DELETE SET NULL,
    next_chunk_id  INTEGER REFERENCES chunks(id) ON DELETE SET NULL
);

CREATE INDEX idx_chunks_file   ON chunks(file_id);
CREATE INDEX idx_chunks_symbol ON chunks(symbol_id);
CREATE INDEX idx_chunks_hash   ON chunks(hash);
CREATE INDEX idx_chunks_kind   ON chunks(kind);

-- ---------- EMBEDDINGS ----------
CREATE TABLE embeddings (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id    INTEGER NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
    model       TEXT    NOT NULL,
    provider    TEXT    NOT NULL,
    dimensions  INTEGER NOT NULL,
    vector      BLOB    NOT NULL,
    norm        REAL    NOT NULL,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(chunk_id, model, provider)
);

CREATE INDEX idx_embeddings_chunk ON embeddings(chunk_id);
CREATE INDEX idx_embeddings_model ON embeddings(model, provider);

-- ---------- FTS5 (BM25) ----------
CREATE VIRTUAL TABLE fts_chunks USING fts5(
    content,
    symbol_name,
    file_path,
    language,
    tokenize = 'unicode61 remove_diacritics 2',
    content = 'chunks',
    content_rowid = 'id'
);

-- Trigger: insertar en FTS5 cuando se inserta un chunk
CREATE TRIGGER fts_chunks_ai AFTER INSERT ON chunks BEGIN
    INSERT INTO fts_chunks(rowid, content, symbol_name, file_path, language)
    VALUES (
        new.id,
        new.content,
        COALESCE((SELECT name FROM symbols WHERE id = new.symbol_id), ''),
        (SELECT path FROM files WHERE id = new.file_id),
        (SELECT language FROM files WHERE id = new.file_id)
    );
END;

-- Trigger: borrar de FTS5 cuando se borra un chunk
CREATE TRIGGER fts_chunks_ad AFTER DELETE ON chunks BEGIN
    INSERT INTO fts_chunks(fts_chunks, rowid, content, symbol_name, file_path, language)
    VALUES ('delete', old.id, old.content, '', '', '');
END;

-- ---------- QUERY CACHE ----------
CREATE TABLE query_cache (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    query_hash  BLOB    NOT NULL UNIQUE,
    query_text  TEXT    NOT NULL,
    mode        TEXT    NOT NULL,
    result_json TEXT    NOT NULL,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    expires_at  TEXT
);

CREATE INDEX idx_query_cache_expires ON query_cache(expires_at);

-- ---------- STATS ----------
CREATE TABLE stats (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
