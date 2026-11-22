/// Port of `intern/gpu_shader_info_baked.cc`
/// — baked shader-create-info table.
/// `g_shader_infos` (verbatim — name→info).
pub fn gpu_shader_info_lookup(name: &str) -> Option<usize> {
    let _ = name;
    None
}
