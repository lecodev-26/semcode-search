//! Procesamiento por lotes de embeddings.
//!
//! Divide una lista larga de textos en lotes y los procesa. Con la feature
//! `rayon` (activada por defecto en el crate), los lotes se pueden
//! procesar en paralelo.
//!
//! ## Ejemplo
//!
//! ```no_run
//! use semcode_embeddings::batch::embed_in_batches;
//! use semcode_embeddings::provider::Provider;
//!
//! let provider = Provider::fastembed().unwrap();
//! let texts = vec!["hola", "mundo", "foo", "bar"];
//! let vecs = embed_in_batches(&provider, &texts, 2).unwrap();
//! assert_eq!(vecs.len(), 4);
//! ```

use semcode_core::error::{Error, Result};
use semcode_core::traits::Embedder;

/// Tamaño de lote por defecto.
pub const DEFAULT_BATCH_SIZE: usize = 32;

/// Genera embeddings para una lista de textos, procesando en lotes.
///
/// - `batch_size`: número de textos por lote. Si es `0`, usa
///   [`DEFAULT_BATCH_SIZE`].
/// - Los lotes se procesan secuencialmente. Para paralelismo, usa
///   [`embed_in_batches_parallel`].
pub fn embed_in_batches<E: Embedder + ?Sized>(
    embedder: &E,
    texts: &[&str],
    batch_size: usize,
) -> Result<Vec<Vec<f32>>> {
    let batch_size = if batch_size == 0 {
        DEFAULT_BATCH_SIZE
    } else {
        batch_size
    };

    let mut out = Vec::with_capacity(texts.len());

    for chunk in texts.chunks(batch_size) {
        let batch_results = embedder.embed_batch(chunk)?;
        if batch_results.len() != chunk.len() {
            return Err(Error::Embedding(format!(
                "batch devolvió {} vectores, esperaba {}",
                batch_results.len(),
                chunk.len()
            )));
        }
        out.extend(batch_results);
    }

    Ok(out)
}

/// Igual que [`embed_in_batches`] pero en paralelo (rayon).
///
/// Cada lote se procesa en un hilo independiente. El orden de salida se
/// preserva.
#[cfg(feature = "rayon")]
pub fn embed_in_batches_parallel<E: Embedder + Sync + ?Sized>(
    embedder: &E,
    texts: &[&str],
    batch_size: usize,
) -> Result<Vec<Vec<f32>>> {
    use rayon::prelude::*;

    let batch_size = if batch_size == 0 {
        DEFAULT_BATCH_SIZE
    } else {
        batch_size
    };

    let batches: Vec<&[&str]> = texts.chunks(batch_size).collect();

    let results: Result<Vec<Vec<Vec<f32>>>> = batches
        .into_par_iter()
        .map(|chunk| embedder.embed_batch(chunk))
        .collect();

    let mut out = Vec::with_capacity(texts.len());
    for batch_result in results? {
        out.extend(batch_result);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyEmbedder;

    impl Embedder for DummyEmbedder {
        fn embed(&self, text: &str) -> Result<Vec<f32>> {
            Ok(vec![text.len() as f32; 4])
        }
        fn dimensions(&self) -> usize {
            4
        }
        fn model_name(&self) -> &str {
            "dummy"
        }
    }

    #[test]
    fn embed_in_batches_basic() {
        let e = DummyEmbedder;
        let texts = vec!["a", "bb", "ccc", "dddd", "eeeee"];
        let out = embed_in_batches(&e, &texts, 2).unwrap();
        assert_eq!(out.len(), 5);
        assert_eq!(out[0].len(), 4);
        assert!((out[0][0] - 1.0).abs() < 1e-6); // len("a")
        assert!((out[4][0] - 5.0).abs() < 1e-6); // len("eeeee")
    }

    #[test]
    fn embed_in_batches_empty() {
        let e = DummyEmbedder;
        let out = embed_in_batches(&e, &[], 4).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn embed_in_batches_zero_uses_default() {
        let e = DummyEmbedder;
        let texts = vec!["x"];
        let out = embed_in_batches(&e, &texts, 0).unwrap();
        assert_eq!(out.len(), 1);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn embed_in_batches_parallel_basic() {
        let e = DummyEmbedder;
        let texts = vec!["a", "bb", "ccc", "dddd"];
        let out = embed_in_batches_parallel(&e, &texts, 2).unwrap();
        assert_eq!(out.len(), 4);
        assert!((out[3][0] - 4.0).abs() < 1e-6);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn embed_in_batches_parallel_empty() {
        let e = DummyEmbedder;
        let out = embed_in_batches_parallel(&e, &[], 4).unwrap();
        assert!(out.is_empty());
    }
}
