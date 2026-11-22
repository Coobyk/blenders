/// Port of `intern/set_curve_type.cc` —
/// convert curves between poly/catmull-rom/
/// bezier/nurbs.

/// `CurveType` ordinals (verbatim from
/// `CURVE_TYPE_*`).
pub const CURVE_TYPE_POLY: i32 = 0;
pub const CURVE_TYPE_BEZIER: i32 = 1;
pub const CURVE_TYPE_NURBS: i32 = 2;
pub const CURVE_TYPE_CATMULL_ROM: i32 = 3;
/// `is_nurbs_to_bezier_one_to_one` (verbatim —
// every curve size=4).
pub fn is_nurbs_to_bezier_one_to_one(sizes: &[i32]) -> bool {
    sizes.iter().all(|&s| s == 4)
}
/// `catmull_rom_to_bezier_handles` (verbatim —
// FD coefficient handles).
pub fn catmull_rom_to_bezier_handles(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    p3: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    let h1 = [
        p1[0] + (p2[0] - p0[0]) / 6.0,
        p1[1] + (p2[1] - p0[1]) / 6.0,
        p1[2] + (p2[2] - p0[2]) / 6.0,
    ];
    let h2 = [
        p2[0] - (p3[0] - p1[0]) / 6.0,
        p2[1] - (p3[1] - p1[1]) / 6.0,
        p2[2] - (p3[2] - p1[2]) / 6.0,
    ];
    (h1, h2)
}
/// `to_bezier_size` (verbatim — per-type
// control-point count).
pub fn to_bezier_size(src_type: i32, src_size: i32) -> i32 {
    match src_type {
        CURVE_TYPE_POLY | CURVE_TYPE_CATMULL_ROM => src_size,
        CURVE_TYPE_NURBS => src_size * 3 - 2,
        _ => src_size,
    }
}
/// `to_nurbs_size` (verbatim).
pub fn to_nurbs_size(src_type: i32, src_size: i32) -> i32 {
    match src_type {
        CURVE_TYPE_BEZIER => src_size,
        _ => src_size,
    }
}
