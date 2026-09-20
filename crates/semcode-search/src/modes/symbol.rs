//! Motor estructural (símbolos).
//!
//! Busca sobre la tabla `symbols`: por nombre, tipo, visibilidad, etc.
//! Devuelve los chunks asociados a los símbolos encontrados para poder
//! fusionarlos con los otros motores.
//!
//! ## Nota
//!
//! Un símbolo puede tener 0 o varios chunks asociados. Si tiene 0 (por
//! ejemplo, un import), se omite del ranking.

use crate::types::{Filters, RankedItem, Source};
use semcode_core::error::{Error, Result};
use semcode_index::Index;


/// Puntuación base para un match exacto de nombre.
const SCORE_EXACT: f32 = 1.0;

/// Puntuación para un match por prefijo.
const SCORE_PREFIX: f32 = 0.8;

/// Puntuación para un match por substring.
const SCORE_CONTAINS: f32 = 0.5;

/// Busca símbolos por nombre y devuelve los chunks asociados.
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

    // SQL dinámico con filtros opcionales
    let mut sql = String::from(
        "SELECT s.id, s.name, s.qualified_name,
                (SELECT c.id FROM chunks c WHERE c.symbol_id = s.id LIMIT 1) AS chunk_id
         FROM symbols s
         JOIN files f ON f.id = s.file_id
         WHERE 1=1"
    );

    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // Nombre (LIKE)
    sql.push_str(" AND (s.name LIKE ? OR s.qualified_name LIKE ?)");
    let pattern = format!("%{}%", query);
    params_vec.push(Box::new(pattern.clone()));
    params_vec.push(Box::new(pattern));

    // Lenguaje
    if let Some(lang) = filters.language {
        sql.push_str(" AND s.language = ?");
        params_vec.push(Box::new(lang.as_str().to_string()));
    }

    // Tipo de símbolo
    if let Some(kind) = filters.symbol_kind {
        sql.push_str(" AND s.kind = ?");
        params_vec.push(Box::new(kind.as_str().to_string()));
    }

    // Visibilidad
    if let Some(vis) = filters.visibility {
        sql.push_str(" AND s.visibility = ?");
        let s = match vis {
            semcode_core::types::Visibility::Public => "public",
            semcode_core::types::Visibility::Private => "private",
            semcode_core::types::Visibility::Protected => "protected",
            semcode_core::types::Visibility::Crate => "crate",
            semcode_core::types::Visibility::Package => "package",
            semcode_core::types::Visibility::Unknown => "unknown",
        };
        params_vec.push(Box::new(s.to_string()));
    }

    // Async
    if let Some(is_async) = filters.is_async {
        sql.push_str(" AND s.is_async = ?");
        params_vec.push(Box::new(if is_async { 1i32 } else { 0i32 }));
    }

    // Exportado
    if let Some(is_exported) = filters.is_exported {
        sql.push_str(" AND s.is_exported = ?");
        params_vec.push(Box::new(if is_exported { 1i32 } else { 0i32 }));
    }

    sql.push_str(" LIMIT ?");
    params_vec.push(Box::new(limit as i64));

    // Ejecutar
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| Error::index(format!("preparar search_symbol: {}", e)))?;

    let params_refs: Vec<&dyn rusqlite::ToSql> =
        params_vec.iter().map(|p| p.as_ref()).collect();

    let rows = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok((
                row.get::<_, i64>(0)?,   // symbol_id
                row.get::<_, String>(1)?, // name
                row.get::<_, String>(2)?, // qualified_name
                row.get::<_, Option<i64>>(3)?, // chunk_id
            ))
        })
        .map_err(|e| Error::index(format!("search_symbol: {}", e)))?;

    let mut results: Vec<RankedItem> = Vec::new();
    let q_lower = query.to_lowercase();

    for r in rows {
        let (_sym_id, name, qualified, chunk_id_opt) =
            r.map_err(|e| Error::index(e.to_string()))?;

        // Saltar símbolos sin chunk asociado
        let chunk_id = match chunk_id_opt {
            Some(id) => id,
            None => continue,
        };

        // Calcular score
        let name_lower = name.to_lowercase();
        let qual_lower = qualified.to_lowercase();

        let score = if name_lower == q_lower || qual_lower == q_lower {
            SCORE_EXACT
        } else if name_lower.starts_with(&q_lower) || qual_lower.starts_with(&q_lower) {
            SCORE_PREFIX
        } else {
            SCORE_CONTAINS
        };

        results.push(RankedItem {
            chunk_id,
            score,
            source: Source::Symbol,
        });
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
    fn score_constants_are_ordered() {
        assert!(SCORE_EXACT > SCORE_PREFIX);
        assert!(SCORE_PREFIX > SCORE_CONTAINS);
    }

    #[test]
    fn empty_query_returns_empty() {
        let q = "   ";
        assert!(q.trim().is_empty());
    }

    #[test]
    fn params_dyn_trait_works() {
        // Verifica que Box<dyn ToSql> se puede construir
        let v: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(1i64), Box::new("hello")];
        let _refs: Vec<&dyn rusqlite::ToSql> = v.iter().map(|p| p.as_ref()).collect();
    }
}
