/// Port of `usd/intern/usd_writer_transform.cc`
/// — Object transform → UsdGeomXform.
/// `write_transform` (verbatim — loc/rot/
/// scale).
pub fn usd_write_transform(loc: [f32; 3]) -> [[f64; 4]; 4] {
    let mut m = [[0.0f64; 4]; 4];
    m[0][0] = 1.0;
    m[1][1] = 1.0;
    m[2][2] = 1.0;
    m[3][3] = 1.0;
    m[0][3] = loc[0] as f64;
    m[1][3] = loc[1] as f64;
    m[2][3] = loc[2] as f64;
    m
}
