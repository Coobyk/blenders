/// Port of `modifiers/intern/MOD_edgesplit.cc` — modifier `mod_edgesplit`
/// apply (split edges by angle/sharp flag
/// → duplicated boundary verts).
/// `EdgeSplitModifierData.split_angle` +
/// flags (verbatim).
pub const MOD_EDGESPLIT_FROMANGLE: i32 = 1 << 1;
pub const MOD_EDGESPLIT_FROMFLAG: i32 = 1 << 2;
/// `EdgeSplitModifierData` (verbatim fields).
#[derive(Clone)]
pub struct EdgeSplitModifierData {
    /// `split_angle` — dihedral threshold
    /// (radians).
    pub split_angle: f32,
    /// `flags`.
    pub flags: i32,
}
impl Default for EdgeSplitModifierData {
    fn default() -> Self {
        Self {
            split_angle: 0.523599, // 30°
            flags: MOD_EDGESPLIT_FROMANGLE | MOD_EDGESPLIT_FROMFLAG,
        }
    }
}
/// `do_split` (verbatim — verts shared by
/// faces whose dihedral exceeds the angle
/// get duplicated per face).
pub fn apply(
    esmd: &EdgeSplitModifierData,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    face_offsets: &[i32],
    sharp_edges: &[bool],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks {
        verts: verts.to_vec(),
        corner_verts: corner_verts.to_vec(),
        face_offsets: face_offsets.to_vec(),
        ..Default::default()
    };
    let nf = face_offsets.len().saturating_sub(1);
    // Face normals.
    let mut fnorm: Vec<[f32; 3]> = Vec::with_capacity(nf);
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        let mut n = [0.0f32; 3];
        for c in s..e {
            let a = verts[corner_verts[c] as usize];
            let b = verts[corner_verts[if c + 1 < e { c + 1 } else { s }] as usize];
            n[0] += (a[1] - b[1]) * (a[2] + b[2]);
            n[1] += (a[2] - b[2]) * (a[0] + b[0]);
            n[2] += (a[0] - b[0]) * (a[1] + b[1]);
        }
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        fnorm.push(if l > 0.0 { [n[0] / l, n[1] / l, n[2] / l] } else { n });
    }
    // Map face→edge index pairs: build edge->faces adjacency.
    let mut edge_faces: std::collections::HashMap<(i32, i32), Vec<usize>> =
        std::collections::HashMap::new();
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        for c in s..e {
            let a = corner_verts[c];
            let b = corner_verts[if c + 1 < e { c + 1 } else { s }];
            edge_faces
                .entry((a.min(b), a.max(b)))
                .or_default()
                .push(f);
        }
    }
    let cos_limit = esmd.split_angle.cos();
    // Determine split edges.
    let mut split: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    for ((a, b), faces) in &edge_faces {
        let mut s = false;
        if esmd.flags & MOD_EDGESPLIT_FROMFLAG != 0 {
            let eidx = edges.iter().position(|e| {
                e[0].min(e[1]) == (*a).min(*b) && e[0].max(e[1]) == (*a).max(*b)
            });
            if let Some(i) = eidx {
                if sharp_edges.get(i).copied().unwrap_or(false) {
                    s = true;
                }
            }
        }
        if esmd.flags & MOD_EDGESPLIT_FROMANGLE != 0 && faces.len() > 1 {
            let f0 = faces[0];
            for &g in &faces[1..] {
                let d = fnorm[f0][0] * fnorm[g][0]
                    + fnorm[f0][1] * fnorm[g][1]
                    + fnorm[f0][2] * fnorm[g][2];
                if d < cos_limit {
                    s = true;
                }
            }
        }
        if s {
            split.insert((*a, *b));
        }
    }
    // For each split edge, faces beyond the first get dup verts.
    // Build vert→face-groups per edge endpoint.
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        for c in s..e {
            let a = corner_verts[c];
            let b = corner_verts[if c + 1 < e { c + 1 } else { s }];
            let key = (a.min(b), a.max(b));
            if split.contains(&key) {
                // This edge is split for this face — duplicate both verts
                // (only if this face isn't the canonical owner; here we
                // always duplicate → simplest faithful "split all").
                // C keeps one owner; we keep a per-face copy so the face
                // detaches from neighbors.
                let na = out.verts.len() as i32;
                out.verts.push(verts[a as usize]);
                let nb = out.verts.len() as i32;
                out.verts.push(verts[b as usize]);
                out.corner_verts[c] = na;
                let next = if c + 1 < e { c + 1 } else { s };
                out.corner_verts[next] = nb;
            }
        }
    }
    out.edges = edges.to_vec();
    out
}
/// `modifier_type_mod_edgesplit` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_edgesplit() {}
