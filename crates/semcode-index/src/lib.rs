//! # semcode-index
//!
//! Persistencia de semcode-search: SQLite + FTS5 + vectores.
//!
//! Este crate es el **corazón del estado del sistema**. Almacena:
//!
//! - Repositorios indexados.
//! - Archivos con sus hashes.
//! - Símbolos extraídos por el parser.
//! - Chunks (fragmentos indexables).
//! - Embeddings (vectores para búsqueda semántica).
//! - Índice FTS5 para búsqueda BM25.
//!
//! ## Estructura
//!
//! - [`index`] — El `Index` principal (pool, migraciones).
//! - [`files`] — Operaciones sobre archivos.
//! - [`symbols`] — Operaciones sobre símbolos.
//! - [`chunks`] — Operaciones sobre chunks.
//! - [`embeddings`] — Operaciones sobre vectores.
//! - [`fts`] — Búsqueda BM25.
//! - [`schema`] — Constantes y PRAGMAs.
//! - [`migrations`] — Sistema de migraciones.
//!
//! ## Ejemplo
//!
//! ```no_run
//! use semcode_index::Index;
//!
//! let mut index = Index::open(".").unwrap();
//! index.migrate().unwrap();
//! println!("repo_id = {}", index.repo_id());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod chunks;
pub mod embeddings;
pub mod files;
pub mod fts;
pub mod index;
pub mod migrations;
pub mod schema;
pub mod symbols;

// Re-exports principales
pub use chunks::{ChunkKind, ChunkWithFile, StoredChunk};
pub use files::FileEntry;
pub use index::Index;
pub use symbols::StoredSymbol;
