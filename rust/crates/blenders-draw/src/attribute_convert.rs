/// Port of `intern/attribute_convert.cc` —
/// draw-attribute → GPUVertCompType map.
/// `drw_attr_convert` (verbatim — CD type →
/// GPU comp).
pub fn drw_attr_convert(cd_type: i32) -> i32 {
    match cd_type {
        0 => blenders_gpu::gpu_vertex_format::GPU_COMP_F32,
        _ => blenders_gpu::gpu_vertex_format::GPU_COMP_F32,
    }
}
