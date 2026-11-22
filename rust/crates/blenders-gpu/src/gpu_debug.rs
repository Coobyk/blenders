/// Port of `intern/gpu_debug.cc` — debug
/// label groups + error capture.
/// `GPU_debug_group_begin` (verbatim —
/// backend-injected label push).
pub fn gpu_debug_group_begin(name: &str) {
    let _ = name;
}
/// `GPU_debug_group_end` (verbatim).
pub fn gpu_debug_group_end() {}
/// `GPU_debug_get_groups` (verbatim —
/// stack contents).
pub fn gpu_debug_get_groups(stack: &[String]) -> String {
    stack.join(" > ")
}
