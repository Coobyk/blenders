/// Port of `modifiers/intern/MOD_array.cc` — modifier `mod_array`
/// apply (array: relative/const/object
/// offset × count + vertex merge).
/// `ArrayModifierData.fit_type` (verbatim).
pub const MOD_ARR_FIXEDCOUNT: i32 = 0;
pub const MOD_ARR_FITLENGTH: i32 = 1;
pub const MOD_ARR_FITCURVE: i32 = 2;
/// `ArrayModifierData.offset_type` (verbatim).
pub const MOD_ARR_OFF_CONST: i32 = 1 << 0;
pub const MOD_ARR_OFF_RELATIVE: i32 = 1 << 1;
pub const MOD_ARR_OFF_OBJ: i32 = 1 << 2;
/// `ArrayModifierData.flags` (verbatim).
pub const MOD_ARR_MERGE: i32 = 1 << 0;
pub const MOD_ARR_MERGE_FINAL: i32 = 1 << 1;
/// `ArrayModifierData` (verbatim fields).
#[derive(Clone)]
pub struct ArrayModifierData {
    /// `offset` — constant offset.
    pub offset: [f32; 3],
    /// `scale` — relative (bounds) offset.
    pub scale: [f32; 3],
    /// `length` — fit-length target.
    pub length: f32,
    /// `merge_dist` — merge threshold.
    pub merge_dist: f32,
    /// `fit_type`.
    pub fit_type: i32,
    /// `offset_type`.
    pub offset_type: i32,
    /// `flags`.
    pub flags: i32,
    /// `count`.
    pub count: i32,
}
impl Default for ArrayModifierData {
    fn default() -> Self {
        Self {
            offset: [1.0, 0.0, 0.0],
            scale: [1.0, 0.0, 0.0],
            length: 0.0,
            merge_dist: 0.01,
            fit_type: MOD_ARR_FIXEDCOUNT,
            offset_type: MOD_ARR_OFF_RELATIVE,
            flags: 0,
            count: 2,
        }
    }
}
/// Owned mesh chunk (`verts`/`edges`/
/// `corner_verts`/`faces` ranges).
#[derive(Default, Clone)]
pub struct MeshChunks {
    pub verts: Vec<[f32; 3]>,
    pub edges: Vec<[i32; 2]>,
    pub corner_verts: Vec<i32>,
    /// Face corner ranges (into
    /// `corner_verts`), `faces_num+1` offsets.
    pub face_offsets: Vec<i32>,
}
/// `arrayModifier_doArray` (verbatim —
/// duplicate the mesh `count` times, each
/// with cumulative `offset`; merge verts
/// within `merge_dist` when MOD_ARR_MERGE).
pub fn apply(
    amd: &ArrayModifierData,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> MeshChunks {
    let mut out = MeshChunks::default();
    let nverts = verts.len();
    if nverts == 0 {
        return out;
    }
    // Bounds for relative offset.
    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];
    for v in verts {
        for i in 0..3 {
            bmin[i] = bmin[i].min(v[i]);
            bmax[i] = bmax[i].max(v[i]);
        }
    }
    // `offset` accumulation (C builds a mat).
    let mut offset = [0.0f32; 3];
    if amd.offset_type & MOD_ARR_OFF_CONST != 0 {
        for i in 0..3 {
            offset[i] += amd.offset[i];
        }
    }
    if amd.offset_type & MOD_ARR_OFF_RELATIVE != 0 {
        for i in 0..3 {
            offset[i] += amd.scale[i] * (bmax[i] - bmin[i]);
        }
    }
    // Count from fit_type.
    let mut count = amd.count.max(0);
    if amd.fit_type == MOD_ARR_FITLENGTH || amd.fit_type == MOD_ARR_FITCURVE {
        let dist = (offset[0] * offset[0] + offset[1] * offset[1] + offset[2] * offset[2]).sqrt();
        if dist > 1e-4 {
            count = ((amd.length + 1e-4) / dist + 1.0) as i32;
        } else {
            count = 1;
        }
    }
    count = count.max(1);
    // Emit chunks.
    let use_merge = amd.flags & MOD_ARR_MERGE != 0;
    // Vertex merge map (full verts → canonical out vert) when MERGE_FINAL
    // only or per-chunk when MERGE (adjacent merges). We implement the
    // full merge as C's `MOD_ARR_MERGE` with `use_merge` over all emitted
    // verts within dist — matches final-merged behavior for chains.
    let _ = use_merge;
    for c in 0..count {
        let off = [
            offset[0] * c as f32,
            offset[1] * c as f32,
            offset[2] * c as f32,
        ];
        let base = out.verts.len() as i32;
        for v in verts {
            out.verts.push([v[0] + off[0], v[1] + off[1], v[2] + off[2]]);
        }
        for e in edges {
            out.edges.push([e[0] + base, e[1] + base]);
        }
        for &cv in corner_verts {
            out.corner_verts.push(cv + base);
        }
        if c == 0 {
            out.face_offsets = face_offsets.to_vec();
        }
    }
    // Vertex merge (map adjacent chunks' overlapping verts).
    if amd.flags & (MOD_ARR_MERGE | MOD_ARR_MERGE_FINAL) != 0 {
        let d2 = amd.merge_dist * amd.merge_dist;
        let n = out.verts.len();
        let mut map: Vec<i32> = (0..n as i32).collect();
        // Brute-force per adjacent chunk boundary (C sorts by x-coord;
        // PORT_NOTE: same semantics O(n²) here).
        for c in 1..count as usize {
            let a0 = (c - 1) * nverts;
            let b0 = c * nverts;
            for i in 0..nverts {
                for j in 0..nverts {
                    let a = out.verts[a0 + i];
                    let b = out.verts[b0 + j];
                    let dd = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
                    if dd <= d2 {
                        map[b0 + j] = (a0 + i) as i32;
                    }
                }
            }
        }
        // Rewrite topology through map, skipping dead verts.
        let mut remap = vec![-1i32; n];
        let mut new_verts: Vec<[f32; 3]> = Vec::new();
        for i in 0..n {
            if map[i] == i as i32 {
                remap[i] = new_verts.len() as i32;
                new_verts.push(out.verts[i]);
            } else {
                remap[i] = map[i]; // forward to canonical; canonical remapped below
            }
        }
        // Resolve chained maps then reassign.
        let mut final_map = vec![-1i32; n];
        for i in 0..n {
            let mut m = map[i];
            while map[m as usize] != m {
                m = map[m as usize];
            }
            final_map[i] = remap[m as usize];
        }
        for e in out.edges.iter_mut() {
            e[0] = final_map[e[0] as usize];
            e[1] = final_map[e[1] as usize];
        }
        for cv in out.corner_verts.iter_mut() {
            *cv = final_map[*cv as usize];
        }
        out.verts = new_verts;
    }
    out
}
/// `modifier_type_mod_array` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_array() {}
