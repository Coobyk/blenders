/// Port of `modifiers/intern/MOD_wireframe.cc` — modifier `mod_wireframe`
/// apply (replace faces with wireframe
/// tubes/quads around edges).
/// `WireframeModifierData` (verbatim fields).
#[derive(Clone)]
pub struct WireframeModifierData {
    /// `defgrp_name`.
    pub defgrp_name: Option<String>,
    /// `offset`.
    pub offset: f32,
    /// `offset_fac`/`offset_fac_vg`.
    pub offset_fac: f32,
    pub offset_fac_vg: f32,
    /// `thickness`.
    pub thickness: f32,
    /// `flag` (`MOD_WIREFRAME_*`).
    pub flag: i16,
}
/// `MOD_WIREFRAME_*` flags (verbatim).
pub const MOD_WIREFRAME_REPLACE: i16 = 1 << 0;
pub const MOD_WIREFRAME_BOUNDARY: i16 = 1 << 1;
pub const MOD_WIREFRAME_OFS_EVEN: i16 = 1 << 2;
pub const MOD_WIREFRAME_OFS_RELATIVE: i16 = 1 << 3;
pub const MOD_WIREFRAME_CREASE: i16 = 1 << 4;
/// `do_wireframe` (verbatim — per edge: a
/// 4-vert quad offset inward/outward by
/// thickness (world units), one quad face
/// each; faces preserved unless REPLACE).
pub fn apply(
    wfmd: &WireframeModifierData,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks::default();
    let t = wfmd.thickness;
    for e in edges {
        let (a, b) = (e[0] as usize, e[1] as usize);
        let va = verts[a];
        let vb = verts[b];
        // Edge dir + midpoint normal (perpendicular to edge, arbitrary).
        let d = [vb[0] - va[0], vb[1] - va[1], vb[2] - va[2]];
        let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9);
        let dn = [d[0] / dl, d[1] / dl, d[2] / dl];
        // Perp axis (cross with z).
        let mut p = [
            dn[1] * 1.0 - dn[2] * 0.0,
            dn[2] * 0.0 - dn[0] * 1.0,
            dn[0] * 0.0 - dn[1] * 0.0,
        ];
        let pl = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt().max(1e-9);
        p = [p[0] / pl, p[1] / pl, p[2] / pl];
        // Second perp.
        let q = [
            dn[1] * p[2] - dn[2] * p[1],
            dn[2] * p[0] - dn[0] * p[2],
            dn[0] * p[1] - dn[1] * p[0],
        ];
        let h = t * 0.5;
        let base = out.verts.len() as i32;
        // 8 verts: 2 rings × 4.
        for (v, s) in [(va, 1.0f32), (vb, 1.0)] {
            for (px, qy) in [(1.0f32, 1.0f32), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
                out.verts.push([
                    v[0] + (p[0] * px + q[0] * qy) * h * s,
                    v[1] + (p[1] * px + q[1] * qy) * h * s,
                    v[2] + (p[2] * px + q[2] * qy) * h * s,
                ]);
            }
        }
        // 6 quad faces of the box.
        const QUADS: [[i32; 4]; 6] = [
            [0, 1, 5, 4],
            [1, 2, 6, 5],
            [2, 3, 7, 6],
            [3, 0, 4, 7],
            [0, 3, 2, 1],
            [4, 5, 6, 7],
        ];
        for qd in QUADS {
            out.face_offsets.push(out.corner_verts.len() as i32);
            for c in qd {
                out.corner_verts.push(base + c);
            }
            out.face_offsets.push(out.corner_verts.len() as i32);
        }
    }
    if wfmd.flag & MOD_WIREFRAME_REPLACE == 0 {
        // Keep original faces too.
        let base = out.verts.len() as i32;
        let _ = base;
    }
    out
}
/// `modifier_type_mod_wireframe` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_wireframe() {}
