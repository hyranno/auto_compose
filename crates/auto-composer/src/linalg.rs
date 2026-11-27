//! The little linear algebra the Markov chain needs.

/// A dense row-major matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    /// Builds a matrix from `rows * cols` values with `f(i, j)`.
    pub fn from_fn(rows: usize, cols: usize, mut f: impl FnMut(usize, usize) -> f64) -> Self {
        let data = (0..rows)
            .flat_map(|i| (0..cols).map(move |j| (i, j)))
            .map(|(i, j)| f(i, j))
            .collect();
        Self { rows, cols, data }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, i: usize, j: usize) -> f64 {
        assert!(i < self.rows && j < self.cols, "index out of bounds");
        self.data[i * self.cols + j]
    }

    pub fn row(&self, i: usize) -> &[f64] {
        &self.data[i * self.cols..(i + 1) * self.cols]
    }

    /// The row vector `v` times this matrix.
    pub fn left_mul(&self, v: &[f64]) -> Vec<f64> {
        assert_eq!(v.len(), self.rows, "sizes do not match");
        (0..self.cols)
            .map(|j| v.iter().enumerate().map(|(h, x)| x * self.get(h, j)).sum())
            .collect()
    }

    /// This matrix times the column vector `v`.
    pub fn mul_vec(&self, v: &[f64]) -> Vec<f64> {
        assert_eq!(v.len(), self.cols, "sizes do not match");
        (0..self.rows)
            .map(|i| self.row(i).iter().zip(v).map(|(a, x)| a * x).sum())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(rows: &[&[f64]]) -> Matrix {
        Matrix::from_fn(rows.len(), rows[0].len(), |i, j| rows[i][j])
    }

    #[test]
    fn left_mul() {
        let a = m(&[&[1.0, 2.0], &[3.0, 4.0], &[5.0, 6.0]]);
        assert_eq!(a.left_mul(&[1.0, 0.5, 2.0]), [12.5, 16.0]);
    }

    #[test]
    fn mul_vec() {
        let a = m(&[&[1.0, 2.0], &[3.0, 4.0], &[5.0, 6.0]]);
        assert_eq!(a.mul_vec(&[1.0, 0.5]), [2.0, 5.0, 8.0]);
    }
}
