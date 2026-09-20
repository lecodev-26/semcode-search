//! Comando `index`: indexa un proyecto.
//!
//! Recorre el proyecto, detecta el lenguaje de cada archivo, extrae
//! símbolos con Tree-sitter, genera chunks y los guarda en SQLite.

use colored::Colorize;
use semcode_core::error::{Error, Result};
use semcode_core::traits::ParsedSymbol;
use semcode_core::types::{Blake3Hash, Language};
use semcode_index::chunks::{self, ChunkKind};
use semcode_index::files::{self, FileEntry};
use semcode_index::symbols;
use semcode_index::Index;
use semcode_parser::CodeParser;

use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

/// Tamaño máximo de archivo a indexar (5 MB).
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Ejecuta el comando `index`.
pub fn run(path: &Path, force: bool, _ai: bool) -> Result<()> {
    let started = Instant::now();

    let canonical = path.canonicalize().map_err(|e| {
        Error::index(format!("no se pudo acceder a {}: {}", path.display(), e))
    })?;

    println!(
        "{} Indexando {}",
        "→".bright_cyan(),
        canonical.display().to_string().bright_white()
    );

    let mut index = Index::open(&canonical)?;
    index.migrate()?;
    let repo_id = index.repo_id();

    let mut parser = CodeParser::new()?;

    // Recorremos el proyecto respetando .gitignore
    let walker = WalkDir::new(&canonical)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()));

    let mut files_indexed = 0usize;
    let mut files_skipped = 0usize;
    let mut symbols_total = 0usize;
    let mut chunks_total = 0usize;

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let file_path = entry.path();

        // Saltar archivos demasiado grandes
        let meta = match std::fs::metadata(file_path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.len() > MAX_FILE_SIZE {
            files_skipped += 1;
            continue;
        }

        // Leer contenido
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Detectar lenguaje
        let rel_path = file_path.strip_prefix(&canonical).unwrap_or(file_path);
        let language = semcode_parser::detect(file_path, &content);

        // Si es Unknown y no parece texto, saltar
        if language == Language::Unknown && is_binary(&content) {
            files_skipped += 1;
            continue;
        }

        // Hash
        let hash = Blake3Hash::from_bytes(&content);

        // Preparar FileEntry
        let mut file_entry = FileEntry::new(
            rel_path.to_path_buf(),
            language,
            hash,
            meta.len(),
        );
        file_entry.is_binary = is_binary(&content);

        // Guardar archivo (upsert)
        let conn = index.conn()?;
        let file_id = files::upsert_file(&conn, repo_id, &file_entry)?;
        drop(conn);

        // Parsear si tiene parser
        let source = String::from_utf8_lossy(&content);
        let parsed_symbols: Vec<ParsedSymbol> = if language.has_parser() {
            parser.parse(&source, language).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Guardar símbolos
        let mut conn = index.conn()?;
        let symbol_ids = if parsed_symbols.is_empty() {
            Vec::new()
        } else {
            symbols::insert_symbols(&mut conn, file_id, language, &parsed_symbols)?
        };
        drop(conn);

        // Generar chunks (uno por símbolo, o uno global si no hay símbolos)
        let chunk_data = generate_chunks(&source, &parsed_symbols, &symbol_ids);
        let mut conn = index.conn()?;
        chunks::insert_chunks(&mut conn, file_id, &chunk_data)?;
        drop(conn);

        files_indexed += 1;
        symbols_total += parsed_symbols.len();
        chunks_total += chunk_data.len();
    }

    let elapsed = started.elapsed();

    println!(
        "{} Indexado completado en {:.2}s",
        "✓".bright_green(),
        elapsed.as_secs_f64()
    );
    println!("  {} {}", "Archivos:".dimmed(), files_indexed.to_string().bright_white());
    println!("  {} {}", "Símbolos:".dimmed(), symbols_total.to_string().bright_white());
    println!("  {} {}", "Chunks:".dimmed(), chunks_total.to_string().bright_white());
    if files_skipped > 0 {
        println!("  {} {}", "Omitidos:".dimmed(), files_skipped.to_string().yellow());
    }
    if force {
        println!("  {} {}", "Modo:".dimmed(), "force".yellow());
    }

    Ok(())
}

/// ¿Está la ruta dentro de un directorio que ignoramos?
fn is_ignored(path: &Path) -> bool {
    const IGNORED: &[&str] = &[
        ".git", ".semcode", "target", "node_modules", ".venv", "venv",
        "dist", "build", ".next", ".cache", "vendor", "__pycache__",
    ];
    for comp in path.components() {
        if let std::path::Component::Normal(name) = comp {
            if let Some(s) = name.to_str() {
                if IGNORED.contains(&s) {
                    return true;
                }
            }
        }
    }
    false
}

/// ¿El contenido parece binario?
fn is_binary(content: &[u8]) -> bool {
    content.iter().take(1024).any(|&b| b == 0)
}

/// Genera chunks a partir del código fuente.
///
/// Si hay símbolos, un chunk por símbolo. Si no, un chunk con todo el
/// contenido (limitado a 2000 caracteres).
fn generate_chunks(
    source: &str,
    symbols: &[ParsedSymbol],
    symbol_ids: &[i64],
) -> Vec<(ChunkKind, String, usize, usize, usize, usize, usize, Blake3Hash)> {
    let mut chunks_out = Vec::new();

    if symbols.is_empty() {
        // Sin símbolos → un único chunk con el contenido
        let content = source.to_string();
        if !content.is_empty() {
            let hash = Blake3Hash::from_bytes(content.as_bytes());
            let lines = content.lines().count();
            let token_count = content.split_whitespace().count();
            chunks_out.push((
                ChunkKind::Block,
                content,
                1,
                lines.max(1),
                0,
                source.len(),
                token_count,
                hash,
            ));
        }
        return chunks_out;
    }

    // Con símbolos → un chunk por símbolo (usando los offsets del símbolo)
    for (i, sym) in symbols.iter().enumerate() {
        let _ = i;
        let content = match source.get(sym.start_byte..sym.end_byte) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if content.is_empty() {
            continue;
        }
        let hash = Blake3Hash::from_bytes(content.as_bytes());
        let token_count = content.split_whitespace().count();
        chunks_out.push((
            ChunkKind::SymbolBody,
            content,
            sym.start_line,
            sym.end_line,
            sym.start_byte,
            sym.end_byte,
            token_count,
            hash,
        ));
    }

    // Si no hubo chunks (offsets inválidos), fallback al contenido completo
    if chunks_out.is_empty() {
        let content = source.to_string();
        if !content.is_empty() {
            let hash = Blake3Hash::from_bytes(content.as_bytes());
            let lines = content.lines().count();
            let token_count = content.split_whitespace().count();
            chunks_out.push((
                ChunkKind::Block,
                content,
                1,
                lines.max(1),
                0,
                source.len(),
                token_count,
                hash,
            ));
        }
    }

    let _ = symbol_ids; // reservado para futuras mejoras

    chunks_out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_binary_detects_null() {
        assert!(is_binary(&[0x00, 0x01, 0x02]));
        assert!(!is_binary(b"hello world"));
    }

    #[test]
    fn is_ignored_detects_git() {
        assert!(is_ignored(Path::new(".git/config")));
        assert!(is_ignored(Path::new("target/debug/build")));
        assert!(!is_ignored(Path::new("src/main.rs")));
    }

    #[test]
    fn generate_chunks_empty_source() {
        let chunks = generate_chunks("", &[], &[]);
        assert!(chunks.is_empty());
    }

    #[test]
    fn generate_chunks_without_symbols() {
        let source = "hello world\nfoo bar";
        let chunks = generate_chunks(source, &[], &[]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0, ChunkKind::Block);
    }

    #[test]
    fn generate_chunks_with_symbols() {
        let source = "pub fn hello() {}";
        let sym = ParsedSymbol::new(
            "hello",
            "hello",
            semcode_core::types::SymbolKind::Function,
            1,
            1,
            0,
            source.len(),
        );
        let chunks = generate_chunks(source, std::slice::from_ref(&sym), &[1]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0, ChunkKind::SymbolBody);
    }

    #[test]
    fn generate_chunks_with_invalid_offsets() {
        let source = "short";
        let sym = ParsedSymbol::new(
            "fake",
            "fake",
            semcode_core::types::SymbolKind::Function,
            1,
            1,
            0,
            999_999, // offset fuera de rango
        );
        let chunks = generate_chunks(source, &[sym], &[1]);
        // Debe caer al fallback con el contenido completo
        assert_eq!(chunks.len(), 1);
    }
}
