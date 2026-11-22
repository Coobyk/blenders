/// Port of `alembic/intern/abc_axis_conversion.cc`
/// — Alembic↔Blender axis conversion.
/// `abc_convert_axis` (verbatim — Y-up↔Z-up
/// matrix).
pub fn abc_convert_axis_matrix() -> [[f32; 3]; 3] {
    // Alembic is Y-up right-handed; Blender is
    // Z-up right-handed.
    [
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, -1.0, 0.0],
    ]
}
