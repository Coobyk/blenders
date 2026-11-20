//! Port of `quadric.cc` — `Quadric` (10-coefficient quadratic form, doubles).

/// `Quadric` — field order matches C (10 doubles).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Quadric {
    pub a2: f64,
    pub b2: f64,
    pub c2: f64,
    pub ab: f64,
    pub ac: f64,
    pub bc: f64,
    pub ad: f64,
    pub bd: f64,
    pub cd: f64,
    pub d2: f64,
}

const QUADRIC_FLT_TOT: usize = 10;

impl Quadric {
    /// `BLI_quadric_from_plane` — build from plane `v[4]`.
    pub fn from_plane(v: [f64; 4]) -> Self {
        Quadric {
            a2: v[0] * v[0],
            b2: v[1] * v[1],
            c2: v[2] * v[2],
            ab: v[0] * v[1],
            ac: v[0] * v[2],
            bc: v[1] * v[2],
            ad: v[0] * v[3],
            bd: v[1] * v[3],
            cd: v[2] * v[3],
            d2: v[3] * v[3],
        }
    }
    /// `BLI_quadric_to_vector_v3`.
    pub fn to_vector_v3(&self) -> [f64; 3] {
        [self.ad, self.bd, self.cd]
    }
    /// `BLI_quadric_clear`.
    pub fn clear(&mut self) {
        *self = Quadric::default();
    }
    /// `BLI_quadric_add_qu_qu` — `a += b`.
    pub fn add_qu_qu(&mut self, b: &Quadric) {
        let av = self.as_mut_slice();
        let bv = b.as_slice();
        for i in 0..QUADRIC_FLT_TOT {
            av[i] += bv[i];
        }
    }
    /// `BLI_quadric_add_qu_ququ` — `r = a + b`.
    pub fn add_qu_ququ(a: &Quadric, b: &Quadric) -> Quadric {
        let mut r = Quadric::default();
        for i in 0..QUADRIC_FLT_TOT {
            r.as_mut_slice()[i] = a.as_slice()[i] + b.as_slice()[i];
        }
        r
    }
    /// `BLI_quadric_mul` — `a *= scalar`.
    pub fn mul(&mut self, scalar: f64) {
        for v in self.as_mut_slice() {
            *v *= scalar;
        }
    }
    /// `BLI_quadric_evaluate` — `v^T Q v` expansion.
    pub fn evaluate(&self, v: [f64; 3]) -> f64 {
        let v00 = v[0] * v[0];
        let v01 = v[0] * v[1];
        let v02 = v[0] * v[2];
        let v11 = v[1] * v[1];
        let v12 = v[1] * v[2];
        let v22 = v[2] * v[2];
        (self.a2 * v00)
            + (self.ab * 2.0 * v01)
            + (self.ac * 2.0 * v02)
            + (self.ad * 2.0 * v[0])
            + (self.b2 * v11)
            + (self.bc * 2.0 * v12)
            + (self.bd * 2.0 * v[1])
            + (self.c2 * v22)
            + (self.cd * 2.0 * v[2])
            + self.d2
    }
    /// `BLI_quadric_optimize` — solve `Q v = 0` for v when tensor invertible.
    pub fn optimize(&self, epsilon: f64) -> Option<[f64; 3]> {
        let m = self.tensor_m3_inverse(epsilon)?;
        let v = self.to_vector_v3();
        // mul_m3_v3_db(m, v); negate
        let out = [
            -(m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2]),
            -(m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2]),
            -(m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2]),
        ];
        Some(out)
    }

    /// `quadric_to_tensor_m3_inverse` — 3x3 adjugate inverse.
    fn tensor_m3_inverse(&self, epsilon: f64) -> Option<[[f64; 3]; 3]> {
        let q = self;
        let det = q.a2 * (q.b2 * q.c2 - q.bc * q.bc)
            - q.ab * (q.ab * q.c2 - q.ac * q.bc)
            + q.ac * (q.ab * q.bc - q.ac * q.b2);
        if det.abs() <= epsilon {
            return None;
        }
        let invdet = 1.0 / det;
        Some([
            [
                (q.b2 * q.c2 - q.bc * q.bc) * invdet,
                (q.ac * q.bc - q.ab * q.c2) * invdet,
                (q.ab * q.bc - q.ac * q.b2) * invdet,
            ],
            [
                (q.bc * q.ac - q.ab * q.c2) * invdet,
                (q.a2 * q.c2 - q.ac * q.ac) * invdet,
                (q.ac * q.ab - q.a2 * q.bc) * invdet,
            ],
            [
                (q.ab * q.bc - q.b2 * q.ac) * invdet,
                (q.ab * q.ac - q.a2 * q.bc) * invdet,
                (q.a2 * q.b2 - q.ab * q.ab) * invdet,
            ],
        ])
    }

    fn as_slice(&self) -> &[f64; QUADRIC_FLT_TOT] {
        unsafe { &*(self as *const Quadric as *const [f64; QUADRIC_FLT_TOT]) }
    }
    fn as_mut_slice(&mut self) -> &mut [f64; QUADRIC_FLT_TOT] {
        unsafe { &mut *(self as *mut Quadric as *mut [f64; QUADRIC_FLT_TOT]) }
    }
}
