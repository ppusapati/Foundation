//! M13: Validation Helpers – reusable validation functions and combinators.

use foundation::errors::{FoundationError, FoundationResult};
use foundation::primitives::{SafeFloat, SafeInteger};

/// Validate that a string is non-empty.
pub fn non_empty_string(value: &str, field_name: &str) -> FoundationResult<()> {
    if value.is_empty() {
        return Err(FoundationError::ValidationFailed(format!(
            "{} must not be empty",
            field_name
        )));
    }
    Ok(())
}

/// Validate that a string has a minimum length.
pub fn min_length(value: &str, min: usize, field_name: &str) -> FoundationResult<()> {
    if value.len() < min {
        return Err(FoundationError::ValidationFailed(format!(
            "{} must be at least {} characters, got {}",
            field_name,
            min,
            value.len()
        )));
    }
    Ok(())
}

/// Validate that a string has a maximum length.
pub fn max_length(value: &str, max: usize, field_name: &str) -> FoundationResult<()> {
    if value.len() > max {
        return Err(FoundationError::ValidationFailed(format!(
            "{} must be at most {} characters, got {}",
            field_name,
            max,
            value.len()
        )));
    }
    Ok(())
}

/// Validate that a string contains only alphanumeric characters and the given extra chars.
pub fn alphanumeric_with(
    value: &str,
    extra_chars: &[char],
    field_name: &str,
) -> FoundationResult<()> {
    for ch in value.chars() {
        if !ch.is_alphanumeric() && !extra_chars.contains(&ch) {
            return Err(FoundationError::ValidationFailed(format!(
                "{} contains invalid character: '{}'",
                field_name, ch
            )));
        }
    }
    Ok(())
}

/// Validate that a SafeInteger is within [min, max].
pub fn integer_in_range(
    value: SafeInteger,
    min: i64,
    max: i64,
    field_name: &str,
) -> FoundationResult<()> {
    if value.value() < min || value.value() > max {
        return Err(FoundationError::OutOfRange(format!(
            "{} must be in [{}, {}], got {}",
            field_name,
            min,
            max,
            value.value()
        )));
    }
    Ok(())
}

/// Validate that a SafeFloat is within [min, max].
pub fn float_in_range(
    value: SafeFloat,
    min: f64,
    max: f64,
    field_name: &str,
) -> FoundationResult<()> {
    if value.value() < min || value.value() > max {
        return Err(FoundationError::OutOfRange(format!(
            "{} must be in [{}, {}], got {}",
            field_name,
            min,
            max,
            value.value()
        )));
    }
    Ok(())
}

/// Validate that a SafeFloat is positive (> 0).
pub fn positive_float(value: SafeFloat, field_name: &str) -> FoundationResult<()> {
    if value.value() <= 0.0 {
        return Err(FoundationError::OutOfRange(format!(
            "{} must be positive, got {}",
            field_name,
            value.value()
        )));
    }
    Ok(())
}

/// Validate that a SafeInteger is positive (> 0).
pub fn positive_integer(value: SafeInteger, field_name: &str) -> FoundationResult<()> {
    if value.value() <= 0 {
        return Err(FoundationError::OutOfRange(format!(
            "{} must be positive, got {}",
            field_name,
            value.value()
        )));
    }
    Ok(())
}

/// Validate that a collection is non-empty.
pub fn non_empty_collection<T>(collection: &[T], field_name: &str) -> FoundationResult<()> {
    if collection.is_empty() {
        return Err(FoundationError::ValidationFailed(format!(
            "{} must not be empty",
            field_name
        )));
    }
    Ok(())
}

/// Validate that a collection has at most `max` elements.
pub fn max_collection_size<T>(
    collection: &[T],
    max: usize,
    field_name: &str,
) -> FoundationResult<()> {
    if collection.len() > max {
        return Err(FoundationError::ValidationFailed(format!(
            "{} must have at most {} elements, got {}",
            field_name,
            max,
            collection.len()
        )));
    }
    Ok(())
}

/// Validate that all elements in a collection are unique (by key).
pub fn unique_by<T, K: Eq + std::hash::Hash>(
    collection: &[T],
    key_fn: impl Fn(&T) -> K,
    field_name: &str,
) -> FoundationResult<()> {
    let mut seen = std::collections::HashSet::new();
    for item in collection {
        if !seen.insert(key_fn(item)) {
            return Err(FoundationError::ValidationFailed(format!(
                "{} contains duplicate entries",
                field_name
            )));
        }
    }
    Ok(())
}

/// A validation result collector that accumulates multiple errors.
#[derive(Debug, Default)]
pub struct ValidationCollector {
    errors: Vec<String>,
}

impl ValidationCollector {
    pub fn new() -> Self {
        ValidationCollector { errors: Vec::new() }
    }

    /// Run a validation check; if it fails, collect the error.
    pub fn check(&mut self, result: FoundationResult<()>) {
        if let Err(e) = result {
            self.errors.push(e.to_string());
        }
    }

    /// Add a custom error message.
    pub fn add_error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }

    /// Return all collected errors.
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// Return Ok if no errors, or Err with all collected errors.
    pub fn result(&self) -> FoundationResult<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(FoundationError::ValidationFailed(
                self.errors.join("; "),
            ))
        }
    }

    /// Whether any errors were collected.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_string_valid() {
        assert!(non_empty_string("hello", "name").is_ok());
    }

    #[test]
    fn non_empty_string_invalid() {
        assert!(non_empty_string("", "name").is_err());
    }

    #[test]
    fn min_length_valid() {
        assert!(min_length("hello", 3, "name").is_ok());
    }

    #[test]
    fn min_length_invalid() {
        assert!(min_length("hi", 3, "name").is_err());
    }

    #[test]
    fn alphanumeric_with_valid() {
        assert!(alphanumeric_with("hello-world_1", &['-', '_'], "id").is_ok());
    }

    #[test]
    fn alphanumeric_with_invalid() {
        assert!(alphanumeric_with("hello world", &['-', '_'], "id").is_err());
    }

    #[test]
    fn integer_in_range_valid() {
        assert!(integer_in_range(SafeInteger::new(5), 0, 10, "val").is_ok());
    }

    #[test]
    fn integer_in_range_invalid() {
        assert!(integer_in_range(SafeInteger::new(15), 0, 10, "val").is_err());
    }

    #[test]
    fn float_in_range_valid() {
        assert!(float_in_range(SafeFloat::new(0.5).unwrap(), 0.0, 1.0, "val").is_ok());
    }

    #[test]
    fn validation_collector() {
        let mut vc = ValidationCollector::new();
        vc.check(non_empty_string("ok", "f1"));
        vc.check(non_empty_string("", "f2"));
        vc.check(non_empty_string("", "f3"));
        assert!(vc.has_errors());
        assert_eq!(vc.errors().len(), 2);
        assert!(vc.result().is_err());
    }

    #[test]
    fn validation_collector_no_errors() {
        let mut vc = ValidationCollector::new();
        vc.check(non_empty_string("ok", "f1"));
        assert!(!vc.has_errors());
        assert!(vc.result().is_ok());
    }

    #[test]
    fn unique_by_valid() {
        let items = vec!["a", "b", "c"];
        assert!(unique_by(&items, |s| *s, "items").is_ok());
    }

    #[test]
    fn unique_by_invalid() {
        let items = vec!["a", "b", "a"];
        assert!(unique_by(&items, |s| *s, "items").is_err());
    }
}
