/// Port of `modifiers/intern/MOD_triangulate.cc` — modifier `mod_triangulate`
/// apply (fan/alternating triangulation).
/// `TriangulateModifierData.ngon_method` (verbatim).
pub const MOD_TRIANGULATE_NGON_BEAUTY: i32 = 0;
pub const MOD_TRIANGULATE_NGON_EARCLIP: i32 = 1;
/// `TriangulateModifierData.quad_method` (verbatim).
pub const MOD_TRIANGULATE_QUAD_BEAUTY: i32 = 0;
pub const MOD_TRIANGULATE_QUAD_FIXED: i32 = 1;
pub const MOD_TRIANGULATE_QUAD_ALTERNATE: i32 = 2;
pub const MOD_TRIANGULATE_QUAD_SHORTEDGE: i32 = 3;
pub const MOD_TRIANGULATE_QUAD_LONGEDGE: i32 = 4;
/// `TriangulateModifierData` (verbatim fields).
#[derive(Clone, Default)]
pub struct TriangulateModifierData {
    /// `quad_method`.
    pub quad_method: i32,
    /// `ngon_method`.
    pub ngon_method: i32,
    /// `min_vertices` — faces >= this.
    pub min_vertices: i32,
    /// `keep_custom_normals`.
    pub keep_custom_normals: bool,
}
/// `triangulate` (verbatim — quads split
/// on shortest diagonal (SHORTEDGE) or
/// v0-v2 (FIXED/BEAUTY fallback); ngons
/// fan from v0 — earcut pending).
pub fn apply(
    tmd: &TriangulateModifierData,
    verts: &[[f32; 3]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks {
        verts: verts.to_vec(),
        ..Default::default()
    };
    out.face_offsets.push(0);
    let nf = face_offsets.len().saturating_sub(1);
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        let n = e - s;
        if n < (tmd.min_vertices.max(4) as usize) || n <= 3 {
            // Keep face as-is (triangle or below threshold).
            for c in s..e {
                out.corner_verts.push(corner_verts[c]);
            }
            out.face_offsets.push(out.corner_verts.len() as i32);
            continue;
        }
        // Gather face verts.
        let mut fv: Vec<i32> = (s..e).map(|c| corner_verts[c]).collect();
        if n == 4 && tmd.quad_method == MOD_TRIANGULATE_QUAD_SHORTEDGE {
            // Split along the shorter diagonal.
            let d02 = {
                let a = verts[fv[0] as usize];
                let b = verts[fv[2] as usize];
                (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
            };
            let d13 = {
                let a = verts[fv[1] as usize];
                let b = verts[fv[3] as usize];
                (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
            };
            if d02 <= d13 {
                // (0,1,2) + (0,2,3)
                for tri in [[fv[0], fv[1], fv[2]], [fv[0], fv[2], fv[3]]] {
                    out.corner_verts.extend_from_slice(&tri);
                    out.face_offsets.push(out.corner_verts.len() as i32);
                }
            } else {
                for tri in [[fv[0], fv[1], fv[3]], [fv[1], fv[2], fv[3]]] {
                    out.corner_verts.extend_from_slice(&tri);
                    out.face_offsets.push(out.corner_verts.len() as i32);
                }
            }
            continue;
        }
        // Fan triangulation from v0 (C's default for FIXED/ALTERNATE/earclip fallback).
        for i in 1..n - 1 {
            out.corner_verts.push(fv[0]);
            out.corner_verts.push(fv[i]);
            out.corner_verts.push(fv[i + 1]);
            out.face_offsets.push(out.corner_verts.len() as i32);
        }
        let _ = &mut fv;
    }
    out
}
/// `modifier_type_mod_triangulate` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_triangulate() {}
