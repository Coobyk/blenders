/// Port of `mathutils_color.cc` —
/// `mathutils.Color` (rgb, hsv/hsv→rgb).
/// `Color` (verbatim — r,g,b + frozen).
#[derive(Clone, Copy)]
pub struct MathutilsColor {
    /// `col` [r,g,b].
    pub col: [f32; 3],
    /// `is_frozen`.
    pub is_frozen: bool,
}
impl MathutilsColor {
    /// `hsv` getter (verbatim — max/min delta).
    pub fn hsv(&self) -> [f32; 3] {
        let [r, g, b] = self.col;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / d) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        let s = if max == 0.0 { 0.0 } else { d / max };
        [if h < 0.0 { h + 360.0 } else { h } / 360.0, s, max]
    }
}
