//! M8: Hash Helpers – deterministic hashing utilities.

use sha2::{Digest, Sha256};

/// Compute the SHA-256 hash of a byte slice and return it as a hex string.
#[inline]
pub fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex_encode(&hash)
}

/// Compute the SHA-256 hash of a string and return it as a hex string.
#[inline]
pub fn sha256_str(s: &str) -> String {
    sha256_hex(s.as_bytes())
}

/// Compute the SHA-256 hash of multiple byte slices concatenated.
pub fn sha256_parts(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    let hash = hasher.finalize();
    hex_encode(&hash)
}

/// Compute the raw SHA-256 bytes of a byte slice.
#[inline]
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let hash = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&hash);
    out
}

/// Hash a serializable value by first converting to canonical JSON.
pub fn hash_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    let json = serde_json::to_string(value).map_err(|e| e.to_string())?;
    Ok(sha256_str(&json))
}

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_deterministic() {
        let h1 = sha256_str("hello");
        let h2 = sha256_str("hello");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn sha256_different_input() {
        let h1 = sha256_str("hello");
        let h2 = sha256_str("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn sha256_parts_works() {
        let combined = sha256_hex(b"helloworld");
        let parts = sha256_parts(&[b"hello", b"world"]);
        assert_eq!(combined, parts);
    }

    #[test]
    fn hash_json_works() {
        let val = serde_json::json!({"a": 1, "b": 2});
        let h = hash_json(&val).unwrap();
        assert_eq!(h.len(), 64);
    }
}
