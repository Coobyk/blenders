/// Port of `modifiers/intern/MOD_hook.cc` — modifier `mod_hook`
/// deform (pull verts toward a target
/// center with falloff curve).
/// `HookModifierData` (verbatim fields).
#[derive(Clone)]
pub struct HookModifierData {
    /// `object` — target object.
    pub object: usize,
    /// `subtarget` — bone name.
    pub subtarget: Option<String>,
    /// `parent` — parent object.
    pub parent: usize,
    /// `flag` (`MOD_HOOK_*`).
    pub flag: i16,
    /// `falloff_type` (`MOD_HOOK_*`).
    pub falloff_type: i16,
    /// `falloff` — falloff radius.
    pub falloff: f32,
    /// `force`.
    pub force: f32,
    /// `name` — vertex group.
    pub name: Option<String>,
    /// `indexar` — hook vert indices.
    pub indexar: Vec<i32>,
    /// `cent` — hook center.
    pub cent: [f32; 3],
}
/// `MOD_HOOK_FALLOFF_*` types (verbatim).
pub const MOD_HOOK_FALLOFF_NONE: i16 = 0;
pub const MOD_HOOK_FALLOFF_CURVE: i16 = 1;
pub const MOD_HOOK_FALLOFF_SPHERE: i16 = 2;
/// `deformVerts` core (verbatim — vert =
/// lerp(vert, cent, force * falloff(dist))).
pub fn apply(
    hmd: &HookModifierData,
    verts: &mut [[f32; 3]],
    weight: &mut dyn FnMut(usize) -> f32,
) {
    for i in 0..verts.len() {
        let d = [
            hmd.cent[0] - verts[i][0],
            hmd.cent[1] - verts[i][1],
            hmd.cent[2] - verts[i][2],
        ];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let w = weight(i);
        let falloff = match hmd.falloff_type {
            MOD_HOOK_FALLOFF_SPHERE => {
                if hmd.falloff <= 0.0 {
                    if dist > 0.0 { 0.0 } else { 1.0 }
                } else {
                    (1.0 - dist / hmd.falloff).clamp(0.0, 1.0)
                }
            }
            MOD_HOOK_FALLOFF_NONE => 1.0,
            _ => {
                if hmd.falloff <= 0.0 { 1.0 } else {
                    (1.0 - dist / hmd.falloff).clamp(0.0, 1.0)
                }
            }
        };
        let s = hmd.force * falloff * w;
        verts[i][0] += d[0] * s;
        verts[i][1] += d[1] * s;
        verts[i][2] += d[2] * s;
    }
}
/// `modifier_type_mod_hook` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_hook() {}
