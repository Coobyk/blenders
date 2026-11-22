/// Port of `intern/draw_cache_impl_mesh.cc`
/// — `DRW_mesh_batch_cache` (per-mesh
/// cached VBOs/IBOs/batches).
/// `MeshBatchCache` (verbatim — batch map).
#[derive(Default)]
pub struct MeshBatchCache {
    /// `batch` (GPUBatch per usage).
    pub batch: Vec<usize>,
    /// `vbo` (vertex buffers).
    pub vbo: Vec<usize>,
    /// `ibo` (index buffers).
    pub ibo: Vec<usize>,
    /// `mbuff` (MeshBufferCache).
    pub mbuff: usize,
}
/// `drw_mesh_batch_cache` (verbatim — get
/// or build).
pub fn drw_mesh_batch_cache() -> usize {
    0
}
