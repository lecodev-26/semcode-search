//! Motor de búsqueda principal.
//!
//! Coordina los 4 motores (Exact, BM25, Semantic, Symbol), fusiona sus
//! rankings con RRF y devuelve resultados hidratados con snippet y
//! metadata.

use crate::modes;
use crate::rrf::{self, RrfWeights, DEFAULT_RRF_K};
use crate::snippet;
use crate::types::{
    FusedItem, Query, RankedItem, SearchMode, SearchResult, Source,
};

use semcode_core::error::Result;
use semcode_embeddings::Provider;
use semcode_index::{chunks, Index};

use std::sync::Arc;

/// Número de resultados por motor antes de fusionar (Top-K).
pub const DEFAULT_TOPK_PER_MODE: usize = 50;

/// Líneas de contexto en los snippets.
pub const DEFAULT_SNIPPET_CONTEXT: usize = 3;

/// Motor de búsqueda híbrida.
pub struct SearchEngine {
    index: Arc<Index>,
    embedder: Arc<Provider>,
    topk_per_mode: usize,
    rrf_k: usize,
    rrf_weights: RrfWeights,
}

impl SearchEngine {
    /// Crea un motor con la configuración por defecto.
    pub fn new(index: Arc<Index>, embedder: Arc<Provider>) -> Self {
        Self {
            index,
            embedder,
            topk_per_mode: DEFAULT_TOPK_PER_MODE,
            rrf_k: DEFAULT_RRF_K,
            rrf_weights: RrfWeights::default(),
        }
    }

    /// Configura el Top-K por motor.
    pub fn with_topk(mut self, topk: usize) -> Self {
        self.topk_per_mode = topk;
        self
    }

    /// Configura la constante k de RRF.
    pub fn with_rrf_k(mut self, k: usize) -> Self {
        self.rrf_k = k;
        self
    }

    /// Configura los pesos de RRF.
    pub fn with_rrf_weights(mut self, weights: RrfWeights) -> Self {
        self.rrf_weights = weights;
        self
    }

    /// Ejecuta una búsqueda según el modo indicado.
    pub fn search(&self, query: Query) -> Result<Vec<SearchResult>> {
        if query.text.trim().is_empty() {
            return Ok(Vec::new());
        }

        let rankings = match query.mode {
            SearchMode::Exact => vec![self.run_exact(&query)?],
            SearchMode::Bm25 => vec![self.run_bm25(&query)?],
            SearchMode::Semantic => vec![self.run_semantic(&query)?],
            SearchMode::Symbol => vec![self.run_symbol(&query)?],
            SearchMode::Hybrid => self.run_hybrid(&query)?,
        };

        // Fusionar (aunque sea un solo motor, RRF ordena)
        let fused = if rankings.len() == 1 {
            // Sin fusión: convertir a FusedItem
            rankings
                .into_iter()
                .next()
                .unwrap_or_default()
                .into_iter()
                .map(|item| FusedItem {
                    chunk_id: item.chunk_id,
                    rrf_score: item.score,
                    sources: vec![item.source],
                    breakdown: breakdown_from(&item),
                })
                .collect()
        } else {
            rrf::fuse_weighted(rankings, self.rrf_k, self.rrf_weights)
        };

        // Hidratar
        let results = self.hydrate(&fused, &query)?;
        Ok(results.into_iter().take(query.limit).collect())
    }

    /// Ejecuta los 4 motores en paralelo.
    fn run_hybrid(&self, query: &Query) -> Result<Vec<Vec<RankedItem>>> {
        let exact = self.run_exact(query)?;
        let bm25 = self.run_bm25(query)?;
        let semantic = self.run_semantic(query)?;
        let symbol = self.run_symbol(query)?;
        Ok(vec![exact, bm25, semantic, symbol])
    }

    fn run_exact(&self, query: &Query) -> Result<Vec<RankedItem>> {
        modes::exact::search(&self.index, &query.text, &query.filters, self.topk_per_mode)
    }

    fn run_bm25(&self, query: &Query) -> Result<Vec<RankedItem>> {
        modes::bm25::search(&self.index, &query.text, &query.filters, self.topk_per_mode)
    }

    fn run_semantic(&self, query: &Query) -> Result<Vec<RankedItem>> {
        modes::semantic::search(
            &self.index,
            &self.embedder,
            &query.text,
            &query.filters,
            self.topk_per_mode,
        )
    }

    fn run_symbol(&self, query: &Query) -> Result<Vec<RankedItem>> {
        modes::symbol::search(&self.index, &query.text, &query.filters, self.topk_per_mode)
    }

    /// Hidrata los items fusionados con contenido, snippet y metadata.
    fn hydrate(&self, fused: &[FusedItem], query: &Query) -> Result<Vec<SearchResult>> {
        if fused.is_empty() {
            return Ok(Vec::new());
        }

        let ids: Vec<i64> = fused.iter().map(|f| f.chunk_id).collect();
        let conn = self.index.conn()?;
        let chunks_data = chunks::get_with_files(&conn, &ids)?;

        // Indexar por chunk_id para lookup O(1)
        let mut map: std::collections::HashMap<i64, _> =
            chunks_data.into_iter().map(|c| (c.chunk.id, c)).collect();

        let mut results = Vec::with_capacity(fused.len());

        for item in fused {
            let chunk_data = match map.remove(&item.chunk_id) {
                Some(c) => c,
                None => continue, // chunk borrado entre la búsqueda y la hidratación
            };

            let content = &chunk_data.chunk.content;
            let snippet_text = snippet::make_snippet(content, &query.text, DEFAULT_SNIPPET_CONTEXT);

            results.push(SearchResult {
                path: chunk_data.file_path.clone(),
                score: item.rrf_score,
                snippet: snippet_text,
                start_line: chunk_data.chunk.start_line,
                end_line: chunk_data.chunk.end_line,
                symbol: None, // TODO: lookup en symbols si chunk.symbol_id
                sources: item.sources.clone(),
                score_breakdown: item.breakdown.clone(),
            });
        }

        // Ordenar por score descendente
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }
}

/// Construye un `ScoreBreakdown` a partir de un `RankedItem` único.
fn breakdown_from(item: &RankedItem) -> crate::types::ScoreBreakdown {
    let mut b = crate::types::ScoreBreakdown::default();
    match item.source {
        Source::Exact => b.exact = Some(item.score),
        Source::Bm25 => b.bm25 = Some(item.score),
        Source::Semantic => b.semantic = Some(item.score),
        Source::Symbol => b.symbol = Some(item.score),
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_sane() {
        assert!(DEFAULT_TOPK_PER_MODE > 0);
        assert!(DEFAULT_SNIPPET_CONTEXT > 0);
    }

    #[test]
    fn breakdown_from_exact() {
        let item = RankedItem {
            chunk_id: 1,
            score: 0.9,
            source: Source::Exact,
        };
        let b = breakdown_from(&item);
        assert_eq!(b.exact, Some(0.9));
        assert!(b.bm25.is_none());
    }

    #[test]
    fn breakdown_from_semantic() {
        let item = RankedItem {
            chunk_id: 1,
            score: 0.75,
            source: Source::Semantic,
        };
        let b = breakdown_from(&item);
        assert_eq!(b.semantic, Some(0.75));
        assert!(b.exact.is_none());
    }
}
