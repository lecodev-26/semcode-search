//! # semcode-embeddings
//!
//! Proveedores de embeddings para semcode-search.
//!
//! Este crate envuelve varios proveedores de embeddings (locales y
//! remotos) detrás de una API común. El proveedor por defecto es
//! **fastembed** con el modelo `all-MiniLM-L6-v2` (384 dims, ONNX local).
//!
//! ## Módulos
//!
//! - [`similarity`] — Funciones matemáticas (coseno, normas, dot).
//! - [`provider`] — `Provider` + `ProviderKind`.
//! - [`fastembed`] — Implementación con fastembed (ONNX local).
//! - [`batch`] — Procesamiento por lotes (secuencial y paralelo).
//!
//! ## Ejemplo
//!
//! ```no_run
//! use semcode_embeddings::{Provider, similarity::cosine_similarity};
//!
//! let provider = Provider::fastembed().unwrap();
//! let a = provider.embed("hello world").unwrap();
//! let b = provider.embed("hi there").unwrap();
//! let sim = cosine_similarity(&a, &b);
//! println!("similitud: {}", sim);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod batch;
#[cfg(feature = "fastembed")]
pub mod fastembed;
pub mod provider;
pub mod similarity;

// Re-exports principales
pub use provider::{Provider, ProviderKind};
pub use similarity::{
    cosine_similarity, cosine_similarity_normalized, cosine_similarity_with_norms, dot, l2_norm,
    normalize, normalized,
};

#[cfg(feature = "fastembed")]
pub use fastembed::{FastEmbedder, DEFAULT_DIMENSIONS, DEFAULT_MODEL_NAME};
