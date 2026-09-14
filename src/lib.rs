//! # semcode-search
//!
//! Biblioteca para busqueda semantica de codigo con TF-IDF, cache y filtros avanzados.

pub mod cache;
pub mod cli;
pub mod core;
pub mod embeddings;
pub mod history;
pub mod i18n;
pub mod watch;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "tui")]
pub mod tui;

// Re-exportar API publica
pub use cache::Cache;
pub use core::index_files;
pub use core::search_files;
pub use core::SearchConfigInternal;
pub use core::SearchEngine;
pub use core::SearchError;
pub use core::SearchResult;
pub use history::History;
pub use i18n::{current_language, t, Language, Translations};
pub use watch::Watcher;

// Funciones con IA (solo si el feature `ai` esta activado)
#[cfg(feature = "ai")]
pub use core::index_files_with_ai;
#[cfg(feature = "ai")]
pub use core::search_files_with_ai;
#[cfg(feature = "ai")]
pub use embeddings::Embedder;

// Servidor (solo si el feature `server` esta activado)
#[cfg(feature = "server")]
pub use server::run_server;

// TUI (solo si el feature `tui` esta activado)
#[cfg(feature = "tui")]
pub use tui::run_tui;
