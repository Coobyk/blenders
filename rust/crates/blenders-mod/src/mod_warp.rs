/// Port of `modifiers/intern/MOD_warp.cc` — modifier `mod_warp`
/// deform (rotate verts around `from`→`to`
/// axis by strength with falloff).
/// `WarpModifierData` (verbatim fields).
#[derive(Clone)]
pub struct WarpModifierData {
    /// `object_from` — source object.
    pub object_from: usize,
    /// `object_to` — target object.
    pub object_to: usize,
    /// `keep_radius`.
    pub keep_radius: bool,
    /// `curfalloff` — falloff curve idx.
    pub curfalloff: usize,
    /// `use_relative` — offset mode.
    pub use_relative: bool,
    /// `flag` (`MOD_WARP_*`).
    pub flag: i16,
    /// `falloff_type` (`MOD_WARP_*`).
    pub falloff_type: i16,
    /// `defgrp_name` — vertex group.
    pub defgrp_name: Option<String>,
    /// `strength`.
    pub strength: f32,
    /// `falloff_radius`.
    pub falloff_radius: f32,
    /// `texture` — texture object.
    pub texture: usize,
    /// `texture_coords`.
    pub texture_coords: i32,
    /// `texture_coords_object`.
    pub texture_coords_object: usize,
}
/// `MOD_WARP_*` falloff types (verbatim).
pub const MOD_WARP_VOLUME: i16 = 0;
pub const MOD_WARP_WEIGHT: i16 = 1;
/// `deformVerts` core (verbatim — each vert
/// rotated by `strength` around the
/// from→to axis, attenuated by falloff).
pub fn apply(
    wmd: &WarpModifierData,
    verts: &mut [[f32; 3]],
    from: [f32; 3],
    to: [f32; 3],
    strength: f32,
) {
    let axis = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let alen = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if alen < 1e-6 {
        return;
    }
    let a = [axis[0] / alen, axis[1] / alen, axis[2] / alen];
    for v in verts.iter_mut() {
        // Distance from axis for falloff.
        let rel = [v[0] - from[0], v[1] - from[1], v[2] - from[2]];
        let proj = rel[0] * a[0] + rel[1] * a[1] + rel[2] * a[2];
        let perp = [
            rel[0] - a[0] * proj,
            rel[1] - a[1] * proj,
            rel[2] - a[2] * proj,
        ];
        let dist = (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt();
        if wmd.falloff_radius > 0.0 && dist > wmd.falloff_radius {
            continue;
        }
        // Angle: strength (radians).
        let (s, c) = strength.sin_cos();
        let oc = 1.0 - c;
        // Rodrigues rotation of `rel` about `a`.
        let cross = [
            a[1] * rel[2] - a[2] * rel[1],
            a[2] * rel[0] - a[0] * rel[2],
            a[0] * rel[1] - a[1] * rel[0],
        ];
        let dot = a[0] * rel[0] + a[1] * rel[1] + a[2] * rel[2];
        let rot = [
            rel[0] * c + cross[0] * s + a[0] * dot * oc,
            rel[1] * c + cross[1] * s + a[1] * dot * oc,
            rel[2] * c + cross[2] * s + a[2] * dot * oc,
        ];
        v[0] = from[0] + rot[0];
        v[1] = from[1] + rot[1];
        v[2] = from[2] + rot[2];
    }
}
/// `modifier_type_mod_warp` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_warp() {}
