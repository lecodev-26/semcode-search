//! Operaciones sobre la tabla `embeddings`.
//!
//! Los vectores se guardan como BLOB con f32 en little-endian. El tamaño
//! es `dimensions * 4` bytes. Para 384 dimensiones (all-MiniLM-L6-v2) son
//! 1536 bytes por vector.

use semcode_core::error::{Error, Result};

use rusqlite::{params, Connection};

/// Embedding persistido.
#[derive(Debug, Clone)]
pub struct StoredEmbedding {
    /// ID interno.
    pub id: i64,

    /// ID del chunk asociado.
    pub chunk_id: i64,

    /// Modelo usado.
    pub model: String,

    /// Proveedor (`fastembed`, `openai`, `ollama`).
    pub provider: String,

    /// Número de dimensiones.
    pub dimensions: usize,

    /// Vector denso (f32).
    pub vector: Vec<f32>,

    /// Norma L2 precalculada.
    pub norm: f32,
}

/// Inserta un embedding para un chunk.
///
/// Si ya existe uno para `(chunk_id, model, provider)`, lo reemplaza.
pub fn insert_embedding(
    conn: &Connection,
    chunk_id: i64,
    model: &str,
    provider: &str,
    vector: &[f32],
) -> Result<i64> {
    let dimensions = vector.len();
    let norm = l2_norm(vector);
    let blob = vec_to_blob(vector);

    conn.execute(
        "INSERT INTO embeddings (chunk_id, model, provider, dimensions, vector, norm)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(chunk_id, model, provider) DO UPDATE SET
            dimensions = excluded.dimensions,
            vector = excluded.vector,
            norm = excluded.norm,
            created_at = datetime('now')",
        params![
            chunk_id,
            model,
            provider,
            dimensions as i64,
            blob,
            norm as f64,
        ],
    )
    .map_err(|e| Error::index(format!("insertar embedding: {}", e)))?;

    let id: i64 = conn
        .query_row(
            "SELECT id FROM embeddings WHERE chunk_id = ?1 AND model = ?2 AND provider = ?3",
            params![chunk_id, model, provider],
            |r| r.get(0),
        )
        .map_err(|e| Error::index(format!("leer embedding_id: {}", e)))?;

    Ok(id)
}

/// Devuelve todos los embeddings de un modelo concreto.
///
/// Devuelve `Vec<(chunk_id, vector, norm)>` que es lo que necesita el
/// motor de búsqueda semántica.
pub fn all_for_model(
    conn: &Connection,
    model: &str,
) -> Result<Vec<(i64, Vec<f32>, f32)>> {
    let mut stmt = conn
        .prepare(
            "SELECT chunk_id, vector, norm
             FROM embeddings
             WHERE model = ?1",
        )
        .map_err(|e| Error::index(format!("preparar all_for_model: {}", e)))?;

    let rows = stmt
        .query_map([model], |row| {
            let chunk_id: i64 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let norm: f64 = row.get(2)?;
            let vector = blob_to_vec(&blob);
            Ok((chunk_id, vector, norm as f32))
        })
        .map_err(|e| Error::index(format!("all_for_model: {}", e)))?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| Error::index(format!("leer embedding: {}", e)))?);
    }
    Ok(out)
}

/// Cuenta cuántos embeddings hay para un modelo.
pub fn count_for_model(conn: &Connection, model: &str) -> Result<usize> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM embeddings WHERE model = ?1",
            [model],
            |r| r.get(0),
        )
        .map_err(|e| Error::index(format!("contar embeddings: {}", e)))?;
    Ok(n as usize)
}

/// Borra todos los embeddings de un chunk.
pub fn delete_by_chunk(conn: &Connection, chunk_id: i64) -> Result<()> {
    conn.execute("DELETE FROM embeddings WHERE chunk_id = ?1", [chunk_id])
        .map_err(|e| Error::index(format!("borrar embeddings del chunk: {}", e)))?;
    Ok(())
}

/// Serializa un vector `f32` a BLOB little-endian.
pub fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

/// Deserializa un BLOB little-endian a vector `f32`.
pub fn blob_to_vec(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Norma L2 de un vector.
pub fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Producto escalar.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Similitud coseno entre dos vectores.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let na = l2_norm(a);
    let nb = l2_norm(b);
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot(a, b) / (na * nb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_blob_roundtrip() {
        let v = vec![1.0_f32, -2.5, 3.75, 0.0];
        let blob = vec_to_blob(&v);
        assert_eq!(blob.len(), 16); // 4 floats * 4 bytes
        let v2 = blob_to_vec(&blob);
        assert_eq!(v, v2);
    }

    #[test]
    fn l2_norm_basic() {
        let v = vec![3.0_f32, 4.0];
        assert!((l2_norm(&v) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_identical_is_one() {
        let v = vec![1.0_f32, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal_is_zero() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![0.0_f32, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }
}
