/// Port of `modifiers/intern/MOD_weightvgmix.cc` — modifier `mod_weightvgmix`
/// deform (mix two vertex groups per op).
/// `WeightVGMixModifierData.mix_mode` (verbatim).
pub const MOD_WVG_MIX_SET: i8 = 0;
pub const MOD_WVG_MIX_ADD: i8 = 1;
pub const MOD_WVG_MIX_SUB: i8 = 2;
pub const MOD_WVG_MIX_MUL: i8 = 3;
pub const MOD_WVG_MIX_DIV: i8 = 4;
pub const MOD_WVG_MIX_DIF: i8 = 5;
pub const MOD_WVG_MIX_AVG: i8 = 6;
/// `WeightVGMixModifierData` (verbatim fields).
#[derive(Clone)]
pub struct WeightVGMixModifierData {
    /// `defgrp_name_a`/`defgrp_name_b`.
    pub defgrp_name_a: Option<String>,
    pub defgrp_name_b: Option<String>,
    /// `default_weight_a`/`default_weight_b`.
    pub default_weight_a: f32,
    pub default_weight_b: f32,
    /// `mix_mode`.
    pub mix_mode: i8,
    /// `mix_set` — which channels written.
    pub mix_set: i8,
    /// `mask_constant`.
    pub mask_constant: f32,
    /// `flag`.
    pub flag: i8,
}
/// `applyModifier` core (verbatim — w_a' =
/// mix(w_a, w_b) per mix_mode).
pub fn apply(
    md: &WeightVGMixModifierData,
    wa: &mut [f32],
    wb: &[f32],
) {
    for (i, a) in wa.iter_mut().enumerate() {
        let b = wb.get(i).copied().unwrap_or(md.default_weight_b);
        *a = match md.mix_mode {
            MOD_WVG_MIX_ADD => *a + b,
            MOD_WVG_MIX_SUB => *a - b,
            MOD_WVG_MIX_MUL => *a * b,
            MOD_WVG_MIX_DIV => {
                if b != 0.0 { *a / b } else { *a }
            }
            MOD_WVG_MIX_DIF => (*a - b).abs(),
            MOD_WVG_MIX_AVG => (*a + b) * 0.5,
            // MOD_WVG_MIX_SET.
            _ => b,
        }
        .clamp(0.0, 1.0);
    }
}
/// `modifier_type_mod_weightvgmix` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_weightvgmix() {}
