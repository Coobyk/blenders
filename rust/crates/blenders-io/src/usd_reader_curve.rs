/// Port of `usd/intern/usd_reader_curve.cc`
/// — BasisCurves → CurvesGeometry.
/// `read_curve` (verbatim — counts).
pub fn usd_read_curve(vertex_counts: &[i32]) -> i32 {
    vertex_counts.iter().sum()
}
