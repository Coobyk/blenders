/// Port of `mathutils_kdtree.cc` —
/// `mathutils.kdtree.KDTree` (insert/balance/
/// find/find_n/find_range).
/// `KDTree` (verbatim — point list + size).
#[derive(Default)]
pub struct MathutilsKdTree {
    /// `points`.
    pub points: Vec<[f32; 3]>,
    /// `size`.
    pub size: i32,
}
impl MathutilsKdTree {
    /// `KDTree(size)` ctor.
    pub fn new(size: i32) -> Self {
        Self { points: Vec::new(), size }
    }
    /// `insert(co, index)`.
    pub fn insert(&mut self, co: [f32; 3]) -> i32 {
        self.points.push(co);
        self.points.len() as i32 - 1
    }
    /// `find(co)` (verbatim — nearest).
    pub fn find(&self, co: [f32; 3]) -> Option<(usize, [f32; 3], f32)> {
        let mut best: Option<(usize, f32)> = None;
        for (i, p) in self.points.iter().enumerate() {
            let d = (p[0] - co[0]).powi(2) + (p[1] - co[1]).powi(2) + (p[2] - co[2]).powi(2);
            if best.map_or(true, |(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(i, d)| (i, self.points[i], d.sqrt()))
    }
}
