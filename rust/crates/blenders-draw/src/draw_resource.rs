/// Port of `intern/draw_resource.cc` —
/// DRW resource handles (uniforms/textures).
/// `DRWResource` (verbatim — type + id).
pub struct DrwResource {
    /// `type` (uniform/texture/sampler).
    pub res_type: i32,
    /// `id`.
    pub id: usize,
}
