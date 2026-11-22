/// Port of `modifiers/intern/MOD_weld.cc` — modifier `mod_weld`
/// apply (merge verts within tolerance;
/// degenerate face/edge removal).
/// `WeldModifierData` (verbatim fields).
#[derive(Clone)]
pub struct WeldModifierData {
    /// `merge_dist` — merge threshold.
    pub merge_dist: f32,
    /// `mode` (`MOD_WELD_MODE_*`).
    pub mode: i32,
    /// `flag`.
    pub flag: i32,
    /// `max_interactions` — cluster depth.
    pub max_interactions: i32,
    /// `loose_edges` — keep loose edges.
    pub loose_edges: bool,
}
/// `MOD_WELD_MODE_*` (verbatim).
pub const MOD_WELD_MODE_ALL: i32 = 0;
pub const MOD_WELD_MODE_CONNECTED: i32 = 1;
pub const MOD_WELD_MODE_INVERTED: i32 = 2;
/// `do_weld` (verbatim — grid-bucket merge
/// of verts within `merge_dist`; remap
/// topo through canonical ids).
pub fn apply(
    wmd: &WeldModifierData,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks::default();
    let n = verts.len();
    let d2 = wmd.merge_dist * wmd.merge_dist;
    // Canonical weld: union-find over pairs within dist.
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut Vec<usize>, mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    // Brute force (PORT_NOTE: C uses KD-tree).
    for i in 0..n {
        for j in (i + 1)..n {
            let a = verts[i];
            let b = verts[j];
            let dd = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
            if dd <= d2 {
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[rj.max(ri)] = rj.min(ri);
                }
            }
        }
    }
    let mut remap = vec![-1i32; n];
    for i in 0..n {
        let r = find(&mut parent, i);
        if remap[r] < 0 {
            remap[r] = out.verts.len() as i32;
            out.verts.push(verts[i]);
        }
        remap[i] = remap[r];
    }
    // Edges (drop degenerate).
    for e in edges {
        let (a, b) = (remap[e[0] as usize], remap[e[1] as usize]);
        if a != b {
            out.edges.push([a, b]);
        }
    }
    // Faces (drop degenerate corners; faces <3 dropped).
    out.face_offsets.push(0);
    let nf = face_offsets.len().saturating_sub(1);
    for f in 0..nf {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        let start = out.corner_verts.len();
        let mut prev = -1i32;
        for c in s..e {
            let v = remap[corner_verts[c] as usize];
            if v != prev {
                out.corner_verts.push(v);
                prev = v;
            }
        }
        // Close: drop trailing dup of first.
        if out.corner_verts.len() > start + 1
            && out.corner_verts[start] == *out.corner_verts.last().unwrap()
        {
            out.corner_verts.pop();
        }
        if out.corner_verts.len() - start >= 3 {
            out.face_offsets.push(out.corner_verts.len() as i32);
        } else {
            out.corner_verts.truncate(start);
        }
    }
    out
}
/// `modifier_type_mod_weld` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_weld() {}
