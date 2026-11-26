//! The little linear algebra the Markov chain needs.
//!
//! The TypeScript version uses vectorious 6 without its native backends (nblas / nlapack),
//! i.e. its JavaScript fallbacks. The operations here perform the same floating point operations
//! in the same order, so that the results match bit for bit.

/// A dense row-major matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    /// Builds a matrix from `rows * cols` values with `f(i, j)`.
    pub fn from_fn(rows: usize, cols: usize, mut f: impl FnMut(usize, usize) -> f64) -> Self {
        let data = (0..rows)
            .flat_map(|i| (0..cols).map(move |j| (i, j)))
            .map(|(i, j)| f(i, j))
            .collect();
        Self { rows, cols, data }
    }

    pub fn identity(n: usize) -> Self {
        Self::from_fn(n, n, |i, j| if i == j { 1.0 } else { 0.0 })
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

    pub fn transpose(&self) -> Self {
        Self::from_fn(self.cols, self.rows, |i, j| self.get(j, i))
    }

    /// `self - other` (vectorious computes `self + (-1) * other`, which is the same value).
    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!((self.rows, self.cols), (other.rows, other.cols));
        Self::from_fn(self.rows, self.cols, |i, j| {
            self.get(i, j) - other.get(i, j)
        })
    }

    /// The row vector `v` times this matrix. Each element is summed left to right from 0.
    pub fn left_mul(&self, v: &[f64]) -> Vec<f64> {
        assert_eq!(v.len(), self.rows, "sizes do not match");
        (0..self.cols)
            .map(|j| (0..self.rows).fold(0.0, |acc, h| acc + v[h] * self.get(h, j)))
            .collect()
    }

    /// Solves `self * x = b` for a square matrix by LU decomposition with partial pivoting
    /// (vectorious' `lu_factor` and `solve`). A singular matrix gives non-finite values.
    pub fn solve(&self, b: &[f64]) -> Vec<f64> {
        let n = self.rows;
        assert_eq!(self.cols, n, "matrix is not square");
        assert_eq!(b.len(), n, "sizes do not match");
        let (lu, pivots) = self.lu_factor();
        let at = |i: usize, j: usize| lu[i * n + j];
        let mut y = b.to_vec();
        for (u, &p) in pivots.iter().enumerate() {
            if u != p {
                y.swap(u, p);
            }
        }
        for u in 0..n {
            for h in 0..u {
                y[u] -= at(u, h) * y[h];
            }
        }
        for u in (0..n).rev() {
            for h in u + 1..n {
                y[u] -= at(u, h) * y[h];
            }
            y[u] /= at(u, u);
        }
        y
    }

    /// In-place LU factorisation (unit lower triangle below the diagonal) and the row swapped
    /// with each row (0-based; vectorious stores them 1-based).
    fn lu_factor(&self) -> (Vec<f64>, Vec<usize>) {
        let n = self.rows;
        let mut r = self.data.clone();
        let mut pivots = vec![0; n];
        for u in 0..n {
            let mut p = u;
            let mut max = r[u * n + u].abs();
            for y in u + 1..n {
                let s = r[y * n + u].abs();
                if max < s {
                    max = s;
                    p = y;
                }
            }
            pivots[u] = p;
            if p != u {
                for j in 0..n {
                    r.swap(u * n + j, p * n + j);
                }
            }
            let m = r[u * n + u];
            for c in u + 1..n {
                r[c * n + u] /= m;
            }
            for c in u + 1..n {
                for y in u + 1..n {
                    r[c * n + y] -= r[c * n + u] * r[u * n + y];
                }
            }
        }
        (r, pivots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(rows: &[&[f64]]) -> Matrix {
        Matrix::from_fn(rows.len(), rows[0].len(), |i, j| rows[i][j])
    }

    #[test]
    fn transpose() {
        // The vectorious test in util.test.ts.
        assert_eq!(
            m(&[&[1.0, 1.0], &[0.0, 0.0]]).transpose(),
            m(&[&[1.0, 0.0], &[1.0, 0.0]])
        );
        let a = m(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
        assert_eq!(a.transpose(), m(&[&[1.0, 4.0], &[2.0, 5.0], &[3.0, 6.0]]));
    }

    #[test]
    fn identity_and_sub() {
        let p = m(&[&[0.25, 0.75], &[0.5, 0.5]]);
        assert_eq!(
            Matrix::identity(2).sub(&p),
            m(&[&[0.75, -0.75], &[-0.5, 0.5]])
        );
    }

    #[test]
    fn left_mul() {
        let a = m(&[&[1.0, 2.0], &[3.0, 4.0], &[5.0, 6.0]]);
        assert_eq!(a.left_mul(&[1.0, 0.5, 2.0]), [12.5, 16.0]);
    }

    #[test]
    fn solve_needs_pivoting() {
        // The first pivot is 0, so the rows must be swapped.
        let a = m(&[&[0.0, 2.0, 1.0], &[1.0, 1.0, 1.0], &[2.0, 1.0, 3.0]]);
        let x = [1.0, -2.0, 3.0];
        let b: Vec<f64> = (0..3)
            .map(|i| (0..3).map(|j| a.get(i, j) * x[j]).sum())
            .collect();
        let got = a.solve(&b);
        for (g, e) in got.iter().zip(x) {
            assert!((g - e).abs() < 1e-12, "{got:?}");
        }
    }

    #[test]
    fn solve_singular_is_not_finite() {
        let a = m(&[&[1.0, 2.0], &[2.0, 4.0]]);
        assert!(a.solve(&[1.0, 1.0]).iter().any(|v| !v.is_finite()));
    }
}
