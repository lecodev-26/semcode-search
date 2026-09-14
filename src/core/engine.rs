//! Motor de bsqueda principal

use crate::core::error::{Result, SearchError};
use crate::core::search_files as core_search_files;
use crate::core::types::{SearchConfig, SearchResult};
use crate::core::SearchConfigInternal;

/// Motor de bsqueda principal
#[derive(Debug, Clone)]
pub struct SearchEngine {
    config: SearchConfig,
}

impl SearchEngine {
    /// Crear un nuevo motor de bsqueda
    pub fn new() -> Self {
        Self {
            config: SearchConfig::default(),
        }
    }

    /// Crear un motor con configuracin personalizada
    pub fn with_config(config: SearchConfig) -> Self {
        Self { config }
    }

    /// Indexar un directorio
    pub fn index(&self, path: &str) -> Result<()> {
        crate::core::index_files(path, Vec::new(), None, None, false)
            .map_err(|e| SearchError::Other(e.to_string()))?;
        Ok(())
    }

    /// Buscar por texto
    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let config = SearchConfigInternal {
            query: query.to_string(),
            path: self.config.path.clone(),
            ext: self.config.extensions.clone(),
            ignore: self.config.ignore_dirs.clone(),
            exact: self.config.exact,
            ignore_case: self.config.ignore_case,
            verbose: self.config.verbose,
            no_cache: self.config.no_cache,
            semantic: false,
            ai: false, //  NUEVO
            file: None,
            summary: self.config.summary,
            max_size: self.config.max_size.clone(),
            ignore_pattern: self.config.ignore_pattern.clone(),
            extract: false,
            interactive: self.config.interactive,
        };

        let results = core_search_files(config).map_err(|e| SearchError::Other(e.to_string()))?;
        Ok(results)
    }

    /// Buscar semnticamente (TF-IDF)
    pub fn search_semantic(&self, query: &str) -> Result<Vec<SearchResult>> {
        let config = SearchConfigInternal {
            query: query.to_string(),
            path: self.config.path.clone(),
            ext: self.config.extensions.clone(),
            ignore: self.config.ignore_dirs.clone(),
            exact: self.config.exact,
            ignore_case: self.config.ignore_case,
            verbose: self.config.verbose,
            no_cache: self.config.no_cache,
            semantic: true,
            ai: false, //  NUEVO
            file: None,
            summary: self.config.summary,
            max_size: self.config.max_size.clone(),
            ignore_pattern: self.config.ignore_pattern.clone(),
            extract: false,
            interactive: self.config.interactive,
        };

        let results = core_search_files(config).map_err(|e| SearchError::Other(e.to_string()))?;
        Ok(results)
    }

    /// Buscar por nombre de archivo
    pub fn search_by_filename(&self, pattern: &str) -> Result<Vec<SearchResult>> {
        let config = SearchConfigInternal {
            query: String::new(),
            path: self.config.path.clone(),
            ext: self.config.extensions.clone(),
            ignore: self.config.ignore_dirs.clone(),
            exact: false,
            ignore_case: self.config.ignore_case,
            verbose: self.config.verbose,
            no_cache: self.config.no_cache,
            semantic: false,
            ai: false, //  NUEVO
            file: Some(pattern.to_string()),
            summary: self.config.summary,
            max_size: self.config.max_size.clone(),
            ignore_pattern: self.config.ignore_pattern.clone(),
            extract: false,
            interactive: self.config.interactive,
        };

        let results = core_search_files(config).map_err(|e| SearchError::Other(e.to_string()))?;
        Ok(results)
    }

    /// Obtener la configuracin actual
    pub fn config(&self) -> &SearchConfig {
        &self.config
    }

    /// Actualizar la configuracin
    pub fn set_config(&mut self, config: SearchConfig) {
        self.config = config;
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}
