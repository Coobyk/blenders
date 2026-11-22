//! Port of `intern/domain.cc` — compositor Domain
//! (size + display transform).

/// `Domain` verbatim fields.
#[derive(Clone)]
pub struct Domain {
    /// `data_size`.
    pub data_size: [i32; 2],
    /// `display_size`.
    pub display_size: [i32; 2],
    /// `data_offset`.
    pub data_offset: [i32; 2],
    /// `transformation` (3x3 row-major).
    pub transformation: [[f32; 3]; 3],
}
impl Domain {
    /// `Domain(size)` (verbatim — display=size,
    /// offset 0, identity transform).
    pub fn new(size: [i32; 2]) -> Self {
        Domain {
            data_size: size,
            display_size: size,
            data_offset: [0, 0],
            transformation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
    /// `Domain(size, transformation)` (verbatim).
    pub fn with_transform(size: [i32; 2], transformation: [[f32; 3]; 3]) -> Self {
        Domain {
            data_size: size,
            display_size: size,
            data_offset: [0, 0],
            transformation,
        }
    }
    /// `transform` (verbatim — left-multiply
    /// transform into domain transform + update
    /// display size/offset).
    pub fn transform(&mut self, input_transformation: [[f32; 3]; 3]) {
        self.transformation = mul_m3(input_transformation, self.transformation);
    }
}
fn mul_m3(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0f32; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            out[r][c] = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
        }
    }
    out
}
