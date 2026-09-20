//! Proveedores de embeddings.
//!
//! Un `Provider` envuelve un `Box<dyn Embedder>` y ofrece una API
//! uniforme. Se puede crear con [`Provider::fastembed`] (por defecto) o
//! con otras variantes si las features correspondientes están activas.

use semcode_core::error::Result;
use semcode_core::traits::Embedder;

/// Tipo de proveedor de embeddings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// Modelo local ONNX (all-MiniLM-L6-v2).
    FastEmbed,
    /// API de OpenAI (requiere `--features openai`).
    OpenAI,
    /// Servidor Ollama local (requiere `--features ollama`).
    Ollama,
}

impl ProviderKind {
    /// Nombre canónico en minúsculas.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::FastEmbed => "fastembed",
            ProviderKind::OpenAI => "openai",
            ProviderKind::Ollama => "ollama",
        }
    }
}

/// Proveedor de embeddings.
///
/// Envuelve un `Box<dyn Embedder>` para poder cambiar de implementación
/// en runtime sin tocar el resto del código.
pub struct Provider {
    inner: Box<dyn Embedder>,
    kind: ProviderKind,
}

impl Provider {
    /// Crea un proveedor a partir de un `Embedder` concreto.
    pub fn new(embedder: Box<dyn Embedder>, kind: ProviderKind) -> Self {
        Self {
            inner: embedder,
            kind,
        }
    }

    /// Crea un proveedor con fastembed (all-MiniLM-L6-v2).
    ///
    /// Requiere la feature `fastembed` (activada por defecto).
    #[cfg(feature = "fastembed")]
    pub fn fastembed() -> Result<Self> {
        let embedder = crate::fastembed::FastEmbedder::new()?;
        Ok(Self::new(Box::new(embedder), ProviderKind::FastEmbed))
    }

    /// Tipo de proveedor.
    pub fn kind(&self) -> ProviderKind {
        self.kind
    }

    /// Nombre del modelo.
    pub fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    /// Número de dimensiones.
    pub fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }

    /// Genera el embedding de un texto.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.inner.embed(text)
    }

    /// Genera embeddings para un lote de textos.
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.inner.embed_batch(texts)
    }

    /// Acceso al `Embedder` interno (por si hace falta).
    pub fn inner(&self) -> &dyn Embedder {
        self.inner.as_ref()
    }
}

impl std::fmt::Debug for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Provider")
            .field("kind", &self.kind)
            .field("model", &self.inner.model_name())
            .field("dimensions", &self.inner.dimensions())
            .finish()
    }
}

impl Embedder for Provider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.inner.embed(text)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.inner.embed_batch(texts)
    }

    fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyEmbedder {
        dims: usize,
    }

    impl Embedder for DummyEmbedder {
        fn embed(&self, text: &str) -> Result<Vec<f32>> {
            Ok(vec![text.len() as f32; self.dims])
        }
        fn dimensions(&self) -> usize {
            self.dims
        }
        fn model_name(&self) -> &str {
            "dummy"
        }
    }

    #[test]
    fn provider_kind_as_str() {
        assert_eq!(ProviderKind::FastEmbed.as_str(), "fastembed");
        assert_eq!(ProviderKind::OpenAI.as_str(), "openai");
        assert_eq!(ProviderKind::Ollama.as_str(), "ollama");
    }

    #[test]
    fn provider_wraps_embedder() {
        let p = Provider::new(Box::new(DummyEmbedder { dims: 8 }), ProviderKind::FastEmbed);
        assert_eq!(p.dimensions(), 8);
        assert_eq!(p.model_name(), "dummy");
        assert_eq!(p.kind(), ProviderKind::FastEmbed);

        let v = p.embed("hello").unwrap();
        assert_eq!(v.len(), 8);
    }

    #[test]
    fn provider_debug_format() {
        let p = Provider::new(Box::new(DummyEmbedder { dims: 4 }), ProviderKind::OpenAI);
        let s = format!("{:?}", p);
        assert!(s.contains("OpenAI"));
        assert!(s.contains("dummy"));
    }
}
