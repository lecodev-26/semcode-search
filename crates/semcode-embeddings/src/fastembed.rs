//! Proveedor de embeddings usando `fastembed` (ONNX runtime local).
//!
//! Modelo por defecto: **all-MiniLM-L6-v2** (384 dimensiones, ~90 MB).
//! El modelo se descarga la primera vez y se cachea en:
//!
//! - Linux: `~/.cache/huggingface/`
//! - Windows: `%USERPROFILE%\.cache\huggingface\`
//! - macOS: `~/Library/Caches/huggingface/`
//!
//! ## Thread-safety
//!
//! `TextEmbedding` no es `Send + Sync` por defecto, pero el trait
//! [`Embedder`] sí lo requiere. Lo envolvemos en un `Mutex` para cumplir
//! el contrato.

use semcode_core::error::{Error, Result};
use semcode_core::traits::Embedder;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Mutex;

/// Número de dimensiones del modelo all-MiniLM-L6-v2.
pub const DEFAULT_DIMENSIONS: usize = 384;

/// Nombre canónico del modelo por defecto.
pub const DEFAULT_MODEL_NAME: &str = "all-MiniLM-L6-v2";

/// Embedder basado en fastembed.
pub struct FastEmbedder {
    inner: Mutex<TextEmbedding>,
    dimensions: usize,
    model_name: String,
}

impl FastEmbedder {
    /// Crea un embedder con el modelo por defecto (all-MiniLM-L6-v2).
    ///
    /// La primera vez descarga el modelo (~90 MB). Las siguientes veces
    /// usa la copia cacheada.
    pub fn new() -> Result<Self> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }

    /// Crea un embedder con un modelo concreto.
    pub fn with_model(model: EmbeddingModel) -> Result<Self> {
        let options = InitOptions::new(model).with_show_download_progress(true);

        let embedding = TextEmbedding::try_new(options)
            .map_err(|e| Error::Embedding(format!("inicializar fastembed: {}", e)))?;

        let dimensions = DEFAULT_DIMENSIONS;
        let model_name = DEFAULT_MODEL_NAME.to_string();

        Ok(Self {
            inner: Mutex::new(embedding),
            dimensions,
            model_name,
        })
    }

    /// Número de dimensiones del modelo.
    pub fn dims(&self) -> usize {
        self.dimensions
    }
}

impl Embedder for FastEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| Error::Embedding(format!("bloquear fastembed: {}", e)))?;

        let mut results = guard
            .embed(vec![text], None)
            .map_err(|e| Error::Embedding(format!("embed: {}", e)))?;

        results
            .pop()
            .ok_or_else(|| Error::Embedding("fastembed devolvió lista vacía".to_string()))
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut guard = self
            .inner
            .lock()
            .map_err(|e| Error::Embedding(format!("bloquear fastembed: {}", e)))?;

        // fastembed recibe un Vec<&str>
        let owned: Vec<&str> = texts.to_vec();
        guard
            .embed(owned, None)
            .map_err(|e| Error::Embedding(format!("embed_batch: {}", e)))
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}

impl std::fmt::Debug for FastEmbedder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastEmbedder")
            .field("model", &self.model_name)
            .field("dimensions", &self.dimensions)
            .finish()
    }
}

// Nota: los tests de `FastEmbedder` requieren descargar el modelo (~90 MB)
// y por eso están marcados con `#[ignore]`. Ejecútalos con:
//   cargo test -p semcode-embeddings -- --ignored
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "descarga el modelo (~90 MB)"]
    fn embed_single_text() {
        let e = FastEmbedder::new().expect("crear FastEmbedder");
        let v = e.embed("hello world").expect("embed");
        assert_eq!(v.len(), DEFAULT_DIMENSIONS);
        assert_eq!(e.dimensions(), DEFAULT_DIMENSIONS);
        assert_eq!(e.model_name(), DEFAULT_MODEL_NAME);
    }

    #[test]
    #[ignore = "descarga el modelo (~90 MB)"]
    fn embed_batch_works() {
        let e = FastEmbedder::new().expect("crear FastEmbedder");
        let vs = e.embed_batch(&["hello", "world", "foo"]).expect("embed_batch");
        assert_eq!(vs.len(), 3);
        assert_eq!(vs[0].len(), DEFAULT_DIMENSIONS);
    }

    #[test]
    #[ignore = "descarga el modelo (~90 MB)"]
    fn embed_batch_empty() {
        let e = FastEmbedder::new().expect("crear FastEmbedder");
        let vs = e.embed_batch(&[]).expect("embed_batch vacío");
        assert!(vs.is_empty());
    }
}
