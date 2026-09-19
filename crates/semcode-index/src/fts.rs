//! Búsqueda BM25 sobre la tabla virtual FTS5 `fts_chunks`.
//!
//! SQLite FTS5 implementa BM25 nativamente con la función `bm25()`.
//! Devuelve valores **negativos** (más negativo = más relevante), así
//! que en el código los negamos para tener "más grande = mejor".

use semcode_core::error::{Error, Result};

use rusqlite::{params, Connection};

/// Resultado de una búsqueda FTS5.
#[derive(Debug, Clone)]
pub struct FtsHit {
    /// ID del chunk en `chunks`.
    pub chunk_id: i64,

    /// Score BM25 (positivo, mayor = mejor).
    pub score: f32,

    /// Snippet con contexto.
    pub snippet: String,
}

/// Busca en el índice FTS5 usando BM25.
///
/// El `query` es una expresión FTS5 (soporta `AND`, `OR`, `NOT`, frases
/// entre comillas, etc.).
pub fn search_fts(conn: &Connection, query: &str, limit: usize) -> Result<Vec<FtsHit>> {
    let mut stmt = conn
        .prepare(
            "SELECT
                fts_chunks.rowid,
                bm25(fts_chunks) AS score,
                snippet(fts_chunks, 0, '<b>', '</b>', '...', 32) AS snip
             FROM fts_chunks
             WHERE fts_chunks MATCH ?1
             ORDER BY score
             LIMIT ?2",
        )
        .map_err(|e| Error::index(format!("preparar search_fts: {}", e)))?;

    let rows = stmt
        .query_map(params![query, limit as i64], |row| {
            let chunk_id: i64 = row.get(0)?;
            let bm25: f64 = row.get(1)?;
            let snippet: String = row.get(2)?;
            Ok(FtsHit {
                chunk_id,
                score: (-bm25) as f32, // bm25 devuelve negativo; lo negamos
                snippet,
            })
        })
        .map_err(|e| Error::index(format!("search_fts: {}", e)))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| Error::index(format!("leer hit FTS: {}", e)))?);
    }
    Ok(out)
}

/// Cuenta cuántos chunks hay en el índice FTS5.
pub fn count_fts(conn: &Connection) -> Result<usize> {
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM fts_chunks", [], |r| r.get(0))
        .map_err(|e| Error::index(format!("contar fts_chunks: {}", e)))?;
    Ok(n as usize)
}

/// Reconstruye el índice FTS5 desde cero.
///
/// Útil tras operaciones masivas o si el índice se corrompe.
pub fn rebuild_fts(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT INTO fts_chunks(fts_chunks) VALUES('rebuild')",
        [],
    )
    .map_err(|e| Error::index(format!("rebuild fts_chunks: {}", e)))?;
    Ok(())
}

/// Escapa una query FTS5 para búsquedas literales.
///
/// Útil si el usuario pasa texto libre y queremos que no interprete
/// operadores (`AND`, `OR`, `NOT`, `*`, `"`, ...).
pub fn escape_query(query: &str) -> String {
    // Envolvemos cada palabra en comillas dobles para tratarla como literal
    query
        .split_whitespace()
        .map(|word| format!("\"{}\"", word.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_query_wraps_words() {
        let q = escape_query("hello world");
        assert_eq!(q, r#""hello" "world""#);
    }

    #[test]
    fn escape_query_strips_quotes() {
        let q = escape_query(r#"say "hi""#);
        assert_eq!(q, r#""say" "hi""#);
    }

    #[test]
    fn escape_query_empty() {
        assert_eq!(escape_query(""), "");
    }
}
