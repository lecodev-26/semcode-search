//! Tipos pblicos de la biblioteca

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Resultado de una bsqueda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Ruta del archivo
    pub path: PathBuf,

    /// Contenido del archivo (o fragmento)
    pub content: String,

    /// Nmero de coincidencias
    pub matches: usize,

    /// Tamao del archivo en bytes
    pub size: u64,

    /// Lnea de inicio de la coincidencia
    pub line_start: Option<usize>,

    /// Lnea de fin de la coincidencia
    pub line_end: Option<usize>,

    /// Puntuacin de similitud (para bsqueda semntica)
    pub score: Option<f32>,

    /// Lista de palabras clave relevantes (para bsqueda semntica)
    pub keywords: Vec<String>,
}

/// Configuracin del motor de bsqueda
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Ruta a buscar
    pub path: String,

    /// Extensiones a incluir (ej: ["rs", "py"])
    pub extensions: Option<Vec<String>>,

    /// Carpetas a ignorar
    pub ignore_dirs: Vec<String>,

    /// Bsqueda exacta (palabra completa)
    pub exact: bool,

    /// Ignorar maysculas/minsculas
    pub ignore_case: bool,

    /// Modo verboso
    pub verbose: bool,

    /// Ignorar cach
    pub no_cache: bool,

    /// Mostrar resumen
    pub summary: bool,

    /// Tamao mximo de archivo
    pub max_size: Option<String>,

    /// Patrn glob para ignorar archivos
    pub ignore_pattern: Option<String>,

    /// Modo interactivo
    pub interactive: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            extensions: None,
            ignore_dirs: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
            ],
            exact: false,
            ignore_case: false,
            verbose: false,
            no_cache: false,
            summary: true,
            max_size: None,
            ignore_pattern: None,
            interactive: false,
        }
    }
}
