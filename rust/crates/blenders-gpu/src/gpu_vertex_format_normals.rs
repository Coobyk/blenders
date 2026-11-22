/// Port of `intern/gpu_vertex_format_normals.cc`
/// — packed normal fetch-mode (snorm oct/short/
/// byte).
/// `GPU_NORMAL_SNORM_I10` (verbatim).
pub const GPU_NORMAL_SNORM_I10: i32 = 0;
/// `convert_normals` (verbatim — f32→packed).
pub fn gpu_pack_normal_i10(n: [f32; 3]) -> u32 {
    let c = |v: f32| -> u32 { (v.clamp(-1.0, 1.0) * 511.0) as i32 as u32 & 0x3ff };
    c(n[0]) | (c(n[1]) << 10) | (c(n[2]) << 20)
}
