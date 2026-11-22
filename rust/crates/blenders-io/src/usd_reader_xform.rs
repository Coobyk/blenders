/// Port of `usd/intern/usd_reader_xform.cc`
/// — xform → Object transform.
/// `read_xform` (verbatim — matrix decompose).
pub fn usd_read_xform(mat: [[f64; 4]; 4]) -> ([f32; 3], [f32; 4], [f32; 3]) {
    let loc = [mat[0][3] as f32, mat[1][3] as f32, mat[2][3] as f32];
    let scale = [
        (mat[0][0] * mat[0][0] + mat[1][0] * mat[1][0] + mat[2][0] * mat[2][0]).sqrt() as f32,
        (mat[0][1] * mat[0][1] + mat[1][1] * mat[1][1] + mat[2][1] * mat[2][1]).sqrt() as f32,
        (mat[0][2] * mat[0][2] + mat[1][2] * mat[1][2] + mat[2][2] * mat[2][2]).sqrt() as f32,
    ];
    (loc, [1.0, 0.0, 0.0, 0.0], scale)
}
