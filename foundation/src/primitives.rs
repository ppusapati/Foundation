//! M2: Primitive Types – deterministic numeric wrappers with overflow protection.

use crate::errors::{FoundationError, FoundationResult};
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// SafeInteger – i64 wrapper with checked arithmetic
// ---------------------------------------------------------------------------

/// A 64-bit signed integer that guards against silent overflow.
///
/// All arithmetic operations return `FoundationResult` and use Rust's
/// checked_* intrinsics internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SafeInteger(i64);

impl SafeInteger {
    pub const ZERO: Self = SafeInteger(0);
    pub const ONE: Self = SafeInteger(1);
    pub const MIN: Self = SafeInteger(i64::MIN);
    pub const MAX: Self = SafeInteger(i64::MAX);

    /// Create a new `SafeInteger` from an `i64`.
    #[inline]
    pub const fn new(value: i64) -> Self {
        SafeInteger(value)
    }

    /// Return the inner `i64`.
    #[inline]
    pub const fn value(&self) -> i64 {
        self.0
    }

    /// Checked addition.
    #[inline]
    pub fn checked_add(&self, rhs: SafeInteger) -> FoundationResult<SafeInteger> {
        self.0
            .checked_add(rhs.0)
            .map(SafeInteger)
            .ok_or_else(|| FoundationError::Overflow(format!("{} + {} overflows", self.0, rhs.0)))
    }

    /// Checked subtraction.
    #[inline]
    pub fn checked_sub(&self, rhs: SafeInteger) -> FoundationResult<SafeInteger> {
        self.0
            .checked_sub(rhs.0)
            .map(SafeInteger)
            .ok_or_else(|| FoundationError::Overflow(format!("{} - {} overflows", self.0, rhs.0)))
    }

    /// Checked multiplication.
    #[inline]
    pub fn checked_mul(&self, rhs: SafeInteger) -> FoundationResult<SafeInteger> {
        self.0
            .checked_mul(rhs.0)
            .map(SafeInteger)
            .ok_or_else(|| FoundationError::Overflow(format!("{} * {} overflows", self.0, rhs.0)))
    }

    /// Checked division.
    pub fn checked_div(&self, rhs: SafeInteger) -> FoundationResult<SafeInteger> {
        if rhs.0 == 0 {
            return Err(FoundationError::Overflow("division by zero".into()));
        }
        self.0
            .checked_div(rhs.0)
            .map(SafeInteger)
            .ok_or_else(|| FoundationError::Overflow(format!("{} / {} overflows", self.0, rhs.0)))
    }

    /// Checked remainder.
    pub fn checked_rem(&self, rhs: SafeInteger) -> FoundationResult<SafeInteger> {
        if rhs.0 == 0 {
            return Err(FoundationError::Overflow("remainder by zero".into()));
        }
        self.0
            .checked_rem(rhs.0)
            .map(SafeInteger)
            .ok_or_else(|| FoundationError::Overflow(format!("{} % {} overflows", self.0, rhs.0)))
    }

    /// Checked negation.
    pub fn checked_neg(&self) -> FoundationResult<SafeInteger> {
        self.0
            .checked_neg()
            .map(SafeInteger)
            .ok_or_else(|| FoundationError::Overflow(format!("-({}) overflows", self.0)))
    }

    /// Checked absolute value.
    pub fn checked_abs(&self) -> FoundationResult<SafeInteger> {
        self.0
            .checked_abs()
            .map(SafeInteger)
            .ok_or_else(|| FoundationError::Overflow(format!("abs({}) overflows", self.0)))
    }

    /// Checked exponentiation.
    pub fn checked_pow(&self, exp: u32) -> FoundationResult<SafeInteger> {
        self.0
            .checked_pow(exp)
            .map(SafeInteger)
            .ok_or_else(|| {
                FoundationError::Overflow(format!("{}^{} overflows", self.0, exp))
            })
    }
}

impl fmt::Display for SafeInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for SafeInteger {
    fn from(v: i64) -> Self {
        SafeInteger(v)
    }
}

impl From<i32> for SafeInteger {
    fn from(v: i32) -> Self {
        SafeInteger(v as i64)
    }
}

impl From<SafeInteger> for i64 {
    fn from(v: SafeInteger) -> Self {
        v.0
    }
}

// ---------------------------------------------------------------------------
// SafeFloat – f64 wrapper that rejects NaN / Infinity
// ---------------------------------------------------------------------------

/// A 64-bit float that enforces finiteness.
///
/// Construction fails (or panics in `new`) for NaN / ±Infinity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SafeFloat(f64);

impl SafeFloat {
    pub const ZERO: Self = SafeFloat(0.0);
    pub const ONE: Self = SafeFloat(1.0);
    pub const EPSILON: f64 = 1e-10;

    /// Create a new `SafeFloat`, returning an error if the value is not finite.
    #[inline]
    pub fn new(value: f64) -> FoundationResult<Self> {
        if value.is_finite() {
            Ok(SafeFloat(value))
        } else {
            Err(FoundationError::OutOfRange(format!(
                "non-finite float: {}",
                value
            )))
        }
    }

    /// Create a `SafeFloat` from a finite value without checking.
    ///
    /// # Safety
    /// Caller must ensure the value is finite.
    pub const fn new_unchecked(value: f64) -> Self {
        SafeFloat(value)
    }

    /// Return the inner `f64`.
    #[inline]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Checked addition.
    #[inline]
    pub fn checked_add(&self, rhs: SafeFloat) -> FoundationResult<SafeFloat> {
        SafeFloat::new(self.0 + rhs.0)
    }

    /// Checked subtraction.
    #[inline]
    pub fn checked_sub(&self, rhs: SafeFloat) -> FoundationResult<SafeFloat> {
        SafeFloat::new(self.0 - rhs.0)
    }

    /// Checked multiplication.
    #[inline]
    pub fn checked_mul(&self, rhs: SafeFloat) -> FoundationResult<SafeFloat> {
        SafeFloat::new(self.0 * rhs.0)
    }

    /// Checked division.
    pub fn checked_div(&self, rhs: SafeFloat) -> FoundationResult<SafeFloat> {
        if rhs.0 == 0.0 {
            return Err(FoundationError::Overflow("float division by zero".into()));
        }
        SafeFloat::new(self.0 / rhs.0)
    }

    /// Approximate equality within the default epsilon.
    pub fn approx_eq(&self, other: &SafeFloat) -> bool {
        (self.0 - other.0).abs() < Self::EPSILON
    }

    /// Approximate equality with a custom epsilon.
    pub fn approx_eq_eps(&self, other: &SafeFloat, epsilon: f64) -> bool {
        (self.0 - other.0).abs() < epsilon
    }

    /// Absolute value.
    pub fn abs(&self) -> SafeFloat {
        SafeFloat(self.0.abs())
    }

    /// Square root, returns error for negative inputs.
    pub fn sqrt(&self) -> FoundationResult<SafeFloat> {
        if self.0 < 0.0 {
            return Err(FoundationError::OutOfRange(
                "sqrt of negative number".into(),
            ));
        }
        SafeFloat::new(self.0.sqrt())
    }
}

impl fmt::Display for SafeFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for SafeFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for SafeFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl From<SafeFloat> for f64 {
    fn from(v: SafeFloat) -> Self {
        v.0
    }
}

// ---------------------------------------------------------------------------
// Percentage – a float clamped to [0.0, 100.0]
// ---------------------------------------------------------------------------

/// A percentage value validated to [0.0, 100.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Percentage(f64);

impl Percentage {
    pub fn new(value: f64) -> FoundationResult<Self> {
        if !value.is_finite() || !(0.0..=100.0).contains(&value) {
            return Err(FoundationError::OutOfRange(format!(
                "percentage must be in [0.0, 100.0], got {}",
                value
            )));
        }
        Ok(Percentage(value))
    }

    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Return as a fraction in [0.0, 1.0].
    pub fn as_fraction(&self) -> f64 {
        self.0 / 100.0
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

// ---------------------------------------------------------------------------
// NonNegativeFloat – f64 >= 0.0
// ---------------------------------------------------------------------------

/// A float that must be >= 0.0 and finite.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NonNegativeFloat(f64);

impl NonNegativeFloat {
    pub fn new(value: f64) -> FoundationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            return Err(FoundationError::OutOfRange(format!(
                "non-negative float expected, got {}",
                value
            )));
        }
        Ok(NonNegativeFloat(value))
    }

    pub const fn value(&self) -> f64 {
        self.0
    }
}

impl fmt::Display for NonNegativeFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// BoundedFloat – f64 within a generic range
// ---------------------------------------------------------------------------

/// A float bounded within [min, max].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundedFloat {
    value: f64,
    min: f64,
    max: f64,
}

impl BoundedFloat {
    pub fn new(value: f64, min: f64, max: f64) -> FoundationResult<Self> {
        if !value.is_finite() || !min.is_finite() || !max.is_finite() {
            return Err(FoundationError::OutOfRange(
                "bounded float values must be finite".into(),
            ));
        }
        if min > max {
            return Err(FoundationError::ValidationFailed(
                "min must be <= max".into(),
            ));
        }
        if value < min || value > max {
            return Err(FoundationError::OutOfRange(format!(
                "value {} not in [{}, {}]",
                value, min, max
            )));
        }
        Ok(BoundedFloat { value, min, max })
    }

    pub const fn value(&self) -> f64 {
        self.value
    }

    pub const fn min(&self) -> f64 {
        self.min
    }

    pub const fn max(&self) -> f64 {
        self.max
    }
}

impl fmt::Display for BoundedFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ∈ [{}, {}]", self.value, self.min, self.max)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_integer_checked_add_ok() {
        let a = SafeInteger::new(10);
        let b = SafeInteger::new(20);
        assert_eq!(a.checked_add(b).unwrap().value(), 30);
    }

    #[test]
    fn safe_integer_checked_add_overflow() {
        let a = SafeInteger::MAX;
        let b = SafeInteger::new(1);
        assert!(a.checked_add(b).is_err());
    }

    #[test]
    fn safe_integer_checked_div_by_zero() {
        let a = SafeInteger::new(10);
        let b = SafeInteger::ZERO;
        assert!(a.checked_div(b).is_err());
    }

    #[test]
    fn safe_float_rejects_nan() {
        assert!(SafeFloat::new(f64::NAN).is_err());
    }

    #[test]
    fn safe_float_rejects_infinity() {
        assert!(SafeFloat::new(f64::INFINITY).is_err());
        assert!(SafeFloat::new(f64::NEG_INFINITY).is_err());
    }

    #[test]
    fn safe_float_approx_eq() {
        let a = SafeFloat::new(1.0).unwrap();
        let b = SafeFloat::new(1.0 + 1e-11).unwrap();
        assert!(a.approx_eq(&b));
    }

    #[test]
    fn safe_float_div_by_zero() {
        let a = SafeFloat::new(1.0).unwrap();
        let b = SafeFloat::ZERO;
        assert!(a.checked_div(b).is_err());
    }

    #[test]
    fn percentage_valid() {
        assert!(Percentage::new(0.0).is_ok());
        assert!(Percentage::new(50.0).is_ok());
        assert!(Percentage::new(100.0).is_ok());
    }

    #[test]
    fn percentage_invalid() {
        assert!(Percentage::new(-1.0).is_err());
        assert!(Percentage::new(101.0).is_err());
        assert!(Percentage::new(f64::NAN).is_err());
    }

    #[test]
    fn non_negative_float_valid() {
        assert!(NonNegativeFloat::new(0.0).is_ok());
        assert!(NonNegativeFloat::new(42.5).is_ok());
    }

    #[test]
    fn non_negative_float_invalid() {
        assert!(NonNegativeFloat::new(-0.001).is_err());
    }

    #[test]
    fn bounded_float_valid() {
        let bf = BoundedFloat::new(5.0, 0.0, 10.0).unwrap();
        assert_eq!(bf.value(), 5.0);
    }

    #[test]
    fn bounded_float_out_of_range() {
        assert!(BoundedFloat::new(11.0, 0.0, 10.0).is_err());
    }
}
