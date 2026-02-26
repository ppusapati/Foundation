//! M12: Linear Algebra Helpers – vectors and matrices with safe arithmetic.

use foundation::errors::{FoundationError, FoundationResult};
use foundation::primitives::SafeFloat;
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Vector
// ---------------------------------------------------------------------------

/// A dense vector of SafeFloats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vector {
    data: Vec<SafeFloat>,
}

impl Vector {
    /// Create a vector from a slice of SafeFloats.
    pub fn new(data: Vec<SafeFloat>) -> Self {
        Vector { data }
    }

    /// Create a vector from a slice of f64s.
    pub fn from_f64(values: &[f64]) -> FoundationResult<Self> {
        let data: FoundationResult<Vec<_>> = values.iter().map(|v| SafeFloat::new(*v)).collect();
        Ok(Vector { data: data? })
    }

    /// Create a zero vector of length n.
    pub fn zeros(n: usize) -> Self {
        Vector {
            data: vec![SafeFloat::ZERO; n],
        }
    }

    /// Dimension of the vector.
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Get element at index.
    pub fn get(&self, i: usize) -> FoundationResult<SafeFloat> {
        self.data
            .get(i)
            .copied()
            .ok_or_else(|| FoundationError::OutOfRange(format!("index {} out of range", i)))
    }

    /// Set element at index.
    pub fn set(&mut self, i: usize, value: SafeFloat) -> FoundationResult<()> {
        if i >= self.data.len() {
            return Err(FoundationError::OutOfRange(format!(
                "index {} out of range",
                i
            )));
        }
        self.data[i] = value;
        Ok(())
    }

    /// Component-wise addition.
    pub fn add(&self, other: &Vector) -> FoundationResult<Vector> {
        if self.dim() != other.dim() {
            return Err(FoundationError::ValidationFailed(
                "vector dimension mismatch".into(),
            ));
        }
        let data: FoundationResult<Vec<_>> = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| a.checked_add(*b))
            .collect();
        Ok(Vector { data: data? })
    }

    /// Component-wise subtraction.
    pub fn sub(&self, other: &Vector) -> FoundationResult<Vector> {
        if self.dim() != other.dim() {
            return Err(FoundationError::ValidationFailed(
                "vector dimension mismatch".into(),
            ));
        }
        let data: FoundationResult<Vec<_>> = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| a.checked_sub(*b))
            .collect();
        Ok(Vector { data: data? })
    }

    /// Scalar multiplication.
    pub fn scale(&self, scalar: SafeFloat) -> FoundationResult<Vector> {
        let data: FoundationResult<Vec<_>> =
            self.data.iter().map(|a| a.checked_mul(scalar)).collect();
        Ok(Vector { data: data? })
    }

    /// Dot product.
    pub fn dot(&self, other: &Vector) -> FoundationResult<SafeFloat> {
        if self.dim() != other.dim() {
            return Err(FoundationError::ValidationFailed(
                "vector dimension mismatch".into(),
            ));
        }
        let mut sum = SafeFloat::ZERO;
        for (a, b) in self.data.iter().zip(&other.data) {
            let prod = a.checked_mul(*b)?;
            sum = sum.checked_add(prod)?;
        }
        Ok(sum)
    }

    /// Euclidean norm (L2).
    pub fn norm(&self) -> FoundationResult<SafeFloat> {
        self.dot(self)?.sqrt()
    }

    /// Normalize to unit length.
    pub fn normalize(&self) -> FoundationResult<Vector> {
        let n = self.norm()?;
        if n.approx_eq(&SafeFloat::ZERO) {
            return Err(FoundationError::ValidationFailed(
                "cannot normalize zero vector".into(),
            ));
        }
        self.scale(SafeFloat::new(1.0 / n.value())?)
    }

    /// Cosine similarity.
    pub fn cosine_similarity(&self, other: &Vector) -> FoundationResult<SafeFloat> {
        let dot = self.dot(other)?;
        let n1 = self.norm()?;
        let n2 = other.norm()?;
        let denom = n1.checked_mul(n2)?;
        if denom.approx_eq(&SafeFloat::ZERO) {
            return Err(FoundationError::ValidationFailed(
                "cannot compute cosine similarity with zero vector".into(),
            ));
        }
        dot.checked_div(denom)
    }

    /// Return the underlying data.
    pub fn as_slice(&self) -> &[SafeFloat] {
        &self.data
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, v) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
        }
        write!(f, "]")
    }
}

// ---------------------------------------------------------------------------
// Matrix
// ---------------------------------------------------------------------------

/// A dense matrix stored in row-major order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<SafeFloat>,
}

impl Matrix {
    /// Create a matrix from dimensions and flat data (row-major).
    pub fn new(rows: usize, cols: usize, data: Vec<SafeFloat>) -> FoundationResult<Self> {
        if data.len() != rows * cols {
            return Err(FoundationError::ValidationFailed(format!(
                "expected {} elements, got {}",
                rows * cols,
                data.len()
            )));
        }
        Ok(Matrix { rows, cols, data })
    }

    /// Create a zero matrix.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Matrix {
            rows,
            cols,
            data: vec![SafeFloat::ZERO; rows * cols],
        }
    }

    /// Create an identity matrix.
    pub fn identity(n: usize) -> Self {
        let mut m = Matrix::zeros(n, n);
        for i in 0..n {
            m.data[i * n + i] = SafeFloat::ONE;
        }
        m
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Get element at (row, col).
    pub fn get(&self, row: usize, col: usize) -> FoundationResult<SafeFloat> {
        if row >= self.rows || col >= self.cols {
            return Err(FoundationError::OutOfRange(format!(
                "({}, {}) out of range",
                row, col
            )));
        }
        Ok(self.data[row * self.cols + col])
    }

    /// Set element at (row, col).
    pub fn set(&mut self, row: usize, col: usize, value: SafeFloat) -> FoundationResult<()> {
        if row >= self.rows || col >= self.cols {
            return Err(FoundationError::OutOfRange(format!(
                "({}, {}) out of range",
                row, col
            )));
        }
        self.data[row * self.cols + col] = value;
        Ok(())
    }

    /// Matrix addition.
    pub fn add(&self, other: &Matrix) -> FoundationResult<Matrix> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err(FoundationError::ValidationFailed(
                "matrix dimension mismatch".into(),
            ));
        }
        let data: FoundationResult<Vec<_>> = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| a.checked_add(*b))
            .collect();
        Ok(Matrix {
            rows: self.rows,
            cols: self.cols,
            data: data?,
        })
    }

    /// Scalar multiplication.
    pub fn scale(&self, scalar: SafeFloat) -> FoundationResult<Matrix> {
        let data: FoundationResult<Vec<_>> =
            self.data.iter().map(|a| a.checked_mul(scalar)).collect();
        Ok(Matrix {
            rows: self.rows,
            cols: self.cols,
            data: data?,
        })
    }

    /// Matrix multiplication.
    pub fn mul(&self, other: &Matrix) -> FoundationResult<Matrix> {
        if self.cols != other.rows {
            return Err(FoundationError::ValidationFailed(format!(
                "incompatible dimensions: {}x{} * {}x{}",
                self.rows, self.cols, other.rows, other.cols
            )));
        }
        let mut result = Matrix::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = SafeFloat::ZERO;
                for k in 0..self.cols {
                    let a = self.get(i, k)?;
                    let b = other.get(k, j)?;
                    sum = sum.checked_add(a.checked_mul(b)?)?;
                }
                result.set(i, j, sum)?;
            }
        }
        Ok(result)
    }

    /// Transpose.
    pub fn transpose(&self) -> Matrix {
        let mut data = vec![SafeFloat::ZERO; self.rows * self.cols];
        for i in 0..self.rows {
            for j in 0..self.cols {
                data[j * self.rows + i] = self.data[i * self.cols + j];
            }
        }
        Matrix {
            rows: self.cols,
            cols: self.rows,
            data,
        }
    }

    /// Matrix-vector multiplication.
    pub fn mul_vec(&self, v: &Vector) -> FoundationResult<Vector> {
        if self.cols != v.dim() {
            return Err(FoundationError::ValidationFailed(
                "matrix-vector dimension mismatch".into(),
            ));
        }
        let mut result = vec![SafeFloat::ZERO; self.rows];
        for i in 0..self.rows {
            let mut sum = SafeFloat::ZERO;
            for j in 0..self.cols {
                let a = self.get(i, j)?;
                let b = v.get(j)?;
                sum = sum.checked_add(a.checked_mul(b)?)?;
            }
            result[i] = sum;
        }
        Ok(Vector::new(result))
    }

    /// Extract a row as a Vector.
    pub fn row(&self, i: usize) -> FoundationResult<Vector> {
        if i >= self.rows {
            return Err(FoundationError::OutOfRange("row index out of range".into()));
        }
        let start = i * self.cols;
        let data = self.data[start..start + self.cols].to_vec();
        Ok(Vector::new(data))
    }

    /// Extract a column as a Vector.
    pub fn col(&self, j: usize) -> FoundationResult<Vector> {
        if j >= self.cols {
            return Err(FoundationError::OutOfRange(
                "column index out of range".into(),
            ));
        }
        let data: Vec<SafeFloat> = (0..self.rows).map(|i| self.data[i * self.cols + j]).collect();
        Ok(Vector::new(data))
    }

    /// Compute the trace (sum of diagonal elements).
    pub fn trace(&self) -> FoundationResult<SafeFloat> {
        if self.rows != self.cols {
            return Err(FoundationError::ValidationFailed(
                "trace requires a square matrix".into(),
            ));
        }
        let mut sum = SafeFloat::ZERO;
        for i in 0..self.rows {
            sum = sum.checked_add(self.get(i, i)?)?;
        }
        Ok(sum)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.rows {
            write!(f, "[")?;
            for j in 0..self.cols {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.data[i * self.cols + j])?;
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_basic_ops() {
        let a = Vector::from_f64(&[1.0, 2.0, 3.0]).unwrap();
        let b = Vector::from_f64(&[4.0, 5.0, 6.0]).unwrap();
        let sum = a.add(&b).unwrap();
        assert!(sum.get(0).unwrap().approx_eq(&SafeFloat::new(5.0).unwrap()));
    }

    #[test]
    fn vector_dot_product() {
        let a = Vector::from_f64(&[1.0, 2.0, 3.0]).unwrap();
        let b = Vector::from_f64(&[4.0, 5.0, 6.0]).unwrap();
        let dot = a.dot(&b).unwrap();
        assert!(dot.approx_eq(&SafeFloat::new(32.0).unwrap()));
    }

    #[test]
    fn vector_norm() {
        let v = Vector::from_f64(&[3.0, 4.0]).unwrap();
        let n = v.norm().unwrap();
        assert!(n.approx_eq(&SafeFloat::new(5.0).unwrap()));
    }

    #[test]
    fn vector_cosine_similarity() {
        let a = Vector::from_f64(&[1.0, 0.0]).unwrap();
        let b = Vector::from_f64(&[0.0, 1.0]).unwrap();
        let sim = a.cosine_similarity(&b).unwrap();
        assert!(sim.approx_eq(&SafeFloat::ZERO));
    }

    #[test]
    fn vector_dimension_mismatch() {
        let a = Vector::from_f64(&[1.0, 2.0]).unwrap();
        let b = Vector::from_f64(&[1.0, 2.0, 3.0]).unwrap();
        assert!(a.add(&b).is_err());
    }

    #[test]
    fn matrix_identity() {
        let id = Matrix::identity(3);
        assert!(id.get(0, 0).unwrap().approx_eq(&SafeFloat::ONE));
        assert!(id.get(0, 1).unwrap().approx_eq(&SafeFloat::ZERO));
    }

    #[test]
    fn matrix_mul() {
        let a = Matrix::new(
            2,
            2,
            vec![
                SafeFloat::new(1.0).unwrap(),
                SafeFloat::new(2.0).unwrap(),
                SafeFloat::new(3.0).unwrap(),
                SafeFloat::new(4.0).unwrap(),
            ],
        )
        .unwrap();
        let id = Matrix::identity(2);
        let result = a.mul(&id).unwrap();
        assert_eq!(result.get(0, 0).unwrap(), a.get(0, 0).unwrap());
    }

    #[test]
    fn matrix_transpose() {
        let m = Matrix::new(
            2,
            3,
            vec![
                SafeFloat::new(1.0).unwrap(),
                SafeFloat::new(2.0).unwrap(),
                SafeFloat::new(3.0).unwrap(),
                SafeFloat::new(4.0).unwrap(),
                SafeFloat::new(5.0).unwrap(),
                SafeFloat::new(6.0).unwrap(),
            ],
        )
        .unwrap();
        let t = m.transpose();
        assert_eq!(t.rows(), 3);
        assert_eq!(t.cols(), 2);
        assert_eq!(t.get(0, 1).unwrap(), m.get(1, 0).unwrap());
    }

    #[test]
    fn matrix_trace() {
        let m = Matrix::identity(3);
        let t = m.trace().unwrap();
        assert!(t.approx_eq(&SafeFloat::new(3.0).unwrap()));
    }

    #[test]
    fn matrix_vec_mul() {
        let m = Matrix::identity(2);
        let v = Vector::from_f64(&[3.0, 4.0]).unwrap();
        let result = m.mul_vec(&v).unwrap();
        assert!(result.get(0).unwrap().approx_eq(&SafeFloat::new(3.0).unwrap()));
        assert!(result.get(1).unwrap().approx_eq(&SafeFloat::new(4.0).unwrap()));
    }
}
