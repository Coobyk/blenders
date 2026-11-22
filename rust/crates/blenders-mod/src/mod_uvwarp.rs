/// Port of `modifiers/intern/MOD_uvwarp.cc` — modifier `mod_uvwarp`
/// apply (UV transform: offset + scale +
/// rotation per axis).
/// `UVWarpModifierData` (verbatim fields).
#[derive(Clone)]
pub struct UVWarpModifierData {
    /// `offset` — uv offset.
    pub offset: [f32; 2],
    /// `scale` — uv scale.
    pub scale: [f32; 2],
    /// `rotate` — uv rotation (radians).
    pub rotate: f32,
    /// `axis_u` — u axis object axis.
    pub axis_u: i8,
    /// `axis_v`.
    pub axis_v: i8,
    /// `object_src`/`object_dst`.
    pub object_src: usize,
    pub object_dst: usize,
    /// `bone_src`/`bone_dst`.
    pub bone_src: Option<String>,
    pub bone_dst: Option<String>,
    /// `vgroup`.
    pub vgroup: i32,
    /// `uv_layer` — target uv layer.
    pub uv_layer: Option<String>,
}
/// `do_uvwarp` (verbatim — uv' = scale *
/// rot * (uv + offset)).
pub fn apply(
    uwmd: &UVWarpModifierData,
    uvs: &mut [[f32; 2]],
) {
    let (s, c) = uwmd.rotate.sin_cos();
    for uv in uvs.iter_mut() {
        let u = (uv[0] + uwmd.offset[0]) * uwmd.scale[0];
        let v = (uv[1] + uwmd.offset[1]) * uwmd.scale[1];
        uv[0] = u * c - v * s;
        uv[1] = u * s + v * c;
    }
}
/// `modifier_type_mod_uvwarp` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_uvwarp() {}
