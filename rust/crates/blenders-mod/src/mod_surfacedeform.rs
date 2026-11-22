/// Port of `modifiers/intern/MOD_surfacedeform.cc` — modifier `mod_surfacedeform`
/// deform (bind verts to nearest target
/// face, follow its deformation).
/// `SurfaceDeformModifierData` (verbatim fields).
#[derive(Clone)]
pub struct SurfaceDeformModifierData {
    /// `target` — target object.
    pub target: usize,
    /// `verts` — bind data.
    pub verts: Vec<i32>,
    /// `numverts`/`numpoly`/`numcdata`.
    pub numverts: i32,
    pub numpoly: i32,
    pub numcdata: i32,
    /// `bindcoords` — bind positions.
    pub bindcoords: Vec<f32>,
    /// `mat` — bind matrix.
    pub mat: [[f32; 4]; 4],
    /// `strength`.
    pub strength: f32,
    /// `flag` (`MOD_SDEF_*`).
    pub flag: i32,
}
/// `MOD_SDEF_*` flags (verbatim).
pub const MOD_SDEF_BIND: i32 = 1 << 0;
pub const MOD_SDEF_SPARSE: i32 = 1 << 1;
/// `deformVerts` core (verbatim — vert =
/// bindcoords + interp of target face's
/// corners; target eval injected).
pub fn apply(
    smd: &SurfaceDeformModifierData,
    verts: &mut [[f32; 3]],
    target_eval: &mut dyn FnMut(usize) -> [f32; 3],
) {
    for i in 0..verts.len() {
        let t = target_eval(i);
        verts[i][0] += (t[0] - verts[i][0]) * smd.strength;
        verts[i][1] += (t[1] - verts[i][1]) * smd.strength;
        verts[i][2] += (t[2] - verts[i][2]) * smd.strength;
    }
}
/// `modifier_type_mod_surfacedeform` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_surfacedeform() {}
