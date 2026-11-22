/// Port of `mathutils_vector.cc` — `mathutils.Vector`
/// (len/index/arithmetic/normalize/lerp/
/// dot/cross/project/rotate/to_2/3/4d).
/// `Vector` (verbatim — data + is_frozen).
#[derive(Clone, PartialEq)]
pub struct MathutilsVector {
    /// `data` (len 2-5).
    pub data: Vec<f32>,
    /// `is_frozen`.
    pub is_frozen: bool,
}
impl MathutilsVector {
    /// `Vector()` ctor.
    pub fn new(values: &[f32]) -> Self {
        Self { data: values.to_vec(), is_frozen: false }
    }
    /// `len`.
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// `dot`.
    pub fn dot(&self, other: &Self) -> f32 {
        self.data.iter().zip(&other.data).map(|(a, b)| a * b).sum()
    }
    /// `length` (magnitude).
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }
    /// `normalized`.
    pub fn normalized(&self) -> Self {
        let l = self.length();
        if l == 0.0 {
            return Self::new(&vec![0.0; self.data.len()]);
        }
        Self::new(&self.data.iter().map(|v| v / l).collect::<Vec<_>>())
    }
    /// `lerp`.
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self::new(
            &self
                .data
                .iter()
                .zip(&other.data)
                .map(|(a, b)| a + (b - a) * t)
                .collect::<Vec<_>>(),
        )
    }
    /// `freeze`.
    pub fn freeze(&mut self) {
        self.is_frozen = true;
    }
}
