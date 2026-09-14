//! # semcode-search
//!
//! Biblioteca para búsqueda semántica de código con TF-IDF, caché y filtros avanzados.

pub mod cache;
pub mod cli;
pub mod core;
pub mod history;
pub mod watch;

// Re-exportar API pública
pub use cache::Cache;
pub use core::index_files;
pub use core::search_files;
pub use core::SearchConfigInternal;
pub use core::SearchEngine;
pub use core::SearchError;
pub use core::SearchResult;
pub use history::History;
pub use watch::Watcher;
