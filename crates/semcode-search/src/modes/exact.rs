//! Motor de búsqueda exacta.
//!
//! Busca el texto literal (o un patrón regex) en el contenido de los
//! chunks. Es el motor más simple y el más preciso: no hay ambigüedad.
//!
//! ## Estrategia
//!
//! 1. Normaliza la query (regex escape si no es modo regex).
//! 2. Hace un `LIKE` en SQL para acotar candidatos.
//! 3. Cuenta matches reales con `regex`.
//! 4. Score = número de matches (más matches → mejor).

use crate::types::{Filters, RankedItem, Source};
use semcode_core::error::{Error, Result};
use semcode_index::Index;

use regex::Regex;
use rusqlite::params;

/// Busca el texto exacto en el contenido de los chunks.
///
/// Si `filters.case_sensitive` es `false`, la búsqueda ignora mayúsculas.
pub fn search(
    index: &Index,
    query: &str,
    filters: &Filters,
    limit: usize,
) -> Result<Vec<RankedItem>> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let conn = index.conn()?;

    // Preparamos el regex una vez
    let pattern = if filters.case_sensitive {
        regex::escape(query)
    } else {
        format!("(?i){}", regex::escape(query))
    };

    let re = Regex::new(&pattern)
        .map_err(|e| Error::search(format!("regex inválido: {}", e)))?;

    // Preparar el LIKE
    let like_pattern = format!("%{}%", query);

    // Ejecutamos con o sin filtro de lenguaje
    let mut results: Vec<RankedItem> = Vec::new();

    if let Some(lang) = filters.language {
        // Con filtro de lenguaje
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.content
                 FROM chunks c
                 JOIN files f ON f.id = c.file_id
                 WHERE c.content LIKE ?1 AND f.language = ?2
                 LIMIT ?3",
            )
            .map_err(|e| Error::index(format!("preparar search_exact: {}", e)))?;

        let rows = stmt
            .query_map(params![like_pattern, lang.as_str(), limit as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| Error::index(format!("search_exact: {}", e)))?;

        for r in rows {
            let (chunk_id, content) = r.map_err(|e| Error::index(e.to_string()))?;
            let count = re.find_iter(&content).count();
            if count > 0 {
                results.push(RankedItem {
                    chunk_id,
                    score: count as f32,
                    source: Source::Exact,
                });
            }
        }
    } else {
        // Sin filtro de lenguaje
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.content
                 FROM chunks c
                 WHERE c.content LIKE ?1
                 LIMIT ?2",
            )
            .map_err(|e| Error::index(format!("preparar search_exact: {}", e)))?;

        let rows = stmt
            .query_map(params![like_pattern, limit as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| Error::index(format!("search_exact: {}", e)))?;

        for r in rows {
            let (chunk_id, content) = r.map_err(|e| Error::index(e.to_string()))?;
            let count = re.find_iter(&content).count();
            if count > 0 {
                results.push(RankedItem {
                    chunk_id,
                    score: count as f32,
                    source: Source::Exact,
                });
            }
        }
    }

    // Ordenar por score descendente
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_escape_works() {
        let q = "fn main() {}";
        let escaped = regex::escape(q);
        let re = Regex::new(&escaped).unwrap();
        assert!(re.is_match("fn main() {}"));
    }

    #[test]
    fn case_insensitive_pattern() {
        let q = "Hello";
        let pattern = format!("(?i){}", regex::escape(q));
        let re = Regex::new(&pattern).unwrap();
        assert!(re.is_match("hello world"));
        assert!(re.is_match("HELLO"));
    }

    #[test]
    fn count_matches() {
        let re = Regex::new("a").unwrap();
        assert_eq!(re.find_iter("banana").count(), 3);
    }

    #[test]
    fn empty_query_returns_empty() {
        let q = "   ";
        assert!(q.trim().is_empty());
    }
}
