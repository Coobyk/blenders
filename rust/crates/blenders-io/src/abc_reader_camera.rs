/// Port of `alembic/intern/abc_reader_camera.cc`
/// — camera sample → Camera.
/// `read_camera` (verbatim — focal/aperture).
pub fn abc_read_camera(focal_length: f32, aperture: f32) -> [f32; 2] {
    [focal_length, aperture]
}
