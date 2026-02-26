//! M11: Math Helpers – deterministic mathematical operations.

use foundation::errors::{FoundationError, FoundationResult};
use foundation::primitives::{SafeFloat, SafeInteger};

/// Compute the greatest common divisor of two integers using Euclid's algorithm.
pub fn gcd(a: SafeInteger, b: SafeInteger) -> FoundationResult<SafeInteger> {
    let mut x = a.checked_abs()?;
    let mut y = b.checked_abs()?;
    while y.value() != 0 {
        let temp = y;
        y = x.checked_rem(y)?;
        x = temp;
    }
    Ok(x)
}

/// Compute the least common multiple of two integers.
pub fn lcm(a: SafeInteger, b: SafeInteger) -> FoundationResult<SafeInteger> {
    if a.value() == 0 || b.value() == 0 {
        return Ok(SafeInteger::ZERO);
    }
    let g = gcd(a, b)?;
    let div = a.checked_div(g)?;
    div.checked_mul(b)?.checked_abs()
}

/// Clamp a value to [min, max].
pub fn clamp_integer(
    value: SafeInteger,
    min: SafeInteger,
    max: SafeInteger,
) -> FoundationResult<SafeInteger> {
    if min > max {
        return Err(FoundationError::ValidationFailed(
            "min must be <= max".into(),
        ));
    }
    if value < min {
        Ok(min)
    } else if value > max {
        Ok(max)
    } else {
        Ok(value)
    }
}

/// Clamp a float to [min, max].
pub fn clamp_float(
    value: SafeFloat,
    min: SafeFloat,
    max: SafeFloat,
) -> FoundationResult<SafeFloat> {
    if min > max {
        return Err(FoundationError::ValidationFailed(
            "min must be <= max".into(),
        ));
    }
    if value < min {
        Ok(min)
    } else if value > max {
        Ok(max)
    } else {
        Ok(value)
    }
}

/// Compute the mean of a slice of SafeFloats.
pub fn mean(values: &[SafeFloat]) -> FoundationResult<SafeFloat> {
    if values.is_empty() {
        return Err(FoundationError::ValidationFailed(
            "cannot compute mean of empty slice".into(),
        ));
    }
    let mut sum = SafeFloat::ZERO;
    for v in values {
        sum = sum.checked_add(*v)?;
    }
    let n = SafeFloat::new(values.len() as f64)?;
    sum.checked_div(n)
}

/// Compute the variance of a slice of SafeFloats.
pub fn variance(values: &[SafeFloat]) -> FoundationResult<SafeFloat> {
    if values.len() < 2 {
        return Err(FoundationError::ValidationFailed(
            "need at least 2 values for variance".into(),
        ));
    }
    let m = mean(values)?;
    let mut sum_sq = SafeFloat::ZERO;
    for v in values {
        let diff = v.checked_sub(m)?;
        let sq = diff.checked_mul(diff)?;
        sum_sq = sum_sq.checked_add(sq)?;
    }
    let n = SafeFloat::new((values.len() - 1) as f64)?;
    sum_sq.checked_div(n)
}

/// Compute the standard deviation.
pub fn std_dev(values: &[SafeFloat]) -> FoundationResult<SafeFloat> {
    variance(values)?.sqrt()
}

/// Linear interpolation between a and b at parameter t ∈ [0, 1].
pub fn lerp(a: SafeFloat, b: SafeFloat, t: SafeFloat) -> FoundationResult<SafeFloat> {
    if t.value() < 0.0 || t.value() > 1.0 {
        return Err(FoundationError::OutOfRange(
            "lerp parameter t must be in [0, 1]".into(),
        ));
    }
    let one_minus_t = SafeFloat::ONE.checked_sub(t)?;
    let left = a.checked_mul(one_minus_t)?;
    let right = b.checked_mul(t)?;
    left.checked_add(right)
}

/// Compute the sum of a slice of SafeIntegers.
pub fn sum_integers(values: &[SafeInteger]) -> FoundationResult<SafeInteger> {
    let mut acc = SafeInteger::ZERO;
    for v in values {
        acc = acc.checked_add(*v)?;
    }
    Ok(acc)
}

/// Compute the sum of a slice of SafeFloats.
pub fn sum_floats(values: &[SafeFloat]) -> FoundationResult<SafeFloat> {
    let mut acc = SafeFloat::ZERO;
    for v in values {
        acc = acc.checked_add(*v)?;
    }
    Ok(acc)
}

/// Compute the min and max of a slice of SafeFloats.
pub fn min_max(values: &[SafeFloat]) -> FoundationResult<(SafeFloat, SafeFloat)> {
    if values.is_empty() {
        return Err(FoundationError::ValidationFailed(
            "cannot compute min/max of empty slice".into(),
        ));
    }
    let mut min = values[0];
    let mut max = values[0];
    for v in &values[1..] {
        if *v < min {
            min = *v;
        }
        if *v > max {
            max = *v;
        }
    }
    Ok((min, max))
}

/// Normalize a slice of SafeFloats to [0, 1].
pub fn normalize(values: &[SafeFloat]) -> FoundationResult<Vec<SafeFloat>> {
    let (min, max) = min_max(values)?;
    let range = max.checked_sub(min)?;
    if range.approx_eq(&SafeFloat::ZERO) {
        return Ok(vec![SafeFloat::ZERO; values.len()]);
    }
    values
        .iter()
        .map(|v| v.checked_sub(min)?.checked_div(range))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcd_basic() {
        let a = SafeInteger::new(12);
        let b = SafeInteger::new(8);
        assert_eq!(gcd(a, b).unwrap().value(), 4);
    }

    #[test]
    fn lcm_basic() {
        let a = SafeInteger::new(4);
        let b = SafeInteger::new(6);
        assert_eq!(lcm(a, b).unwrap().value(), 12);
    }

    #[test]
    fn clamp_integer_works() {
        let val = SafeInteger::new(15);
        let min = SafeInteger::new(0);
        let max = SafeInteger::new(10);
        assert_eq!(clamp_integer(val, min, max).unwrap().value(), 10);
    }

    #[test]
    fn mean_basic() {
        let values = vec![
            SafeFloat::new(1.0).unwrap(),
            SafeFloat::new(2.0).unwrap(),
            SafeFloat::new(3.0).unwrap(),
        ];
        let m = mean(&values).unwrap();
        assert!(m.approx_eq(&SafeFloat::new(2.0).unwrap()));
    }

    #[test]
    fn mean_empty() {
        let values: Vec<SafeFloat> = vec![];
        assert!(mean(&values).is_err());
    }

    #[test]
    fn variance_basic() {
        let values = vec![
            SafeFloat::new(2.0).unwrap(),
            SafeFloat::new(4.0).unwrap(),
            SafeFloat::new(4.0).unwrap(),
            SafeFloat::new(4.0).unwrap(),
            SafeFloat::new(5.0).unwrap(),
            SafeFloat::new(5.0).unwrap(),
            SafeFloat::new(7.0).unwrap(),
            SafeFloat::new(9.0).unwrap(),
        ];
        let v = variance(&values).unwrap();
        assert!(v.value() > 0.0);
    }

    #[test]
    fn lerp_basic() {
        let a = SafeFloat::new(0.0).unwrap();
        let b = SafeFloat::new(10.0).unwrap();
        let t = SafeFloat::new(0.5).unwrap();
        let result = lerp(a, b, t).unwrap();
        assert!(result.approx_eq(&SafeFloat::new(5.0).unwrap()));
    }

    #[test]
    fn lerp_out_of_range() {
        let a = SafeFloat::ZERO;
        let b = SafeFloat::ONE;
        let t = SafeFloat::new(1.5).unwrap();
        assert!(lerp(a, b, t).is_err());
    }

    #[test]
    fn normalize_basic() {
        let values = vec![
            SafeFloat::new(1.0).unwrap(),
            SafeFloat::new(3.0).unwrap(),
            SafeFloat::new(5.0).unwrap(),
        ];
        let normed = normalize(&values).unwrap();
        assert!(normed[0].approx_eq(&SafeFloat::ZERO));
        assert!(normed[2].approx_eq(&SafeFloat::ONE));
    }

    #[test]
    fn sum_integers_basic() {
        let vals = vec![SafeInteger::new(1), SafeInteger::new(2), SafeInteger::new(3)];
        assert_eq!(sum_integers(&vals).unwrap().value(), 6);
    }
}
