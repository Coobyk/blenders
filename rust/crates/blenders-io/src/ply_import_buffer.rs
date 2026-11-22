/// Port of `ply/importer/ply_import_buffer.cc`
/// — raw PLY input buffer.
pub struct PlyImportBuffer<'a> {
    /// `buf`.
    pub data: &'a [u8],
    /// `pos`.
    pub pos: usize,
}
impl<'a> PlyImportBuffer<'a> {
    /// `read<T>` (verbatim — LE bytes).
    pub fn read_f32(&mut self) -> f32 {
        let v = f32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        v
    }
}
