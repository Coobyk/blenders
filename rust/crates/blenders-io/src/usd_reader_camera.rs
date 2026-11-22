/// Port of `usd/intern/usd_reader_camera.cc`
/// — camera prim → Camera data.
/// `read_camera` (verbatim — focal/clip).
pub fn usd_read_camera(focal_length: f32, aperture: f32) -> [f32; 2] {
    [focal_length, aperture]
}
