//! Detección de lenguaje de programación.
//!
//! Estrategia en cascada:
//!
//! 1. Nombre de archivo exacto (`Makefile`, `Dockerfile`, `Cargo.toml`).
//! 2. Extensión (`.rs`, `.py`, `.ts`, ...).
//! 3. Shebang (`#!/usr/bin/env python`).
//! 4. Fallback a [`Language::Unknown`].
//!
//! El enum `Language` en sí vive en `semcode-core` para que todos los crates
//! compartan el mismo tipo. Aquí solo añadimos la lógica de detección.

use semcode_core::types::Language;
use std::path::Path;

/// Detecta el lenguaje de un archivo a partir de su ruta y contenido.
///
/// El contenido es necesario para el paso de shebang; si no quieres leer
/// el archivo, pasa un slice vacío (`&[]`) y se saltará ese paso.
pub fn detect(path: &Path, content: &[u8]) -> Language {
    // 1. Nombre de archivo exacto
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        match name {
            "Makefile" | "makefile" | "GNUmakefile" => return Language::Make,
            "Dockerfile" | "Containerfile" => return Language::Dockerfile,
            "Cargo.toml" | "pyproject.toml" | "Cargo.lock" => return Language::Toml,
            _ => {}
        }
    }

    // 2. Extensión
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if let Some(lang) = Language::from_extension(ext) {
            return lang;
        }
    }

    // 3. Shebang
    if let Some(lang) = detect_from_shebang(content) {
        return lang;
    }

    // 4. Fallback
    Language::Unknown
}

/// Detecta el lenguaje a partir de la primera línea si es un shebang.
///
/// Reconoce los intérpretes más comunes: `python`, `bash`, `node`, `sh`.
pub fn detect_from_shebang(content: &[u8]) -> Option<Language> {
    if !content.starts_with(b"#!") {
        return None;
    }

    // Tomamos los primeros 256 bytes y buscamos el final de la línea
    let head = &content[..content.len().min(256)];
    let line_end = head.iter().position(|&b| b == b'\n').unwrap_or(head.len());
    let line = std::str::from_utf8(&head[..line_end]).ok()?;

    if line.contains("python") {
        Some(Language::Python)
    } else if line.contains("bash") || line.contains("/sh") || line.contains("zsh") {
        Some(Language::Bash)
    } else if line.contains("node") {
        Some(Language::JavaScript)
    } else {
        None
    }
}

/// ¿El lenguaje es parseable con Tree-sitter en la v4.0.0?
///
/// Es un alias de [`Language::has_parser`] que se queda aquí por comodidad
/// del crate parser.
pub fn is_parseable(lang: Language) -> bool {
    lang.has_parser()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rust_by_extension() {
        assert_eq!(detect(Path::new("src/main.rs"), b""), Language::Rust);
        assert_eq!(detect(Path::new("lib.rs"), b""), Language::Rust);
    }

    #[test]
    fn detects_makefile_by_name() {
        assert_eq!(detect(Path::new("Makefile"), b""), Language::Make);
        assert_eq!(detect(Path::new("Dockerfile"), b""), Language::Dockerfile);
        assert_eq!(detect(Path::new("Cargo.toml"), b""), Language::Toml);
    }

    #[test]
    fn detects_python_by_shebang() {
        let content = b"#!/usr/bin/env python3\nprint('hi')\n";
        assert_eq!(detect(Path::new("script"), content), Language::Python);
    }

    #[test]
    fn detects_bash_by_shebang() {
        let content = b"#!/bin/bash\necho hi\n";
        assert_eq!(detect(Path::new("script"), content), Language::Bash);
    }

    #[test]
    fn detects_unknown() {
        assert_eq!(detect(Path::new("file.xyz"), b""), Language::Unknown);
        assert_eq!(detect(Path::new("random"), b"no shebang"), Language::Unknown);
    }

    #[test]
    fn shebang_without_hash_is_ignored() {
        assert_eq!(detect_from_shebang(b"no shebang here"), None);
    }

    #[test]
    fn extension_wins_over_shebang() {
        // Un .rs con shebang sigue siendo Rust
        let content = b"#!/usr/bin/env python\nfn main() {}";
        assert_eq!(detect(Path::new("main.rs"), content), Language::Rust);
    }
}
