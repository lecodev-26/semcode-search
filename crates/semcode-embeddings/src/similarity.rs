//! Funciones de similitud entre vectores.
//!
//! Estas funciones son puras y no dependen de ningún proveedor de
//! embeddings. Se usan tanto en el almacenamiento (para precalcular normas)
//! como en la búsqueda semántica.

/// Producto escalar entre dos vectores.
///
/// Si los vectores tienen distinta longitud, se usa la longitud mínima.
/// (No debería pasar, pero evitamos panic.)
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Norma L2 (euclídea) de un vector.
pub fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Similitud coseno entre dos vectores.
///
/// Rango: `[-1.0, 1.0]`. Devuelve `0.0` si alguno de los vectores es cero.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let na = l2_norm(a);
    let nb = l2_norm(b);
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot(a, b) / (na * nb)
}

/// Similitud coseno asumiendo que ambos vectores ya están normalizados.
///
/// Si ya tienes la norma calculada, es más rápido usar
/// [`cosine_similarity_with_norms`].
pub fn cosine_similarity_normalized(a: &[f32], b: &[f32]) -> f32 {
    dot(a, b)
}

/// Similitud coseno con normas precalculadas.
///
/// Útil en búsquedas masivas donde no quieres recalcular `l2_norm(b)`
/// por cada comparación.
pub fn cosine_similarity_with_norms(a: &[f32], b: &[f32], norm_a: f32, norm_b: f32) -> f32 {
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot(a, b) / (norm_a * norm_b)
}

/// Normaliza un vector in-place (lo divide por su norma L2).
pub fn normalize(v: &mut [f32]) {
    let norm = l2_norm(v);
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// Devuelve una copia normalizada del vector.
pub fn normalized(v: &[f32]) -> Vec<f32> {
    let mut out = v.to_vec();
    normalize(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_basic() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let b = vec![4.0_f32, 5.0, 6.0];
        assert!((dot(&a, &b) - 32.0).abs() < 1e-6);
    }

    #[test]
    fn dot_different_lengths() {
        // Usa la longitud mínima, no panickea
        let a = vec![1.0_f32, 2.0];
        let b = vec![3.0_f32, 4.0, 5.0];
        assert!((dot(&a, &b) - 11.0).abs() < 1e-6);
    }

    #[test]
    fn l2_norm_3_4_5() {
        let v = vec![3.0_f32, 4.0];
        assert!((l2_norm(&v) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_identical() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![0.0_f32, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn cosine_opposite() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![-1.0_f32, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_zero_vector() {
        let a = vec![0.0_f32, 0.0];
        let b = vec![1.0_f32, 2.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn normalize_makes_unit() {
        let mut v = vec![3.0_f32, 4.0];
        normalize(&mut v);
        assert!((l2_norm(&v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalized_returns_copy() {
        let v = vec![3.0_f32, 4.0];
        let n = normalized(&v);
        // El original no cambia
        assert!((v[0] - 3.0).abs() < 1e-6);
        assert!((l2_norm(&n) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_with_norms_matches_regular() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let b = vec![4.0_f32, 5.0, 6.0];
        let na = l2_norm(&a);
        let nb = l2_norm(&b);
        let s1 = cosine_similarity(&a, &b);
        let s2 = cosine_similarity_with_norms(&a, &b, na, nb);
        assert!((s1 - s2).abs() < 1e-6);
    }

    #[test]
    fn cosine_normalized_matches_regular() {
        let a = normalized(&[1.0_f32, 2.0, 3.0]);
        let b = normalized(&[4.0_f32, 5.0, 6.0]);
        let s1 = cosine_similarity(&a, &b);
        let s2 = cosine_similarity_normalized(&a, &b);
        assert!((s1 - s2).abs() < 1e-6);
    }
}
