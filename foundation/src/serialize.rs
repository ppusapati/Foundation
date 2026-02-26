//! M9: Serialization Helpers – canonical JSON serialization / deserialization.

use crate::errors::{FoundationError, FoundationResult};
use serde::{de::DeserializeOwned, Serialize};

/// Serialize a value to a canonical JSON string (compact, sorted keys).
pub fn to_canonical_json<T: Serialize>(value: &T) -> FoundationResult<String> {
    serde_json::to_string(value).map_err(|e| {
        FoundationError::SerializationError(format!("JSON serialization failed: {}", e))
    })
}

/// Serialize a value to a pretty-printed JSON string.
pub fn to_pretty_json<T: Serialize>(value: &T) -> FoundationResult<String> {
    serde_json::to_string_pretty(value).map_err(|e| {
        FoundationError::SerializationError(format!("JSON serialization failed: {}", e))
    })
}

/// Serialize a value to JSON bytes.
pub fn to_json_bytes<T: Serialize>(value: &T) -> FoundationResult<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| {
        FoundationError::SerializationError(format!("JSON serialization failed: {}", e))
    })
}

/// Deserialize a value from a JSON string.
pub fn from_json_str<T: DeserializeOwned>(json: &str) -> FoundationResult<T> {
    serde_json::from_str(json).map_err(|e| {
        FoundationError::SerializationError(format!("JSON deserialization failed: {}", e))
    })
}

/// Deserialize a value from JSON bytes.
pub fn from_json_bytes<T: DeserializeOwned>(bytes: &[u8]) -> FoundationResult<T> {
    serde_json::from_slice(bytes).map_err(|e| {
        FoundationError::SerializationError(format!("JSON deserialization failed: {}", e))
    })
}

/// Round-trip a value through JSON to produce a canonical form.
pub fn canonicalize<T: Serialize + DeserializeOwned>(value: &T) -> FoundationResult<T> {
    let json = to_canonical_json(value)?;
    from_json_str(&json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        name: String,
        value: i32,
    }

    #[test]
    fn json_roundtrip() {
        let s = Sample {
            name: "test".into(),
            value: 42,
        };
        let json = to_canonical_json(&s).unwrap();
        let s2: Sample = from_json_str(&json).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn json_bytes_roundtrip() {
        let s = Sample {
            name: "bytes".into(),
            value: 7,
        };
        let bytes = to_json_bytes(&s).unwrap();
        let s2: Sample = from_json_bytes(&bytes).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn canonicalize_roundtrip() {
        let s = Sample {
            name: "canon".into(),
            value: 99,
        };
        let s2 = canonicalize(&s).unwrap();
        assert_eq!(s, s2);
    }
}
