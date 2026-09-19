//! # semcode-core
//!
//! Tipos, errores y traits compartidos por todos los crates de
//! **semcode-search v4.0.0**.
//!
//! Este crate es la base del sistema. No contiene lógica pesada: solo
//! define los contratos y estructuras que usan los demás.
//!
//! ## Módulos
//!
//! - [`error`] — Error y Result unificados.
//! - [`types`] — Enums y structs del dominio (Language, SymbolKind, ...).
//! - [`traits`] — Traits compartidos (Embedder, Parser).
//! - [`config`] — Configuración global.
//!
//! ## Ejemplo
//!
//! ```
//! use semcode_core::types::{Language, Blake3Hash};
//! use semcode_core::error::Result;
//!
//! fn detect(text: &str) -> Language {
//!     Language::from_extension(text).unwrap_or(Language::Unknown)
//! }
//!
//! assert_eq!(detect("rs"), Language::Rust);
//! assert_eq!(detect("xyz"), Language::Unknown);
//!
//! let h = Blake3Hash::from_str("hello");
//! assert_eq!(h.to_hex().len(), 64);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod traits;
pub mod types;

// Re-exports principales para que los consumidores no tengan que escribir
// `use semcode_core::types::Language;` sino `use semcode_core::Language;`.
pub use config::Config;
pub use error::{Error, Result};
pub use traits::{Embedder, ParsedSymbol, Parser};
pub use types::{Blake3Hash, Language, SymbolKind, Visibility};
