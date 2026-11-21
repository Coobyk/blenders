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
    /// Knife op (no boolean): `boolean_mesh` returns the intersected mesh.
    None,
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

/* ---- Cell winding methods ------------------------------------------------ */

impl Cell {
    fn init_winding(&mut self, winding_len: usize) {
        self.winding = vec![0; winding_len];
    }
    fn seed_ambient_winding(&mut self) {
        self.winding.iter_mut().for_each(|w| *w = 0);
        self.winding_assigned = true;
    }
    fn set_winding_and_in_output_volume(
        &mut self,
        from_cell: &Cell,
        shape: i32,
        delta: i32,
        op: BoolOpType,
    ) {
        self.winding = from_cell.winding.clone();
        if shape >= 0 {
            self.winding[shape as usize] += delta;
        }
        self.winding_assigned = true;
        self.in_output_volume = apply_bool_op(op, &self.winding);
    }
}
impl CellsInfo {
    fn init_windings(&mut self, winding_len: usize) {
        for cell in &mut self.cell {
            cell.init_winding(winding_len);
        }
    }
}

/// `finish_patch_cell_graph` — connect disconnected components via nesting.
fn finish_patch_cell_graph(
    arena: &mut IMeshArena,
    tm: &IMesh,
    cinfo: &mut CellsInfo,
    pinfo: &mut PatchesInfo,
    tmtopo: &TriMeshTopology,
) {
    let components = find_patch_components(cinfo, pinfo);
    if components.len() <= 1 {
        return;
    }
    let mut ambient_cell = vec![0i32; components.len()];
    for comp in 0..components.len() {
        ambient_cell[comp] =
            find_ambient_cell(arena, tm, Some(&components[comp]), tmtopo, pinfo);
    }
    let tot_components = components.len();
    let mut comp_cont: Vec<Vec<ComponentContainer>> =
        (0..tot_components).map(|_| Vec::new()).collect();
    if tot_components > 1 {
        let mut comp_bb = vec![BoundingBox::new(); tot_components];
        populate_comp_bbs(&components, pinfo, arena, tm, &mut comp_bb);
        for comp in 0..tot_components {
            comp_cont[comp] = find_component_containers(
                comp,
                &components,
                &ambient_cell,
                arena,
                tm,
                pinfo,
                tmtopo,
                &comp_bb,
            );
        }
    }
    /* Nested components: merge their ambient cell with nearest containing cell. */
    let mut outer_components: Vec<usize> = Vec::new();
    for comp in 0..comp_cont.len() {
        if comp_cont[comp].is_empty() {
            outer_components.push(comp);
        } else {
            let mut closest_i = 0usize;
            for i in 1..comp_cont[comp].len() {
                if comp_cont[comp][i].dist_to_cell < comp_cont[comp][closest_i].dist_to_cell {
                    closest_i = i;
                }
            }
            let comp_ambient = ambient_cell[comp];
            let cont_cell = comp_cont[comp][closest_i].nearest_cell;
            merge_cells(cont_cell, comp_ambient, cinfo, pinfo);
        }
    }
    /* Outer components: merge their ambient cells. */
    if outer_components.len() > 1 {
        let merged_ambient = ambient_cell[outer_components[0]];
        for i in 1..outer_components.len() {
            merge_cells(merged_ambient, ambient_cell[outer_components[i]], cinfo, pinfo);
        }
    }
}

/// `propagate_windings_and_in_output_volume`.
fn propagate_windings_and_in_output_volume(
    pinfo: &mut PatchesInfo,
    cinfo: &mut CellsInfo,
    c_ambient: i32,
    op: BoolOpType,
    nshapes: i32,
    shape_fn: &dyn Fn(i32) -> i32,
) {
    cinfo.cell_mut(c_ambient).seed_ambient_winding();
    let mut queue: Vec<i32> = Vec::with_capacity(cinfo.tot_cell() as usize);
    queue.push(c_ambient);
    let mut queue_head = 0usize;
    while queue_head < queue.len() {
        let c = queue[queue_head];
        queue_head += 1;
        let patches: Vec<i32> = cinfo.cell(c).patches.iter().copied().collect();
        for p in patches {
            let (p_above_c, c_neighbor, t) = {
                let patch = pinfo.patch(p);
                let p_above_c = patch.cell_below == c;
                let c_neighbor = if p_above_c {
                    patch.cell_above
                } else {
                    patch.cell_below
                };
                (p_above_c, c_neighbor, patch.tri[0])
            };
            if !cinfo.cell(c_neighbor).winding_assigned {
                let winding_delta = if p_above_c { -1 } else { 1 };
                let shape = shape_fn(t);
                debug_assert!(shape < nshapes);
                let winding = cinfo.cell(c).winding.clone();
                let neighbor = cinfo.cell_mut(c_neighbor);
                neighbor.winding = winding;
                if shape >= 0 {
                    neighbor.winding[shape as usize] += winding_delta;
                }
                neighbor.winding_assigned = true;
                neighbor.in_output_volume = apply_bool_op(op, &neighbor.winding);
                queue.push(c_neighbor);
            }
        }
    }
}

/// `apply_bool_op` — include cell in output volume?
pub fn apply_bool_op(op: BoolOpType, winding: &[i32]) -> bool {
    let nw = winding.len();
    debug_assert!(nw > 0);
    match op {
        BoolOpType::Intersect => winding.iter().all(|&w| w != 0),
        BoolOpType::Union => winding.iter().any(|&w| w != 0),
        BoolOpType::None => false,
        BoolOpType::Difference => {
            if winding[0] == 0 {
                return false;
            }
            if nw == 1 {
                return true;
            }
            for &w in &winding[1..] {
                if w >= 1 {
                    return false;
                }
            }
            true
        }
    }
}

/// `extract_zero_volume_cell_tris`.
fn extract_zero_volume_cell_tris(
    r_tris: &mut Vec<usize>,
    arena: &mut IMeshArena,
    tm_subdivided: &IMesh,
    pinfo: &PatchesInfo,
    cinfo: &CellsInfo,
) {
    let tot_patch = pinfo.tot_patch() as usize;
    let mut adj_to_zv = vec![false; tot_patch];
    for p in 0..tot_patch as i32 {
        let patch = pinfo.patch(p);
        adj_to_zv[p as usize] = cinfo.cell(patch.cell_above).zero_volume
            || cinfo.cell(patch.cell_below).zero_volume;
    }
    let mut allocated_to_stack = vec![false; tot_patch];
    for p in 0..tot_patch as i32 {
        if !adj_to_zv[p as usize] || allocated_to_stack[p as usize] {
            continue;
        }
        let mut stack: Vec<i32> = vec![p];
        let mut flipped: Vec<bool> = vec![false];
        allocated_to_stack[p as usize] = true;
        /* Walk in above direction from p. */
        let mut pwalk = p;
        let mut c = pinfo.patch(pwalk).cell_above;
        while cinfo.cell(c).zero_volume {
            debug_assert_eq!(cinfo.cell(c).patches.len(), 2);
            let pother = cinfo.cell(c).patch_other(pwalk);
            let flip = pinfo.patch(pother).cell_above == c;
            flipped.push(flip);
            stack.push(pother);
            allocated_to_stack[pother as usize] = true;
            pwalk = pother;
            c = if flip {
                pinfo.patch(pwalk).cell_below
            } else {
                pinfo.patch(pwalk).cell_above
            };
        }
        let above_stack_cell = c;
        /* Walk in below direction from p. */
        let mut pwalk = p;
        let mut c = pinfo.patch(pwalk).cell_below;
        while cinfo.cell(c).zero_volume {
            let pother = cinfo.cell(c).patch_other(pwalk);
            let flip = pinfo.patch(pother).cell_below == c;
            flipped.push(flip);
            stack.push(pother);
            allocated_to_stack[pother as usize] = true;
            pwalk = pother;
            c = if flip {
                pinfo.patch(pwalk).cell_above
            } else {
                pinfo.patch(pwalk).cell_below
            };
        }
        let below_stack_cell = c;
        if cinfo.cell(above_stack_cell).in_output_volume
            != cinfo.cell(below_stack_cell).in_output_volume
        {
            let need_flipped_tri = cinfo.cell(above_stack_cell).in_output_volume;
            let mut t_to_add = NO_INDEX;
            for i in 0..stack.len() {
                if flipped[i] == need_flipped_tri {
                    t_to_add = pinfo.patch(stack[i]).tri[0];
                    r_tris.push(tm_subdivided.face[t_to_add as usize]);
                    break;
                }
            }
            if t_to_add == NO_INDEX {
                let f = tm_subdivided.face[pinfo.patch(p).tri[0] as usize];
                let tri = &arena.faces[f];
                let flipped_vs = vec![tri.vert[0], tri.vert[2], tri.vert[1]];
                let flipped_e_origs =
                    vec![tri.edge_orig[2], tri.edge_orig[1], tri.edge_orig[0]];
                let flipped_is_intersect =
                    vec![tri.is_intersect[2], tri.is_intersect[1], tri.is_intersect[0]];
                let orig = tri.orig;
                let flipped_f = arena.add_face_full(
                    flipped_vs,
                    orig,
                    flipped_e_origs,
                    flipped_is_intersect,
                );
                r_tris.push(flipped_f);
            }
        }
    }
}

/// `extract_from_in_output_volume_diffs`.
fn extract_from_in_output_volume_diffs(
    arena: &mut IMeshArena,
    tm_subdivided: &IMesh,
    pinfo: &PatchesInfo,
    cinfo: &CellsInfo,
) -> IMesh {
    let mut out_tris: Vec<usize> = Vec::with_capacity(tm_subdivided.face_size());
    let mut any_zero_volume_cell = false;
    for t in 0..tm_subdivided.face_size() {
        let p = pinfo.tri_patch(t as i32);
        let patch = pinfo.patch(p);
        let cell_above = cinfo.cell(patch.cell_above);
        let cell_below = cinfo.cell(patch.cell_below);
        let adjacent_zero_volume_cell = cell_above.zero_volume || cell_below.zero_volume;
        any_zero_volume_cell |= adjacent_zero_volume_cell;
        if (cell_above.in_output_volume != cell_below.in_output_volume)
            && !adjacent_zero_volume_cell
        {
            let flip = cell_above.in_output_volume;
            let f = tm_subdivided.face[t];
            if flip {
                let tri = &arena.faces[f];
                let flipped_vs = vec![tri.vert[0], tri.vert[2], tri.vert[1]];
                let flipped_e_origs =
                    vec![tri.edge_orig[2], tri.edge_orig[1], tri.edge_orig[0]];
                let flipped_is_intersect =
                    vec![tri.is_intersect[2], tri.is_intersect[1], tri.is_intersect[0]];
                let orig = tri.orig;
                let flipped_f = arena.add_face_full(
                    flipped_vs,
                    orig,
                    flipped_e_origs,
                    flipped_is_intersect,
                );
                out_tris.push(flipped_f);
            } else {
                out_tris.push(f);
            }
        }
    }
    if any_zero_volume_cell {
        extract_zero_volume_cell_tris(&mut out_tris, arena, tm_subdivided, pinfo, cinfo);
    }
    IMesh::from_faces(out_tris)
}

/* ---- Raycast fallback (non-PWN input) ------------------------------------ */

fn calc_point_inside_tri_db(arena: &IMeshArena, tri: &Face) -> [f64; 3] {
    let v0 = arena.verts[tri.vert[0]].co;
    let v1 = arena.verts[tri.vert[1]].co;
    let v2 = arena.verts[tri.vert[2]].co;
    [
        v0[0] / 3.0 + v1[0] / 3.0 + v2[0] / 3.0,
        v0[1] / 3.0 + v1[1] / 3.0 + v2[1] / 3.0,
        v0[2] / 3.0 + v1[2] / 3.0 + v2[2] / 3.0,
    ]
}

/// `orient3d` on double3 — f64 determinant sign of (b-a, c-a, d-a).
fn orient3d_f64(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> i32 {
    let adx = b[0] - a[0];
    let ady = b[1] - a[1];
    let adz = b[2] - a[2];
    let bdx = c[0] - a[0];
    let bdy = c[1] - a[1];
    let bdz = c[2] - a[2];
    let cdx = d[0] - a[0];
    let cdy = d[1] - a[1];
    let cdz = d[2] - a[2];
    let det = adx * (bdy * cdz - bdz * cdy) + bdx * (cdy * adz - cdz * ady)
        + cdx * (ady * bdz - adz * bdy);
    det.signum() as i32
}

/// `inside_shape_callback` state.
struct InsideShapeTestData {
    hit_parity: Vec<i32>,
}

fn inside_shape_callback(
    arena: &IMeshArena,
    tm: &IMesh,
    shape_fn: &dyn Fn(i32) -> i32,
    data: &mut InsideShapeTestData,
    index: i32,
    ray: &crate::kdopbvh::BVHTreeRay,
) {
    let tri = &arena.faces[tm.face[index as usize]];
    let shape = shape_fn(tri.orig);
    if shape == -1 {
        return;
    }
    let fv0 = [
        tri.vert[0] as usize,
    ];
    let _ = fv0;
    let fv0 = arena.verts[tri.vert[0]].co;
    let fv1 = arena.verts[tri.vert[1]].co;
    let fv2 = arena.verts[tri.vert[2]].co;
    let co3 = |c: [f64; 3]| blenders_math::vector::Vec3::new(c[0] as f32, c[1] as f32, c[2] as f32);
    if blenders_math::geom::isect_ray_tri_epsilon(
        blenders_math::vector::Vec3::new(ray.origin[0], ray.origin[1], ray.origin[2]),
        blenders_math::vector::Vec3::new(ray.direction[0], ray.direction[1], ray.direction[2]),
        co3(fv0),
        co3(fv1),
        co3(fv2),
        f32::EPSILON,
    )
    .is_some()
    {
        /* +1 if ray agrees with tri normal direction, -1 if opposite. */
        let o_db = [
            ray.origin[0] as f64,
            ray.origin[1] as f64,
            ray.origin[2] as f64,
        ];
        let parity = orient3d_f64(fv0, fv1, fv2, o_db);
        data.hit_parity[shape as usize] += parity;
    }
}

/// `test_tri_inside_shapes` — 6-ray inside-shape confidence test.
fn test_tri_inside_shapes(
    arena: &mut IMeshArena,
    tm: &IMesh,
    shape_fn: &dyn Fn(i32) -> i32,
    nshapes: usize,
    test_t_index: usize,
    tree: &crate::kdopbvh::BVHTree,
    in_shape: &mut [f32],
) {
    let tri_test = &arena.faces[tm.face[test_t_index]];
    let shape = shape_fn(tri_test.orig);
    if shape == -1 {
        in_shape.fill(0.0);
        return;
    }
    let test_point = calc_point_inside_tri_db(arena, tri_test);
    /* Offset a tiny bit in the plane normal direction. */
    if tri_test.plane.is_none() {
        let fi = tm.face[test_t_index];
        let mut f = arena.faces[fi].clone();
        f.populate_plane(arena, false);
        arena.faces[fi] = f;
    }
    let norm_d = arena.planes[arena.faces[tm.face[test_t_index]].plane.unwrap()].norm;
    let nlen = (norm_d[0] * norm_d[0] + norm_d[1] * norm_d[1] + norm_d[2] * norm_d[2]).sqrt();
    let norm = [norm_d[0] / nlen, norm_d[1] / nlen, norm_d[2] / nlen];
    let offset_amount = 1e-5f64;
    let offset_test_point = [
        test_point[0] + offset_amount * norm[0],
        test_point[1] + offset_amount * norm[1],
        test_point[2] + offset_amount * norm[2],
    ];
    const RAYS_NUM: usize = 6;
    const R1: f32 = 0.9987025295199663;
    const RA: f32 = 0.04993512647599832;
    const RB: f32 = 0.009987025295199663;
    let test_rays: [[f32; 3]; RAYS_NUM] = [
        [R1, RA, RB],
        [-R1, -RA, -RB],
        [RB, R1, RA],
        [-RB, -R1, -RA],
        [RA, RB, R1],
        [-RA, -RB, -R1],
    ];
    let mut data = InsideShapeTestData {
        hit_parity: vec![0; nshapes],
    };
    let mut count_insides = vec![0i32; nshapes];
    let co = [
        offset_test_point[0] as f32,
        offset_test_point[1] as f32,
        offset_test_point[2] as f32,
    ];
    for dir in test_rays {
        crate::kdopbvh::bvhtree_ray_cast_all(
            tree,
            co,
            dir,
            0.0,
            f32::MAX,
            Some(&mut |index, ray, _hit| {
                inside_shape_callback(arena, tm, shape_fn, &mut data, index, ray);
            }),
        );
        for j in 0..nshapes {
            if j != shape as usize && data.hit_parity[j] > 0 {
                count_insides[j] += 1;
            }
        }
        data.hit_parity.iter_mut().for_each(|h| *h = 0);
    }
    for j in 0..nshapes {
        if j == shape as usize {
            in_shape[j] = 1.0;
        } else {
            in_shape[j] = count_insides[j] as f32 / RAYS_NUM as f32;
        }
    }
}

/// `raycast_tree`.
fn raycast_tree(arena: &IMeshArena, tm: &IMesh) -> crate::kdopbvh::BVHTree {
    let mut tree =
        crate::kdopbvh::bvhtree_new(tm.face_size() as i32, f32::EPSILON, 4, 6);
    for i in 0..tm.face_size() {
        let f = &arena.faces[tm.face[i]];
        let mut t_cos = [0f32; 9];
        for j in 0..3 {
            for k in 0..3 {
                t_cos[3 * j + k] = arena.verts[f.vert[j]].co[k] as f32;
            }
        }
        crate::kdopbvh::bvhtree_insert(&mut tree, i as i32, &t_cos, 3);
    }
    crate::kdopbvh::bvhtree_balance(&mut tree);
    tree
}

/// `raycast_test_remove` — returns (do_remove, do_flip).
fn raycast_test_remove(op: BoolOpType, winding: &mut [i32], shape: usize) -> (bool, bool) {
    winding[shape] = 0;
    let in_output_volume_0 = apply_bool_op(op, winding);
    winding[shape] = 1;
    let in_output_volume_1 = apply_bool_op(op, winding);
    let do_remove = in_output_volume_0 == in_output_volume_1;
    let do_flip = !do_remove && op == BoolOpType::Difference && shape != 0;
    (do_remove, do_flip)
}

/// `raycast_add_flipped`.
fn raycast_add_flipped(out_faces: &mut Vec<usize>, arena: &mut IMeshArena, tri_fi: usize) {
    let tri = &arena.faces[tri_fi];
    let flipped_vs = vec![tri.vert[0], tri.vert[2], tri.vert[1]];
    let flipped_e_origs = vec![tri.edge_orig[2], tri.edge_orig[1], tri.edge_orig[0]];
    let flipped_is_intersect =
        vec![tri.is_intersect[2], tri.is_intersect[1], tri.is_intersect[0]];
    let orig = tri.orig;
    let flipped_f = arena.add_face_full(
        flipped_vs,
        orig,
        flipped_e_origs,
        flipped_is_intersect,
    );
    out_faces.push(flipped_f);
}

/// `raycast_tris_boolean` (serial).
fn raycast_tris_boolean(
    arena: &mut IMeshArena,
    tm: &IMesh,
    op: BoolOpType,
    nshapes: usize,
    shape_fn: &dyn Fn(i32) -> i32,
) -> IMesh {
    let tree = raycast_tree(arena, tm);
    let mut out_faces: Vec<usize> = Vec::with_capacity(tm.face_size());
    let mut in_shape = vec![0f32; nshapes];
    let mut winding = vec![0i32; nshapes];
    for t in 0..tm.face_size() {
        let tri_fi = tm.face[t];
        let shape = shape_fn(arena.faces[tri_fi].orig);
        test_tri_inside_shapes(arena, tm, shape_fn, nshapes, t, &tree, &mut in_shape);
        for other_shape in 0..nshapes {
            if other_shape == shape as usize {
                continue;
            }
            let need_high_confidence =
                (op == BoolOpType::Difference && shape != 0) || op == BoolOpType::Intersect;
            let inside = in_shape[other_shape] >= if need_high_confidence { 0.5 } else { 0.1 };
            winding[other_shape] = inside as i32;
        }
        let (do_remove, do_flip) = raycast_test_remove(op, &mut winding, shape as usize);
        if !do_remove {
            if !do_flip {
                out_faces.push(tri_fi);
            } else {
                raycast_add_flipped(&mut out_faces, arena, tri_fi);
            }
        }
    }
    let mut ans = IMesh::new();
    ans.set_faces(out_faces);
    ans
}

/// `raycast_patches_boolean`.
fn raycast_patches_boolean(
    arena: &mut IMeshArena,
    tm: &IMesh,
    op: BoolOpType,
    nshapes: usize,
    shape_fn: &dyn Fn(i32) -> i32,
    pinfo: &PatchesInfo,
) -> IMesh {
    let tree = raycast_tree(arena, tm);
    let mut out_faces: Vec<usize> = Vec::with_capacity(tm.face_size());
    let mut in_shape = vec![0f32; nshapes];
    let mut winding = vec![0i32; nshapes];
    for p in 0..pinfo.tot_patch() {
        let patch = pinfo.patch(p);
        let test_t_index = patch.tri[patch.tot_tri() / 2];
        let tri_test = &arena.faces[tm.face[test_t_index as usize]];
        let shape = shape_fn(tri_test.orig);
        if shape == -1 {
            continue;
        }
        test_tri_inside_shapes(
            arena,
            tm,
            shape_fn,
            nshapes,
            test_t_index as usize,
            &tree,
            &mut in_shape,
        );
        for other_shape in 0..nshapes {
            if other_shape == shape as usize {
                continue;
            }
            let need_high_confidence =
                (op == BoolOpType::Difference && shape != 0) || op == BoolOpType::Intersect;
            let inside = in_shape[other_shape] >= if need_high_confidence { 0.5 } else { 0.1 };
            winding[other_shape] = inside as i32;
        }
        let (do_remove, do_flip) = raycast_test_remove(op, &mut winding, shape as usize);
        if !do_remove {
            for &t in &patch.tri {
                let f = tm.face[t as usize];
                if !do_flip {
                    out_faces.push(f);
                } else {
                    raycast_add_flipped(&mut out_faces, arena, f);
                }
            }
        }
    }
    let mut ans = IMesh::new();
    ans.set_faces(out_faces);
    ans
}

/* ---- Merge / dissolve ---------------------------------------------------- */

/// `find_tris_common_edge` — (i, j) edge start indices, else (-1, -1).
fn find_tris_common_edge(tri1: &Face, tri2: &Face) -> (i32, i32) {
    for i in 0..3 {
        for j in 0..3 {
            if tri1.vert[(i + 1) % 3] == tri2.vert[j]
                && tri1.vert[i] == tri2.vert[(j + 1) % 3]
            {
                return (i as i32, j as i32);
            }
        }
    }
    (-1, -1)
}

/// `MergeEdge`.
struct MergeEdge {
    len_squared: f64,
    v1: usize,
    v2: usize,
    left_face: i32,
    right_face: i32,
    orig: i32,
    dissolvable: bool,
    is_intersect: bool,
}
impl MergeEdge {
    fn new(arena: &IMeshArena, va: usize, vb: usize) -> Self {
        let (v1, v2) = if arena.verts[va].id < arena.verts[vb].id {
            (va, vb)
        } else {
            (vb, va)
        };
        MergeEdge {
            len_squared: 0.0,
            v1,
            v2,
            left_face: -1,
            right_face: -1,
            orig: -1,
            dissolvable: false,
            is_intersect: false,
        }
    }
}

/// `MergeFace`.
struct MergeFace {
    vert: Vec<usize>,
    edge: Vec<i32>,
    merge_to: i32,
    orig: i32,
}

/// `FaceMergeState`.
struct FaceMergeState {
    face: Vec<MergeFace>,
    edge: Vec<MergeEdge>,
    edge_map: HashMap<(i32, i32), i32>,
}

/// `init_face_merge_state`.
fn init_face_merge_state(
    fms: &mut FaceMergeState,
    arena: &IMeshArena,
    tris: &[i32],
    tm: &IMesh,
    norm: &[f64; 3],
) {
    fms.face.reserve(tris.len() + 1);
    fms.edge.reserve(3 * tris.len());
    fms.edge_map.reserve(3 * tris.len());
    for &t in tris {
        let tri = &arena.faces[tm.face[t as usize]];
        debug_assert!(tri.plane.is_some());
        let pn = &arena.planes[tri.plane.unwrap()].norm;
        if norm[0] * pn[0] + norm[1] * pn[1] + norm[2] * pn[2] <= 0.0 {
            continue;
        }
        let mut mf = MergeFace {
            vert: vec![tri.vert[0], tri.vert[1], tri.vert[2]],
            edge: Vec::new(),
            merge_to: -1,
            orig: tri.orig,
        };
        let edge_origs: Vec<i32> = tri.edge_orig.clone();
        let is_inters: Vec<bool> = tri.is_intersect.clone();
        fms.face.push(mf);
        let f = (fms.face.len() - 1) as i32;
        for i in 0..3 {
            let inext = (i + 1) % 3;
            let new_me = MergeEdge::new(
                arena,
                fms.face[f as usize].vert[i],
                fms.face[f as usize].vert[inext],
            );
            let canon_vs = (arena.verts[new_me.v1].id, arena.verts[new_me.v2].id);
            let mut me_index = *fms.edge_map.get(&canon_vs).unwrap_or(&-1);
            if me_index == -1 {
                let v1co = arena.verts[new_me.v1].co;
                let v2co = arena.verts[new_me.v2].co;
                let vec = [
                    v2co[0] - v1co[0],
                    v2co[1] - v1co[1],
                    v2co[2] - v1co[2],
                ];
                let mut new_me = new_me;
                new_me.len_squared =
                    vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2];
                new_me.orig = edge_origs[i];
                new_me.is_intersect = is_inters[i];
                new_me.dissolvable = new_me.orig == NO_INDEX && !new_me.is_intersect;
                fms.edge.push(new_me);
                me_index = (fms.edge.len() - 1) as i32;
                fms.edge_map.insert(canon_vs, me_index);
            }
            if fms.edge[me_index as usize].dissolvable && edge_origs[i] != NO_INDEX {
                fms.edge[me_index as usize].dissolvable = false;
                fms.edge[me_index as usize].orig = edge_origs[i];
            }
            if fms.edge[me_index as usize].dissolvable && is_inters[i] {
                fms.edge[me_index as usize].dissolvable = false;
                fms.edge[me_index as usize].is_intersect = true;
            }
            if fms.edge[me_index as usize].v1 == fms.face[f as usize].vert[i] {
                if fms.edge[me_index as usize].left_face != -1 {
                    fms.edge[me_index as usize].dissolvable = false;
                } else {
                    fms.edge[me_index as usize].left_face = f;
                }
            } else if fms.edge[me_index as usize].right_face != -1 {
                fms.edge[me_index as usize].dissolvable = false;
            } else {
                fms.edge[me_index as usize].right_face = f;
            }
            fms.face[f as usize].edge.push(me_index);
        }
    }
}

/// `dissolve_leaves_valid_bmesh`.
fn dissolve_leaves_valid_bmesh(
    fms: &FaceMergeState,
    me: &MergeEdge,
    me_index: i32,
    mf_left: &MergeFace,
    mf_right: &MergeFace,
) -> bool {
    let a_edge_start = mf_left.edge.iter().position(|&e| e == me_index).unwrap() as i32;
    let alen = mf_left.vert.len() as i32;
    let blen = mf_right.vert.len() as i32;
    let b_left_face = me.right_face;
    let mut ok = true;
    /* Another edge, not me, in A's face whose right face is B's left? */
    let mut a_e_index = (a_edge_start + 1) % alen;
    while ok && a_e_index != a_edge_start {
        let a_me_cur = &fms.edge[mf_left.edge[a_e_index as usize] as usize];
        if a_me_cur.right_face == b_left_face {
            ok = false;
        }
        a_e_index = (a_e_index + 1) % alen;
    }
    /* Vert in A, not me's verts, also in B? */
    let mut a_v_index = 0i32;
    while ok && a_v_index < alen {
        let a_v = mf_left.vert[a_v_index as usize];
        if a_v != me.v1 && a_v != me.v2 {
            for b_v_index in 0..blen {
                let b_v = mf_right.vert[b_v_index as usize];
                if a_v == b_v {
                    ok = false;
                }
            }
        }
        a_v_index += 1;
    }
    ok
}

/// `splice_faces`.
fn splice_faces(fms: &mut FaceMergeState, me_index: i32, mf_left_i: i32, mf_right_i: i32) {
    let (a_edge_start, alen) = {
        let mf_left = &fms.face[mf_left_i as usize];
        (
            mf_left
                .edge
                .iter()
                .position(|&e| e == me_index)
                .unwrap(),
            mf_left.vert.len(),
        )
    };
    let (b_edge_start, blen) = {
        let mf_right = &fms.face[mf_right_i as usize];
        (
            mf_right
                .edge
                .iter()
                .position(|&e| e == me_index)
                .unwrap(),
            mf_right.vert.len(),
        )
    };
    let mut splice_vert: Vec<usize> = Vec::with_capacity(alen + blen - 2);
    let mut splice_edge: Vec<i32> = Vec::with_capacity(alen + blen - 2);
    let mut ai = 0usize;
    while ai < a_edge_start {
        splice_vert.push(fms.face[mf_left_i as usize].vert[ai]);
        splice_edge.push(fms.face[mf_left_i as usize].edge[ai]);
        ai += 1;
    }
    let mut bi = b_edge_start + 1;
    while bi != b_edge_start {
        if bi >= blen {
            bi = 0;
            if bi == b_edge_start {
                break;
            }
        }
        let (bv, be) = {
            let mf_right = &fms.face[mf_right_i as usize];
            (mf_right.vert[bi], mf_right.edge[bi])
        };
        splice_vert.push(bv);
        splice_edge.push(be);
        let left_of_me = fms.edge[me_index as usize].left_face;
        if bv == fms.edge[be as usize].v1 {
            fms.edge[be as usize].left_face = left_of_me;
        } else {
            fms.edge[be as usize].right_face = left_of_me;
        }
        bi += 1;
    }
    let mut ai = a_edge_start + 1;
    while ai < alen {
        splice_vert.push(fms.face[mf_left_i as usize].vert[ai]);
        splice_edge.push(fms.face[mf_left_i as usize].edge[ai]);
        ai += 1;
    }
    fms.face[mf_right_i as usize].merge_to = fms.edge[me_index as usize].left_face;
    let mf_left = &mut fms.face[mf_left_i as usize];
    mf_left.vert = splice_vert;
    mf_left.edge = splice_edge;
    fms.edge[me_index as usize].left_face = -1;
    fms.edge[me_index as usize].right_face = -1;
}

/// `do_dissolve` — dissolve edges, longest first (serial).
fn do_dissolve(fms: &mut FaceMergeState) {
    let mut dissolve_edges: Vec<i32> = (0..fms.edge.len())
        .filter(|&e| fms.edge[e].dissolvable)
        .map(|e| e as i32)
        .collect();
    if dissolve_edges.is_empty() {
        return;
    }
    dissolve_edges.sort_by(|&a, &b| {
        fms.edge[b as usize]
            .len_squared
            .partial_cmp(&fms.edge[a as usize].len_squared)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for me_index in dissolve_edges {
        let (left, right, v1, v2) = {
            let me = &fms.edge[me_index as usize];
            (me.left_face, me.right_face, me.v1, me.v2)
        };
        let _ = (v1, v2);
        if left == -1 || right == -1 {
            continue;
        }
        let ok = {
            let me = &fms.edge[me_index as usize];
            let mf_left = &fms.face[left as usize];
            let mf_right = &fms.face[right as usize];
            dissolve_leaves_valid_bmesh(fms, me, me_index, mf_left, mf_right)
        };
        if !ok {
            continue;
        }
        splice_faces(fms, me_index, left, right);
    }
}

/// `merge_tris_for_face`.
fn merge_tris_for_face(
    arena: &mut IMeshArena,
    tris: &[i32],
    tm: &IMesh,
    imesh_in: &IMesh,
) -> Vec<usize> {
    let mut ans: Vec<usize> = Vec::new();
    if tris.len() <= 1 {
        if tris.len() == 1 {
            ans.push(tm.face[tris[0] as usize]);
        }
        return ans;
    }
    let mut done = false;
    let first_tri_normal =
        arena.planes[arena.faces[tm.face[tris[0] as usize]].plane.unwrap()].norm;
    let second_tri_normal =
        arena.planes[arena.faces[tm.face[tris[1] as usize]].plane.unwrap()].norm;
    let dot = first_tri_normal[0] * second_tri_normal[0]
        + first_tri_normal[1] * second_tri_normal[1]
        + first_tri_normal[2] * second_tri_normal[2];
    if tris.len() == 2 && dot > 0.0 {
        /* Quad-with-one-diagonal special case. */
        let tri1_i = tm.face[tris[0] as usize];
        let tri2_i = tm.face[tris[1] as usize];
        let tri1 = &arena.faces[tri1_i];
        let tri2 = &arena.faces[tri2_i];
        let in_face = &arena.faces[imesh_in.face[tri1.orig as usize]];
        if in_face.size() == 4 {
            let estarts = find_tris_common_edge(tri1, tri2);
            if estarts.0 != -1 && tri1.edge_orig[estarts.0 as usize] == NO_INDEX {
                let i0 = estarts.0 as usize;
                let i1 = (i0 + 1) % 3;
                let i2 = (i0 + 2) % 3;
                let j2 = (estarts.1 as usize + 2) % 3;
                let tryface = Face::new(
                    vec![
                        tri1.vert[i1],
                        tri1.vert[i2],
                        tri1.vert[i0],
                        tri2.vert[j2],
                    ],
                    -1,
                    -1,
                    Vec::new(),
                    Vec::new(),
                );
                if tryface.cyclic_equal(arena, in_face) {
                    ans.push(imesh_in.face[tri1.orig as usize]);
                    done = true;
                }
            }
        }
    }
    if done {
        return ans;
    }
    let first_tri_normal_rev = [-first_tri_normal[0], -first_tri_normal[1], -first_tri_normal[2]];
    for norm in [first_tri_normal, first_tri_normal_rev] {
        let mut fms = FaceMergeState {
            face: Vec::new(),
            edge: Vec::new(),
            edge_map: HashMap::new(),
        };
        init_face_merge_state(&mut fms, arena, tris, tm, &norm);
        do_dissolve(&mut fms);
        for mf in &fms.face {
            if mf.merge_to == -1 {
                let e_orig: Vec<i32> = mf
                    .edge
                    .iter()
                    .map(|&e| fms.edge[e as usize].orig)
                    .collect();
                let is_intersect: Vec<bool> = mf
                    .edge
                    .iter()
                    .map(|&e| fms.edge[e as usize].is_intersect)
                    .collect();
                let facep = arena.add_face_full(mf.vert.clone(), mf.orig, e_orig, is_intersect);
                ans.push(facep);
            }
        }
    }
    ans
}

/// `approx_in_line`.
fn approx_in_line(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> bool {
    let vec1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let vec2 = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
    let n1 = (vec1[0] * vec1[0] + vec1[1] * vec1[1] + vec1[2] * vec1[2]).sqrt();
    let n2 = (vec2[0] * vec2[0] + vec2[1] * vec2[1] + vec2[2] * vec2[2]).sqrt();
    let cos_ang = (vec1[0] * vec2[0] + vec1[1] * vec2[1] + vec1[2] * vec2[2]) / (n1 * n2);
    (cos_ang - 1.0).abs() < 1e-4
}

/// `find_dissolve_verts` — returns (dissolve flags, count).
fn find_dissolve_verts(arena: &IMeshArena, imesh_out: &mut IMesh) -> (Vec<bool>, i32) {
    imesh_out.populate_vert(arena);
    let mut dissolve: Vec<bool> = imesh_out
        .vert
        .iter()
        .map(|&v| arena.verts[v].orig == NO_INDEX)
        .collect();
    /* neighbors[i]: up-to-two neighbors of vert i, else can't dissolve. */
    let mut neighbors: Vec<(i32, i32)> = vec![(-1, -1); imesh_out.vert.len()];
    for &f in &imesh_out.face {
        if f == NIL {
            continue;
        }
        let face = &arena.faces[f];
        let flen = face.vert.len();
        for i in 0..flen {
            let v = face.vert[i];
            let v_index = imesh_out.lookup_vert(v) as usize;
            if dissolve[v_index] {
                let n1 = face.vert[(i + 1) % flen] as i32;
                let n2 = face.vert[(i + flen - 1) % flen] as i32;
                let (f_n1, f_n2) = neighbors[v_index];
                if f_n1 != -1 {
                    /* Already has neighbor in another face; same pair required. */
                    if !((n1 == f_n2 && n2 == f_n1) || (n1 == f_n1 && n2 == f_n2)) {
                        dissolve[v_index] = false;
                    }
                } else {
                    neighbors[v_index] = (n1, n2);
                }
            }
        }
    }
    let mut count = 0;
    for v_out in 0..imesh_out.vert.len() {
        if dissolve[v_out] {
            dissolve[v_out] = false; /* Set true if final condition satisfied. */
            let (n1, n2) = neighbors[v_out];
            if n1 != -1 {
                debug_assert_ne!(n2, -1);
                let v_v_out = arena.verts[imesh_out.vert[v_out]].co;
                if approx_in_line(
                    arena.verts[n1 as usize].co,
                    v_v_out,
                    arena.verts[n2 as usize].co,
                ) {
                    dissolve[v_out] = true;
                    count += 1;
                }
            }
        }
    }
    (dissolve, count)
}

/// `dissolve_verts`.
fn dissolve_verts(imesh: &mut IMesh, dissolve: &[bool], arena: &mut IMeshArena) {
    let mut any_faces_erased = false;
    for f in 0..imesh.face.len() {
        if imesh.face[f] == NIL {
            continue;
        }
        let face = &arena.faces[imesh.face[f]];
        let mut face_pos_erase = Vec::with_capacity(face.vert.len());
        let mut erase_num = 0;
        for &v in &face.vert {
            let v_index = imesh.lookup_vert(v);
            debug_assert_ne!(v_index, NO_INDEX);
            if dissolve[v_index as usize] {
                face_pos_erase.push(true);
                erase_num += 1;
            } else {
                face_pos_erase.push(false);
            }
        }
        if erase_num > 0 {
            any_faces_erased |= imesh.erase_face_positions(f, &face_pos_erase, arena);
        }
    }
    imesh.set_dirty_verts();
    if any_faces_erased {
        imesh.remove_null_faces();
    }
}

/// `polymesh_from_trimesh_with_dissolve`.
fn polymesh_from_trimesh_with_dissolve(
    arena: &mut IMeshArena,
    tm_out: &IMesh,
    imesh_in: &IMesh,
) -> IMesh {
    /* Need plane normals for all triangles. */
    for i in 0..tm_out.face_size() {
        let fi = tm_out.face[i];
        if arena.faces[fi].plane.is_none() {
            let mut f = arena.faces[fi].clone();
            f.populate_plane(arena, false);
            arena.faces[fi] = f;
        }
    }
    /* face_output_tris[f]: tm_out tris with original face f. */
    let tot_in_face = imesh_in.face_size();
    let mut face_output_tris: Vec<Vec<i32>> = vec![Vec::new(); tot_in_face];
    for t in 0..tm_out.face_size() {
        let tri = &arena.faces[tm_out.face[t]];
        face_output_tris[tri.orig as usize].push(t as i32);
    }
    /* Merge output tris into faces. */
    let mut face_output_face: Vec<Vec<usize>> = vec![Vec::new(); tot_in_face];
    let mut tot_out_face = 0usize;
    for in_f in 0..tot_in_face {
        if face_output_tris[in_f].is_empty() {
            continue;
        }
        face_output_face[in_f] =
            merge_tris_for_face(arena, &face_output_tris[in_f].clone(), tm_out, imesh_in);
        tot_out_face += face_output_face[in_f].len();
    }
    let mut face: Vec<usize> = Vec::with_capacity(tot_out_face);
    for in_f in 0..tot_in_face {
        face.extend_from_slice(&face_output_face[in_f]);
    }
    let mut imesh_out = IMesh::from_faces(face);
    /* Dissolve non-original valence-2 in-line verts. */
    let (v_dissolve, count_dissolve) = find_dissolve_verts(arena, &mut imesh_out);
    if count_dissolve > 0 {
        dissolve_verts(&mut imesh_out, &v_dissolve, arena);
    }
    imesh_out
}

/* ---- Public entry points -------------------------------------------------- */

/// `boolean_trimesh` — main boolean on a triangle IMesh.
pub fn boolean_trimesh(
    arena: &mut IMeshArena,
    tm_in: &IMesh,
    op: BoolOpType,
    nshapes: i32,
    shape_fn: &dyn Fn(i32) -> i32,
    use_self: bool,
    hole_tolerant: bool,
) -> IMesh {
    if tm_in.face_size() == 0 {
        return tm_in.clone();
    }
    let tm_si = crate::mesh_intersect::trimesh_nary_intersect(
        arena, tm_in, nshapes, shape_fn, use_self,
    );
    /* tm_si may be empty if all input triangles were degenerate. */
    if tm_si.face_size() == 0 || op == BoolOpType::None {
        return tm_si;
    }
    let si_origs: Vec<i32> = tm_si.face.iter().map(|&f| arena.faces[f].orig).collect();
    let si_shape_fn = |t: i32| shape_fn(si_origs[t as usize]);
    let tm_si_topo = TriMeshTopology::new(arena, &tm_si);
    let pwn = is_pwn(arena, &tm_si, &tm_si_topo);
    let tm_out;
    if !pwn {
        if hole_tolerant {
            tm_out = raycast_tris_boolean(
                arena,
                &tm_si,
                op,
                nshapes as usize,
                shape_fn,
            );
        } else {
            let pinfo = find_patches(arena, &tm_si, &tm_si_topo);
            tm_out = raycast_patches_boolean(
                arena,
                &tm_si,
                op,
                nshapes as usize,
                shape_fn,
                &pinfo,
            );
        }
    } else {
        let mut pinfo = find_patches(arena, &tm_si, &tm_si_topo);
        let mut cinfo = find_cells(arena, &tm_si, &tm_si_topo, &mut pinfo);
        finish_patch_cell_graph(arena, &tm_si, &mut cinfo, &mut pinfo, &tm_si_topo);
        let pc_ok = patch_cell_graph_ok(&cinfo, &pinfo);
        if !pc_ok {
            return tm_in.clone();
        }
        cinfo.init_windings(nshapes as usize);
        let c_ambient = find_ambient_cell(arena, &tm_si, None, &tm_si_topo, &pinfo);
        if c_ambient == NO_INDEX {
            return tm_si.clone();
        }
        propagate_windings_and_in_output_volume(
            &mut pinfo,
            &mut cinfo,
            c_ambient,
            op,
            nshapes,
            &si_shape_fn,
        );
        tm_out = extract_from_in_output_volume_diffs(arena, &tm_si, &pinfo, &cinfo);
    }
    tm_out
}

/// `boolean_mesh` — boolean on a general polygonal IMesh.
pub fn boolean_mesh(
    arena: &mut IMeshArena,
    imesh: &IMesh,
    imesh_triangulated: Option<&IMesh>,
    op: BoolOpType,
    nshapes: i32,
    shape_fn: &dyn Fn(i32) -> i32,
    use_self: bool,
    hole_tolerant: bool,
) -> IMesh {
    let our_triangulation;
    let tm_in: &IMesh = match imesh_triangulated {
        Some(t) => t,
        None => {
            our_triangulation = crate::mesh_intersect::triangulate_polymesh(arena, imesh);
            &our_triangulation
        }
    };
    let tm_out = boolean_trimesh(
        arena,
        tm_in,
        op,
        nshapes,
        shape_fn,
        use_self,
        hole_tolerant,
    );
    polymesh_from_trimesh_with_dissolve(arena, &tm_out, imesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two overlapping tetrahedra (outward-oriented faces), one shifted by 0.5.
    fn two_tetras() -> (IMeshArena, IMesh) {
        let mut arena = IMeshArena::new();
        let verts = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.25, 0.25, 0.25],
            [1.25, 0.25, 0.25],
            [0.25, 1.25, 0.25],
            [0.25, 0.25, 1.25],
        ];
        let mut vi = [0usize; 8];
        for (i, c) in verts.iter().enumerate() {
            vi[i] = arena.add_or_find_vert_d(*c, 0);
        }
        /* Outward faces for each tetra. */
        let tetra_faces = [[1, 2, 3], [0, 3, 2], [0, 1, 3], [0, 2, 1]];
        let mut faces: Vec<usize> = Vec::new();
        for shape in 0..2 {
            for (f_i, f) in tetra_faces.iter().enumerate() {
                let vs = vec![vi[shape * 4 + f[0]], vi[shape * 4 + f[1]], vi[shape * 4 + f[2]]];
                faces.push(arena.add_face(vs, (shape * 4 + f_i) as i32));
            }
        }
        let mut mesh = IMesh::from_faces(faces);
        mesh.populate_vert(&arena);
        (arena, mesh)
    }

    #[test]
    fn two_tetra_boolean() {
        let (mut arena, tm) = two_tetras();
        let shape_fn = |t: i32| t / 4;
        for op in [
            BoolOpType::Intersect,
            BoolOpType::Union,
            BoolOpType::Difference,
        ] {
            let out = boolean_trimesh(
                &mut arena,
                &tm,
                op,
                2,
                &shape_fn,
                false,
                false,
            );
            assert!(
                out.face_size() > 0,
                "boolean {:?} produced empty mesh",
                op
            );
        }
    }

}
