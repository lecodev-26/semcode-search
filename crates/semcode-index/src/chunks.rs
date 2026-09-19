//! Operaciones sobre la tabla `chunks`.

use semcode_core::error::{Error, Result};
use semcode_core::types::Blake3Hash;

use rusqlite::{params, Connection};

/// Tipo de chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    /// Cuerpo completo de un símbolo.
    SymbolBody,
    /// Solo la firma de un símbolo.
    SymbolSignature,
    /// Comentario de documentación.
    DocComment,
    /// Cabecera del archivo (imports, módulos...).
    FileHeader,
    /// Bloque arbitrario (sin AST).
    Block,
    /// Ventana deslizante con overlap.
    Window,
}

impl ChunkKind {
    /// Nombre canónico.
    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkKind::SymbolBody => "symbol_body",
            ChunkKind::SymbolSignature => "symbol_signature",
            ChunkKind::DocComment => "doc_comment",
            ChunkKind::FileHeader => "file_header",
            ChunkKind::Block => "block",
            ChunkKind::Window => "window",
        }
    }

    /// Parsea desde string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "symbol_body" => ChunkKind::SymbolBody,
            "symbol_signature" => ChunkKind::SymbolSignature,
            "doc_comment" => ChunkKind::DocComment,
            "file_header" => ChunkKind::FileHeader,
            "window" => ChunkKind::Window,
            _ => ChunkKind::Block,
        }
    }
}

/// Chunk persistido.
#[derive(Debug, Clone)]
pub struct StoredChunk {
    /// ID interno.
    pub id: i64,

    /// ID del archivo.
    pub file_id: i64,

    /// ID del símbolo asociado (si lo hay).
    pub symbol_id: Option<i64>,

    /// Tipo de chunk.
    pub kind: ChunkKind,

    /// Contenido textual.
    pub content: String,

    /// Línea inicial.
    pub start_line: usize,

    /// Línea final.
    pub end_line: usize,

    /// Offset inicial.
    pub start_byte: usize,

    /// Offset final.
    pub end_byte: usize,

    /// Tokens aproximados.
    pub token_count: usize,

    /// Hash del contenido.
    pub hash: Blake3Hash,
}

/// Chunk con información del archivo (para búsquedas).
#[derive(Debug, Clone)]
pub struct ChunkWithFile {
    /// El chunk.
    pub chunk: StoredChunk,

    /// Ruta del archivo.
    pub file_path: String,

    /// Lenguaje del archivo.
    pub language: String,
}

/// Inserta un lote de chunks para un archivo.
pub fn insert_chunks(
    conn: &mut Connection,
    file_id: i64,
    chunks: &[(ChunkKind, String, usize, usize, usize, usize, usize, Blake3Hash)],
) -> Result<Vec<i64>> {
    let tx = conn
        .transaction()
        .map_err(|e| Error::index(format!("iniciar transacción chunks: {}", e)))?;

    // Borrar chunks antiguos
    tx.execute("DELETE FROM chunks WHERE file_id = ?1", [file_id])
        .map_err(|e| Error::index(format!("borrar chunks antiguos: {}", e)))?;

    let mut ids = Vec::with_capacity(chunks.len());

    for (kind, content, start_line, end_line, start_byte, end_byte, token_count, hash) in chunks {
        tx.execute(
            "INSERT INTO chunks (
                file_id, symbol_id, kind, content,
                start_line, end_line, start_byte, end_byte,
                token_count, hash
             ) VALUES (
                ?1, NULL, ?2, ?3,
                ?4, ?5, ?6, ?7,
                ?8, ?9
             )",
            params![
                file_id,
                kind.as_str(),
                content,
                *start_line as i64,
                *end_line as i64,
                *start_byte as i64,
                *end_byte as i64,
                *token_count as i64,
                &hash.0[..],
            ],
        )
        .map_err(|e| Error::index(format!("insertar chunk: {}", e)))?;

        ids.push(tx.last_insert_rowid());
    }

    tx.commit()
        .map_err(|e| Error::index(format!("commit chunks: {}", e)))?;

    Ok(ids)
}

/// Cuenta los chunks de un archivo.
pub fn count_by_file(conn: &Connection, file_id: i64) -> Result<usize> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM chunks WHERE file_id = ?1",
            [file_id],
            |r| r.get(0),
        )
        .map_err(|e| Error::index(format!("contar chunks: {}", e)))?;
    Ok(n as usize)
}

/// Lista todos los chunks de un archivo.
pub fn list_by_file(conn: &Connection, file_id: i64) -> Result<Vec<StoredChunk>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, file_id, symbol_id, kind, content,
                    start_line, end_line, start_byte, end_byte,
                    token_count, hash
             FROM chunks
             WHERE file_id = ?1
             ORDER BY start_line",
        )
        .map_err(|e| Error::index(format!("preparar list_by_file chunks: {}", e)))?;

    let rows = stmt
        .query_map([file_id], row_to_chunk)
        .map_err(|e| Error::index(format!("list_by_file chunks: {}", e)))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| Error::index(format!("leer chunk: {}", e)))?);
    }
    Ok(out)
}

/// Obtiene chunks por IDs, junto con info del archivo.
pub fn get_with_files(conn: &Connection, ids: &[i64]) -> Result<Vec<ChunkWithFile>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    // SQL con IN dinámico
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT c.id, c.file_id, c.symbol_id, c.kind, c.content,
                c.start_line, c.end_line, c.start_byte, c.end_byte,
                c.token_count, c.hash,
                f.path, f.language
         FROM chunks c
         JOIN files f ON f.id = c.file_id
         WHERE c.id IN ({})",
        placeholders
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| Error::index(format!("preparar get_with_files: {}", e)))?;

    let params: Vec<&dyn rusqlite::ToSql> = ids
        .iter()
        .map(|id| id as &dyn rusqlite::ToSql)
        .collect();

    let rows = stmt
        .query_map(params.as_slice(), |row| {
            let chunk = row_to_chunk(row)?;
            Ok(ChunkWithFile {
                chunk,
                file_path: row.get(11)?,
                language: row.get(12)?,
            })
        })
        .map_err(|e| Error::index(format!("get_with_files: {}", e)))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| Error::index(format!("leer chunk con archivo: {}", e)))?);
    }
    Ok(out)
}

/// Convierte una fila SQL en `StoredChunk`.
fn row_to_chunk(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredChunk> {
    let kind_str: String = row.get(3)?;
    let kind = ChunkKind::from_str(&kind_str);

    let hash_blob: Vec<u8> = row.get(10)?;
    let mut hash_arr = [0u8; 32];
    if hash_blob.len() == 32 {
        hash_arr.copy_from_slice(&hash_blob);
    }

    Ok(StoredChunk {
        id: row.get(0)?,
        file_id: row.get(1)?,
        symbol_id: row.get(2)?,
        kind,
        content: row.get(4)?,
        start_line: row.get::<_, i64>(5)? as usize,
        end_line: row.get::<_, i64>(6)? as usize,
        start_byte: row.get::<_, i64>(7)? as usize,
        end_byte: row.get::<_, i64>(8)? as usize,
        token_count: row.get::<_, i64>(9)? as usize,
        hash: Blake3Hash(hash_arr),
    })
}
