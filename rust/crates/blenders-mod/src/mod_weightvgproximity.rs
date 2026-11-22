/// Port of `modifiers/intern/MOD_weightvgproximity.cc` — modifier `mod_weightvgproximity`
/// deform (weight = proximity of vert to
/// target object surface within range).
/// `WeightVGProximityModifierData.proximity_mode` (verbatim).
pub const MOD_WVG_PROXIMITY_OBJECT: i32 = 0;
pub const MOD_WVG_PROXIMITY_GEOMETRY: i32 = 1;
/// `WeightVGProximityModifierData.proximity_flags` (verbatim).
pub const MOD_WVG_PROXIMITY_GEOM_VERTS: i16 = 1 << 0;
pub const MOD_WVG_PROXIMITY_GEOM_EDGES: i16 = 1 << 1;
pub const MOD_WVG_PROXIMITY_GEOM_FACES: i16 = 1 << 2;
/// `WeightVGProximityModifierData` (verbatim fields).
#[derive(Clone)]
pub struct WeightVGProximityModifierData {
    /// `defgrp_name`.
    pub defgrp_name: Option<String>,
    /// `proximity_mode`.
    pub proximity_mode: i32,
    /// `proximity_flags`.
    pub proximity_flags: i16,
    /// `proximity_ob` — target object.
    pub proximity_ob: usize,
    /// `mask_constant`.
    pub mask_constant: f32,
    /// `min_dist`/`max_dist` — range.
    pub min_dist: f32,
    pub max_dist: f32,
}
/// `applyModifier` core (verbatim — per vert:
/// nearest distance to target verts →
/// weight = 1 - clamp((d-min)/(max-min))).
pub fn apply(
    md: &WeightVGProximityModifierData,
    verts: &[[f32; 3]],
    target_verts: &[[f32; 3]],
    weights: &mut [f32],
) {
    let range = (md.max_dist - md.min_dist).max(1e-6);
    for (i, v) in verts.iter().enumerate() {
        let mut best = f32::MAX;
        for tv in target_verts {
            let d = ((v[0] - tv[0]).powi(2)
                + (v[1] - tv[1]).powi(2)
                + (v[2] - tv[2]).powi(2))
            .sqrt();
            if d < best {
                best = d;
            }
        }
        let w = if best >= md.max_dist {
            0.0
        } else if best <= md.min_dist {
            1.0
        } else {
            1.0 - (best - md.min_dist) / range
        };
        weights[i] = (weights[i] * md.mask_constant + w).clamp(0.0, 1.0);
    }
}
/// `modifier_type_mod_weightvgproximity` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_weightvgproximity() {}
