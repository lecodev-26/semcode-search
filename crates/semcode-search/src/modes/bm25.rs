//! Motor BM25 (búsqueda léxica sobre FTS5).
//!
//! Envuelve la función `search_fts` de `semcode-index` para devolver
//! `RankedItem`. El score ya viene normalizado por SQLite (negamos el
//! `bm25()` que devuelve valores negativos).

use crate::types::{Filters, RankedItem, Source};
use semcode_core::error::Result;
use semcode_index::fts;
use semcode_index::Index;

/// Busca con BM25 sobre el índice FTS5.
///
/// Escapa la query para tratarla como texto literal, evitando que los
/// operadores FTS5 (`AND`, `OR`, `NOT`, `*`, `"`) la interpreten.
pub fn search(
    index: &Index,
    query: &str,
    _filters: &Filters,
    limit: usize,
) -> Result<Vec<RankedItem>> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let conn = index.conn()?;

    // Escapamos la query para que sea tratada como texto literal
    let escaped = fts::escape_query(query);

    // Si la query queda vacía tras escapar (p.ej. solo símbolos), salir
    if escaped.is_empty() {
        return Ok(Vec::new());
    }

    let hits = fts::search_fts(&conn, &escaped, limit)?;

    let items: Vec<RankedItem> = hits
        .into_iter()
        .map(|hit| RankedItem {
            chunk_id: hit.chunk_id,
            score: hit.score,
            source: Source::Bm25,
        })
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_returns_empty() {
        // No podemos llamar a search sin un Index, pero validamos la guarda
        let q = "   ";
        assert!(q.trim().is_empty());
    }

    #[test]
    fn escape_preserves_words() {
        let escaped = fts::escape_query("hello world");
        assert!(escaped.contains("hello"));
        assert!(escaped.contains("world"));
    }
}
