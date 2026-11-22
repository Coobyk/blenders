/// Port of `alembic/intern/abc_reader_curves.cc`
/// — curves sample → CurvesGeometry.
/// `read_curves` (verbatim — point counts).
pub fn abc_read_curves(vertex_counts: &[i32]) -> i32 {
    vertex_counts.iter().sum()
}
