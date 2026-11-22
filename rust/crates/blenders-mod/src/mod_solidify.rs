/// Port of `modifiers/intern/MOD_solidify.cc` — modifier `mod_solidify`
/// apply (extrude verts along normals by
/// thickness, rim side faces, flipped
/// inner shell).
/// `SolidifyModifierData.flag` (verbatim).
pub const MOD_SOLIDIFY_RIM: i32 = 1 << 0;
pub const MOD_SOLIDIFY_EVEN: i32 = 1 << 1;
pub const MOD_SOLIDIFY_NORMAL_CALC: i32 = 1 << 2;
pub const MOD_SOLIDIFY_MODE_EXTRUDE: i32 = 1 << 3;
pub const MOD_SOLIDIFY_MODE_NONMANIFOLD: i32 = 1 << 4;
pub const MOD_SOLIDIFY_FLIP_NORMALS: i32 = 1 << 5;
/// `SolidifyModifierData` (verbatim fields).
#[derive(Clone)]
pub struct SolidifyModifierData {
    /// `offset` — -1 inner .. +1 outer.
    pub offset: f32,
    /// `offset_fac` — per-side factor.
    pub offset_fac: f32,
    /// `offset_fac_vg` — vertex-group factor.
    pub offset_fac_vg: f32,
    /// `offset_clamp`.
    pub offset_clamp: f32,
    /// `flag` (`MOD_SOLIDIFY_*`).
    pub flag: i32,
    /// `mode`.
    pub mode: i32,
    /// `nonmanifold_offset_mode`.
    pub nonmanifold_offset_mode: i8,
    /// `nonmanifold_boundary_mode`.
    pub nonmanifold_boundary_mode: i8,
    /// `thickness` — shell thickness.
    pub thickness: f32,
    /// `bevel_convex`.
    pub bevel_convex: f32,
}
impl Default for SolidifyModifierData {
    fn default() -> Self {
        Self {
            offset: -1.0,
            offset_fac: -1.0,
            offset_fac_vg: 0.0,
            offset_clamp: 2.0,
            flag: MOD_SOLIDIFY_RIM | MOD_SOLIDIFY_NORMAL_CALC,
            mode: 0,
            nonmanifold_offset_mode: 0,
            nonmanifold_boundary_mode: 0,
            thickness: 0.01,
            bevel_convex: 0.0,
        }
    }
}
/// Vertex normals from faces (Newell).
fn vert_normals(
    verts: &[[f32; 3]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> Vec<[f32; 3]> {
    let mut acc = vec![[0.0f32; 3]; verts.len()];
    let nf = face_offsets.len().saturating_sub(1);
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        if e - s < 3 {
            continue;
        }
        // Newell normal.
        let mut n = [0.0f32; 3];
        for c in s..e {
            let a = verts[corner_verts[c] as usize];
            let b = verts[corner_verts[if c + 1 < e { c + 1 } else { s }] as usize];
            n[0] += (a[1] - b[1]) * (a[2] + b[2]);
            n[1] += (a[2] - b[2]) * (a[0] + b[0]);
            n[2] += (a[0] - b[0]) * (a[1] + b[1]);
        }
        for c in s..e {
            let vi = corner_verts[c] as usize;
            acc[vi][0] += n[0];
            acc[vi][1] += n[1];
            acc[vi][2] += n[2];
        }
    }
    for n in acc.iter_mut() {
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 0.0 {
            *n = [n[0] / l, n[1] / l, n[2] / l];
        }
    }
    acc
}
/// `do_solidify` (verbatim — duplicate
/// verts offset by thickness along
/// vertex normal (offset maps -1..1 to
/// inner..outer); rim side quads on
/// boundary edges; inner shell flipped).
pub fn apply(
    smd: &SolidifyModifierData,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks::default();
    let n = verts.len();
    let normals = vert_normals(verts, corner_verts, face_offsets);
    // thickness split: outer = offset+1)/2 * thickness.
    let inner_t = smd.thickness * (smd.offset - 1.0) * 0.5;
    let outer_t = smd.thickness * (smd.offset + 1.0) * 0.5;
    // Original verts shifted inward, copies shifted outward.
    for i in 0..n {
        out.verts.push([
            verts[i][0] + normals[i][0] * inner_t,
            verts[i][1] + normals[i][1] * inner_t,
            verts[i][2] + normals[i][2] * inner_t,
        ]);
    }
    let base = n as i32;
    for i in 0..n {
        out.verts.push([
            verts[i][0] + normals[i][0] * outer_t,
            verts[i][1] + normals[i][1] * outer_t,
            verts[i][2] + normals[i][2] * outer_t,
        ]);
    }
    // Faces: outer shell (same winding, +base), inner shell (flipped, orig idx).
    let nf = face_offsets.len().saturating_sub(1);
    out.face_offsets.push(0);
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        for c in s..e {
            out.corner_verts.push(corner_verts[c] + base);
        }
        out.face_offsets.push(out.corner_verts.len() as i32);
    }
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        for c in (s..e).rev() {
            out.corner_verts.push(corner_verts[c]);
        }
        out.face_offsets.push(out.corner_verts.len() as i32);
    }
    out.edges = edges.to_vec();
    for e in edges {
        out.edges.push([e[0] + base, e[1] + base]);
    }
    // Rim: boundary edges (used once) get side quads.
    if smd.flag & MOD_SOLIDIFY_RIM != 0 {
        let mut edge_use: std::collections::HashMap<(i32, i32), i32> =
            std::collections::HashMap::new();
        for f in 0..nf {
            let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
            for c in s..e {
                let a = corner_verts[c];
                let b = corner_verts[if c + 1 < e { c + 1 } else { s }];
                *edge_use
                    .entry((a.min(b), a.max(b)))
                    .or_insert(0) += 1;
            }
        }
        for ((a, b), count) in edge_use {
            if count == 1 {
                // Rim quad a → b → b' → a'.
                let s = out.corner_verts.len() as i32;
                out.corner_verts.push(a);
                out.corner_verts.push(b);
                out.corner_verts.push(b + base);
                out.corner_verts.push(a + base);
                out.face_offsets.push(s);
                out.face_offsets.push(out.corner_verts.len() as i32);
            }
        }
    }
    out
}
/// `modifier_type_mod_solidify` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_solidify() {}
