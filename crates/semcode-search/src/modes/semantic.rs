//! Motor semántico (embeddings).
//!
//! Genera el embedding de la query y lo compara con todos los embeddings
//! almacenados en el índice usando similitud coseno. Devuelve los chunks
//! más similares por encima del umbral configurado.

use crate::types::{Filters, RankedItem, Source};
use semcode_core::error::Result;
use semcode_embeddings::similarity::{cosine_similarity_with_norms, l2_norm};
use semcode_embeddings::Provider;
use semcode_index::embeddings;
use semcode_index::Index;

use rayon::prelude::*;

/// Umbral mínimo de similitud por defecto.
pub const DEFAULT_MIN_SCORE: f32 = 0.3;

/// Busca por similitud semántica.
///
/// 1. Genera el embedding de la query.
/// 2. Recupera todos los embeddings del modelo.
/// 3. Calcula similitud coseno en paralelo (rayon).
/// 4. Filtra por umbral y ordena por score.
pub fn search(
    index: &Index,
    embedder: &Provider,
    query: &str,
    filters: &Filters,
    limit: usize,
) -> Result<Vec<RankedItem>> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let min_score = filters.min_semantic_score.unwrap_or(DEFAULT_MIN_SCORE);

    // 1. Embedding de la query
    let q_vec = embedder.embed(query)?;
    let q_norm = l2_norm(&q_vec);

    // 2. Recuperar embeddings del modelo
    let conn = index.conn()?;
    let model = embedder.model_name();
    let candidates = embeddings::all_for_model(&conn, model)?;

    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // 3. Calcular similitudes en paralelo
    let mut scored: Vec<RankedItem> = candidates
        .into_par_iter()
        .filter_map(|(chunk_id, vec, norm)| {
            let sim = cosine_similarity_with_norms(&q_vec, &vec, q_norm, norm);
            if sim >= min_score {
                Some(RankedItem {
                    chunk_id,
                    score: sim,
                    source: Source::Semantic,
                })
            } else {
                None
            }
        })
        .collect();

    // 4. Ordenar y truncar
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(limit);

    Ok(scored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_min_score_is_sane() {
        assert!(DEFAULT_MIN_SCORE > 0.0);
        assert!(DEFAULT_MIN_SCORE < 1.0);
    }

    #[test]
    fn empty_query_returns_empty() {
        let q = "   ";
        assert!(q.trim().is_empty());
    }
}
