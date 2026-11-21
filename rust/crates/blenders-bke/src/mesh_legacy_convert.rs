//! Port of `intern/mesh_legacy_convert.cc` — legacy mesh-data
//! conversion kernels: `EdgeSort` edge-synthesis sort,
//! `mesh_calc_edges_mdata`, `BKE_mesh_strip_loose_faces`,
//! `BKE_mesh_mface_index_validate`, `mesh_tessface_calc`
//! (mpoly→mface tessellation), and the flag→generic-attribute
//! name tables. `CustomData_*`/`AttributeAccessor` plumbing is
//! injected — attribute layers are owned by `mesh`/`attributes`
//! which call these kernels with concrete spans.
//! `PORT_PENDING`: bm_corners_to_loops/mfaces→mpolys bmesh
//! conversion and `*_to_generic` versions that need live
//! `CustomData`/`AttributeWriter` infrastructure.

use std::collections::HashMap;

use blenders_dna::meshdata_types::{MEdge, MFace, MLoop, MPoly};

/// `EdgeSort` — edge key used by `mesh_calc_edges_mdata`.
#[derive(Clone, Copy, Default)]
pub struct EdgeSort {
    /// `v1`.
    pub v1: u32,
    /// `v2`.
    pub v2: u32,
    /// `is_loose`.
    pub is_loose: u8,
    /// `is_draw`.
    pub is_draw: i8,
}

/// `to_edgesort`.
pub fn to_edgesort(ed: &mut EdgeSort, v1: u32, v2: u32, is_loose: u8, is_draw: i8) {
    if v1 < v2 {
        ed.v1 = v1;
        ed.v2 = v2;
    } else {
        ed.v1 = v2;
        ed.v2 = v1;
    }
    ed.is_loose = is_loose;
    ed.is_draw = is_draw;
}

/// `vergedgesort`.
fn vergedgesort(a: &EdgeSort, b: &EdgeSort) -> std::cmp::Ordering {
    a.v1.cmp(&b.v1).then(a.v2.cmp(&b.v2))
}

/// `ME_V1V2` / `ME_V2V3` / `ME_V3V4` / `ME_V4V1` / `ME_V3V1`
/// legacy draw-code bits and `ME_SMOOTH`.
pub const ME_V1V2: i8 = 1 << 0;
/// `ME_V2V3`.
pub const ME_V2V3: i8 = 1 << 1;
/// `ME_V3V4`.
pub const ME_V3V4: i8 = 1 << 2;
/// `ME_V4V1`.
pub const ME_V4V1: i8 = 1 << 3;
/// `ME_V3V1`.
pub const ME_V3V1: i8 = 1 << 4;
/// `ME_SMOOTH`.
pub const ME_SMOOTH: i8 = 1 << 0;

/// `mesh_calc_edges_mdata` — synthesize the edge array from
/// `allface`/`allpoly`/`allloop`; returns the deduplicated
/// `MEdge` table and assigns `ml->e` via the loop.
pub fn mesh_calc_edges_mdata(
    allface: &[MFace],
    allloop: &mut [MLoop],
    allpoly: &[MPoly],
) -> Vec<MEdge> {
    let mut totedge = 0usize;
    for mface in allface {
        if mface.v4 != 0 {
            totedge += 4;
        } else if mface.v3 != 0 {
            totedge += 3;
        } else {
            totedge += 1;
        }
    }
    if totedge == 0 {
        return Vec::new();
    }

    let mut edsort: Vec<EdgeSort> = Vec::with_capacity(totedge);
    for mface in allface {
        let mut ed = EdgeSort::default();
        to_edgesort(&mut ed, mface.v1, mface.v2, (mface.v3 == 0) as u8, mface.edcode & ME_V1V2);
        edsort.push(ed);
        if mface.v4 != 0 {
            let mut e2 = EdgeSort::default();
            let mut e3 = EdgeSort::default();
            let mut e4 = EdgeSort::default();
            to_edgesort(&mut e2, mface.v2, mface.v3, 0, mface.edcode & ME_V2V3);
            to_edgesort(&mut e3, mface.v3, mface.v4, 0, mface.edcode & ME_V3V4);
            to_edgesort(&mut e4, mface.v4, mface.v1, 0, mface.edcode & ME_V4V1);
            edsort.extend_from_slice(&[e2, e3, e4]);
        } else if mface.v3 != 0 {
            let mut e2 = EdgeSort::default();
            let mut e3 = EdgeSort::default();
            to_edgesort(&mut e2, mface.v2, mface.v3, 0, mface.edcode & ME_V2V3);
            to_edgesort(&mut e3, mface.v3, mface.v1, 0, mface.edcode & ME_V3V1);
            edsort.extend_from_slice(&[e2, e3]);
        }
    }
    edsort.sort_by(vergedgesort);

    let mut totedge_final = 0usize;
    for a in 0..totedge.saturating_sub(1) {
        if edsort[a].v1 != edsort[a + 1].v1 || edsort[a].v2 != edsort[a + 1].v2 {
            totedge_final += 1;
        } else {
            /* Equal edge, merge the draw-flag. */
            edsort[a + 1].is_draw |= edsort[a].is_draw;
        }
    }
    totedge_final += 1;

    let mut edges: Vec<MEdge> = Vec::with_capacity(totedge_final);
    for a in 0..totedge - 1 {
        if edsort[a].v1 != edsort[a + 1].v1 || edsort[a].v2 != edsort[a + 1].v2 {
            let mut e = MEdge::default();
            e.v1 = edsort[a].v1;
            e.v2 = edsort[a].v2;
            /* Order is swapped so extruding this edge as a surface
             * won't flip face normals with cyclic curves. */
            if e.v1 + 1 != e.v2 {
                std::mem::swap(&mut e.v1, &mut e.v2);
            }
            edges.push(e);
        }
    }
    let mut e = MEdge::default();
    e.v1 = edsort[totedge - 1].v1;
    e.v2 = edsort[totedge - 1].v2;
    if e.v1 + 1 != e.v2 {
        std::mem::swap(&mut e.v1, &mut e.v2);
    }
    edges.push(e);

    /* Set edge members of mloops. */
    let mut hash: HashMap<(u32, u32), u32> = HashMap::with_capacity(edges.len());
    for (i, edge) in edges.iter().enumerate() {
        hash.insert((edge.v1, edge.v2), i as u32);
    }
    for mpoly in allpoly {
        let start = mpoly.loopstart as usize;
        let n = mpoly.totloop as usize;
        for i in 0..n {
            let cur = start + i;
            let next = if i + 1 < n { cur + 1 } else { start };
            let key = (allloop[cur].v.min(allloop[next].v), allloop[cur].v.max(allloop[next].v));
            if let Some(&e) = hash.get(&key) {
                allloop[cur].e = e;
            }
        }
    }
    edges
}

/// `BKE_mesh_strip_loose_faces` — drop `v3 == 0` faces
/// (CD-copy callback injected; pass `Some` to mirror
/// `CustomData_copy_data` for surviving indices).
pub fn mesh_strip_loose_faces(
    mfaces: &mut Vec<MFace>,
    mut copy_data: Option<&mut dyn FnMut(usize, usize)>,
) {
    let mut b = 0usize;
    for a in 0..mfaces.len() {
        if mfaces[a].v3 != 0 {
            if a != b {
                mfaces[b] = mfaces[a];
                if let Some(cb) = copy_data.as_mut() {
                    cb(a, b);
                }
            }
            b += 1;
        }
    }
    mfaces.truncate(b);
}

/// `BKE_mesh_mface_index_validate` — normalize degenerate vertex
/// indices; `swap_corners` mirrors `CustomData_swap_corners`.
pub fn mesh_mface_index_validate(
    mface: &mut MFace,
    mut nr: i32,
    swap_corners: Option<&mut dyn FnMut([i32; 4])>,
) -> i32 {
    /* First test if the face is legal. */
    if (mface.v3 != 0 || nr == 4) && mface.v3 == mface.v4 {
        mface.v4 = 0;
        nr -= 1;
    }
    if (mface.v2 != 0 || mface.v4 != 0) && mface.v2 == mface.v3 {
        mface.v3 = mface.v4;
        mface.v4 = 0;
        nr -= 1;
    }
    if mface.v1 == mface.v2 {
        mface.v2 = mface.v3;
        mface.v3 = mface.v4;
        mface.v4 = 0;
        nr -= 1;
    }

    /* Check corrupt cases, bow-tie geometry. */
    if nr == 3 {
        if mface.v1 == mface.v2 || mface.v2 == mface.v3 || mface.v3 == mface.v1 {
            return 0;
        }
    } else if nr == 4 {
        if mface.v1 == mface.v2
            || mface.v2 == mface.v3
            || mface.v3 == mface.v4
            || mface.v4 == mface.v1
            || mface.v1 == mface.v3
            || mface.v2 == mface.v4
        {
            return 0;
        }
    }

    /* Prevent a zero at wrong index location. */
    if nr == 3 {
        if mface.v3 == 0 {
            std::mem::swap(&mut mface.v1, &mut mface.v2);
            std::mem::swap(&mut mface.v2, &mut mface.v3);
            if let Some(cb) = swap_corners {
                cb([1, 2, 0, 3]);
            }
        }
    } else if nr == 4 {
        if mface.v3 == 0 || mface.v4 == 0 {
            std::mem::swap(&mut mface.v1, &mut mface.v3);
            std::mem::swap(&mut mface.v2, &mut mface.v4);
            if let Some(cb) = swap_corners {
                cb([2, 3, 0, 1]);
            }
        }
    }
    nr
}

/// `TESSFACE_IS_QUAD` — `edcode` bit tagging quad mfaces.
pub const TESSFACE_IS_QUAD: i8 = 1;

/// Output of `mesh_tessface_calc`.
pub struct TessfaceResult {
    /// `mface`.
    pub mface: Vec<MFace>,
    /// `mface_to_poly_map`.
    pub mface_to_poly_map: Vec<i32>,
    /// `lindices`.
    pub lindices: Vec<[u32; 4]>,
}

/// `poly_to_tri_count` — ngon → tri count across `faces`.
pub fn poly_to_tri_count(faces: &[(u32, u32)]) -> usize {
    faces
        .iter()
        .map(|&(_, tot)| {
            if tot <= 4 {
                1usize
            } else {
                (tot - 2) as usize
            }
        })
        .sum()
}

/// `mesh_tessface_calc` — fan/quad tessellation of each `mpoly`
/// into `MFace`s. `faces` = `(loopstart, totloop)` spans; ngons
/// (>4) go through the injected `triangulate` callback returning
/// corner triples (`tri_indices`).
pub fn mesh_tessface_calc(
    faces: &[(u32, u32)],
    corner_verts: &[u32],
    material_indices: &[i32],
    sharp_faces: &[bool],
    triangulate: &mut dyn FnMut(usize, usize) -> Vec<[u32; 3]>,
) -> TessfaceResult {
    let mut out = TessfaceResult {
        mface: Vec::new(),
        mface_to_poly_map: Vec::new(),
        lindices: Vec::new(),
    };
    for (poly_index, &(loopstart, totloop)) in faces.iter().enumerate() {
        if totloop < 3 {
            continue;
        }
        let mat = material_indices.get(poly_index).copied().unwrap_or(0);
        let smooth = if sharp_faces.get(poly_index).copied().unwrap_or(false) {
            0
        } else {
            ME_SMOOTH
        };
        let emit = |out: &mut TessfaceResult, i1: u32, i2: u32, i3: u32, i4: u32| {
            out.mface_to_poly_map.push(poly_index as i32);
            let (l1, l2, l3, l4) = (
                loopstart + i1,
                loopstart + i2,
                loopstart + i3,
                loopstart + i4,
            );
            out.mface.push(MFace {
                v1: corner_verts[l1 as usize],
                v2: corner_verts[l2 as usize],
                v3: corner_verts[l3 as usize],
                v4: if i4 != u32::MAX {
                    corner_verts[l4 as usize]
                } else {
                    0
                },
                mat_nr: mat as i16,
                flag: smooth,
                edcode: if i4 != u32::MAX { TESSFACE_IS_QUAD } else { 0 },
            });
            out.lindices.push([
                l1,
                l2,
                l3,
                if i4 != u32::MAX { l4 } else { 0 },
            ]);
        };
        if totloop == 3 {
            emit(&mut out, 0, 1, 2, u32::MAX);
        } else if totloop == 4 {
            emit(&mut out, 0, 1, 2, 3);
        } else {
            for tri in triangulate(poly_index, loopstart as usize) {
                emit(&mut out, tri[0], tri[1], tri[2], u32::MAX);
            }
        }
    }
    out
}

/// `BKE_mesh_legacy_sharp_faces_from_flags` — collect face
/// indices where `ME_SHARP` is set (attribute write injected).
pub const ME_SHARP: i8 = 1 << 3;
/// `BKE_mesh_legacy_sharp_edges_from_flags` flag bit.
pub const ME_SEAM: i8 = 1 << 2;
/// `ME_HIDE` face flag.
pub const ME_HIDE: i8 = 1 << 4;
/// `ME_FACE_SEL`.
pub const ME_FACE_SEL: i8 = 1 << 1;
/// `BKE_mesh_legacy_uv_seam_from_flags` edge flag.
pub const ME_SEAM_EDGE: i8 = ME_SEAM;

/// Generic attribute names the `*_to_generic` versions write.
/// `material_index` / `sharp_face` / `hide_face` / `hide_edge` /
/// `uv_seam` / `crease_edge` / `bevel_weight_edge` /
/// `bevel_weight_vert` / `.select_vert`/`.select_edge`/
/// `.select_poly` — verbatim attribute names used by
/// `mesh_legacy_convert.cc`.
pub const ATTR_MATERIAL_INDEX: &str = "material_index";
/// `sharp_face`.
pub const ATTR_SHARP_FACE: &str = "sharp_face";
/// `sharp_edge`.
pub const ATTR_SHARP_EDGE: &str = "sharp_edge";
/// `hide_face`.
pub const ATTR_HIDE_FACE: &str = "hide_face";
/// `hide_edge`.
pub const ATTR_HIDE_EDGE: &str = "hide_edge";
/// `hide_vert`.
pub const ATTR_HIDE_VERT: &str = "hide_vert";
/// `uv_seam`.
pub const ATTR_UV_SEAM: &str = "uv_seam";
/// `crease_edge`.
pub const ATTR_CREASE_EDGE: &str = "crease_edge";
/// `bevel_weight_edge`.
pub const ATTR_BEVEL_WEIGHT_EDGE: &str = "bevel_weight_edge";
/// `bevel_weight_vert`.
pub const ATTR_BEVEL_WEIGHT_VERT: &str = "bevel_weight_vert";
/// `.select_vert`.
pub const ATTR_SELECT_VERT: &str = ".select_vert";
/// `.select_edge`.
pub const ATTR_SELECT_EDGE: &str = ".select_edge";
/// `.select_poly`.
pub const ATTR_SELECT_POLY: &str = ".select_poly";
/// `position` — `BKE_mesh_legacy_convert_verts_to_positions`.
pub const ATTR_POSITION: &str = "position";

/// `BKE_mesh_legacy_convert_flags_to_hide_layers` — per-element
/// flag→bool attribute mapping table (verbatim mapping).
pub const FLAG_HIDE_MAPPINGS: [(&str, i8); 3] = [
    (ATTR_HIDE_VERT, ME_HIDE),
    (ATTR_HIDE_EDGE, ME_HIDE),
    (ATTR_HIDE_FACE, ME_HIDE),
];
