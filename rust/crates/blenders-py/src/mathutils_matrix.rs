/// Port of `mathutils_matrix.cc` —
/// `mathutils.Matrix` (row-major, index,
/// invert/transpose/determinant/mult).
/// `Matrix` (verbatim — dim 2/3/4 + data).
#[derive(Clone)]
pub struct MathutilsMatrix {
    /// `num_row`/`num_col`.
    pub num_row: usize,
    pub num_col: usize,
    /// `matrix` row-major.
    pub matrix: Vec<f32>,
}
impl MathutilsMatrix {
    /// `Matrix.Identity(n)`.
    pub fn identity(n: usize) -> Self {
        let mut m = vec![0.0; n * n];
        for i in 0..n {
            m[i * n + i] = 1.0;
        }
        Self { num_row: n, num_col: n, matrix: m }
    }
    /// `determinant`.
    pub fn determinant(&self) -> f32 {
        if self.num_row != self.num_col {
            return 0.0;
        }
        let n = self.num_row;
        let mut m = self.matrix.clone();
        let mut det = 1.0f32;
        for i in 0..n {
            let mut piv = i;
            for r in (i + 1)..n {
                if m[r * n + i].abs() > m[piv * n + i].abs() {
                    piv = r;
                }
            }
            if piv != i {
                for c in 0..n {
                    m.swap(i * n + c, piv * n + c);
                }
                det = -det;
            }
            let d = m[i * n + i];
            if d == 0.0 {
                return 0.0;
            }
            det *= d;
            for r in (i + 1)..n {
                let f = m[r * n + i] / d;
                for c in i..n {
                    m[r * n + c] -= f * m[i * n + c];
                }
            }
        }
        det
    }
    /// `transposed`.
    pub fn transposed(&self) -> Self {
        let mut m = vec![0.0; self.num_row * self.num_col];
        for r in 0..self.num_row {
            for c in 0..self.num_col {
                m[c * self.num_row + r] = self.matrix[r * self.num_col + c];
            }
        }
        Self {
            num_row: self.num_col,
            num_col: self.num_row,
            matrix: m,
        }
    }
}
