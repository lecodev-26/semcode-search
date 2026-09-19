//! Traits compartidos entre crates.
//!
//! Estos traits definen las interfaces que implementan los crates superiores
//! (`semcode-embeddings`, `semcode-parser`). Mantenerlos aquí evita
//! dependencias circulares.

use crate::error::Result;
use crate::types::Language;

/// Proveedor de embeddings.
///
/// Cualquier implementación (fastembed, OpenAI, Ollama, ...) debe cumplir
/// este contrato. Se usa `Send + Sync` para poder compartirlo entre hilos.
pub trait Embedder: Send + Sync {
    /// Genera el embedding de un texto.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Genera embeddings para un lote de textos.
    ///
    /// Por defecto itera llamando a `embed`, pero las implementaciones
    /// pueden optimizarlo (batch real en el proveedor).
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }

    /// Número de dimensiones del vector.
    fn dimensions(&self) -> usize;

    /// Nombre del modelo (por ejemplo `"all-MiniLM-L6-v2"`).
    fn model_name(&self) -> &str;
}

/// Parser de código.
///
/// Se implementa por lenguaje o por conjunto de lenguajes. El contrato es
/// simple: dado un texto fuente y su lenguaje, devolver los símbolos
/// encontrados.
pub trait Parser: Send + Sync {
    /// Parsea el texto y devuelve los símbolos encontrados.
    fn parse_symbols(&self, source: &str, language: Language) -> Result<Vec<ParsedSymbol>>;

    /// Lenguajes soportados por esta implementación.
    fn supported_languages(&self) -> Vec<Language>;
}

/// Símbolo mínimo devuelto por un parser.
///
/// Es una versión "plana" del símbolo, sin IDs ni referencias a base de datos.
/// El crate `semcode-index` se encarga de enriquecerlo y persistirlo.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSymbol {
    /// Nombre simple (`"validate_email"`).
    pub name: String,

    /// Nombre cualificado (`"User::validate_email"`).
    pub qualified_name: String,

    /// Tipo del símbolo.
    pub kind: crate::types::SymbolKind,

    /// Línea inicial (1-indexed).
    pub start_line: usize,

    /// Línea final (1-indexed, inclusiva).
    pub end_line: usize,

    /// Offset inicial en bytes.
    pub start_byte: usize,

    /// Offset final en bytes.
    pub end_byte: usize,

    /// Firma legible (`"pub fn validate_email(email: &str) -> bool"`).
    pub signature: Option<String>,

    /// Comentario de documentación asociado, si existe.
    pub doc_comment: Option<String>,
}

impl ParsedSymbol {
    /// Constructor de conveniencia.
    pub fn new(
        name: impl Into<String>,
        qualified_name: impl Into<String>,
        kind: crate::types::SymbolKind,
        start_line: usize,
        end_line: usize,
        start_byte: usize,
        end_byte: usize,
    ) -> Self {
        Self {
            name: name.into(),
            qualified_name: qualified_name.into(),
            kind,
            start_line,
            end_line,
            start_byte,
            end_byte,
            signature: None,
            doc_comment: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SymbolKind;

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
    fn embedder_works() {
        let e = DummyEmbedder;
        let v = e.embed("hello").unwrap();
        assert_eq!(v.len(), 4);
        assert_eq!(e.dimensions(), 4);
        assert_eq!(e.model_name(), "dummy");
    }

    #[test]
    fn embedder_batch_default() {
        let e = DummyEmbedder;
        let vs = e.embed_batch(&["a", "bb", "ccc"]).unwrap();
        assert_eq!(vs.len(), 3);
        assert_eq!(vs[1].len(), 4);
    }

    #[test]
    fn parsed_symbol_new() {
        let s = ParsedSymbol::new(
            "foo",
            "Bar::foo",
            SymbolKind::Method,
            10,
            20,
            100,
            250,
        );
        assert_eq!(s.name, "foo");
        assert_eq!(s.qualified_name, "Bar::foo");
        assert_eq!(s.kind, SymbolKind::Method);
        assert_eq!(s.start_line, 10);
        assert_eq!(s.end_byte, 250);
    }
}
