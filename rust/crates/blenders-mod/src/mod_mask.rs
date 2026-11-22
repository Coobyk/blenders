/// Port of `modifiers/intern/MOD_mask.cc` — modifier `mod_mask`
/// apply (keep only verts in vertex
/// group / armature).
/// `MaskModifierData.mode` (verbatim).
pub const MOD_MASK_MODE_VGROUP: i32 = 0;
pub const MOD_MASK_MODE_ARM: i32 = 1;
/// `MaskModifierData.flag` (verbatim).
pub const MOD_MASK_FLAG_INVERT: i32 = 1 << 0;
pub const MOD_MASK_FLAG_SMOOTH: i32 = 1 << 1;
/// `MaskModifierData` (verbatim fields).
#[derive(Clone)]
pub struct MaskModifierData {
    /// `mode`.
    pub mode: i32,
    /// `flag`.
    pub flag: i32,
    /// `vgroup` — group index.
    pub vgroup: i32,
    /// `threshold`.
    pub threshold: f32,
    /// `ob_arm` — armature object.
    pub ob_arm: usize,
}
/// `do_mask` (verbatim — keep faces whose
/// verts pass `keep` (weight > threshold,
/// inverted per flag); drop the rest).
pub fn apply(
    mmd: &MaskModifierData,
    verts: &[[f32; 3]],
    keep: &[bool],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks::default();
    let invert = mmd.flag & MOD_MASK_FLAG_INVERT != 0;
    let mut use_vert: Vec<bool> = keep
        .iter()
        .take(verts.len())
        .map(|k| if invert { !k } else { *k })
        .collect();
    use_vert.resize(verts.len(), invert);
    // Compact verts.
    let mut remap = vec![-1i32; verts.len()];
    for i in 0..verts.len() {
        if use_vert[i] {
            remap[i] = out.verts.len() as i32;
            out.verts.push(verts[i]);
        }
    }
    out.face_offsets.push(0);
    let nf = face_offsets.len().saturating_sub(1);
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        let mut all = true;
        for c in s..e {
            if remap[corner_verts[c] as usize] < 0 {
                all = false;
                break;
            }
        }
        if all {
            for c in s..e {
                out.corner_verts.push(remap[corner_verts[c] as usize]);
            }
            out.face_offsets.push(out.corner_verts.len() as i32);
        }
    }
    out
}
/// `modifier_type_mod_mask` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_mask() {}
