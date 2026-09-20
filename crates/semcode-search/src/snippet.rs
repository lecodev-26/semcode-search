//! Generador de snippets.
//!
//! Dado el contenido de un chunk y una query, extrae el fragmento más
//! relevante con contexto alrededor de los matches.

use unicode_segmentation::UnicodeSegmentation;

/// Número de líneas de contexto por defecto.
pub const DEFAULT_CONTEXT_LINES: usize = 3;

/// Longitud máxima del snippet por defecto (en caracteres).
pub const DEFAULT_MAX_CHARS: usize = 500;

/// Genera un snippet de `content` centrado en la mejor línea para `query`.
///
/// Estrategia:
/// 1. Divide `content` en líneas.
/// 2. Para cada línea, calcula un score simple (número de matches de la query).
/// 3. Elige la mejor línea.
/// 4. Extrae `context_lines` antes y después.
/// 5. Si el snippet excede `max_chars`, lo trunca por el centro.
pub fn make_snippet(content: &str, query: &str, context_lines: usize) -> String {
    make_snippet_opts(content, query, context_lines, DEFAULT_MAX_CHARS)
}

/// Igual que [`make_snippet`] pero con `max_chars` configurable.
pub fn make_snippet_opts(
    content: &str,
    query: &str,
    context_lines: usize,
    max_chars: usize,
) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let best = best_line_index(&lines, query);
    let start = best.saturating_sub(context_lines);
    let end = (best + context_lines + 1).min(lines.len());

    let snippet = lines[start..end].join("\n");
    truncate_to(&snippet, max_chars)
}

/// Encuentra el índice de la mejor línea para la query.
///
/// El "mejor" es el que tiene más coincidencias de las palabras de la
/// query (case-insensitive). En caso de empate, gana el primero.
fn best_line_index(lines: &[&str], query: &str) -> usize {
    let words = query_words(query);
    if words.is_empty() {
        return 0;
    }

    let mut best_idx = 0;
    let mut best_score = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let score = score_line(line, &words);
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    best_idx
}

/// Extrae las palabras "útiles" de la query (longitud ≥ 3, sin símbolos).
fn query_words(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_lowercase())
        .collect()
}

/// Cuenta cuántas palabras aparecen en la línea (case-insensitive).
fn score_line(line: &str, words: &[String]) -> usize {
    let lower = line.to_lowercase();
    words.iter().filter(|w| lower.contains(w.as_str())).count()
}

/// Trunca un string a `max_chars` caracteres, por el centro si es necesario.
///
/// Cuenta por grafemas (correcto con emojis y unicode).
fn truncate_to(s: &str, max_chars: usize) -> String {
    let graphemes: Vec<&str> = s.graphemes(true).collect();
    if graphemes.len() <= max_chars {
        return s.to_string();
    }

    // Cortamos por el centro
    let half = max_chars / 2;
    let start: String = graphemes[..half].concat();
    let end: String = graphemes[graphemes.len() - half..].concat();

    format!("{}...\n...{}", start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_empty_content() {
        assert_eq!(make_snippet("", "hello", 3), "");
    }

    #[test]
    fn snippet_short_content() {
        let content = "line 1\nline 2\nline 3";
        let s = make_snippet(content, "line", 5);
        assert_eq!(s, content);
    }

    #[test]
    fn snippet_centers_on_best_match() {
        let content = "aaa\nbbb\ntarget_line\nccc\nddd";
        let s = make_snippet(content, "target", 1);
        assert!(s.contains("target_line"));
        assert!(s.contains("bbb"));
        assert!(s.contains("ccc"));
    }

    #[test]
    fn snippet_no_match_uses_first_line() {
        let content = "first\nsecond\nthird";
        let s = make_snippet(content, "xyz", 1);
        assert!(s.contains("first"));
    }

    #[test]
    fn snippet_respects_max_chars() {
        let long_line = "x".repeat(1000);
        let content = format!("a\n{}\nb", long_line);
        let s = make_snippet_opts(&content, "x", 1, 100);
        // El resultado está truncado: contiene "..." en medio
        assert!(s.contains("..."));
        assert!(s.graphemes(true).count() <= 110); // 100 + "..." + "\n..."
    }

    #[test]
    fn snippet_with_unicode() {
        let content = "🎉 fiesta\n😀 emoji\n🔥 fuego";
        let s = make_snippet(content, "fiesta", 1);
        assert!(s.contains("fiesta"));
    }

    #[test]
    fn query_words_filters_short() {
        let words = query_words("a to validate email");
        assert_eq!(words, vec!["validate", "email"]);
    }

    #[test]
    fn query_words_handles_punctuation() {
        let words = query_words("validate_email(x)");
        assert_eq!(words, vec!["validate_email"]);
    }

    #[test]
    fn score_line_case_insensitive() {
        let words = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(score_line("HELLO WORLD", &words), 2);
        assert_eq!(score_line("hello", &words), 1);
        assert_eq!(score_line("nothing", &words), 0);
    }

    #[test]
    fn best_line_picks_highest_score() {
        let lines = vec!["aaa", "hello world", "hello", "nothing"];
        let words = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(best_line_index(&lines, "hello world"), 1);
    }
}
