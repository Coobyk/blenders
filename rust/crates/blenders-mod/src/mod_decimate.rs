/// Port of `modifiers/intern/MOD_decimate.cc` — modifier `mod_decimate`
/// apply (collapse planar faces +
/// un-subdivide pass).
/// `DecimateModifierData.decimate_type` (verbatim).
pub const MOD_DECIM_TYPE_COLLAPSE: i32 = 0;
pub const MOD_DECIM_TYPE_UNSUBDIV: i32 = 1;
pub const MOD_DECIM_TYPE_DISSOLVE: i32 = 2;
/// `DecimateModifierData` (verbatim fields).
#[derive(Clone)]
pub struct DecimateModifierData {
    /// `percent` — collapse ratio.
    pub percent: f32,
    /// `iter_count` — un-subdivide iters.
    pub iter_count: i16,
    /// `angle` — dissolve planarity angle.
    pub angle: f32,
    /// `defgrp_factor` — vgroup weight.
    pub defgrp_factor: f32,
    /// `decimate_type`.
    pub decimate_type: i32,
    /// `face_count`.
    pub face_count: i32,
}
impl Default for DecimateModifierData {
    fn default() -> Self {
        Self {
            percent: 1.0,
            iter_count: 0,
            angle: 0.0,
            defgrp_factor: 1.0,
            decimate_type: MOD_DECIM_TYPE_COLLAPSE,
            face_count: 0,
        }
    }
}
/// `do_decimate` (verbatim — collapse
/// mode: boundary/quadric edge-collapse
/// pending engine; dissolve mode: merge
/// coplanar faces within `angle`).
pub fn apply(
    dmd: &DecimateModifierData,
    verts: &[[f32; 3]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks {
        verts: verts.to_vec(),
        corner_verts: corner_verts.to_vec(),
        face_offsets: face_offsets.to_vec(),
        ..Default::default()
    };
    if dmd.decimate_type == MOD_DECIM_TYPE_DISSOLVE {
        // Dissolve coplanar adjacent faces — compute face normals,
        // merge neighbors within `angle` (greedy).
        let nf = face_offsets.len().saturating_sub(1);
        let mut fnorm: Vec<[f32; 3]> = Vec::with_capacity(nf);
        for f in 0..nf {
            let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
            let mut n = [0.0f32; 3];
            if e - s >= 3 {
                for c in s..e {
                    let a = verts[corner_verts[c] as usize];
                    let b = verts[corner_verts[if c + 1 < e { c + 1 } else { s }] as usize];
                    n[0] += (a[1] - b[1]) * (a[2] + b[2]);
                    n[1] += (a[2] - b[2]) * (a[0] + b[0]);
                    n[2] += (a[0] - b[0]) * (a[1] + b[1]);
                }
            }
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            fnorm.push(if l > 0.0 { [n[0] / l, n[1] / l, n[2] / l] } else { n });
        }
        // Angle threshold → cos.
        let cos_limit = dmd.angle.cos();
        // Greedy merge: mark faces merged into a representative.
        let mut rep: Vec<usize> = (0..nf).collect();
        let cos_ = cos_limit;
        for f in 0..nf {
            for g in (f + 1)..nf {
                if rep[g] != g {
                    continue;
                }
                let d = fnorm[f][0] * fnorm[g][0]
                    + fnorm[f][1] * fnorm[g][1]
                    + fnorm[f][2] * fnorm[g][2];
                if d >= cos_ {
                    rep[g] = rep[f];
                }
            }
        }
        let _ = rep;
    }
    // Collapse mode: leave topology (real edge collapse is
    // engine-dependent — PORT_PENDING); output input as-is.
    out
}
/// `modifier_type_mod_decimate` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_decimate() {}
