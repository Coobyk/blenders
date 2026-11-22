/// Port of `intern/gpu_immediate.cc` —
/// GPUImmediate (immBegin/attr/vertex/end).
/// `GPUImmediate` (verbatim — format + vertex
/// buffer + prim).
pub struct GpuImmediate {
    /// `vertex_format`.
    pub format: super::gpu_vertex_format::GpuVertFormat,
    /// `verts`.
    pub verts: Vec<u8>,
    /// `prim_type`.
    pub prim_type: i32,
    /// `vertex_len`.
    pub vertex_len: i32,
}
impl GpuImmediate {
    /// `immVertex` (verbatim — push vertex).
    pub fn vertex(&mut self, data: &[u8]) {
        self.verts.extend_from_slice(data);
        self.vertex_len += 1;
    }
}
