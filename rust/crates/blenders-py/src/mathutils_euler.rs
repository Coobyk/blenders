/// Port of `mathutils_euler.cc` —
/// `mathutils.Euler` (xyz order, to_matrix/
/// quaternion, rotate_axis).
/// `Euler` (verbatim — euler + order + frozen).
#[derive(Clone, Copy)]
pub struct MathutilsEuler {
    /// `euler` [x,y,z].
    pub euler: [f32; 3],
    /// `order` (EULER_ORDER_*).
    pub order: i32,
    /// `is_frozen`.
    pub is_frozen: bool,
}
