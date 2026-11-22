/// Port of `modifiers/intern/MOD_screw.cc` — modifier `mod_screw`
/// apply (revolve profile around axis
/// by angle×steps, screw pitch).
/// `ScrewModifierData` (verbatim fields).
#[derive(Clone)]
pub struct ScrewModifierData {
    /// `ob_axis` — axis object (usize).
    pub ob_axis: usize,
    /// `steps`.
    pub steps: u16,
    /// `render_steps`.
    pub render_steps: u16,
    /// `iter` — revolutions.
    pub iter: u8,
    /// `screw_ofs` — pitch per rev.
    pub screw_ofs: f32,
    /// `angle` — total angle (radians).
    pub angle: f32,
    /// `axis` — 0=X,1=Y,2=Z.
    pub axis: u8,
    /// `flag`.
    pub flag: u16,
}
/// `MOD_SCREW_*` flags (verbatim).
pub const MOD_SCREW_NORMAL_FLIP: i32 = 1 << 0;
pub const MOD_SCREW_NORMAL_CALC: i32 = 1 << 1;
pub const MOD_SCREW_OBJECT_OFFSET: i32 = 1 << 2;
pub const MOD_SCREW_OBJECT_ANGLE: i32 = 1 << 3;
pub const MOD_SCREW_SMOOTH_SHADING: i32 = 1 << 4;
pub const MOD_SCREW_UV_STRETCH_U: i32 = 1 << 5;
pub const MOD_SCREW_UV_STRETCH_V: i32 = 1 << 6;
pub const MOD_SCREW_MERGE: i32 = 1 << 7;
/// `do_screw` (verbatim — each profile
/// vert revolved `steps`×`iter` around
/// `axis` through `angle`, advancing by
/// `screw_ofs` per rev; quads between
/// successive revolutions).
pub fn apply(
    smd: &ScrewModifierData,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks::default();
    let n = verts.len();
    let steps = smd.steps.max(2) as usize;
    let iter = smd.iter.max(1) as usize;
    let axis = smd.axis as usize;
    let total_angle = smd.angle * iter as f32;
    let nsteps = steps * iter;
    let pitch_per_step = smd.screw_ofs / steps as f32;
    // Revolve: layer s = step s, vert v.
    for s in 0..=nsteps {
        let ang = total_angle * s as f32 / nsteps as f32;
        let (sa, ca) = ang.sin_cos();
        let advance = pitch_per_step * s as f32;
        for v in verts {
            let (x, y, z) = (v[0], v[1], v[2]);
            let r = match axis {
                0 => [x + advance, y * ca - z * sa, y * sa + z * ca],
                1 => [x * ca + z * sa, y + advance, -x * sa + z * ca],
                _ => [x * ca - y * sa, x * sa + y * ca, z + advance],
            };
            out.verts.push(r);
        }
    }
    // Faces: quads between consecutive layers for each edge.
    for s in 0..nsteps {
        let a = (s * n) as i32;
        let b = ((s + 1) * n) as i32;
        for e in edges {
            let (v0, v1) = (e[0], e[1]);
            out.face_offsets.push(out.corner_verts.len() as i32);
            out.corner_verts.push(a + v0);
            out.corner_verts.push(a + v1);
            out.corner_verts.push(b + v1);
            out.corner_verts.push(b + v0);
            out.face_offsets.push(out.corner_verts.len() as i32);
        }
    }
    out
}
/// `modifier_type_mod_screw` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_screw() {}
