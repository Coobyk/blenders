/// Port of `extract_mesh_compress_gpu.cc`
/// — GPU-side VBO compression (dst range).
/// `compress_gpu` (verbatim — dst offset).
pub fn compress_gpu(src_len: usize, dst_off: usize) -> usize {
    src_len + dst_off
}
