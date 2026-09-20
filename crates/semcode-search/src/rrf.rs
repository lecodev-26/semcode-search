//! Reciprocal Rank Fusion (RRF).
//!
//! RRF combina varios rankings en uno solo sin necesidad de normalizar
//! scores. Es el estándar de facto en búsqueda híbrida.
//!
//! ## Fórmula
//!
//! ```text
//! score_rrf(d) = Σ_i  w_i / (k + rank_i(d))
//! ```
//!
//! Donde:
//! - `d` es el documento (chunk).
//! - `i` es cada motor (Exact, BM25, Semantic, Symbol).
//! - `rank_i(d)` es la posición de `d` en el ranking del motor `i` (1-indexed).
//! - `k` es la constante de suavizado (default 60).
//! - `w_i` es el peso del motor (opcional, default 1.0).

use crate::types::{FusedItem, RankedItem, ScoreBreakdown, Source};
use std::collections::HashMap;

/// Constante de suavizado por defecto (del paper original de Cormack et al.).
pub const DEFAULT_RRF_K: usize = 60;

/// Pesos por motor para RRF ponderado.
#[derive(Debug, Clone, Copy)]
pub struct RrfWeights {
    /// Peso del motor Exact.
    pub exact: f32,
    /// Peso del motor BM25.
    pub bm25: f32,
    /// Peso del motor Semantic.
    pub semantic: f32,
    /// Peso del motor Symbol.
    pub symbol: f32,
}

impl Default for RrfWeights {
    fn default() -> Self {
        Self {
            exact: 1.0,
            bm25: 1.2,
            semantic: 1.5,
            symbol: 1.0,
        }
    }
}

impl RrfWeights {
    /// Peso de un motor concreto.
    pub fn for_source(&self, source: Source) -> f32 {
        match source {
            Source::Exact => self.exact,
            Source::Bm25 => self.bm25,
            Source::Semantic => self.semantic,
            Source::Symbol => self.symbol,
        }
    }
}

/// Fusiona varios rankings con RRF usando pesos por defecto.
pub fn fuse(rankings: Vec<Vec<RankedItem>>, k: usize) -> Vec<FusedItem> {
    fuse_weighted(rankings, k, RrfWeights::default())
}

/// Fusiona varios rankings con RRF ponderado.
///
/// - `rankings`: uno por motor, ya ordenados de mejor a peor.
/// - `k`: constante de suavizado (usa [`DEFAULT_RRF_K`] si no sabes).
/// - `weights`: pesos por motor.
pub fn fuse_weighted(
    rankings: Vec<Vec<RankedItem>>,
    k: usize,
    weights: RrfWeights,
) -> Vec<FusedItem> {
    let k = if k == 0 { DEFAULT_RRF_K } else { k };
    let k_f = k as f32;

    // Acumulador por chunk_id
    let mut scores: HashMap<i64, FusedItem> = HashMap::new();

    for ranking in rankings {
        for (rank_idx, item) in ranking.iter().enumerate() {
            let rank = (rank_idx + 1) as f32; // 1-indexed
            let weight = weights.for_source(item.source);
            let contribution = weight / (k_f + rank);

            let entry = scores.entry(item.chunk_id).or_insert_with(|| FusedItem {
                chunk_id: item.chunk_id,
                rrf_score: 0.0,
                sources: Vec::new(),
                breakdown: ScoreBreakdown::default(),
            });

            entry.rrf_score += contribution;

            if !entry.sources.contains(&item.source) {
                entry.sources.push(item.source);
            }

            // Guardar el score crudo del motor en el breakdown
            match item.source {
                Source::Exact => entry.breakdown.exact = Some(item.score),
                Source::Bm25 => entry.breakdown.bm25 = Some(item.score),
                Source::Semantic => entry.breakdown.semantic = Some(item.score),
                Source::Symbol => entry.breakdown.symbol = Some(item.score),
            }
        }
    }

    let mut fused: Vec<FusedItem> = scores.into_values().collect();

    // Ordenar por score descendente, y como tie-breaker, por ID ascendente
    // (para que sea determinista).
    fused.sort_by(|a, b| {
        b.rrf_score
            .partial_cmp(&a.rrf_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.chunk_id.cmp(&b.chunk_id))
    });

    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(chunk_id: i64, score: f32, source: Source) -> RankedItem {
        RankedItem {
            chunk_id,
            score,
            source,
        }
    }

    #[test]
    fn fuse_empty_rankings() {
        let out = fuse(vec![], DEFAULT_RRF_K);
        assert!(out.is_empty());
    }

    #[test]
    fn fuse_single_ranking() {
        let r1 = vec![
            item(1, 0.9, Source::Exact),
            item(2, 0.8, Source::Exact),
            item(3, 0.7, Source::Exact),
        ];
        let out = fuse(vec![r1], DEFAULT_RRF_K);
        assert_eq!(out.len(), 3);
        // El primero debe seguir siendo el primero (mismo orden)
        assert_eq!(out[0].chunk_id, 1);
        assert_eq!(out[1].chunk_id, 2);
        assert_eq!(out[2].chunk_id, 3);
    }

    #[test]
    fn fuse_favors_documents_in_multiple_rankings() {
        let r1 = vec![item(1, 1.0, Source::Exact), item(2, 0.9, Source::Exact)];
        let r2 = vec![item(2, 1.0, Source::Bm25), item(3, 0.9, Source::Bm25)];
        let out = fuse(vec![r1, r2], DEFAULT_RRF_K);

        // El chunk 2 aparece en ambos rankings → debe ganar
        assert_eq!(out[0].chunk_id, 2);
        // Y su breakdown debe tener exact y bm25
        assert!(out[0].breakdown.exact.is_some());
        assert!(out[0].breakdown.bm25.is_some());
        // Y sus sources debe contener ambos
        assert!(out[0].sources.contains(&Source::Exact));
        assert!(out[0].sources.contains(&Source::Bm25));
    }

    #[test]
    fn fuse_empty_rankings_in_list() {
        let r1 = vec![item(1, 1.0, Source::Exact)];
        let r2: Vec<RankedItem> = vec![];
        let out = fuse(vec![r1, r2], DEFAULT_RRF_K);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].chunk_id, 1);
    }

    #[test]
    fn fuse_default_k_if_zero() {
        let r1 = vec![item(1, 1.0, Source::Exact)];
        let out = fuse(vec![r1.clone()], 0);
        let out2 = fuse(vec![r1], DEFAULT_RRF_K);
        // Mismos resultados con k=0 (usa default) y k=60
        assert_eq!(out[0].chunk_id, out2[0].chunk_id);
        assert!((out[0].rrf_score - out2[0].rrf_score).abs() < 1e-9);
    }

    #[test]
    fn fuse_is_deterministic() {
        // Mismos inputs → mismos outputs (orden estable)
        let make = || {
            vec![
                vec![item(1, 1.0, Source::Exact), item(2, 1.0, Source::Exact)],
                vec![item(2, 1.0, Source::Bm25), item(1, 1.0, Source::Bm25)],
            ]
        };

        let out1 = fuse(make(), DEFAULT_RRF_K);
        let out2 = fuse(make(), DEFAULT_RRF_K);
        assert_eq!(out1.len(), out2.len());
        for (a, b) in out1.iter().zip(out2.iter()) {
            assert_eq!(a.chunk_id, b.chunk_id);
        }
    }

    #[test]
    fn rrf_weights_default() {
        let w = RrfWeights::default();
        assert_eq!(w.for_source(Source::Semantic), 1.5);
        assert_eq!(w.for_source(Source::Exact), 1.0);
    }

    #[test]
    fn fuse_weighted_respects_weights() {
        let r1 = vec![item(1, 1.0, Source::Exact)];
        let r2 = vec![item(2, 1.0, Source::Semantic)];

        // Con pesos default, semantic (1.5) > exact (1.0), debería ganar el 2
        let out = fuse(vec![r1.clone(), r2.clone()], DEFAULT_RRF_K);
        assert_eq!(out[0].chunk_id, 2);

        // Pero si subimos exact, debería ganar el 1
        let weights = RrfWeights {
            exact: 10.0,
            bm25: 1.0,
            semantic: 1.0,
            symbol: 1.0,
        };
        let out2 = fuse_weighted(vec![r1, r2], DEFAULT_RRF_K, weights);
        assert_eq!(out2[0].chunk_id, 1);
    }
}
