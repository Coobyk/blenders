/// Port of `io_stl/importer/stl_import_binary_reader.cc` —
/// `STLBinaryReader` — parse binary STL
/// (80B header + u32 count + 50B/tris).
/// `STLBinaryReader::read` (verbatim —
/// header 80B, tri count u32 LE, then per
/// tri: normal 3×f32 + 3×vertex 3×f32 +
/// u16 attr = 50 bytes).
pub struct STLBinaryReader<'a> {
    /// `m_file` — input bytes.
    src: &'a [u8],
}
impl<'a> STLBinaryReader<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self { src }
    }
    /// `read` → tris `[(normal, v0, v1, v2)]`.
    pub fn read(&self) -> Vec<([f32; 3], [[f32; 3]; 3])> {
        let mut tris = Vec::new();
        if self.src.len() < 84 {
            return tris;
        }
        let count = u32::from_le_bytes([
            self.src[80], self.src[81], self.src[82], self.src[83],
        ]) as usize;
        let read_f32 = |off: usize| -> f32 {
            f32::from_le_bytes([
                self.src[off], self.src[off + 1], self.src[off + 2], self.src[off + 3],
            ])
        };
        for i in 0..count {
            let base = 84 + i * 50;
            if base + 50 > self.src.len() {
                break;
            }
            let normal = [
                read_f32(base),
                read_f32(base + 4),
                read_f32(base + 8),
            ];
            let mut verts = [[0.0f32; 3]; 3];
            for v in 0..3 {
                let vb = base + 12 + v * 12;
                verts[v] = [read_f32(vb), read_f32(vb + 4), read_f32(vb + 8)];
            }
            tris.push((normal, verts));
        }
        tris
    }
}
/// `is_stl_binary` — file len == 84 +
/// 50×count.
pub fn is_stl_binary(src: &[u8]) -> bool {
    if src.len() < 84 {
        return false;
    }
    let count = u32::from_le_bytes([src[80], src[81], src[82], src[83]]) as usize;
    src.len() >= 84 + count * 50
}
