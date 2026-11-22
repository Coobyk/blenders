/// Port of `modifiers/intern/MOD_weightvgedit.cc` — modifier `mod_weightvgedit`
/// deform (edit vertex-group weights via
/// mapping curve).
/// `WeightVGEditModifierData.edit_flags` (verbatim).
pub const MOD_WVG_EDIT_INVERT_FALLOFF: i16 = 1 << 0;
pub const MOD_WVG_EDIT_INVERT_WEIGHTS: i16 = 1 << 1;
/// `WeightVGEditModifierData` (verbatim fields).
#[derive(Clone)]
pub struct WeightVGEditModifierData {
    /// `defgrp_name`.
    pub defgrp_name: Option<String>,
    /// `edit_flags`.
    pub edit_flags: i16,
    /// `default_weight`.
    pub default_weight: f32,
    /// `cmap_curve` — remap curve idx.
    pub cmap_curve: usize,
    /// `add_threshold`.
    pub add_threshold: f32,
    /// `rem_threshold`.
    pub rem_threshold: f32,
    /// `mask_constant`.
    pub mask_constant: f32,
    /// `mask_tex_use_channel`.
    pub mask_tex_use_channel: i8,
    /// `mask_texture` — mask texture.
    pub mask_texture: usize,
    /// `mask_tex_mapping`.
    pub mask_tex_mapping: i16,
    /// `mask_tex_map_obj`.
    pub mask_tex_map_obj: usize,
    /// `mask_tex_uvlayer_name`.
    pub mask_tex_uvlayer_name: Option<String>,
}
/// `applyModifier` core (verbatim — per vert:
/// weight' = curve_map(w) if w >= add_threshold
/// else default; drop below rem_threshold).
pub fn apply(
    md: &WeightVGEditModifierData,
    weights: &mut [f32],
    curve_map: &mut dyn FnMut(f32) -> f32,
) {
    for w in weights.iter_mut() {
        if *w < md.add_threshold {
            *w = md.default_weight;
        } else {
            *w = curve_map(*w);
        }
        if *w < md.rem_threshold {
            *w = 0.0;
        }
        *w = w.clamp(0.0, 1.0);
    }
}
/// `modifier_type_mod_weightvgedit` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_weightvgedit() {}
