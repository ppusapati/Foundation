//! M14: Determinism Helpers – utilities for ensuring deterministic execution.

use foundation::context::DeterministicContext;
use foundation::errors::{FoundationError, FoundationResult};
use foundation::hash;
use std::collections::BTreeMap;

/// Sort a mutable slice in a deterministic order (stable sort).
pub fn deterministic_sort<T: Ord>(slice: &mut [T]) {
    slice.sort();
}

/// Sort by a key function (stable).
pub fn deterministic_sort_by_key<T, K: Ord>(slice: &mut [T], key_fn: impl Fn(&T) -> K) {
    slice.sort_by(|a, b| key_fn(a).cmp(&key_fn(b)));
}

/// Deterministic iteration over a HashMap-like structure.
/// Converts to BTreeMap which has deterministic iteration order.
pub fn deterministic_entries<K: Ord + Clone, V: Clone>(
    map: &std::collections::HashMap<K, V>,
) -> Vec<(K, V)> {
    let btree: BTreeMap<K, V> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    btree.into_iter().collect()
}

/// A deterministic tie-breaker that uses content hashing.
/// Given two items with the same priority, use their hash to break the tie.
pub fn hash_tiebreak(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let ha = hash::sha256_hex(a);
    let hb = hash::sha256_hex(b);
    ha.cmp(&hb)
}

/// A deterministic shuffle using a seed (Fisher-Yates with deterministic PRNG).
pub fn deterministic_shuffle<T>(slice: &mut [T], seed: u64) {
    let len = slice.len();
    if len <= 1 {
        return;
    }
    let mut state = seed;
    for i in (1..len).rev() {
        // Simple LCG PRNG
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        slice.swap(i, j);
    }
}

/// A deterministic selection of `k` items from a slice using a seed.
pub fn deterministic_select<T: Clone>(slice: &[T], k: usize, seed: u64) -> Vec<T> {
    if k >= slice.len() {
        return slice.to_vec();
    }
    let mut indices: Vec<usize> = (0..slice.len()).collect();
    deterministic_shuffle(&mut indices, seed);
    indices[..k].iter().map(|&i| slice[i].clone()).collect()
}

/// Verify that two values produce the same canonical hash.
pub fn verify_determinism<T: serde::Serialize>(a: &T, b: &T) -> FoundationResult<bool> {
    let ha = hash::hash_json(a).map_err(|e| FoundationError::HashError(e))?;
    let hb = hash::hash_json(b).map_err(|e| FoundationError::HashError(e))?;
    Ok(ha == hb)
}

/// A guard that records a content hash before and after an operation,
/// verifying no unintended mutation occurred.
pub struct DeterminismGuard {
    hash_before: String,
}

impl DeterminismGuard {
    /// Begin a determinism guard by capturing the hash of the current state.
    pub fn begin<T: serde::Serialize>(value: &T) -> FoundationResult<Self> {
        let h = hash::hash_json(value).map_err(|e| FoundationError::HashError(e))?;
        Ok(DeterminismGuard { hash_before: h })
    }

    /// Verify that the value has not changed since the guard was created.
    pub fn verify_unchanged<T: serde::Serialize>(&self, value: &T) -> FoundationResult<bool> {
        let h = hash::hash_json(value).map_err(|e| FoundationError::HashError(e))?;
        Ok(self.hash_before == h)
    }

    /// Verify that the value HAS changed since the guard was created.
    pub fn verify_changed<T: serde::Serialize>(&self, value: &T) -> FoundationResult<bool> {
        let h = hash::hash_json(value).map_err(|e| FoundationError::HashError(e))?;
        Ok(self.hash_before != h)
    }
}

/// Execute a function twice with the same context seed and verify determinism.
pub fn verify_deterministic_execution<F, T>(seed: u64, f: F) -> FoundationResult<T>
where
    F: Fn(&mut DeterministicContext) -> FoundationResult<T>,
    T: serde::Serialize + PartialEq + std::fmt::Debug,
{
    let mut ctx1 = DeterministicContext::new(seed);
    let result1 = f(&mut ctx1)?;

    let mut ctx2 = DeterministicContext::new(seed);
    let result2 = f(&mut ctx2)?;

    if result1 != result2 {
        return Err(FoundationError::ValidationFailed(
            "non-deterministic execution detected".into(),
        ));
    }

    Ok(result1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sort_basic() {
        let mut v = vec![3, 1, 4, 1, 5, 9];
        deterministic_sort(&mut v);
        assert_eq!(v, vec![1, 1, 3, 4, 5, 9]);
    }

    #[test]
    fn deterministic_shuffle_is_deterministic() {
        let mut a = vec![1, 2, 3, 4, 5];
        let mut b = vec![1, 2, 3, 4, 5];
        deterministic_shuffle(&mut a, 42);
        deterministic_shuffle(&mut b, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn deterministic_shuffle_different_seeds() {
        let mut a = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut b = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        deterministic_shuffle(&mut a, 42);
        deterministic_shuffle(&mut b, 99);
        assert_ne!(a, b);
    }

    #[test]
    fn deterministic_select_basic() {
        let items = vec![1, 2, 3, 4, 5];
        let s1 = deterministic_select(&items, 3, 42);
        let s2 = deterministic_select(&items, 3, 42);
        assert_eq!(s1, s2);
        assert_eq!(s1.len(), 3);
    }

    #[test]
    fn hash_tiebreak_deterministic() {
        let o1 = hash_tiebreak(b"alpha", b"beta");
        let o2 = hash_tiebreak(b"alpha", b"beta");
        assert_eq!(o1, o2);
    }

    #[test]
    fn verify_determinism_equal() {
        let a = serde_json::json!({"x": 1});
        let b = serde_json::json!({"x": 1});
        assert!(verify_determinism(&a, &b).unwrap());
    }

    #[test]
    fn verify_determinism_different() {
        let a = serde_json::json!({"x": 1});
        let b = serde_json::json!({"x": 2});
        assert!(!verify_determinism(&a, &b).unwrap());
    }

    #[test]
    fn determinism_guard() {
        let val = serde_json::json!({"state": "initial"});
        let guard = DeterminismGuard::begin(&val).unwrap();
        assert!(guard.verify_unchanged(&val).unwrap());

        let changed = serde_json::json!({"state": "modified"});
        assert!(guard.verify_changed(&changed).unwrap());
    }

    #[test]
    fn verify_deterministic_execution_pass() {
        let result = verify_deterministic_execution(42, |ctx| {
            let _ = ctx.tick("op", "test")?;
            Ok(42i32)
        });
        assert_eq!(result.unwrap(), 42);
    }
}
