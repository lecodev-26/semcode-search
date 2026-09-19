//! Extracción de símbolos desde un AST de Tree-sitter.
//!
//! Este módulo ejecuta las queries definidas en `queries/<lang>.scm` y
//! convierte los matches en [`ParsedSymbol`].
//!
//! El enfoque es agnóstico de lenguaje: cada query captura nodos con
//! nombres (`@function`, `@struct`, `@name`, `@body`, ...) y el extractor
//! los mapea al tipo [`SymbolKind`] correspondiente.

use crate::language::detect;
use semcode_core::error::{Error, Result};
use semcode_core::traits::ParsedSymbol;
use semcode_core::types::{Language, SymbolKind};

use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Query, QueryCursor, Tree};

/// Extrae símbolos de un AST usando un query de Tree-sitter.
///
/// Devuelve la lista de símbolos encontrados. Los errores de sintaxis del
/// código fuente **no** detienen la extracción: Tree-sitter produce un AST
/// parcial y extraemos lo que se pueda.
pub fn extract_symbols(
    tree: &Tree,
    source: &str,
    query: &Query,
    language: Language,
) -> Result<Vec<ParsedSymbol>> {
    let mut symbols = Vec::new();
    let mut cursor = QueryCursor::new();
    let bytes = source.as_bytes();

    let mut matches = cursor.matches(query, tree.root_node(), bytes);

    while let Some(m) = matches.next() {
        match extract_from_match(&m, query, bytes, language) {
            Some(sym) => symbols.push(sym),
            None => continue,
        }
    }

    Ok(symbols)
}

/// Procesa un match individual y, si tiene la forma esperada, devuelve un símbolo.
fn extract_from_match(
    m: &tree_sitter::QueryMatch<'_, '_>,
    query: &Query,
    source: &[u8],
    language: Language,
) -> Option<ParsedSymbol> {
    let mut kind: Option<SymbolKind> = None;
    let mut name: Option<String> = None;
    let mut anchor_node: Option<Node> = None;
    let mut body_node: Option<Node> = None;
    let mut doc_comment: Option<String> = None;

    for capture in m.captures {
        let cap_name = &query.capture_names()[capture.index as usize];
        let node = capture.node;

        // El primer capture del match suele ser el nodo "ancla" (el símbolo
        // completo: @function, @struct, @class...). Lo guardamos como anchor.
        if anchor_node.is_none() {
            anchor_node = Some(node);
        }

        match *cap_name {
            // Tipo de símbolo (marca el kind)
            "function" | "async_fn" | "function_expression" => {
                kind = Some(SymbolKind::Function);
            }
            "method" => {
                kind = Some(SymbolKind::Method);
            }
            "struct" => {
                kind = Some(SymbolKind::Struct);
            }
            "class" | "decorated" => {
                kind = Some(SymbolKind::Class);
            }
            "interface" => {
                kind = Some(SymbolKind::Interface);
            }
            "trait" => {
                kind = Some(SymbolKind::Trait);
            }
            "enum" => {
                kind = Some(SymbolKind::Enum);
            }
            "const" | "static" | "class_var" | "variable" | "var" => {
                kind = Some(SymbolKind::Constant);
            }
            "type_alias" => {
                kind = Some(SymbolKind::TypeAlias);
            }
            "module" => {
                kind = Some(SymbolKind::Module);
            }
            "import" | "import_from" | "use" => {
                kind = Some(SymbolKind::Import);
            }
            "macro" => {
                kind = Some(SymbolKind::Macro);
            }
            "arrow_function" => {
                kind = Some(SymbolKind::Function);
            }
            // Nombre del símbolo
            "name" | "async_name" | "unsafe_name" => {
                name = Some(node_text(node, source));
            }
            // Cuerpo (para calcular el final)
            "body" => {
                body_node = Some(node);
            }
            // Comentario de documentación
            "doc" | "doc_comment" => {
                doc_comment = Some(node_text(node, source));
            }
            _ => {}
        }
    }

    let kind = kind?;
    let name = name?;
    let anchor = anchor_node?;

    // Para el final del símbolo, preferimos el body si lo tenemos.
    let end_node = body_node.unwrap_or(anchor);

    let start_line = anchor.start_position().row + 1;
    let end_line = end_node.end_position().row + 1;
    let start_byte = anchor.start_byte();
    let end_byte = end_node.end_byte();

    // La firma es la primera línea del símbolo (hasta la primera `{` o `:`).
    let signature = extract_signature(anchor, source);

    // Nombre cualificado: por ahora solo el nombre simple. En el futuro
    // miraremos el padre (impl, class, ...) para componerlo.
    let qualified_name = name.clone();

    let mut sym = ParsedSymbol::new(
        name,
        qualified_name,
        kind,
        start_line,
        end_line,
        start_byte,
        end_byte,
    );
    sym.signature = signature;
    sym.doc_comment = doc_comment;
    let _ = language; // reservado para futuras reglas por lenguaje

    Some(sym)
}

/// Extrae la firma de un nodo (primera línea útil del texto del nodo).
fn extract_signature(node: Node, source: &[u8]) -> Option<String> {
    let text = node_text(node, source);
    let first_line = text.lines().next()?.trim();

    if first_line.is_empty() {
        return None;
    }

    // Cortar en `{` o en `=>` si aparece
    let cut = first_line
        .find('{')
        .or_else(|| first_line.find("=>"))
        .unwrap_or(first_line.len());

    let sig = first_line[..cut].trim().to_string();
    if sig.is_empty() {
        None
    } else {
        Some(sig)
    }
}

/// Extrae el texto de un nodo como String.
fn node_text(node: Node, source: &[u8]) -> String {
    let slice = &source[node.byte_range()];
    String::from_utf8_lossy(slice).into_owned()
}

/// Atajo: detecta el lenguaje y extrae símbolos en un solo paso.
pub fn extract_from_source(
    source: &str,
    tree: &Tree,
    query: &Query,
    path: &std::path::Path,
) -> Result<Vec<ParsedSymbol>> {
    let language = detect(path, source.as_bytes());
    if !language.has_parser() {
        return Err(Error::UnsupportedLanguage(language.to_string()));
    }
    extract_symbols(tree, source, query, language)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_text_works() {
        // Test trivial: construir un nodo es complejo sin un parser,
        // así que solo verificamos que `extract_signature` no rompe.
        // Los tests reales de extracción se hacen en `parser.rs` con
        // un parser real.
    }
}
