//! Operaciones sobre la tabla `files`.

use semcode_core::error::{Error, Result};
use semcode_core::types::{Blake3Hash, Language};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use std::path::{Path, PathBuf};

/// Entrada de un archivo en el índice.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// ID interno (0 si no está persistido aún).
    pub id: i64,

    /// Ruta relativa al repo.
    pub path: PathBuf,

    /// Lenguaje detectado.
    pub language: Language,

    /// Hash BLAKE3 del contenido.
    pub hash: Blake3Hash,

    /// Tamaño en bytes.
    pub size: u64,

    /// Número de líneas.
    pub lines: usize,

    /// Fecha de modificación del archivo.
    pub modified_at: DateTime<Utc>,

    /// Fecha de indexación.
    pub indexed_at: DateTime<Utc>,

    /// Número de símbolos extraídos.
    pub symbol_count: usize,

    /// Número de chunks generados.
    pub chunk_count: usize,

    /// ¿Es un archivo binario?
    pub is_binary: bool,

    /// ¿Es un archivo generado (código autogenerado)?
    pub is_generated: bool,
}

impl FileEntry {
    /// Crea una entrada nueva (sin ID).
    pub fn new(path: impl Into<PathBuf>, language: Language, hash: Blake3Hash, size: u64) -> Self {
        Self {
            id: 0,
            path: path.into(),
            language,
            hash,
            size,
            lines: 0,
            modified_at: Utc::now(),
            indexed_at: Utc::now(),
            symbol_count: 0,
            chunk_count: 0,
            is_binary: false,
            is_generated: false,
        }
    }
}

/// Inserta o actualiza un archivo, devolviendo su ID.
///
/// La clave única es `(repo_id, path)`. Si ya existe, se actualiza.
pub fn upsert_file(conn: &Connection, repo_id: i64, entry: &FileEntry) -> Result<i64> {
    let path_str = entry.path.to_string_lossy().to_string();
    let language_str = entry.language.as_str();
    let hash_bytes = &entry.hash.0[..];

    conn.execute(
        "INSERT INTO files (
            repo_id, path, language, hash, size, lines,
            modified_at, indexed_at, symbol_count, chunk_count,
            is_binary, is_generated
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(repo_id, path) DO UPDATE SET
            language = excluded.language,
            hash = excluded.hash,
            size = excluded.size,
            lines = excluded.lines,
            modified_at = excluded.modified_at,
            indexed_at = excluded.indexed_at,
            symbol_count = excluded.symbol_count,
            chunk_count = excluded.chunk_count,
            is_binary = excluded.is_binary,
            is_generated = excluded.is_generated",
        params![
            repo_id,
            path_str,
            language_str,
            hash_bytes,
            entry.size as i64,
            entry.lines as i64,
            entry.modified_at.to_rfc3339(),
            entry.indexed_at.to_rfc3339(),
            entry.symbol_count as i64,
            entry.chunk_count as i64,
            entry.is_binary as i32,
            entry.is_generated as i32,
        ],
    )
    .map_err(|e| Error::index(format!("upsert archivo {:?}: {}", entry.path, e)))?;

    let id: i64 = conn
        .query_row(
            "SELECT id FROM files WHERE repo_id = ?1 AND path = ?2",
            params![repo_id, path_str],
            |r| r.get(0),
        )
        .map_err(|e| Error::index(format!("leer file_id: {}", e)))?;

    Ok(id)
}

/// Busca un archivo por su ruta (relativa al repo).
pub fn find_by_path(conn: &Connection, repo_id: i64, path: &Path) -> Result<Option<FileEntry>> {
    let path_str = path.to_string_lossy().to_string();

    let result = conn
        .query_row(
            "SELECT id, path, language, hash, size, lines,
                    modified_at, indexed_at, symbol_count, chunk_count,
                    is_binary, is_generated
             FROM files
             WHERE repo_id = ?1 AND path = ?2",
            params![repo_id, path_str],
            row_to_file_entry,
        )
        .optional()
        .map_err(|e| Error::index(format!("buscar archivo {:?}: {}", path, e)))?;

    Ok(result)
}

/// Devuelve todos los archivos de un repo.
pub fn list_files(conn: &Connection, repo_id: i64) -> Result<Vec<FileEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, path, language, hash, size, lines,
                    modified_at, indexed_at, symbol_count, chunk_count,
                    is_binary, is_generated
             FROM files
             WHERE repo_id = ?1
             ORDER BY path",
        )
        .map_err(|e| Error::index(format!("preparar list_files: {}", e)))?;

    let rows = stmt
        .query_map([repo_id], row_to_file_entry)
        .map_err(|e| Error::index(format!("list_files: {}", e)))?;

    let mut files = Vec::new();
    for r in rows {
        files.push(r.map_err(|e| Error::index(format!("leer fila: {}", e)))?);
    }
    Ok(files)
}

/// Borra un archivo por ID.
///
/// Los símbolos, chunks y embeddings asociados se borran en cascada.
pub fn delete_file(conn: &Connection, file_id: i64) -> Result<()> {
    conn.execute("DELETE FROM files WHERE id = ?1", [file_id])
        .map_err(|e| Error::index(format!("borrar archivo {}: {}", file_id, e)))?;
    Ok(())
}

/// Convierte una fila SQL en un `FileEntry`.
fn row_to_file_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileEntry> {
    let hash_blob: Vec<u8> = row.get(3)?;
    let mut hash_arr = [0u8; 32];
    if hash_blob.len() == 32 {
        hash_arr.copy_from_slice(&hash_blob);
    }

    let language_str: String = row.get(2)?;
    let language = Language::from_extension(&language_str).unwrap_or(Language::Unknown);

    Ok(FileEntry {
        id: row.get(0)?,
        path: PathBuf::from(row.get::<_, String>(1)?),
        language,
        hash: Blake3Hash(hash_arr),
        size: row.get::<_, i64>(4)? as u64,
        lines: row.get::<_, i64>(5)? as usize,
        modified_at: parse_dt(row.get::<_, String>(6)?),
        indexed_at: parse_dt(row.get::<_, String>(7)?),
        symbol_count: row.get::<_, i64>(8)? as usize,
        chunk_count: row.get::<_, i64>(9)? as usize,
        is_binary: row.get::<_, i32>(10)? != 0,
        is_generated: row.get::<_, i32>(11)? != 0,
    })
}

/// Parsea una fecha RFC3339, con fallback a "ahora".
fn parse_dt(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
