/// Port of `engines/workbench/workbench_shader_cache.cc`
/// — workbench shader permutations.
/// `workbench_shader_cache_get` (verbatim —
/// key).
pub fn workbench_shader_cache_get(light: i32, shadow: i32, cavity: i32) -> i32 {
    light | shadow << 2 | cavity << 4
}
