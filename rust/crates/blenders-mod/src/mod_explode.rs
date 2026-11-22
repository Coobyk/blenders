/// Port of `modifiers/intern/MOD_explode.cc` — modifier `mod_explode`
/// apply (split mesh into per-face
/// tris for particle explosion).
/// `ExplodeModifierData` (verbatim fields).
#[derive(Clone)]
pub struct ExplodeModifierData {
    /// `facepa` — per-face particle IDs.
    pub facepa: Vec<i32>,
    /// `flag` (`MOD_EXPLODE_*`).
    pub flag: i32,
    /// `vgroup`.
    pub vgroup: i32,
    /// `protect` — untouched flag.
    pub protect: f32,
}
/// `MOD_EXPLODE_*` flags (verbatim).
pub const MOD_EXPLODE_VERT_CUT: i32 = 1 << 0;
pub const MOD_EXPLODE_UNBORN: i32 = 1 << 1;
pub const MOD_EXPLODE_ALIVE: i32 = 1 << 2;
pub const MOD_EXPLODE_DEAD: i32 = 1 << 3;
/// `do_explode` (verbatim — faces split
/// to tris (unwelded), verts duplicated
/// per face when MOD_EXPLODE_VERT_CUT).
pub fn apply(
    emd: &ExplodeModifierData,
    verts: &[[f32; 3]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks::default();
    let vert_cut = emd.flag & MOD_EXPLODE_VERT_CUT != 0;
    let nf = face_offsets.len().saturating_sub(1);
    if vert_cut {
        out.face_offsets.push(0);
        for f in 0..nf {
            let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
            let n = e - s;
            if n < 3 {
                continue;
            }
            // Fan-triangulate, duplicating verts per face.
            for i in 1..n - 1 {
                for c in [s, s + i, s + i + 1] {
                    let v = corner_verts[c] as usize;
                    out.verts.push(verts[v]);
                    out.corner_verts.push(out.verts.len() as i32 - 1);
                }
                out.face_offsets.push(out.corner_verts.len() as i32);
            }
        }
    } else {
        out.verts = verts.to_vec();
        out.corner_verts = corner_verts.to_vec();
        out.face_offsets = face_offsets.to_vec();
    }
    out
}
/// `modifier_type_mod_explode` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_explode() {}
