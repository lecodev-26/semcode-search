//! Tipos del motor de búsqueda.
//!
//! Aquí viven las estructuras que se usan entre los distintos motores
//! (Exact, BM25, Semantic, Symbol) y el motor híbrido.

use semcode_core::types::{Language, SymbolKind, Visibility};

use serde::{Deserialize, Serialize};

/// Modo de búsqueda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    /// Solo búsqueda exacta (literal/regex).
    Exact,
    /// Solo BM25 (FTS5).
    Bm25,
    /// Solo semántica (embeddings).
    Semantic,
    /// Solo estructural (símbolos).
    Symbol,
    /// Híbrida: los 4 motores + RRF.
    Hybrid,
}

impl SearchMode {
    /// Nombre canónico.
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchMode::Exact => "exact",
            SearchMode::Bm25 => "bm25",
            SearchMode::Semantic => "semantic",
            SearchMode::Symbol => "symbol",
            SearchMode::Hybrid => "hybrid",
        }
    }

    /// Parsea desde string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "exact" => Some(SearchMode::Exact),
            "bm25" | "keyword" => Some(SearchMode::Bm25),
            "semantic" | "ai" => Some(SearchMode::Semantic),
            "symbol" => Some(SearchMode::Symbol),
            "hybrid" | "all" => Some(SearchMode::Hybrid),
            _ => None,
        }
    }
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Hybrid
    }
}

/// Origen de un resultado (qué motor lo encontró).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// Búsqueda exacta.
    Exact,
    /// Búsqueda BM25.
    Bm25,
    /// Búsqueda semántica.
    Semantic,
    /// Búsqueda estructural.
    Symbol,
}

impl Source {
    /// Nombre canónico.
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::Exact => "exact",
            Source::Bm25 => "bm25",
            Source::Semantic => "semantic",
            Source::Symbol => "symbol",
        }
    }
}

/// Filtros de búsqueda.
///
/// Todos los campos son opcionales. Los que están a `None` no filtran.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filters {
    /// Filtrar por lenguaje.
    pub language: Option<Language>,

    /// Filtrar por tipo de símbolo.
    pub symbol_kind: Option<SymbolKind>,

    /// Filtrar por visibilidad.
    pub visibility: Option<Visibility>,

    /// Solo símbolos async.
    pub is_async: Option<bool>,

    /// Solo símbolos exportados/públicos.
    pub is_exported: Option<bool>,

    /// Tamaño mínimo del archivo (bytes).
    pub min_size: Option<u64>,

    /// Tamaño máximo del archivo (bytes).
    pub max_size: Option<u64>,

    /// Umbral mínimo de similitud semántica.
    pub min_semantic_score: Option<f32>,

    /// Búsqueda case-sensitive (solo para Exact).
    pub case_sensitive: bool,

    /// Activar reranker (feature `rerank`).
    pub deep: bool,
}

/// Consulta de búsqueda.
#[derive(Debug, Clone)]
pub struct Query {
    /// Texto de la consulta.
    pub text: String,

    /// Modo de búsqueda.
    pub mode: SearchMode,

    /// Número máximo de resultados.
    pub limit: usize,

    /// Filtros.
    pub filters: Filters,
}

impl Query {
    /// Constructor de conveniencia.
    pub fn new(text: impl Into<String>, mode: SearchMode, limit: usize) -> Self {
        Self {
            text: text.into(),
            mode,
            limit,
            filters: Filters::default(),
        }
    }

    /// Constructor con modo híbrido (default).
    pub fn hybrid(text: impl Into<String>, limit: usize) -> Self {
        Self::new(text, SearchMode::Hybrid, limit)
    }
}

/// Información del símbolo asociado a un resultado.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Nombre simple.
    pub name: String,

    /// Nombre cualificado.
    pub qualified_name: String,

    /// Tipo de símbolo.
    pub kind: SymbolKind,

    /// Firma (si existe).
    pub signature: Option<String>,

    /// Comentario de doc (si existe).
    pub doc_comment: Option<String>,
}

/// Resultado de búsqueda.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Ruta del archivo (relativa al repo).
    pub path: String,

    /// Score final (RRF o del motor específico).
    pub score: f32,

    /// Snippet con contexto.
    pub snippet: String,

    /// Línea inicial del match.
    pub start_line: usize,

    /// Línea final del match.
    pub end_line: usize,

    /// Símbolo asociado (si aplica).
    pub symbol: Option<SymbolInfo>,

    /// Motores que encontraron este resultado.
    pub sources: Vec<Source>,

    /// Breakdown de scores por motor (para debug).
    pub score_breakdown: ScoreBreakdown,
}

/// Breakdown de scores por motor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Score del motor Exact (si aplica).
    pub exact: Option<f32>,

    /// Score del motor BM25 (si aplica).
    pub bm25: Option<f32>,

    /// Score del motor Semantic (si aplica).
    pub semantic: Option<f32>,

    /// Score del motor Symbol (si aplica).
    pub symbol: Option<f32>,
}

/// Item rankeado devuelto por un motor concreto.
///
/// Es un tipo interno que se usa antes de hidratar los resultados con
/// contenido, símbolo, etc.
#[derive(Debug, Clone)]
pub struct RankedItem {
    /// ID del chunk (o del símbolo, según motor).
    pub chunk_id: i64,

    /// Score asignado por el motor.
    pub score: f32,

    /// Origen del match.
    pub source: Source,
}

/// Item fusionado por RRF.
#[derive(Debug, Clone)]
pub struct FusedItem {
    /// ID del chunk.
    pub chunk_id: i64,

    /// Score RRF final.
    pub rrf_score: f32,

    /// Motores que lo encontraron.
    pub sources: Vec<Source>,

    /// Breakdown de scores.
    pub breakdown: ScoreBreakdown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_mode_as_str() {
        assert_eq!(SearchMode::Exact.as_str(), "exact");
        assert_eq!(SearchMode::Hybrid.as_str(), "hybrid");
    }

    #[test]
    fn search_mode_from_str() {
        assert_eq!(SearchMode::from_str("exact"), Some(SearchMode::Exact));
        assert_eq!(SearchMode::from_str("KEYWORD"), Some(SearchMode::Bm25));
        assert_eq!(SearchMode::from_str("ai"), Some(SearchMode::Semantic));
        assert_eq!(SearchMode::from_str("unknown"), None);
    }

    #[test]
    fn search_mode_default_is_hybrid() {
        assert_eq!(SearchMode::default(), SearchMode::Hybrid);
    }

    #[test]
    fn source_as_str() {
        assert_eq!(Source::Exact.as_str(), "exact");
        assert_eq!(Source::Semantic.as_str(), "semantic");
    }

    #[test]
    fn query_hybrid_constructor() {
        let q = Query::hybrid("hello", 10);
        assert_eq!(q.text, "hello");
        assert_eq!(q.mode, SearchMode::Hybrid);
        assert_eq!(q.limit, 10);
    }

    #[test]
    fn filters_default_is_empty() {
        let f = Filters::default();
        assert!(f.language.is_none());
        assert!(f.symbol_kind.is_none());
        assert!(!f.case_sensitive);
        assert!(!f.deep);
    }
}
