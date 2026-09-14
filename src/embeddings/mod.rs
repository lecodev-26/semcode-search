//! Módulo Embeddings - Búsqueda con IA real (opcional)
//!
//! Solo se compila si el feature `ai` está activado.

#[cfg(feature = "ai")]
mod ai {
    use anyhow::Result;
    use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

    /// Modelo de embeddings encapsulado
    pub struct Embedder {
        model: TextEmbedding,
        dimensions: usize,
    }

    impl Embedder {
        /// Crea un nuevo Embedder y descarga el modelo si es necesario
        pub fn new() -> Result<Self> {
            println!("🧠 Cargando modelo de IA (la primera vez descarga ~90MB)...");

            let model = TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                    .with_show_download_progress(true),
            )?;

            let dimensions = 384;
            println!("✅ Modelo cargado (dimensiones: {})", dimensions);

            Ok(Self { model, dimensions })
        }

        /// Genera un embedding para un texto
        pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
            let embeddings = self.model.embed(vec![text], None)?;
            Ok(embeddings.into_iter().next().unwrap_or_default())
        }

        /// Genera embeddings para múltiples textos (batch)
        pub fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
            let embeddings = self.model.embed(texts, None)?;
            Ok(embeddings)
        }

        /// Devuelve las dimensiones del modelo
        pub fn dimensions(&self) -> usize {
            self.dimensions
        }
    }

    impl Default for Embedder {
        fn default() -> Self {
            Self::new().expect("Failed to create Embedder")
        }
    }

    /// Calcula la similitud coseno entre dos vectores
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }

    /// Busca el archivo más parecido a la query usando embeddings
    pub fn semantic_search(
        query_embedding: &[f32],
        documents: &[(String, Vec<f32>)],
        threshold: f32,
    ) -> Vec<(String, f32)> {
        let mut results: Vec<(String, f32)> = documents
            .iter()
            .filter_map(|(path, emb)| {
                let similarity = cosine_similarity(query_embedding, emb);
                if similarity >= threshold {
                    Some((path.clone(), similarity))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[cfg(feature = "ai")]
pub use ai::*;

/// Indica si el feature de IA está activado
pub const AI_ENABLED: bool = cfg!(feature = "ai");