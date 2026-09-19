//! Configuración global de semcode-search.
//!
//! Se puede cargar desde un archivo TOML o construir programáticamente.
//! Los valores por defecto son razonables para empezar a usar la herramienta
//! sin tener que configurar nada.

use serde::{Deserialize, Serialize};

/// Configuración completa.
///
/// Agrupa todas las secciones de configuración del sistema. Se puede cargar
/// desde TOML con `toml::from_str::<Config>(...)` o construir directamente
/// con `Config::default()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Configuración del proyecto.
    pub project: ProjectConfig,

    /// Configuración del índice.
    pub index: IndexConfig,

    /// Configuración de búsqueda.
    pub search: SearchConfig,

    /// Configuración de chunking.
    pub chunking: ChunkingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            index: IndexConfig::default(),
            search: SearchConfig::default(),
            chunking: ChunkingConfig::default(),
        }
    }
}

/// Configuración del proyecto.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    /// Ruta raíz del proyecto. Por defecto `.` (directorio actual).
    pub root: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            root: ".".to_string(),
        }
    }
}

/// Configuración del índice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IndexConfig {
    /// Ruta del archivo SQLite, relativa a la raíz del proyecto.
    pub database: String,

    /// Activar watch mode por defecto (reindexado automático).
    pub watch: bool,

    /// Tamaño máximo de archivo a indexar (en bytes). `0` = sin límite.
    pub max_file_size: u64,

    /// Extensiones a ignorar, además de las del `.gitignore`.
    pub ignore_extensions: Vec<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            database: ".semcode/index.db".to_string(),
            watch: false,
            max_file_size: 5 * 1024 * 1024, // 5 MB
            ignore_extensions: vec![
                "lock".into(),
                "min.js".into(),
                "min.css".into(),
            ],
        }
    }
}

/// Configuración de búsqueda.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    /// Modo de búsqueda por defecto.
    ///
    /// Valores posibles: `"hybrid"`, `"exact"`, `"bm25"`, `"semantic"`,
    /// `"symbol"`.
    pub mode: String,

    /// Número de resultados devueltos por defecto.
    pub default_limit: usize,

    /// Umbral mínimo de similitud semántica (0.0 a 1.0).
    pub min_semantic_score: f32,

    /// Pesos para la fusión RRF.
    pub weights: SearchWeights,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            mode: "hybrid".to_string(),
            default_limit: 10,
            min_semantic_score: 0.3,
            weights: SearchWeights::default(),
        }
    }
}

/// Pesos por motor en la fusión RRF.
///
/// Cada motor contribuye al ranking final con su peso multiplicado por
/// `1 / (k + rank)`. Un peso mayor hace que ese motor domine más.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchWeights {
    /// Peso del motor de búsqueda exacta (literal/regex).
    pub exact: f32,

    /// Peso del motor BM25 (FTS5).
    pub bm25: f32,

    /// Peso del motor semántico (embeddings).
    pub semantic: f32,

    /// Peso del motor estructural (símbolos AST).
    pub symbol: f32,
}

impl Default for SearchWeights {
    fn default() -> Self {
        Self {
            exact: 1.0,
            bm25: 1.2,
            semantic: 1.5,
            symbol: 1.0,
        }
    }
}

/// Configuración de chunking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChunkingConfig {
    /// Máximo de tokens por chunk.
    pub max_tokens_per_chunk: usize,

    /// Solapamiento entre chunks consecutivos (en tokens).
    pub overlap_tokens: usize,

    /// Mínimo de tokens para que un chunk se considere válido.
    pub min_chunk_tokens: usize,

    /// Dividir símbolos grandes en varios chunks con overlap.
    pub split_large_symbols: bool,

    /// Extraer comentarios de documentación como chunks separados.
    pub separate_doc_comments: bool,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_chunk: 512,
            overlap_tokens: 64,
            min_chunk_tokens: 16,
            split_large_symbols: true,
            separate_doc_comments: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_sane() {
        let c = Config::default();
        assert_eq!(c.project.root, ".");
        assert_eq!(c.index.database, ".semcode/index.db");
        assert_eq!(c.index.max_file_size, 5 * 1024 * 1024);
        assert_eq!(c.search.mode, "hybrid");
        assert_eq!(c.search.default_limit, 10);
        assert_eq!(c.chunking.max_tokens_per_chunk, 512);
    }

    #[test]
    fn search_weights_default() {
        let w = SearchWeights::default();
        assert!(w.semantic > w.bm25);
        assert!(w.bm25 > w.exact);
    }

    #[test]
    fn config_serializes_to_toml() {
        let c = Config::default();
        let toml_str = toml::to_string(&c).unwrap();
        assert!(toml_str.contains("[project]"));
        assert!(toml_str.contains("[index]"));
        assert!(toml_str.contains("[search]"));
        assert!(toml_str.contains("[chunking]"));
    }

    #[test]
    fn config_deserializes_from_empty_toml() {
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.search.mode, "hybrid");
    }
}
