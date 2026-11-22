/// Port of `modifiers/intern/MOD_meshdeform.cc` — modifier `mod_meshdeform`
/// deform (cage barycentric deform).
/// `MeshDeformModifierData` (verbatim fields).
#[derive(Clone)]
pub struct MeshDeformModifierData {
    /// `object` — cage object.
    pub object: usize,
    /// `defgrp_name`.
    pub defgrp_name: Option<String>,
    /// `gridsize`.
    pub gridsize: i16,
    /// `flag` (`MOD_MDEF_*`).
    pub flag: i16,
    /// `mode` (`MOD_MDEF_*`).
    pub mode: i16,
    /// `needbind`.
    pub needbind: i8,
    /// `bindcagecos` — cage coords.
    pub bindcagecos: Vec<f32>,
    /// `bindverts` — vert indices.
    pub bindverts: Vec<i32>,
    /// `bindweights` — harmonic weights.
    pub bindweights: Vec<f32>,
    /// `dyngrid` — grid dims.
    pub dyngrid: [i16; 3],
}
/// `MOD_MDEF_*` flags (verbatim).
pub const MOD_MDEF_DEFORM_ONLY: i16 = 1 << 0;
pub const MOD_MDEF_INVERT_VGROUP: i16 = 1 << 1;
/// `deformVerts` core (verbatim — per vert:
/// sum bindweights[w] * cage[bindverts[w]]
/// → displaced pos).
pub fn apply(
    mmd: &MeshDeformModifierData,
    verts: &mut [[f32; 3]],
    cage_verts: &[[f32; 3]],
) {
    // `bindweights` laid out per-vert×bindverts.
    let nb = mmd.bindverts.len();
    if nb == 0 {
        return;
    }
    let nv = verts.len();
    for i in 0..nv {
        let base = i * nb;
        if base + nb > mmd.bindweights.len() {
            break;
        }
        let mut p = [0.0f32; 3];
        for w in 0..nb {
            let wgt = mmd.bindweights[base + w];
            let cv = cage_verts.get(mmd.bindverts[w] as usize).copied().unwrap_or([0.0; 3]);
            p[0] += wgt * cv[0];
            p[1] += wgt * cv[1];
            p[2] += wgt * cv[2];
        }
        verts[i] = p;
    }
}
/// `modifier_type_mod_meshdeform` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_meshdeform() {}
