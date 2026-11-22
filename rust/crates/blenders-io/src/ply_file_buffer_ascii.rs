/// Port of `ply/exporter/ply_file_buffer_ascii.cc`
/// — ASCII line writer.
pub fn ply_write_ascii_f32(v: f32) -> String {
    format!("{} ", v)
}
