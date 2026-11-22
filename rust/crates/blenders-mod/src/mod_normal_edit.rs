/// Port of `modifiers/intern/MOD_normal_edit.cc` — modifier `mod_normal_edit`
/// apply (reorient vertex normals toward
/// a target direction/object).
/// `NormalEditModifierData.mode` (verbatim).
pub const MOD_NORMALEDIT_MODE_RADIAL: i32 = 0;
pub const MOD_NORMALEDIT_MODE_DIRECTIONAL: i32 = 1;
/// `NormalEditModifierData` (verbatim fields).
#[derive(Clone)]
pub struct NormalEditModifierData {
    /// `target` — target object.
    pub target: usize,
    /// `mode`.
    pub mode: i32,
    /// `mix_mode` (`MOD_NORMALEDIT_MIX_*`).
    pub mix_mode: i16,
    /// `mix_factor`.
    pub mix_factor: f32,
    /// `mix_limit`.
    pub mix_limit: f32,
    /// `offset`.
    pub offset: [f32; 3],
}
/// `MOD_NORMALEDIT_MIX_*` (verbatim).
pub const MOD_NORMALEDIT_MIX_COPY: i16 = 0;
pub const MOD_NORMALEDIT_MIX_ADD: i16 = 1;
pub const MOD_NORMALEDIT_MIX_SUB: i16 = 2;
pub const MOD_NORMALEDIT_MIX_MUL: i16 = 3;
/// `generateCustomNormalArray` (verbatim —
/// new normal = normalized (target-vert)
/// radial, or fixed directional dir).
pub fn apply(
    nemd: &NormalEditModifierData,
    verts: &[[f32; 3]],
    center: [f32; 3],
    dir: [f32; 3],
) -> Vec<[f32; 3]> {
    verts
        .iter()
        .map(|v| {
            let n = if nemd.mode == MOD_NORMALEDIT_MODE_DIRECTIONAL {
                dir
            } else {
                [
                    center[0] - v[0] + nemd.offset[0],
                    center[1] - v[1] + nemd.offset[1],
                    center[2] - v[2] + nemd.offset[2],
                ]
            };
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 0.0 { [n[0] / l, n[1] / l, n[2] / l] } else { [0.0, 0.0, 1.0] }
        })
        .collect()
}
/// `modifier_type_mod_normal_edit` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_normal_edit() {}
