/// Port of `ply/exporter/ply_file_buffer.cc`
/// — buffered PLY writer.
pub struct PlyFileBuffer {
    /// `buf`.
    pub data: Vec<u8>,
}
impl PlyFileBuffer {
    /// `write` (verbatim).
    pub fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
}
