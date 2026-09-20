//! Clasificador heurístico de consultas.
//!
//! Dada una query en texto libre, decide qué [`SearchMode`] es más
//! apropiado. No usa LLM: solo patrones simples para que sea rápido y
//! predecible.
//!
//! ## Reglas
//!
//! 1. Si empieza por `fn `, `struct `, `class `, ... → **Symbol**
//! 2. Si contiene comillas, `.*`, `^`, `$` → **Exact**
//! 3. Si tiene ≥ 5 palabras → **Semantic**
//! 4. Si tiene 1-2 palabras cortas → **Bm25**
//! 5. Por defecto → **Hybrid**

use crate::types::SearchMode;

/// Palabras clave que sugieren búsqueda estructural.
const SYMBOL_PREFIXES: &[&str] = &[
    "fn ", "func ", "struct ", "class ", "trait ",
    "interface ", "enum ", "type ", "impl ", "async ", "pub ",
    "const ", "let ", "def ", "module ", "method ",
];

/// Caracteres que sugieren búsqueda exacta/regex.
const EXACT_CHARS: &[char] = &['"', '\'', '^', '$', '\\'];

/// Clasifica una query y devuelve el modo sugerido.
pub fn classify(query: &str) -> SearchMode {
    let q = query.trim();

    if q.is_empty() {
        return SearchMode::Hybrid;
    }

    // 1. Prefijos estructurales
    let lower = q.to_lowercase();
    for prefix in SYMBOL_PREFIXES {
        if lower.starts_with(prefix) {
            return SearchMode::Symbol;
        }
    }

    // 2. Caracteres de exact/regex
    if q.contains(".*") || q.chars().any(|c| EXACT_CHARS.contains(&c)) {
        return SearchMode::Exact;
    }

    // 3. Texto natural (≥ 5 palabras)
    let word_count = q.split_whitespace().count();
    if word_count >= 5 {
        return SearchMode::Semantic;
    }

    // 4. Query corta tipo keyword (1-2 palabras sin espacios raros)
    if word_count <= 2 && q.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return SearchMode::Bm25;
    }

    // 5. Por defecto: híbrida
    SearchMode::Hybrid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_rust_function() {
        assert_eq!(classify("fn validate_email"), SearchMode::Symbol);
        assert_eq!(classify("struct User"), SearchMode::Symbol);
        assert_eq!(classify("async fn main"), SearchMode::Symbol);
    }

    #[test]
    fn classifies_python_function() {
        assert_eq!(classify("def validate_email"), SearchMode::Symbol);
        assert_eq!(classify("class UserRepository"), SearchMode::Symbol);
    }

    #[test]
    fn classifies_regex() {
        assert_eq!(classify("Result<.*>"), SearchMode::Exact);
        assert_eq!(classify("\"exact string\""), SearchMode::Exact);
        assert_eq!(classify("^start"), SearchMode::Exact);
        assert_eq!(classify("end$"), SearchMode::Exact);
    }

    #[test]
    fn classifies_natural_language() {
        assert_eq!(
            classify("function that validates email addresses"),
            SearchMode::Semantic
        );
        assert_eq!(
            classify("where are JWT tokens validated"),
            SearchMode::Semantic
        );
    }

    #[test]
    fn classifies_short_keyword() {
        assert_eq!(classify("authenticate"), SearchMode::Bm25);
        assert_eq!(classify("validate_email"), SearchMode::Bm25);
        assert_eq!(classify("user-service"), SearchMode::Bm25);
    }

    #[test]
    fn classifies_empty_as_hybrid() {
        assert_eq!(classify(""), SearchMode::Hybrid);
        assert_eq!(classify("   "), SearchMode::Hybrid);
    }

    #[test]
    fn classifies_multi_word_short_as_hybrid() {
        // 3 palabras sin prefijo ni caracteres exact → hybrid
        assert_eq!(classify("validate user email"), SearchMode::Hybrid);
    }
}
