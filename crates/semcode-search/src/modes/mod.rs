//! Motores de búsqueda.
//!
//! Cada submódulo implementa una estrategia distinta:
//!
//! - [`exact`] — Búsqueda literal / regex.
//! - [`bm25`] — Relevancia léxica (FTS5).
//! - [`semantic`] — Similitud semántica (embeddings).
//! - [`symbol`] — Búsqueda estructural (AST).

pub mod bm25;
pub mod exact;
pub mod semantic;
pub mod symbol;
