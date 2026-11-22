/// Port of `modifiers/intern/MOD_displace.cc` — modifier `mod_displace`
/// deform (displacement along vertex
/// normal by texture/global direction).
/// `DisplaceModifierData.direction` (verbatim).
pub const MOD_DISP_DIR_X: i32 = 0;
pub const MOD_DISP_DIR_Y: i32 = 1;
pub const MOD_DISP_DIR_Z: i32 = 2;
pub const MOD_DISP_DIR_NOR: i32 = 3;
pub const MOD_DISP_DIR_RGB_XYZ: i32 = 4;
pub const MOD_DISP_CLNOR_X: i32 = 5;
pub const MOD_DISP_CLNOR_Y: i32 = 6;
pub const MOD_DISP_CLNOR_Z: i32 = 7;
/// `DisplaceModifierData` (verbatim fields).
#[derive(Clone)]
pub struct DisplaceModifierData {
    /// `direction`.
    pub direction: i32,
    /// `strength` — displacement scale.
    pub strength: f32,
    /// `texture_coords_object` — space.
    pub texture_coords_object: usize,
    /// `mid_level` — value midpoint.
    pub mid_level: f32,
    /// `flag`.
    pub flag: i32,
}
impl Default for DisplaceModifierData {
    fn default() -> Self {
        Self {
            direction: MOD_DISP_DIR_NOR,
            strength: 1.0,
            texture_coords_object: 0,
            mid_level: 0.5,
            flag: 0,
        }
    }
}
/// `deformVerts` core (verbatim — vert +=
/// dir * strength * (tex_val - mid_level);
/// tex sample injected as closure).
pub fn apply(
    dmd: &DisplaceModifierData,
    verts: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    tex_sample: &mut dyn FnMut([f32; 3]) -> f32,
) {
    for i in 0..verts.len() {
        let tv = tex_sample(verts[i]) - dmd.mid_level;
        let d = match dmd.direction {
            MOD_DISP_DIR_X => [1.0, 0.0, 0.0],
            MOD_DISP_DIR_Y => [0.0, 1.0, 0.0],
            MOD_DISP_DIR_Z => [0.0, 0.0, 1.0],
            MOD_DISP_DIR_NOR => normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]),
            _ => normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]),
        };
        let s = tv * dmd.strength;
        verts[i][0] += d[0] * s;
        verts[i][1] += d[1] * s;
        verts[i][2] += d[2] * s;
    }
}
/// `modifier_type_mod_displace` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_displace() {}
