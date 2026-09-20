//! # semcode-search
//!
//! Motor de búsqueda híbrida de semcode-search.
//!
//! Combina 4 motores complementarios para encontrar código:
//!
//! - **Exact** — Búsqueda literal / regex.
//! - **BM25** — Relevancia léxica sobre FTS5.
//! - **Semantic** — Similitud por embeddings.
//! - **Symbol** — Búsqueda estructural (AST).
//!
//! Los rankings se fusionan con **Reciprocal Rank Fusion** (RRF).
//!
//! ## Módulos
//!
//! - [`types`] — `Query`, `SearchResult`, `Filters`, `SearchMode`.
//! - [`classifier`] — Clasificador heurístico de consultas.
//! - [`rrf`] — Reciprocal Rank Fusion.
//! - [`snippet`] — Generador de snippets.
//! - [`modes`] — Los 4 motores individuales.
//! - [`engine`] — `SearchEngine` (coordinador).
//!
//! ## Ejemplo
//!
//! ```ignore
//! use semcode_search::{SearchEngine, Query};
//! use semcode_index::Index;
//! use semcode_embeddings::Provider;
//! use std::sync::Arc;
//!
//! let mut index = Index::open(".").unwrap();
//! index.migrate().unwrap();
//! let index = Arc::new(index);
//!
//! let provider = Arc::new(Provider::fastembed().unwrap());
//! let engine = SearchEngine::new(index, provider);
//!
//! let results = engine.search(Query::hybrid("validate email", 10)).unwrap();
//! for r in results {
//!     println!("{}:{} — {}", r.path, r.start_line, r.score);
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod classifier;
pub mod engine;
pub mod modes;
pub mod rrf;
pub mod snippet;
pub mod types;

// Re-exports principales
pub use classifier::classify;
pub use engine::SearchEngine;
pub use rrf::{fuse, fuse_weighted, RrfWeights, DEFAULT_RRF_K};
pub use types::{
    Filters, FusedItem, Query, RankedItem, ScoreBreakdown, SearchMode, SearchResult, Source,
    SymbolInfo,
};
