/// Port of `view3d_navigate_smoothview.cc`
/// — `smooth_view` camera interpolation
/// (quat slerp + ofs lerp).
/// `smooth_view` (verbatim — t-quad
/// ease + slerp).
pub fn smooth_view(t: f32) -> f32 {
    // C's `smoothview` uses smoothstep.
    t * t * (3.0 - 2.0 * t)
}
