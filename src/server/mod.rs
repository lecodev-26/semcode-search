//! Módulo Server - API REST con Axum
//!
//! Expone una API HTTP para buscar desde cualquier herramienta.

use anyhow::Result;
use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::cache::Cache;

/// Respuesta genérica de error
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Respuesta para `/`
#[derive(Serialize)]
struct InfoResponse {
    name: String,
    version: String,
    description: String,
    endpoints: Vec<String>,
}

/// Query params para `/search`
#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    semantic: Option<bool>,
    limit: Option<usize>,
}

/// Respuesta de `/search`
#[derive(Serialize)]
struct SearchResponse {
    query: String,
    semantic: bool,
    total: usize,
    results: Vec<SearchResultJson>,
}

/// Resultado individual en JSON
#[derive(Serialize)]
struct SearchResultJson {
    path: String,
    score: Option<f32>,
    matches: usize,
    size: u64,
    preview: String,
}

/// Respuesta de `/stats`
#[derive(Serialize)]
struct StatsResponse {
    total_files: usize,
    total_lines: usize,
    total_size: u64,
    languages: HashMap<String, LanguageStats>,
    has_embeddings: bool,
    last_indexed: String,
}

#[derive(Serialize)]
struct LanguageStats {
    files: usize,
    size: u64,
    percentage: f32,
}

/// Endpoint `/`
async fn index() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: "semcode-search".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Fast semantic code search API".to_string(),
        endpoints: vec![
            "GET /".to_string(),
            "GET /health".to_string(),
            "GET /stats".to_string(),
            "GET /search?q=query".to_string(),
            "GET /search?q=query&semantic=true".to_string(),
        ],
    })
}

/// Endpoint `/health`
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "semcode-search",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Endpoint `/stats`
async fn stats() -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_path = Path::new(".semantic-index.json");

    if !cache_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No cache found. Run 'semcode-search index' first.".to_string(),
            }),
        ));
    }

    let cache = Cache::load(cache_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to load cache: {}", e),
            }),
        )
    })?;

    let total_files = cache.entries.len();
    let total_size: u64 = cache.entries.values().map(|e| e.size).sum();
    let total_lines: usize = cache
        .entries
        .values()
        .map(|e| e.content.lines().count())
        .sum();

    let mut languages: HashMap<String, LanguageStats> = HashMap::new();
    for entry in cache.entries.values() {
        if let Some(ext) = entry.path.extension().and_then(|e| e.to_str()) {
            let language = languages
                .entry(ext.to_string())
                .or_insert(LanguageStats {
                    files: 0,
                    size: 0,
                    percentage: 0.0,
                });
            language.files += 1;
            language.size += entry.size;
        }
    }

    let total = total_files as f32;
    for lang in languages.values_mut() {
        lang.percentage = (lang.files as f32 / total) * 100.0;
    }

    Ok(Json(StatsResponse {
        total_files,
        total_lines,
        total_size,
        languages,
        has_embeddings: cache.has_embeddings,
        last_indexed: cache.updated.clone(),
    }))
}

/// Endpoint `/search`
async fn search(
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let query = params.q.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing parameter 'q'".to_string(),
            }),
        )
    })?;

    let cache_path = Path::new(".semantic-index.json");
    if !cache_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No cache found. Run 'semcode-search index' first.".to_string(),
            }),
        ));
    }

    let cache = Cache::load(cache_path).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to load cache: {}", e),
            }),
        )
    })?;

    let limit = params.limit.unwrap_or(10);
    let semantic = params.semantic.unwrap_or(false);

    // Búsqueda simple por texto
    let query_lower = query.to_lowercase();
    let mut results: Vec<SearchResultJson> = cache
        .entries
        .iter()
        .filter_map(|(path, entry)| {
            let content_lower = entry.content.to_lowercase();
            if content_lower.contains(&query_lower) {
                let matches = content_lower.matches(&query_lower).count();
                let preview: String = entry
                    .content
                    .lines()
                    .take(5)
                    .collect::<Vec<_>>()
                    .join("\n");

                Some(SearchResultJson {
                    path: path.display().to_string(),
                    score: None,
                    matches,
                    size: entry.size,
                    preview,
                })
            } else {
                None
            }
        })
        .collect();

    results.sort_by_key(|r| std::cmp::Reverse(r.matches));
    results.truncate(limit);

    Ok(Json(SearchResponse {
        query,
        semantic,
        total: results.len(),
        results,
    }))
}

/// Inicia el servidor
pub async fn run_server(host: &str, port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/stats", get(stats))
        .route("/search", get(search));

    let addr = format!("{}:{}", host, port);
    println!("🚀 Servidor iniciado en http://{}", addr);
    println!("   Presiona Ctrl+C para detener");
    println!();
    println!("   Endpoints disponibles:");
    println!("   → GET http://{}/", addr);
    println!("   → GET http://{}/health", addr);
    println!("   → GET http://{}/stats", addr);
    println!("   → GET http://{}/search?q=query", addr);
    println!();

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind: {}", e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}