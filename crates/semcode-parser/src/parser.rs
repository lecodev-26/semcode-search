//! Parser principal de código.
//!
//! Mantiene un parser de Tree-sitter por lenguaje y un query compilado
//! para cada uno. El consumidor solo llama a [`CodeParser::parse`] y
//! recibe los símbolos extraídos.

use crate::extractor::extract_symbols;
use semcode_core::error::{Error, Result};
use semcode_core::traits::ParsedSymbol;
use semcode_core::types::Language;

use std::collections::HashMap;
use tree_sitter::{Language as TsLanguage, Parser, Query, QueryError};

/// Parser multi-lenguaje.
///
/// Los parsers y queries se inicializan **una sola vez** (lazy) y se
/// reutilizan. No es `Send + Sync` porque `tree_sitter::Parser` no lo es,
/// pero se puede crear uno por hilo si hace falta.
pub struct CodeParser {
    parsers: HashMap<Language, Parser>,
    queries: HashMap<Language, Query>,
}

impl CodeParser {
    /// Crea un nuevo parser con todos los lenguajes soportados.
    pub fn new() -> Result<Self> {
        let mut parsers = HashMap::new();
        let mut queries = HashMap::new();

        // Rust
        register(
            &mut parsers,
            &mut queries,
            Language::Rust,
            tree_sitter_rust::LANGUAGE.into(),
            include_str!("queries/rust.scm"),
        )?;

        // Python
        register(
            &mut parsers,
            &mut queries,
            Language::Python,
            tree_sitter_python::LANGUAGE.into(),
            include_str!("queries/python.scm"),
        )?;

        // JavaScript
        register(
            &mut parsers,
            &mut queries,
            Language::JavaScript,
            tree_sitter_javascript::LANGUAGE.into(),
            include_str!("queries/javascript.scm"),
        )?;

        // TypeScript (usamos la gramática .ts; el .tsx requiere otra)
        register(
            &mut parsers,
            &mut queries,
            Language::TypeScript,
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            include_str!("queries/typescript.scm"),
        )?;

        // Go
        register(
            &mut parsers,
            &mut queries,
            Language::Go,
            tree_sitter_go::LANGUAGE.into(),
            include_str!("queries/go.scm"),
        )?;

        // JSON
        register(
            &mut parsers,
            &mut queries,
            Language::Json,
            tree_sitter_json::LANGUAGE.into(),
            include_str!("queries/json.scm"),
        )?;

        Ok(Self { parsers, queries })
    }

    /// Lenguajes soportados por este parser.
    pub fn supported_languages(&self) -> Vec<Language> {
        self.parsers.keys().copied().collect()
    }

    /// ¿Soporta este lenguaje?
    pub fn supports(&self, language: Language) -> bool {
        self.parsers.contains_key(&language)
    }

    /// Parsea el código y devuelve los símbolos encontrados.
    ///
    /// Si el lenguaje no está soportado, devuelve
    /// [`Error::UnsupportedLanguage`].
    pub fn parse(&mut self, source: &str, language: Language) -> Result<Vec<ParsedSymbol>> {
        if !self.supports(language) {
            return Err(Error::UnsupportedLanguage(language.to_string()));
        }

        let parser = self
            .parsers
            .get_mut(&language)
            .expect("invariant: language registered");
        let query = self
            .queries
            .get(&language)
            .expect("invariant: query registered");

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| Error::parse("Tree-sitter devolvió None al parsear"))?;

        extract_symbols(&tree, source, query, language)
    }
}

impl Default for CodeParser {
    fn default() -> Self {
        Self::new().expect("CodeParser::new() no debería fallar con lenguajes válidos")
    }
}

/// Registra un lenguaje en los mapas internos.
///
/// Encapsula el `into()` y la compilación de la query para que el código
/// de `new()` quede limpio.
fn register(
    parsers: &mut HashMap<Language, Parser>,
    queries: &mut HashMap<Language, Query>,
    language: Language,
    ts_lang: TsLanguage,
    query_src: &str,
) -> Result<()> {
    let mut parser = Parser::new();
    parser
        .set_language(&ts_lang)
        .map_err(|e| Error::parse(format!("error al setear lenguaje {:?}: {}", language, e)))?;

    let query = Query::new(&ts_lang, query_src).map_err(query_error_to_error)?;

    parsers.insert(language, parser);
    queries.insert(language, query);
    Ok(())
}

/// Convierte un `QueryError` de Tree-sitter en nuestro `Error::Parse`.
fn query_error_to_error(e: QueryError) -> Error {
    Error::parse(format!("query Tree-sitter inválida: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_parser_with_all_languages() {
        let parser = CodeParser::new().expect("crear CodeParser");
        let langs = parser.supported_languages();
        assert!(langs.contains(&Language::Rust));
        assert!(langs.contains(&Language::Python));
        assert!(langs.contains(&Language::JavaScript));
        assert!(langs.contains(&Language::TypeScript));
        assert!(langs.contains(&Language::Go));
        assert!(langs.contains(&Language::Json));
        assert_eq!(langs.len(), 6);
    }

    #[test]
    fn rejects_unsupported_language() {
        let mut parser = CodeParser::new().unwrap();
        let err = parser.parse("body { color: red }", Language::Css);
        assert!(err.is_err());
        match err.unwrap_err() {
            Error::UnsupportedLanguage(_) => {}
            other => panic!("esperaba UnsupportedLanguage, obtuve {:?}", other),
        }
    }

    #[test]
    fn parses_trivial_rust_function() {
        let mut parser = CodeParser::new().unwrap();
        let src = r#"
pub fn hello(name: &str) -> String {
    format!("Hello, {}", name)
}
"#;
        let symbols = parser.parse(src, Language::Rust).unwrap();
        assert!(!symbols.is_empty(), "debería extraer al menos un símbolo");

        let hello = symbols
            .iter()
            .find(|s| s.name == "hello")
            .expect("debería encontrar `hello`");

        assert_eq!(hello.kind, semcode_core::types::SymbolKind::Function);
        assert_eq!(hello.start_line, 2);
    }
}
