//! Port of `mesh_boolean.cc` (`WITH_GMP` build) — exact mesh booleans built on
//! `mesh_intersect`: patches, cells (partition of 3-space), winding-number
//! propagation, and extraction of the boolean result.
//!
//! `const Vert *` -> usize vert index in `IMeshArena`; `Face *` -> usize face
//! index. Parallel sections serial per PORTING.md. `BoolOpType` lives in
//! `BLI_mesh_boolean.hh` -> `BoolOpType` below.

use crate::mesh_intersect::{
    bbs_might_intersect, BoundingBox, Face, IMesh, IMeshArena, NO_INDEX,
};
use blenders_math::boolean::orient3d_mpq;
use blenders_math::mpq::{mpq, mpq_abs, mpq_i, mpq_to_f64, Mpq, Mpq3};
use num_traits::{Signed, Zero};
use std::collections::{HashMap, HashSet};
const NIL: usize = usize::MAX;
const EXTRA_TRI_INDEX: i32 = i32::MAX;

/// `BoolOpType` (`BLI_mesh_boolean.hh`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoolOpType {
    Difference,
    Union,
    Intersect,
}

/// `Edge` — canonical vert-index pair (lower vert id first).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Edge {
    v: [usize; 2],
}
impl Edge {
    fn new(arena: &IMeshArena, v0: usize, v1: usize) -> Self {
        if arena.verts[v0].id <= arena.verts[v1].id {
            Edge { v: [v0, v1] }
        } else {
            Edge { v: [v1, v0] }
        }
    }
    fn new_null() -> Self {
        Edge { v: [NIL, NIL] }
    }
    fn is_null(&self) -> bool {
        self.v[0] == NIL
    }
    fn v0(&self) -> usize {
        self.v[0]
    }
    fn v1(&self) -> usize {
        self.v[1]
    }
}

/// `TriMeshTopology` — edge -> triangles, vert -> incident edges.
struct TriMeshTopology {
    edge_tri: HashMap<Edge, Vec<i32>>,
    vert_edges: HashMap<usize, Vec<Edge>>,
}
impl TriMeshTopology {
    fn new(arena: &IMeshArena, tm: &IMesh) -> Self {
        let mut edge_tri: HashMap<Edge, Vec<i32>> = HashMap::with_capacity(2 * tm.face_size());
        let mut vert_edges: HashMap<usize, Vec<Edge>> =
            HashMap::with_capacity(tm.face_size());
        for t in 0..tm.face_size() {
            let tri = &arena.faces[tm.face[t]];
            for i in 0..3 {
                let v = tri.vert[i];
                let vnext = tri.vert[(i + 1) % 3];
                let e = Edge::new(arena, v, vnext);
                let edges = vert_edges.entry(v).or_default();
                if !edges.contains(&e) {
                    edges.push(e);
                }
                let tris = edge_tri.entry(e).or_default();
                if !tris.contains(&(t as i32)) {
                    tris.push(t as i32);
                }
            }
        }
        TriMeshTopology {
            edge_tri,
            vert_edges,
        }
    }
    /// `other_tri_if_manifold`.
    fn other_tri_if_manifold(&self, e: Edge, t: i32) -> i32 {
        if let Some(tris) = self.edge_tri.get(&e) {
            if tris.len() == 2 {
                return if tris[0] == t { tris[1] } else { tris[0] };
            }
        }
        NO_INDEX
    }
    fn edge_tris(&self, e: Edge) -> Option<&Vec<i32>> {
        self.edge_tri.get(&e)
    }
    fn vert_edges(&self, v: usize) -> &[Edge] {
        self.vert_edges.get(&v).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// `Patch` — maximal set of triangles sharing manifold edges only.
#[derive(Clone, Default)]
struct Patch {
    tri: Vec<i32>,
    cell_above: i32,
    cell_below: i32,
    component: i32,
}
impl Patch {
    fn add_tri(&mut self, t: i32) {
        self.tri.push(t);
    }
    fn tot_tri(&self) -> usize {
        self.tri.len()
    }
}

/// `PatchesInfo`.
struct PatchesInfo {
    patch: Vec<Patch>,
    tri_patch: Vec<i32>,
    pp_edge: HashMap<(i32, i32), Edge>,
}
impl PatchesInfo {
    fn new(ntri: usize) -> Self {
        PatchesInfo {
            patch: Vec::new(),
            tri_patch: vec![NO_INDEX; ntri],
            pp_edge: HashMap::with_capacity(100),
        }
    }
    fn tri_patch(&self, t: i32) -> i32 {
        self.tri_patch[t as usize]
    }
    fn add_patch(&mut self) -> i32 {
        self.patch.push(Patch {
            cell_above: NO_INDEX,
            cell_below: NO_INDEX,
            component: NO_INDEX,
            ..Patch::default()
        });
        (self.patch.len() - 1) as i32
    }
    fn grow_patch(&mut self, patch_index: i32, t: i32) {
        self.tri_patch[t as usize] = patch_index;
        self.patch[patch_index as usize].add_tri(t);
    }
    fn tri_is_assigned(&self, t: i32) -> bool {
        self.tri_patch[t as usize] != NO_INDEX
    }
    fn patch(&self, p: i32) -> &Patch {
        &self.patch[p as usize]
    }
    fn patch_mut(&mut self, p: i32) -> &mut Patch {
        &mut self.patch[p as usize]
    }
    fn tot_patch(&self) -> i32 {
        self.patch.len() as i32
    }
    fn add_new_patch_patch_edge(&mut self, p1: i32, p2: i32, e: Edge) {
        self.pp_edge.insert((p1, p2), e);
        self.pp_edge.insert((p2, p1), e);
    }
    fn patch_patch_edge(&self, p1: i32, p2: i32) -> Edge {
        self.pp_edge.get(&(p1, p2)).copied().unwrap_or_else(Edge::new_null)
    }
}

/// `Cell` — a volume of 3-space surrounded by patches.
#[derive(Clone)]
struct Cell {
    patches: HashSet<i32>,
    winding: Vec<i32>,
    merged_to: i32,
    winding_assigned: bool,
    in_output_volume: bool,
    zero_volume: bool,
}
impl Default for Cell {
    fn default() -> Self {
        Cell {
            patches: HashSet::new(),
            winding: Vec::new(),
            merged_to: NO_INDEX,
            winding_assigned: false,
            in_output_volume: false,
            zero_volume: false,
        }
    }
}
impl Cell {
    fn add_patch(&mut self, p: i32) {
        self.patches.insert(p);
        self.zero_volume = false;
    }
    fn patch_other(&self, p: i32) -> i32 {
        if self.patches.len() != 2 {
            return NO_INDEX;
        }
        for &pother in &self.patches {
            if pother != p {
                return pother;
            }
        }
        NO_INDEX
    }
    /// `check_for_zero_volume`.
    fn check_for_zero_volume(&mut self, pinfo: &PatchesInfo, arena: &IMeshArena, mesh: &IMesh) {
        if self.patches.len() == 2 {
            let mut it = self.patches.iter();
            let p1_index = *it.next().unwrap();
            let p2_index = *it.next().unwrap();
            let p1 = pinfo.patch(p1_index);
            let p2 = pinfo.patch(p2_index);
            if p1.tot_tri() == 1 && p2.tot_tri() == 1 {
                if tris_have_same_verts(arena, mesh, p1.tri[0], p2.tri[0]) {
                    self.zero_volume = true;
                }
            }
        }
    }
}

/// `tris_have_same_verts`.
fn tris_have_same_verts(arena: &IMeshArena, mesh: &IMesh, t1: i32, t2: i32) -> bool {
    let v1 = &arena.faces[mesh.face[t1 as usize]].vert;
    let v2 = &arena.faces[mesh.face[t2 as usize]].vert;
    debug_assert_eq!(v1.len(), 3);
    debug_assert_eq!(v2.len(), 3);
    if v1[0] == v2[0] {
        return (v1[1] == v2[1] && v1[2] == v2[2]) || (v1[1] == v2[2] && v1[2] == v2[1]);
    }
    if v1[0] == v2[1] {
        return (v1[1] == v2[0] && v1[2] == v2[2]) || (v1[1] == v2[2] && v1[2] == v2[0]);
    }
    if v1[0] == v2[2] {
        return (v1[1] == v2[0] && v1[2] == v2[1]) || (v1[1] == v2[1] && v1[2] == v2[0]);
    }
    false
}

/// `CellsInfo`.
struct CellsInfo {
    cell: Vec<Cell>,
}
impl CellsInfo {
    fn new() -> Self {
        CellsInfo { cell: Vec::new() }
    }
    fn add_cell(&mut self) -> i32 {
        self.cell.push(Cell::default());
        (self.cell.len() - 1) as i32
    }
    fn cell(&self, c: i32) -> &Cell {
        &self.cell[c as usize]
    }
    fn cell_mut(&mut self, c: i32) -> &mut Cell {
        &mut self.cell[c as usize]
    }
    fn tot_cell(&self) -> i32 {
        self.cell.len() as i32
    }
}

/// `merge_cells`.
fn merge_cells(merge_to: i32, merge_from: i32, cinfo: &mut CellsInfo, pinfo: &mut PatchesInfo) {
    if merge_to == merge_from {
        return;
    }
    let mut final_merge_to = merge_to;
    while cinfo.cell(final_merge_to).merged_to != NO_INDEX {
        final_merge_to = cinfo.cell(final_merge_to).merged_to;
    }
    let patches: Vec<i32> = cinfo.cell(merge_from).patches.iter().copied().collect();
    for cell_p in patches {
        cinfo.cell_mut(final_merge_to).add_patch(cell_p);
        let patch = pinfo.patch_mut(cell_p);
        if patch.cell_above == merge_from {
            patch.cell_above = final_merge_to;
        }
        if patch.cell_below == merge_from {
            patch.cell_below = final_merge_to;
        }
    }
    cinfo.cell_mut(merge_from).merged_to = final_merge_to;
}

/// `find_patches` — partition triangles into manifold-edge-connected patches.
fn find_patches(arena: &IMeshArena, tm: &IMesh, tmtopo: &TriMeshTopology) -> PatchesInfo {
    let ntri = tm.face_size();
    let mut pinfo = PatchesInfo::new(ntri);
    /* t_others[t][i] = other tri across edge i, if manifold. */
    let mut t_others: Vec<[i32; 3]> = vec![[0; 3]; ntri];
    for t in 0..ntri {
        let tri = &arena.faces[tm.face[t]];
        for i in 0..3 {
            let e = Edge::new(arena, tri.vert[i], tri.vert[(i + 1) % 3]);
            t_others[t][i] = tmtopo.other_tri_if_manifold(e, t as i32);
        }
    }
    let mut cur_patch_grow: Vec<i32> = Vec::new();
    for t in 0..ntri {
        if pinfo.tri_patch(t as i32) == -1 {
            cur_patch_grow.push(t as i32);
            let cur_patch_index = pinfo.add_patch();
            while let Some(tcand) = cur_patch_grow.pop() {
                if pinfo.tri_is_assigned(tcand) {
                    continue;
                }
                pinfo.grow_patch(cur_patch_index, tcand);
                let tri = &arena.faces[tm.face[tcand as usize]];
                for i in 0..3 {
                    let e = Edge::new(arena, tri.vert[i], tri.vert[(i + 1) % 3]);
                    let t_other = t_others[tcand as usize][i];
                    if t_other != NO_INDEX {
                        if !pinfo.tri_is_assigned(t_other) {
                            cur_patch_grow.push(t_other);
                        }
                    } else {
                        /* Non-manifold edge: record patch-patch incidences. */
                        if let Some(etris) = tmtopo.edge_tris(e) {
                            for &tt in etris {
                                if tt != tcand && pinfo.tri_is_assigned(tt) {
                                    let p_other = pinfo.tri_patch(tt);
                                    if p_other == cur_patch_index {
                                        continue;
                                    }
                                    if pinfo
                                        .patch_patch_edge(cur_patch_index, p_other)
                                        .is_null()
                                    {
                                        pinfo.add_new_patch_patch_edge(
                                            cur_patch_index,
                                            p_other,
                                            e,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pinfo
}

/// `find_flap_vert` — vert of tri not on edge e; r_rev = e reversed in tri.
fn find_flap_vert(tri: &Face, e: Edge, r_rev: &mut bool) -> Option<usize> {
    *r_rev = false;
    let flapv;
    if tri.vert[0] == e.v0() {
        if tri.vert[1] == e.v1() {
            flapv = tri.vert[2];
        } else {
            if tri.vert[2] != e.v1() {
                return None;
            }
            *r_rev = true;
            flapv = tri.vert[1];
        }
    } else if tri.vert[1] == e.v0() {
        if tri.vert[2] == e.v1() {
            flapv = tri.vert[0];
        } else {
            if tri.vert[0] != e.v1() {
                return None;
            }
            *r_rev = true;
            flapv = tri.vert[2];
        }
    } else {
        if tri.vert[2] != e.v0() {
            return None;
        }
        if tri.vert[0] == e.v1() {
            flapv = tri.vert[1];
        } else {
            if tri.vert[1] != e.v1() {
                return None;
            }
            *r_rev = true;
            flapv = tri.vert[0];
        }
    }
    Some(flapv)
}

/// `sort_tris_class` — classify tri vs tri0 around shared edge e.
fn sort_tris_class(
    arena: &IMeshArena,
    tri: &Face,
    tri0: &Face,
    e: Edge,
) -> i32 {
    let a0 = &arena.verts[tri0.vert[0]].co_exact;
    let a1 = &arena.verts[tri0.vert[1]].co_exact;
    let a2 = &arena.verts[tri0.vert[2]].co_exact;
    let mut rev = false;
    let mut rev0 = false;
    let flapv0 = find_flap_vert(tri0, e, &mut rev0);
    let flapv = find_flap_vert(tri, e, &mut rev);
    let flap = &arena.verts[flapv.unwrap()].co_exact;
    /* Positive => flap below oriented plane of (a0,a1,a2). */
    let orient = orient3d_mpq(a0, a1, a2, flap);
    if orient > 0 {
        if rev0 {
            4
        } else {
            3
        }
    } else if orient < 0 {
        if rev0 {
            3
        } else {
            4
        }
    } else if flapv == flapv0 {
        1
    } else {
        2
    }
}

/// `sort_by_signed_triangle_index` — sign by e-orientation for stable order.
fn sort_by_signed_triangle_index(
    g: &mut Vec<i32>,
    e: Edge,
    arena: &IMeshArena,
    tm: &IMesh,
    extra_tri: Option<&Face>,
) {
    let mut signed_g: Vec<i64> = g
        .iter()
        .map(|&gi| {
            let tri = if gi == EXTRA_TRI_INDEX {
                extra_tri.unwrap()
            } else {
                &arena.faces[tm.face[gi as usize]]
            };
            let mut rev = false;
            find_flap_vert(tri, e, &mut rev);
            if rev {
                -(gi as i64)
            } else {
                gi as i64
            }
        })
        .collect();
    signed_g.sort();
    for (i, &s) in signed_g.iter().enumerate() {
        g[i] = s.unsigned_abs() as i32;
    }
}

/// `sort_tris_around_edge` — clockwise sort of edge-sharing triangles.
fn sort_tris_around_edge(
    arena: &IMeshArena,
    tm: &IMesh,
    e: Edge,
    tris: &[i32],
    t0: i32,
    extra_tri: Option<&Face>,
) -> Vec<i32> {
    if tris.is_empty() {
        return Vec::new();
    }
    let mut groups: [Vec<i32>; 4] = [vec![tris[0]], Vec::new(), Vec::new(), Vec::new()];
    let triref = &arena.faces[tm.face[tris[0] as usize]];
    for (i, &t) in tris.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let tri = if t == EXTRA_TRI_INDEX {
            extra_tri.unwrap()
        } else {
            &arena.faces[tm.face[t as usize]]
        };
        let group_num = sort_tris_class(arena, tri, triref, e);
        groups[(group_num - 1) as usize].push(t);
    }
    let [mut g1, mut g2, mut g3, mut g4] = groups;
    if g1.len() > 1 {
        sort_by_signed_triangle_index(&mut g1, e, arena, tm, extra_tri);
    }
    if g2.len() > 1 {
        sort_by_signed_triangle_index(&mut g2, e, arena, tm, extra_tri);
    }
    if g3.len() > 1 {
        g3 = sort_tris_around_edge(arena, tm, e, &g3, t0, extra_tri);
    }
    if g4.len() > 1 {
        g4 = sort_tris_around_edge(arena, tm, e, &g4, t0, extra_tri);
    }
    let mut ans = Vec::with_capacity(g1.len() + g2.len() + g3.len() + g4.len());
    if tris[0] == t0 {
        ans.extend_from_slice(&g1);
        ans.extend_from_slice(&g4);
        ans.extend_from_slice(&g2);
        ans.extend_from_slice(&g3);
    } else {
        ans.extend_from_slice(&g3);
        ans.extend_from_slice(&g1);
        ans.extend_from_slice(&g4);
        ans.extend_from_slice(&g2);
    }
    ans
}

/// `find_cells_from_edge` — build cells around non-manifold edge e.
fn find_cells_from_edge(
    arena: &IMeshArena,
    tm: &IMesh,
    tmtopo: &TriMeshTopology,
    pinfo: &mut PatchesInfo,
    cinfo: &mut CellsInfo,
    e: Edge,
) {
    let edge_tris = tmtopo.edge_tris(e).expect("edge_tris");
    let sorted_tris = sort_tris_around_edge(arena, tm, e, edge_tris, edge_tris[0], None);
    let n_edge_tris = edge_tris.len();
    let mut edge_patches = vec![0i32; n_edge_tris];
    for i in 0..n_edge_tris {
        edge_patches[i] = pinfo.tri_patch(sorted_tris[i]);
    }
    for i in 0..n_edge_tris {
        let inext = (i + 1) % n_edge_tris;
        let r_index = edge_patches[i];
        let rnext_index = edge_patches[inext];
        let mut r_flipped = false;
        let mut rnext_flipped = false;
        find_flap_vert(&arena.faces[tm.face[sorted_tris[i] as usize]], e, &mut r_flipped);
        find_flap_vert(
            &arena.faces[tm.face[sorted_tris[inext] as usize]],
            e,
            &mut rnext_flipped,
        );
        let r_follow_cell = {
            let r = pinfo.patch(r_index);
            if r_flipped {
                r.cell_below
            } else {
                r.cell_above
            }
        };
        let rnext_prev_cell = {
            let rnext = pinfo.patch(rnext_index);
            if rnext_flipped {
                rnext.cell_above
            } else {
                rnext.cell_below
            }
        };
        if r_follow_cell == NO_INDEX && rnext_prev_cell == NO_INDEX {
            let c = cinfo.add_cell();
            {
                let r = pinfo.patch_mut(r_index);
                if r_flipped {
                    r.cell_below = c;
                } else {
                    r.cell_above = c;
                }
            }
            {
                let rnext = pinfo.patch_mut(rnext_index);
                if rnext_flipped {
                    rnext.cell_above = c;
                } else {
                    rnext.cell_below = c;
                }
            }
            cinfo.cell_mut(c).add_patch(r_index);
            cinfo.cell_mut(c).add_patch(rnext_index);
            cinfo.cell_mut(c).check_for_zero_volume(pinfo, arena, tm);
        } else if r_follow_cell != NO_INDEX && rnext_prev_cell == NO_INDEX {
            let c = r_follow_cell;
            {
                let rnext = pinfo.patch_mut(rnext_index);
                if rnext_flipped {
                    rnext.cell_above = c;
                } else {
                    rnext.cell_below = c;
                }
            }
            cinfo.cell_mut(c).add_patch(rnext_index);
            cinfo.cell_mut(c).check_for_zero_volume(pinfo, arena, tm);
        } else if r_follow_cell == NO_INDEX && rnext_prev_cell != NO_INDEX {
            let c = rnext_prev_cell;
            {
                let r = pinfo.patch_mut(r_index);
                if r_flipped {
                    r.cell_below = c;
                } else {
                    r.cell_above = c;
                }
            }
            cinfo.cell_mut(c).add_patch(r_index);
            cinfo.cell_mut(c).check_for_zero_volume(pinfo, arena, tm);
        } else if r_follow_cell != rnext_prev_cell {
            let follow_n = cinfo.cell(r_follow_cell).patches.len();
            let prev_n = cinfo.cell(rnext_prev_cell).patches.len();
            if follow_n >= prev_n {
                merge_cells(r_follow_cell, rnext_prev_cell, cinfo, pinfo);
            } else {
                merge_cells(rnext_prev_cell, r_follow_cell, cinfo, pinfo);
            }
        }
    }
}

/// `find_cells` — partition 3-space into cells.
fn find_cells(arena: &IMeshArena, tm: &IMesh, tmtopo: &TriMeshTopology, pinfo: &mut PatchesInfo) -> CellsInfo {
    let mut cinfo = CellsInfo::new();
    let mut processed_edges: HashSet<Edge> = HashSet::new();
    /* Collect (p,q) pairs with p<q first — HashMap iteration order is arbitrary,
     * but each edge is only processed once via processed_edges anyway. */
    let mut items: Vec<((i32, i32), Edge)> = pinfo.pp_edge.iter().map(|(k,&v)|(*k,v)).collect();
    items.sort_by_key(|((p, q), _)| (*p, *q));
    for ((p, q), e) in items {
        if p < q && !processed_edges.contains(&e) {
            processed_edges.insert(e);
            find_cells_from_edge(arena, tm, tmtopo, pinfo, &mut cinfo, e);
        }
    }
    /* Patches with no cells (closed/open manifold self-incident): make cells. */
    for p in 0..pinfo.tot_patch() {
        if pinfo.patch(p).cell_above == NO_INDEX {
            let c = cinfo.add_cell();
            pinfo.patch_mut(p).cell_above = c;
            cinfo.cell_mut(c).add_patch(p);
        }
        if pinfo.patch(p).cell_below == NO_INDEX {
            let c = cinfo.add_cell();
            pinfo.patch_mut(p).cell_below = c;
            cinfo.cell_mut(c).add_patch(p);
        }
    }
    cinfo
}

/// `find_patch_components` — connected patch sets via cells.
fn find_patch_components(cinfo: &CellsInfo, pinfo: &mut PatchesInfo) -> Vec<Vec<i32>> {
    if pinfo.tot_patch() == 0 {
        return Vec::new();
    }
    let mut current_component = 0usize;
    let mut cell_processed = vec![false; cinfo.tot_cell() as usize];
    let mut ans: Vec<Vec<i32>> = Vec::new();
    for pstart in 0..pinfo.tot_patch() {
        if pinfo.patch(pstart).component != NO_INDEX {
            continue;
        }
        ans.push(vec![pstart]);
        let mut stack = vec![pstart];
        pinfo.patch_mut(pstart).component = current_component as i32;
        while let Some(p) = stack.pop() {
            let (ca, cb) = {
                let patch = pinfo.patch(p);
                (patch.cell_above, patch.cell_below)
            };
            for c in [ca, cb] {
                if c < 0 || cell_processed[c as usize] {
                    continue;
                }
                cell_processed[c as usize] = true;
                let pns: Vec<i32> = cinfo.cell(c).patches.iter().copied().collect();
                for pn in pns {
                    if pinfo.patch(pn).component == NO_INDEX {
                        pinfo.patch_mut(pn).component = current_component as i32;
                        stack.push(pn);
                        ans[current_component].push(pn);
                    }
                }
            }
        }
        current_component += 1;
    }
    ans
}

/// `patch_cell_graph_ok` — debug validation.
fn patch_cell_graph_ok(cinfo: &CellsInfo, pinfo: &PatchesInfo) -> bool {
    for c in 0..cinfo.tot_cell() {
        let cell = cinfo.cell(c);
        if cell.merged_to != NO_INDEX {
            continue;
        }
        if cell.patches.is_empty() {
            return false;
        }
        for &p in &cell.patches {
            if p >= pinfo.tot_patch() {
                return false;
            }
        }
    }
    for p in 0..pinfo.tot_patch() {
        let patch = pinfo.patch(p);
        if patch.cell_above == NO_INDEX || patch.cell_below == NO_INDEX {
            return false;
        }
        if patch.cell_above >= cinfo.tot_cell() || patch.cell_below >= cinfo.tot_cell() {
            return false;
        }
    }
    true
}

/// `is_pwn` — piece-wise-constant-winding-number check (approximate).
fn is_pwn(arena: &IMeshArena, tm: &IMesh, tmtopo: &TriMeshTopology) -> bool {
    for (edge, tris) in &tmtopo.edge_tri {
        let mut tot_orient = 0i32;
        for &t in tris {
            let face = &arena.faces[tm.face[t as usize]];
            for i in 0..3 {
                if face.vert[i] == edge.v0() {
                    if face.vert[(i + 1) % 3] == edge.v1() {
                        tot_orient += 1;
                    } else {
                        debug_assert_eq!(face.vert[(i + 3 - 1) % 3], edge.v1());
                        tot_orient -= 1;
                    }
                }
            }
        }
        if tot_orient != 0 {
            return false;
        }
    }
    true
}

/// `find_cell_for_point_near_edge` — which cell holds point p near edge e.
fn find_cell_for_point_near_edge(
    p: &Mpq3,
    e: Edge,
    arena: &mut IMeshArena,
    tm: &IMesh,
    tmtopo: &TriMeshTopology,
    pinfo: &PatchesInfo,
) -> i32 {
    let etris = tmtopo.edge_tris(e).expect("edge_tris");
    let dummy_vert = arena.add_or_find_vert_mpq(p, NO_INDEX);
    let dummy_fi = arena.add_face_full(
        vec![e.v0(), e.v1(), dummy_vert],
        NO_INDEX,
        vec![NO_INDEX; 3],
        vec![false; 3],
    );
    let mut edge_tris: Vec<i32> = etris.clone();
    edge_tris.push(EXTRA_TRI_INDEX);
    let dummy_face = arena.faces[dummy_fi].clone();
    let sorted_tris =
        sort_tris_around_edge(arena, tm, e, &edge_tris, edge_tris[0], Some(&dummy_face));
    let dummy_index = sorted_tris
        .iter()
        .position(|&t| t == EXTRA_TRI_INDEX)
        .expect("dummy in sorted");
    let prev_tri = if dummy_index == 0 {
        sorted_tris[sorted_tris.len() - 1]
    } else {
        sorted_tris[dummy_index - 1]
    };
    let prev_patch = pinfo.patch(pinfo.tri_patch(prev_tri));
    let mut prev_flipped = false;
    find_flap_vert(
        &arena.faces[tm.face[prev_tri as usize]],
        e,
        &mut prev_flipped,
    );
    if prev_flipped {
        prev_patch.cell_below
    } else {
        prev_patch.cell_above
    }
}

/// `find_ambient_cell` — the outside cell (optionally within component_patches).
fn find_ambient_cell(
    arena: &mut IMeshArena,
    tm: &IMesh,
    component_patches: Option<&Vec<i32>>,
    tmtopo: &TriMeshTopology,
    pinfo: &PatchesInfo,
) -> i32 {
    /* Vertex with maximum x value. */
    let mut v_extreme: usize = match component_patches {
        None => arena.faces[tm.face[0]].vert[0],
        Some(cp) => arena.faces[tm.face[pinfo.patch(cp[0]).tri[0] as usize]].vert[0],
    };
    let extreme_x = |v: usize, arena: &IMeshArena| -> Mpq {
        arena.verts[v].co_exact.x.clone()
    };
    match component_patches {
        None => {
            for &f in &tm.face {
                for &v in &arena.faces[f].vert {
                    if extreme_x(v, arena) > extreme_x(v_extreme, arena) {
                        v_extreme = v;
                    }
                }
            }
        }
        Some(cp) => {
            for &p in cp {
                for i in 0..pinfo.patch(p).tot_tri() {
                    let t = pinfo.patch(p).tri[i];
                    for &v in &arena.faces[tm.face[t as usize]].vert {
                        if extreme_x(v, arena) > extreme_x(v_extreme, arena) {
                            v_extreme = v;
                        }
                    }
                }
            }
        }
    }
    /* Edge at v_extreme with max absolute slope in XY — on convex hull. */
    let edges = tmtopo.vert_edges(v_extreme).to_vec();
    let ex = &arena.verts[v_extreme].co_exact;
    let mut ehull = Edge::new_null();
    let mut max_abs_slope = mpq(-1.0);
    for e in edges {
        let v_other = if e.v0() == v_extreme { e.v1() } else { e.v0() };
        let co_other = &arena.verts[v_other].co_exact;
        let delta_x = &co_other.x - &ex.x;
        if delta_x.is_zero() {
            ehull = e;
            break;
        }
        let abs_slope = mpq_abs(&((&co_other.y - &ex.y) / &delta_x));
        if abs_slope > max_abs_slope {
            ehull = e;
            max_abs_slope = abs_slope;
        }
    }
    let mut p_in_ambient = ex.clone();
    p_in_ambient.x += mpq(1.0);
    find_cell_for_point_near_edge(&p_in_ambient, ehull, arena, tm, tmtopo, pinfo)
}

/// `find_good_sorting_edge` — hull edge at closestp not colinear with testp.
fn find_good_sorting_edge(
    arena: &IMeshArena,
    testp: usize,
    closestp: usize,
    tmtopo: &TriMeshTopology,
) -> Edge {
    let co_closest = &arena.verts[closestp].co_exact;
    let co_test = &arena.verts[testp].co_exact;
    let abscissa = co_test - co_closest;
    let mut axis = 0;
    while axis < 3 && abscissa[axis].is_zero() {
        axis += 1;
    }
    debug_assert!(axis < 3);
    let axis_next = (axis + 1) % 3;
    let axis_next_next = (axis_next + 1) % 3;
    let mut ordinate = Mpq3::zero();
    ordinate[axis] = abscissa[axis_next].clone();
    ordinate[axis_next] = -abscissa[axis].clone();
    ordinate[axis_next_next] = mpq_i(0);
    let normal = Mpq3::cross(&abscissa, &ordinate);
    let nlen2 = normal.dot(&normal);
    let mut max_abs_slope = mpq(-1.0);
    let mut esort = Edge::new_null();
    for e in tmtopo.vert_edges(closestp).to_vec() {
        let v_other = if e.v0() == closestp { e.v1() } else { e.v0() };
        let co_other = &arena.verts[v_other].co_exact;
        let evec = co_other - co_closest;
        let proj_evec = &evec - &(&normal * &(&evec.dot(&normal) / &nlen2));
        let evec_a = proj_evec.dot(&abscissa);
        let evec_o = proj_evec.dot(&ordinate);
        if evec_a.is_zero() {
            esort = e;
            break;
        }
        let abs_slope = mpq_abs(&(evec_o / evec_a));
        if abs_slope > max_abs_slope {
            esort = e;
            max_abs_slope = abs_slope;
        }
    }
    esort
}

/// `find_containing_cell` — cell containing vert v near triangle t.
fn find_containing_cell(
    v: usize,
    t: i32,
    close_edge: i32,
    close_vert: i32,
    pinfo: &PatchesInfo,
    arena: &mut IMeshArena,
    tm: &IMesh,
    tmtopo: &TriMeshTopology,
) -> i32 {
    let tri = &arena.faces[tm.face[t as usize]];
    let mut etest = Edge::new_null();
    let mut close_edge = close_edge;
    if close_edge == -1 && close_vert == -1 {
        close_edge = 0;
    }
    if close_edge != -1 {
        let v0 = tri.vert[close_edge as usize];
        let v1 = tri.vert[(close_edge as usize + 1) % 3];
        for e in tmtopo.vert_edges(v0).to_vec() {
            if (e.v0() == v0 && e.v1() == v1) || (e.v0() == v1 && e.v1() == v0) {
                etest = e;
                break;
            }
        }
    } else {
        let cv = close_vert as usize;
        let mut vert_cv = tri.vert[cv];
        if vert_cv == v {
            vert_cv = tri.vert[(cv + 1) % 3];
            debug_assert_ne!(vert_cv, v);
        }
        etest = find_good_sorting_edge(arena, v, vert_cv, tmtopo);
    }
    debug_assert!(!etest.is_null());
    let co = arena.verts[v].co_exact.clone();
    find_cell_for_point_near_edge(&co, etest, arena, tm, tmtopo, pinfo)
}

/// `closest_on_tri_to_point` — returns (dist², r_edge, r_vert).
fn closest_on_tri_to_point(
    p: &Mpq3,
    a: &Mpq3,
    b: &Mpq3,
    c: &Mpq3,
) -> (Mpq, i32, i32) {
    let dist2 = |p: &Mpq3, q: &Mpq3| -> Mpq {
        let d = p - q;
        d.dot(&d)
    };
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(&ap);
    let d2 = ac.dot(&ap);
    let zero = mpq_i(0);
    if d1 <= zero && d2 <= zero {
        return (dist2(p, a), -1, 0);
    }
    let bp = p - b;
    let d3 = ab.dot(&bp);
    let d4 = ac.dot(&bp);
    if d3 >= zero && d4 <= d3 {
        return (dist2(p, b), -1, 1);
    }
    let vc = &d1 * &d4 - &d3 * &d2;
    if vc <= zero && d1 >= zero && d3 <= zero {
        let vv = &d1 / &(&d1 - &d3);
        let r = a + &(&ab * &vv);
        return (dist2(p, &r), 0, -1);
    }
    let cp = p - c;
    let d5 = ab.dot(&cp);
    let d6 = ac.dot(&cp);
    if d6 >= zero && d5 <= d6 {
        return (dist2(p, c), -1, 2);
    }
    let vb = &d5 * &d2 - &d1 * &d6;
    if vb <= zero && d2 >= zero && d6 <= zero {
        let w = &d2 / &(&d2 - &d6);
        let r = a + &(&ac * &w);
        return (dist2(p, &r), 2, -1);
    }
    let va = &d3 * &d6 - &d5 * &d4;
    if va <= zero && &d4 - &d3 >= zero && &d5 - &d6 >= zero {
        let w = &(&d4 - &d3) / &(&(&d4 - &d3) + &(&d5 - &d6));
        let r = b + &(&(c - b) * &w);
        return (dist2(p, &r), 1, -1);
    }
    /* Inside face region — barycentric. */
    let denom = mpq_i(1) / &(&va + &vb + &vc);
    let vv = &vb * &denom;
    let w = &vc * &denom;
    let r = &(a + &(&ab * &vv)) + &(&ac * &w);
    (dist2(p, &r), -1, -1)
}

/// `closest_on_tri_to_point_float_dist_squared` — cheap float version.
fn closest_on_tri_to_point_float_dist_squared(
    p: [f32; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> f32 {
    let fa = [a[0] as f32, a[1] as f32, a[2] as f32];
    let fb = [b[0] as f32, b[1] as f32, b[2] as f32];
    let fc = [c[0] as f32, c[1] as f32, c[2] as f32];
    let v = blenders_math::vector::Vec3::new(p[0], p[1], p[2]);
    let closest = blenders_math::geom::closest_on_tri_to_point(
        v,
        blenders_math::vector::Vec3::new(fa[0], fa[1], fa[2]),
        blenders_math::vector::Vec3::new(fb[0], fb[1], fb[2]),
        blenders_math::vector::Vec3::new(fc[0], fc[1], fc[2]),
    ).0;
    (v - closest).dot(v - closest)
}

/// `ComponentContainer`.
struct ComponentContainer {
    containing_component: i32,
    nearest_cell: i32,
    dist_to_cell: Mpq,
}

/// `find_component_containers`.
#[allow(clippy::too_many_arguments)]
fn find_component_containers(
    comp: usize,
    components: &[Vec<i32>],
    ambient_cell: &[i32],
    arena: &mut IMeshArena,
    tm: &IMesh,
    pinfo: &PatchesInfo,
    tmtopo: &TriMeshTopology,
    comp_bb: &[BoundingBox],
) -> Vec<ComponentContainer> {
    let mut ans = Vec::new();
    let test_p = components[comp][0];
    let test_t = pinfo.patch(test_p).tri[0];
    let test_v = arena.faces[tm.face[test_t as usize]].vert[0];
    let test_v_d = arena.verts[test_v].co;
    let test_v_f = [
        test_v_d[0] as f32,
        test_v_d[1] as f32,
        test_v_d[2] as f32,
    ];
    for comp_other in 0..components.len() {
        if comp == comp_other {
            continue;
        }
        if !bbs_might_intersect(&comp_bb[comp], &comp_bb[comp_other]) {
            continue;
        }
        let mut nearest_tri = NO_INDEX;
        let mut nearest_tri_close_vert = -1;
        let mut nearest_tri_close_edge = -1;
        let mut nearest_tri_dist_squared = mpq_i(0);
        let mut nearest_tri_dist_squared_float = f32::MAX;
        for &p in &components[comp_other] {
            for &t in &pinfo.patch(p).tri {
                let tri = &arena.faces[tm.face[t as usize]];
                let d2_f = closest_on_tri_to_point_float_dist_squared(
                    test_v_f,
                    arena.verts[tri.vert[0]].co,
                    arena.verts[tri.vert[1]].co,
                    arena.verts[tri.vert[2]].co,
                );
                if d2_f - f32::EPSILON > nearest_tri_dist_squared_float {
                    continue;
                }
                let (d2, close_edge, close_vert) = closest_on_tri_to_point(
                    &arena.verts[test_v].co_exact,
                    &arena.verts[tri.vert[0]].co_exact,
                    &arena.verts[tri.vert[1]].co_exact,
                    &arena.verts[tri.vert[2]].co_exact,
                );
                if nearest_tri == NO_INDEX || d2 < nearest_tri_dist_squared {
                    nearest_tri = t;
                    nearest_tri_close_edge = close_edge;
                    nearest_tri_close_vert = close_vert;
                    nearest_tri_dist_squared = d2;
                    nearest_tri_dist_squared_float = d2_f;
                }
            }
        }
        let containing_cell = find_containing_cell(
            test_v,
            nearest_tri,
            nearest_tri_close_edge,
            nearest_tri_close_vert,
            pinfo,
            arena,
            tm,
            tmtopo,
        );
        if containing_cell != ambient_cell[comp_other] {
            ans.push(ComponentContainer {
                containing_component: comp_other as i32,
                nearest_cell: containing_cell,
                dist_to_cell: nearest_tri_dist_squared,
            });
        }
    }
    ans
}

/// `populate_comp_bbs` (serial).
fn populate_comp_bbs(
    components: &[Vec<i32>],
    pinfo: &PatchesInfo,
    arena: &IMeshArena,
    im: &IMesh,
    comp_bb: &mut Vec<BoundingBox>,
) {
    let mut max_abs = vec![0.0f64; components.len()];
    for c in 0..components.len() {
        for &p in &components[c] {
            for &t in &pinfo.patch(p).tri {
                for &v in &arena.faces[im.face[t as usize]].vert {
                    let co = arena.verts[v].co;
                    comp_bb[c].combine(co);
                    for i in 0..3 {
                        max_abs[c] = max_abs[c].max(co[i].abs());
                    }
                }
            }
        }
    }
    let all_max_abs = max_abs.iter().copied().fold(0.0, f64::max);
    let pad_factor = 10.0f64;
    let mut pad = if all_max_abs == 0.0 {
        f32::EPSILON as f64
    } else {
        2.0 * f32::EPSILON as f64 * all_max_abs
    };
    pad *= pad_factor;
    for bb in comp_bb.iter_mut() {
        bb.expand(pad);
    }
}
