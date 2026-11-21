//! Port of `bmesh/tools/bmesh_bevel.cc` — the bevel tool.
//!
//! Pointer-graph port: `BoundVert`/`EdgeHalf`/`NewVert` objects live in
//! `Vec` arenas on `BevelParams`/`BevVert`/`VMesh`; C pointers become
//! `usize` arena indexes (`BM_NIL` = null). EdgeHalf refs are `(bv, idx)`
//! pairs via [`EHRef`]; BoundVert refs are global indexes into
//! `bp.boundverts`. CustomData-driven pieces (UV merge, math layers,
//! `BM_elem_attrs_copy`) are `PORT_PENDING` stubs.

use crate::iterators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_FACE, BM_LOOP, BM_VERT};
use crate::{BM_ELEM_SELECT, BM_ELEM_TAG};
use blenders_math::vector::{Vec2, Vec3};
use std::collections::{HashMap, HashSet};

/// `BM_ELEM_LONG_TAG` (hflag bit 6 — `TAG_ALT` in our hflag map).
pub const BM_ELEM_LONG_TAG: u8 = 1 << 6;

/// output flags (oflag) for `edges.out`/`verts.out` collection.
const EDGE_OUT: u8 = 4;
const VERT_OUT: u8 = 8;

pub const BEVEL_EPSILON_D: f32 = 1e-6;
pub const BEVEL_EPSILON: f32 = 1e-6;
pub const BEVEL_EPSILON_SQ: f32 = 1e-12;
pub const BEVEL_EPSILON_BIG: f32 = 1e-4;
pub const BEVEL_EPSILON_BIG_SQ: f32 = 1e-8;
pub const BEVEL_EPSILON_ANG: f32 = 2.0 * std::f32::consts::PI / 180.0;
pub const BEVEL_SMALL_ANG: f32 = 10.0 * std::f32::consts::PI / 180.0;
pub const BEVEL_SMALL_ANG_DOT: f32 = 1.0 - 0.984807753012208;
pub const BEVEL_EPSILON_ANG_DOT: f32 = 1.0 - 0.9993908270190958;
pub const BEVEL_MAX_ADJUST_PCT: f32 = 10.0;
pub const BEVEL_MAX_AUTO_ADJUST_PCT: f32 = 300.0;
pub const BEVEL_MATCH_SPEC_WEIGHT: f64 = 0.2;
pub const STD_UV_CONNECT_LIMIT: f32 = 0.0001;

/// `BMAmtType` (`bmesh_operators.hh`).
pub const BEVEL_AMT_OFFSET: i32 = 0;
pub const BEVEL_AMT_WIDTH: i32 = 1;
pub const BEVEL_AMT_DEPTH: i32 = 2;
pub const BEVEL_AMT_PERCENT: i32 = 3;
pub const BEVEL_AMT_ABSOLUTE: i32 = 4;
/// `BEVEL_PROFILE_*`.
pub const BEVEL_PROFILE_SUPERELLIPSE: i32 = 0;
pub const BEVEL_PROFILE_CUSTOM: i32 = 1;
/// `BEVEL_FACE_STRENGTH_*`.
pub const BEVEL_FACE_STRENGTH_NONE: i32 = 0;
pub const BEVEL_FACE_STRENGTH_NEW: i32 = 1;
pub const BEVEL_FACE_STRENGTH_AFFECTED: i32 = 2;
pub const BEVEL_FACE_STRENGTH_ALL: i32 = 3;
/// `BVMiterType`.
pub const BEVEL_MITER_SHARP: i32 = 0;
pub const BEVEL_MITER_PATCH: i32 = 1;
pub const BEVEL_MITER_ARC: i32 = 2;
/// `BVMeshMethod`.
pub const BEVEL_VMESH_ADJ: i32 = 0;
pub const BEVEL_VMESH_CUTOFF: i32 = 1;
/// `BVAffectType`.
pub const BEVEL_AFFECT_VERTICES: i32 = 0;
pub const BEVEL_AFFECT_EDGES: i32 = 1;

/// `PRO_SQUARE_R`/`PRO_CIRCLE_R`/`PRO_LINE_R`/`PRO_SQUARE_IN_R`.
pub const PRO_SQUARE_R: f32 = 1e4;
pub const PRO_CIRCLE_R: f32 = 2.0;
pub const PRO_LINE_R: f32 = 1.0;
pub const PRO_SQUARE_IN_R: f32 = 0.0;

/// Reference to an `EdgeHalf` inside `bp.bverts[bv].edges` — `(bv_idx, eh_idx)`.
pub type EHRef = (usize, usize);
const EH_NIL: EHRef = (usize::MAX, usize::MAX);

#[derive(Clone, Copy, Default)]
pub struct NewVert {
    pub v: usize,
    pub co: Vec3,
}

impl NewVert {
    fn new() -> Self {
        NewVert { v: BM_NIL, co: Vec3::ZERO }
    }
}

#[derive(Clone)]
pub struct EdgeHalf {
    /// circular links within `bp.bverts[bv].edges`
    pub next: usize,
    pub prev: usize,
    pub e: usize,
    pub fprev: usize,
    pub fnext: usize,
    /// `BoundVert` indexes in `bp.boundverts` (BM_NIL when unset).
    pub leftv: usize,
    pub rightv: usize,
    pub profile_index: i32,
    pub seg: i32,
    pub offset_l: f32,
    pub offset_r: f32,
    pub offset_l_spec: f32,
    pub offset_r_spec: f32,
    pub is_bev: bool,
    pub is_rev: bool,
    pub is_seam: bool,
    pub visited_rpo: bool,
}

impl Default for EdgeHalf {
    fn default() -> Self {
        Self {
            next: BM_NIL,
            prev: BM_NIL,
            e: BM_NIL,
            fprev: BM_NIL,
            fnext: BM_NIL,
            leftv: BM_NIL,
            rightv: BM_NIL,
            profile_index: 0,
            seg: 0,
            offset_l: 0.0,
            offset_r: 0.0,
            offset_l_spec: 0.0,
            offset_r_spec: 0.0,
            is_bev: false,
            is_rev: false,
            is_seam: false,
            visited_rpo: false,
        }
    }
}

#[derive(Clone)]
pub struct Profile {
    pub super_r: f32,
    pub height: f32,
    pub start: Vec3,
    pub middle: Vec3,
    pub end: Vec3,
    pub plane_no: Vec3,
    pub plane_co: Vec3,
    pub proj_dir: Vec3,
    /// `prof_co`, `prof_co_2` — per-seg cached profile coords.
    pub prof_co: Vec<Vec3>,
    pub prof_co_2: Vec<Vec3>,
    pub special_params: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            super_r: 0.0,
            height: 0.0,
            start: Vec3::ZERO,
            middle: Vec3::ZERO,
            end: Vec3::ZERO,
            plane_no: Vec3::ZERO,
            plane_co: Vec3::ZERO,
            proj_dir: Vec3::ZERO,
            prof_co: Vec::new(),
            prof_co_2: Vec::new(),
            special_params: false,
        }
    }
}

#[derive(Clone, Default)]
pub struct ProfileSpacing {
    pub xvals: Vec<f64>,
    pub yvals: Vec<f64>,
    pub xvals_2: Vec<f64>,
    pub yvals_2: Vec<f64>,
    pub seg_2: i32,
    pub fullness: f32,
}

#[derive(Default)]
pub struct MathLayerInfo {
    pub face_component: Vec<i32>,
    pub has_math_layers: bool,
}

#[derive(Clone)]
pub struct UVFace {
    pub f: usize,
    pub attached_frep: usize,
}

#[derive(Clone)]
pub struct BoundVert {
    /// circular doubly-linked list links (indexes into `bp.boundverts`).
    pub next: usize,
    pub prev: usize,
    pub nv: NewVert,
    /// owning `BevVert` index (`bp.bverts`).
    pub bv: usize,
    /// `EdgeHalf` indexes into `bv.edges` (BM_NIL when unset).
    pub efirst: usize,
    pub elast: usize,
    pub eon: usize,
    pub ebev: usize,
    pub index: i32,
    pub sinratio: f32,
    /// index into `bp.boundverts` (weld chain — cross-bv).
    pub adjchain: usize,
    pub profile: Profile,
    pub any_seam: bool,
    pub visited: bool,
    pub is_arc_start: bool,
    pub is_patch_start: bool,
    pub is_profile_start: bool,
    pub seam_len: i32,
    pub sharp_len: i32,
}

impl Default for BoundVert {
    fn default() -> Self {
        Self {
            next: BM_NIL,
            prev: BM_NIL,
            nv: NewVert::new(),
            bv: BM_NIL,
            efirst: BM_NIL,
            elast: BM_NIL,
            eon: BM_NIL,
            ebev: BM_NIL,
            index: -1,
            sinratio: 1.0,
            adjchain: BM_NIL,
            profile: Profile::default(),
            any_seam: false,
            visited: false,
            is_arc_start: false,
            is_patch_start: false,
            is_profile_start: false,
            seam_len: 0,
            sharp_len: 0,
        }
    }
}

/// `enum MeshKind`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum MeshKind {
    #[default]
    None,
    Poly,
    Adj,
    TriFan,
    Cutoff,
}

#[derive(Clone, Default)]
pub struct VMesh {
    /// `(count, seg/2+1, seg+1)` flat `NewVert` array.
    pub mesh: Vec<NewVert>,
    /// head of the boundvert circular list (`bp.boundverts` index).
    pub boundstart: usize,
    pub count: i32,
    pub seg: i32,
    pub mesh_kind: MeshKind,
}

impl VMesh {
    fn new() -> Self {
        VMesh { mesh: Vec::new(), boundstart: BM_NIL, count: 0, seg: 0, mesh_kind: MeshKind::None }
    }
    #[inline]
    pub fn nj(&self) -> usize {
        (self.seg as usize) / 2 + 1
    }
    #[inline]
    pub fn nk(&self) -> usize {
        self.seg as usize + 1
    }
    #[inline]
    pub fn mv_index(&self, i: i32, j: i32, k: i32) -> usize {
        let (i, j, k) = (i as usize, j as usize, k as usize);
        i * self.nk() * self.nj() + j * self.nk() + k
    }
    pub fn mesh_vert(&self, i: i32, j: i32, k: i32) -> usize {
        self.mv_index(i, j, k)
    }
}

#[derive(Default)]
pub struct BevVert {
    pub v: usize,
    pub edgecount: i32,
    pub selcount: i32,
    pub wirecount: i32,
    pub offset: f32,
    pub any_seam: bool,
    pub visited: bool,
    /// `edges` — EdgeHalf arena local to this bv (circular via next/prev idx).
    pub edges: Vec<EdgeHalf>,
    pub wire_edges: Vec<usize>,
    pub vmesh: Option<VMesh>,
    /// `boundverts` owned by this bv are linked circularly inside
    /// `bp.boundverts`; `boundstart` duplicates `vmesh.boundstart` head idx.
    pub boundstart: usize,
}

/// `enum FKind`.
pub const F_NONE: i32 = 0;
pub const F_ORIG: i32 = 1;
pub const F_VERT: i32 = 2;
pub const F_EDGE: i32 = 3;
pub const F_RECON: i32 = 4;

/// `enum AngleKind`.
pub const ANGLE_SMALLER: i32 = -1;
pub const ANGLE_STRAIGHT: i32 = 0;
pub const ANGLE_LARGER: i32 = 1;

/// `UVVertBucket` = set of loop indexes; `UVVertMap` per uv layer.
type UVVertBucket = HashSet<usize>;
type UVVertMap = HashMap<usize, Vec<UVVertBucket>>;

/// `struct BevelParams`.
#[derive(Default)]
pub struct BevelParams {
    /// `vert_hash` — vert index → `bverts` index.
    pub vert_hash: HashMap<usize, usize>,
    pub bverts: Vec<BevVert>,
    /// Global `BoundVert` arena (adjchain links cross bvs).
    pub boundverts: Vec<BoundVert>,
    /// `face_hash` — `Option` mirrors C `std::optional` (enabled for vmesh rebuild).
    pub face_hash: Option<HashMap<usize, i32>>,
    /// `uv_face_hash` — face → `uvfaces` index.
    pub uv_face_hash: HashMap<usize, usize>,
    pub uvfaces: Vec<UVFace>,
    pub uv_vert_maps: Vec<UVVertMap>,
    pub pro_spacing: ProfileSpacing,
    pub pro_spacing_miter: ProfileSpacing,
    pub math_layer_info: MathLayerInfo,

    pub offset: f32,
    pub offset_type: i32,
    pub profile_type: i32,
    pub affect_type: i32,
    pub affect_vertices_odd: bool,
    pub seg: i32,
    pub profile: f32,
    pub pro_super_r: f32,
    pub use_weights: bool,
    pub bweight_offset_vert: i32,
    pub bweight_offset_edge: i32,
    pub loop_slide: bool,
    pub limit_offset: bool,
    pub offset_adjust: bool,
    pub mark_seam: bool,
    pub mark_sharp: bool,
    pub harden_normals: bool,
    /// `custom_profile`, `dvert`, `vertex_group` — PORT_PENDING (CustomData).
    pub vertex_group: i32,
    pub mat_nr: i32,
    pub face_strength_mode: i32,
    pub miter_outer: i32,
    pub miter_inner: i32,
    pub vmesh_method: i32,
    pub spread: f32,
}

impl BevelParams {
    #[inline]
    pub fn bv(&self, bvi: usize) -> &BevVert {
        &self.bverts[bvi]
    }
    #[inline]
    pub fn bv_mut(&mut self, bvi: usize) -> &mut BevVert {
        &mut self.bverts[bvi]
    }
    #[inline]
    pub fn eh(&self, ehref: EHRef) -> &EdgeHalf {
        &self.bverts[ehref.0].edges[ehref.1]
    }
    #[inline]
    pub fn bnd(&self, i: usize) -> &BoundVert {
        &self.boundverts[i]
    }
    #[inline]
    pub fn bnd_mut(&mut self, i: usize) -> &mut BoundVert {
        &mut self.boundverts[i]
    }
    /// `BoundVert`'s EdgeHalf fields live in its own bv's `edges` arena.
    #[inline]
    pub fn bnd_eh(&self, bnd: usize, efield: usize) -> &EdgeHalf {
        &self.bverts[self.boundverts[bnd].bv].edges[efield]
    }
}

/* `angle_v3v3` — angle between (non-normalized) vectors. */
#[inline]
fn angle_v3v3(d1: Vec3, d2: Vec3) -> f32 {
    d1.angle(d2)
}

fn flag_out_edge(bm: &mut BMesh, bme: usize) {
    crate::operators::bmo_elem_flag_enable(bm, (BM_EDGE, bme), EDGE_OUT);
}

fn flag_out_vert(bm: &mut BMesh, bmv: usize) {
    crate::operators::bmo_elem_flag_enable(bm, (BM_VERT, bmv), VERT_OUT);
}

fn disable_flag_out_edge(bm: &mut BMesh, bme: usize) {
    bm.epool[bme].head.api_flag &= !EDGE_OUT;
}

fn record_face_kind(bp: &mut BevelParams, f: usize, fkind: i32) {
    if let Some(fh) = &mut bp.face_hash {
        fh.insert(f, fkind);
    }
}

fn get_face_kind(bp: &BevelParams, f: usize) -> i32 {
    *bp.face_hash.as_ref().and_then(|fh| fh.get(&f)).unwrap_or(&F_ORIG)
}

fn nearly_parallel(d1: Vec3, d2: Vec3) -> bool {
    let ang = angle_v3v3(d1, d2);
    ang.abs() < BEVEL_EPSILON_ANG || (ang - std::f32::consts::PI).abs() < BEVEL_EPSILON_ANG
}

fn nearly_parallel_normalized(d1: Vec3, d2: Vec3) -> bool {
    blenders_math::base::compare_ff(d1.dot(d2).abs(), 1.0, BEVEL_EPSILON_ANG_DOT)
}

/// `add_new_bound_vert` — append to `vm`'s circular boundvert list.
fn add_new_bound_vert(bp: &mut BevelParams, bvi: usize, co: Vec3) -> usize {
    let mut ans = BoundVert { nv: NewVert { v: BM_NIL, co }, bv: bvi, ..Default::default() };
    ans.profile.super_r = PRO_LINE_R;
    let idx = bp.boundverts.len();
    let boundstart = bp.bverts[bvi]
        .vmesh
        .as_ref()
        .map(|vm| vm.boundstart)
        .unwrap_or(BM_NIL);
    if boundstart == BM_NIL {
        ans.index = 0;
        ans.next = idx;
        ans.prev = idx;
        if let Some(vm) = &mut bp.bverts[bvi].vmesh {
            vm.boundstart = idx;
        }
    } else {
        let tail = bp.boundverts[boundstart].prev;
        ans.index = bp.boundverts[tail].index + 1;
        ans.prev = tail;
        ans.next = boundstart;
        bp.boundverts[tail].next = idx;
        bp.boundverts[boundstart].prev = idx;
    }
    bp.boundverts.push(ans);
    if let Some(vm) = &mut bp.bverts[bvi].vmesh {
        vm.count += 1;
    }
    idx
}

/// `mesh_vert` — index into `vm.mesh` flat array.
#[inline]
fn mesh_vert(vm: &VMesh, i: i32, j: i32, k: i32) -> usize {
    vm.mv_index(i, j, k)
}

fn create_mesh_bmvert(bm: &mut BMesh, vm: &mut VMesh, i: i32, j: i32, k: i32, eg: Option<usize>) {
    let ni = mesh_vert(vm, i, j, k);
    let co = vm.mesh[ni].co;
    let v = crate::core::bm_vert_create(bm, co, eg, 0);
    vm.mesh[ni].v = v;
    bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
    flag_out_vert(bm, v);
}

fn copy_mesh_vert(vm: &mut VMesh, ito: i32, jto: i32, kto: i32, ifrom: i32, jfrom: i32, kfrom: i32) {
    let (to, from) = (mesh_vert(vm, ito, jto, kto), mesh_vert(vm, ifrom, jfrom, kfrom));
    vm.mesh[to].v = vm.mesh[from].v;
    vm.mesh[to].co = vm.mesh[from].co;
}

/// `find_edge_half` — EHRef of `bme` in `bv.edges`, or `EH_NIL`.
fn find_edge_half(bp: &BevelParams, bvi: usize, bme: usize) -> EHRef {
    for i in 0..bp.bverts[bvi].edges.len() {
        if bp.bverts[bvi].edges[i].e == bme {
            return (bvi, i);
        }
    }
    EH_NIL
}

fn find_bevvert(bp: &BevelParams, bmv: usize) -> usize {
    *bp.vert_hash.get(&bmv).unwrap_or(&BM_NIL)
}

fn find_uv_face(bp: &BevelParams, bmf: usize) -> usize {
    *bp.uv_face_hash.get(&bmf).unwrap_or(&BM_NIL)
}

/// `find_other_end_edge_half` — returns `(EHRef, bv_other)`.
fn find_other_end_edge_half(bm: &BMesh, bp: &BevelParams, e: EHRef) -> (EHRef, usize) {
    let eh = bp.eh(e);
    let other_v = if eh.is_rev { bm.edge(eh.e).v1 } else { bm.edge(eh.e).v2 };
    let bvo = find_bevvert(bp, other_v);
    if bvo != BM_NIL {
        let eother = find_edge_half(bp, bvo, eh.e);
        debug_assert!(eother != EH_NIL);
        (eother, bvo)
    } else {
        (EH_NIL, BM_NIL)
    }
}

/// `next_bev` — next beveled EdgeHalf in circular order.
fn next_bev(bp: &BevelParams, bvi: usize, from_e: EHRef) -> EHRef {
    let edges = &bp.bverts[bvi].edges;
    let mut from = if from_e == EH_NIL { edges.len() - 1 } else { from_e.1 };
    let start = from;
    loop {
        let e = &edges[from];
        if e.is_bev {
            return (bvi, from);
        }
        from = edges[from].next;
        if from == start {
            break;
        }
    }
    EH_NIL
}

/// `count_ccw_edges_between` — count eh steps from e1 to e2 (exclusive).
fn count_ccw_edges_between(bp: &BevelParams, e1: EHRef, e2: EHRef) -> i32 {
    let edges = &bp.bverts[e1.0].edges;
    let mut count = 0;
    let mut e = e1.1;
    loop {
        if e == e2.1 {
            break;
        }
        e = edges[e].next;
        count += 1;
        if e == e1.1 {
            break;
        }
    }
    count
}

/// `edges_face_connected_at_vert` — do the two edges share a face corner?
fn edges_face_connected_at_vert(bm: &BMesh, bme1: usize, bme2: usize) -> bool {
    for l in LoopsOfEdge::new(bm, bme1) {
        if bm.lpool[bm.lpool[l].prev].e == bme2 || bm.lpool[bm.lpool[l].next].e == bme2 {
            return true;
        }
    }
    false
}

/// `register_uv_face` — record a new bevel face + its rep face.
fn register_uv_face(bp: &mut BevelParams, fnew: usize, frep: usize, frep_arr: Option<&[usize]>) -> usize {
    if fnew == BM_NIL {
        return BM_NIL;
    }
    let mut attached = BM_NIL;
    if let Some(arr) = frep_arr {
        if let Some(&f0) = arr.first() {
            if f0 != BM_NIL {
                attached = f0;
            }
        }
    }
    if attached == BM_NIL && frep != BM_NIL {
        attached = frep;
    }
    let idx = bp.uvfaces.len();
    bp.uvfaces.push(UVFace { f: fnew, attached_frep: attached });
    bp.uv_face_hash.insert(fnew, idx);
    idx
}

/// `update_uv_vert_map` — bucket new-face loops into per-layer uv buckets.
fn update_uv_vert_map(
    bm: &BMesh,
    bp: &mut BevelParams,
    uv_face: usize,
    bv: usize,
    nv_bv_map: Option<&HashMap<usize, usize>>,
) {
    if uv_face == BM_NIL || bp.uvfaces[uv_face].attached_frep == BM_NIL {
        return;
    }
    let f_new = bp.uvfaces[uv_face].f;
    let attached_frep = bp.uvfaces[uv_face].attached_frep;
    for mi in 0..bp.uv_vert_maps.len() {
        let loops: Vec<usize> = LoopsOfFace::new(bm, f_new).collect();
        for l in loops {
            let lv = bm.lpool[l].v;
            if !bp.uv_vert_maps[mi].contains_key(&lv) {
                let mut b = UVVertBucket::new();
                b.insert(l);
                bp.uv_vert_maps[mi].insert(lv, vec![b]);
                continue;
            }
            let orig_v = match nv_bv_map {
                Some(map) => *map.get(&lv).unwrap_or(&bv),
                None => bv,
            };
            let orig_l = crate::query::bm_face_vert_share_loop(bm, attached_frep, orig_v);
            debug_assert!(orig_l != BM_NIL);
            let mut is_bucket_found = false;
            let vert_loops: Vec<usize> = LoopsOfVert::new(bm, lv).collect();
            'outer: for l2 in vert_loops {
                if l == l2 {
                    continue;
                }
                let f2 = bm.lpool[l2].f;
                let uv_face2 = find_uv_face(bp, f2);
                if uv_face2 == BM_NIL || bp.uvfaces[uv_face2].attached_frep == BM_NIL {
                    continue;
                }
                let frep2 = bp.uvfaces[uv_face2].attached_frep;
                let orig_l2 = crate::query::bm_face_vert_share_loop(bm, frep2, orig_v);
                debug_assert!(orig_l2 != BM_NIL);
                let mut is_orig_connected = false;
                if let Some(orig_buckets) = bp.uv_vert_maps[mi].get(&orig_v) {
                    for b in orig_buckets {
                        if b.contains(&orig_l) && b.contains(&orig_l2) {
                            is_orig_connected = true;
                            break;
                        }
                    }
                }
                if attached_frep == frep2 || is_orig_connected {
                    for bucket in bp.uv_vert_maps[mi].get_mut(&lv).unwrap().iter_mut() {
                        if bucket.contains(&l2) {
                            bucket.insert(l);
                            is_bucket_found = true;
                            break;
                        }
                    }
                }
                if is_bucket_found {
                    break 'outer;
                }
            }
            if !is_bucket_found {
                let mut b = UVVertBucket::new();
                b.insert(l);
                bp.uv_vert_maps[mi].get_mut(&lv).unwrap().push(b);
            }
        }
    }
}

/// `determine_uv_vert_connectivity` — group `v`'s loops by equal UVs.
/// PORT_PENDING: needs per-layer `MLoopUV` access (CustomData); computes one
/// bucket per loop unless `uv_at` is provided for layer 0.
fn determine_uv_vert_connectivity(
    bm: &BMesh,
    bp: &mut BevelParams,
    v: usize,
    uv_at: Option<&dyn Fn(&BMesh, usize) -> Vec2>,
) {
    /* uv_vert_maps currently tracks one map (layer 0) pending CustomData. */
    if bp.uv_vert_maps.is_empty() {
        bp.uv_vert_maps.resize(1, HashMap::new());
    }
    let mut buckets: Vec<UVVertBucket> = Vec::new();
    for l in LoopsOfVert::new(bm, v) {
        let luv = uv_at.map(|f| f(bm, l));
        let mut found = false;
        if let Some(luv) = luv {
            'b: for bucket in buckets.iter_mut() {
                for &l2 in bucket.iter() {
                    if (uv_at.unwrap()(bm, l2) - luv).length() <= STD_UV_CONNECT_LIMIT {
                        bucket.insert(l);
                        found = true;
                        break 'b;
                    }
                }
            }
        }
        if !found {
            let mut b = UVVertBucket::new();
            b.insert(l);
            buckets.push(b);
        }
    }
    debug_assert!(!bp.uv_vert_maps[0].contains_key(&v));
    bp.uv_vert_maps[0].insert(v, buckets);
}

/// `bevel_merge_uvs` — average UVs in each bucket. PORT_PENDING (CustomData).
fn bevel_merge_uvs(
    bm: &mut BMesh,
    bp: &mut BevelParams,
    mut uv_set: Option<&mut dyn FnMut(&mut BMesh, usize, Vec2)>,
    uv_at: Option<&dyn Fn(&BMesh, usize) -> Vec2>,
) {
    if let (Some(setter), Some(getter)) = (&mut uv_set, uv_at) {
        for map in &bp.uv_vert_maps {
            for buckets in map.values() {
                for bucket in buckets {
                    if bucket.len() <= 1 {
                        continue;
                    }
                    let mut uv = Vec2::ZERO;
                    for &l in bucket.iter() {
                        uv += getter(bm, l);
                    }
                    uv /= bucket.len() as f32;
                    for &l in bucket.iter() {
                        setter(bm, l, uv);
                    }
                }
            }
        }
    }
}

/// `boundvert_rep_face` — rep face for `v` (and optionally `r_fother`).
fn boundvert_rep_face(bp: &BevelParams, v: usize) -> (usize, usize) {
    let bv = &bp.boundverts[v];
    let mut frep = BM_NIL;
    let mut frep2 = BM_NIL;
    if bv.ebev != BM_NIL {
        frep = bp.bnd_eh(v, bv.ebev).fprev;
        if bp.bnd_eh(v, bv.efirst).fprev != frep {
            frep2 = bp.bnd_eh(v, bv.efirst).fprev;
        }
    } else if bv.efirst != BM_NIL {
        frep = bp.bnd_eh(v, bv.efirst).fprev;
        if frep != BM_NIL {
            if bp.bnd_eh(v, bv.elast).fnext != frep {
                frep2 = bp.bnd_eh(v, bv.elast).fnext;
            } else if bp.bnd_eh(v, bv.efirst).fnext != frep {
                frep2 = bp.bnd_eh(v, bv.efirst).fnext;
            } else if bp.bnd_eh(v, bv.elast).fprev != frep {
                frep2 = bp.bnd_eh(v, bv.efirst).fprev;
            }
        } else if bp.bnd_eh(v, bv.efirst).fnext != BM_NIL {
            frep = bp.bnd_eh(v, bv.efirst).fnext;
            if bp.bnd_eh(v, bv.elast).fnext != frep {
                frep2 = bp.bnd_eh(v, bv.elast).fnext;
            }
        } else if bp.bnd_eh(v, bv.elast).fprev != BM_NIL {
            frep = bp.bnd_eh(v, bv.elast).fprev;
        }
    } else if bp.boundverts[bv.prev].elast != BM_NIL {
        let pv = bv.prev;
        frep = bp.bnd_eh(pv, bp.boundverts[pv].elast).fnext;
        let nv = bv.next;
        if bp.boundverts[nv].efirst != BM_NIL {
            if frep != BM_NIL {
                frep2 = bp.bnd_eh(nv, bp.boundverts[nv].efirst).fprev;
            } else {
                frep = bp.bnd_eh(nv, bp.boundverts[nv].efirst).fprev;
            }
        }
    }
    (frep, frep2)
}

/// `bev_create_ngon` — create face, copy attrs, interp loops, tag, flag edges.
#[allow(clippy::too_many_arguments)]
fn bev_create_ngon(
    bm: &mut BMesh,
    bp: &mut BevelParams,
    vert_arr: &[usize],
    face_arr: Option<&[usize]>,
    facerep: usize,
    snap_edge_arr: Option<&[usize]>,
    bv_v: usize,
    nv_bv_map: Option<&HashMap<usize, usize>>,
    mat_nr: i32,
    do_interp: bool,
) -> usize {
    let f = crate::core::bm_face_create_verts(bm, vert_arr, None, 0, true, false);
    if f == BM_NIL {
        return BM_NIL;
    }
    let rep = if facerep != BM_NIL {
        facerep
    } else if let Some(arr) = face_arr {
        *arr.first().unwrap_or(&BM_NIL)
    } else {
        BM_NIL
    };
    if rep != BM_NIL {
        /* `BM_elem_attrs_copy` — PORT_PENDING (CustomData). */
        if do_interp {
            let new_loops: Vec<usize> = LoopsOfFace::new(bm, f).collect();
            for (i, l) in new_loops.into_iter().enumerate() {
                let interp_f = face_arr.map(|a| a[i]).unwrap_or(facerep);
                if interp_f != BM_NIL {
                    let mut bme = BM_NIL;
                    if let Some(se) = snap_edge_arr {
                        bme = se[i];
                    }
                    let mut save_co = Vec3::ZERO;
                    if bme != BM_NIL {
                        let lv = bm.lpool[l].v;
                        save_co = bm.vert(lv).co;
                        let (c, _) = blenders_math::geom::closest_to_line_segment_v3(
                            save_co,
                            bm.vert(bm.edge(bme).v1).co,
                            bm.vert(bm.edge(bme).v2).co,
                        );
                        bm.vpool[lv].co = c;
                    }
                    crate::interp::bm_loop_interp_from_face(bm, l, interp_f, true, true);
                    if bme != BM_NIL {
                        bm.vpool[bm.lpool[l].v].co = save_co;
                    }
                }
            }
        }
    }
    bm.fpool[f].head.hflag |= BM_ELEM_TAG;
    for e in EdgesOfFace::new(bm, f).collect::<Vec<usize>>() {
        flag_out_edge(bm, e);
    }
    if mat_nr >= 0 {
        bm.fpool[f].mat_nr = mat_nr as i16;
    }
    let uv_face = register_uv_face(bp, f, facerep, face_arr);
    update_uv_vert_map(bm, bp, uv_face, bv_v, nv_bv_map);
    f
}

/// `contig_ldata_across_loops` — PORT_PENDING (CustomData); true when no layers.
fn contig_ldata_across_loops(_bm: &BMesh, _l1: usize, _l2: usize, _layer: i32) -> bool {
    true
}

/// `contig_ldata_across_edge`.
fn contig_ldata_across_edge(bm: &BMesh, e: usize, f1: usize, f2: usize) -> bool {
    /* `bm->ldata.totlayer == 0` — no CustomData yet → true. */
    let (lef1, lef2) = match crate::query::bm_edge_loop_pair(bm, e) {
        Some(p) => p,
        None => return false,
    };
    let (lef1, lef2) = if bm.lpool[lef1].f == f2 { (lef2, lef1) } else { (lef1, lef2) };
    if bm.lpool[lef1].f != f1 || bm.lpool[lef2].f != f2 {
        return false;
    }
    if bm.lpool[lef1].v == bm.lpool[lef2].v {
        return false;
    }
    let (lv1f1, lv2f1) = (lef1, bm.lpool[lef1].next);
    let (lv1f2, lv2f2) = (bm.lpool[lef2].next, lef2);
    for i in 0..0 /* no ldata layers yet */ {
        if !contig_ldata_across_loops(bm, lv1f1, lv1f2, i)
            || !contig_ldata_across_loops(bm, lv2f1, lv2f2, i)
        {
            return false;
        }
    }
    let _ = (lv1f1, lv2f1, lv1f2, lv2f2);
    true
}

/// `contig_ldata_around_vert` — true when vert has uniform loop data.
fn contig_ldata_around_vert(bm: &BMesh, v: usize) -> bool {
    let mut l_first = BM_NIL;
    for l in LoopsOfVert::new(bm, v) {
        if l_first == BM_NIL {
            l_first = l;
            continue;
        }
        for i in 0..0 /* no ldata layers yet */ {
            if !contig_ldata_across_loops(bm, l_first, l, i) {
                return false;
            }
        }
    }
    true
}

fn swap_face_components(face_component: &mut [i32], c1: i32, c2: i32) {
    if c1 == c2 {
        return;
    }
    for fc in face_component.iter_mut() {
        if *fc == c1 {
            *fc = c2;
        } else if *fc == c2 {
            *fc = c1;
        }
    }
}

/// `uv_vert_map_init` — one map per UV layer (PORT_PENDING: 1 placeholder).
fn uv_vert_map_init(bp: &mut BevelParams) {
    bp.uv_vert_maps.clear();
    bp.uv_vert_maps.resize(1, HashMap::new());
}

fn uv_vert_map_pop(bp: &mut BevelParams, v: usize) {
    for m in &mut bp.uv_vert_maps {
        m.remove(&v);
    }
}

/// `math_layer_info_init` — face components via ldata continuity + z-order.
fn math_layer_info_init(bm: &mut BMesh, bp: &mut BevelParams) {
    bp.math_layer_info.has_math_layers = false; /* no CD_PROP_FLOAT2 layers yet */
    bp.math_layer_info.face_component = Vec::new();
    if !bp.math_layer_info.has_math_layers || (bp.seg % 2) == 0 {
        return;
    }
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_FACE);
    let totface = bm.totface as usize;
    let mut face_component = vec![-1i32; totface];
    let mut in_stack = vec![false; totface];
    let mut ftable: Vec<usize> = vec![BM_NIL; totface];
    for f in FacesOfMesh::new(bm) {
        ftable[bm.face(f).head.index as usize] = f;
    }
    let mut stack: Vec<usize> = Vec::with_capacity(totface);
    let mut current_component = -1;
    for f in 0..totface {
        if face_component[f] == -1 && !in_stack[f] {
            current_component += 1;
            stack.push(ftable[f]);
            in_stack[f] = true;
            while let Some(bmf) = stack.pop() {
                let bmf_index = bm.face(bmf).head.index as usize;
                in_stack[bmf_index] = false;
                if face_component[bmf_index] != -1 {
                    continue;
                }
                face_component[bmf_index] = current_component;
                for bme in EdgesOfFace::new(bm, bmf) {
                    for bmf_other in FacesOfEdge::new(bm, bme) {
                        if bmf_other == bmf {
                            continue;
                        }
                        let oi = bm.face(bmf_other).head.index as usize;
                        if face_component[oi] != -1 || in_stack[oi] {
                            continue;
                        }
                        if contig_ldata_across_edge(bm, bme, bmf, bmf_other) {
                            stack.push(bmf_other);
                            in_stack[oi] = true;
                        }
                    }
                }
            }
        }
    }
    if current_component <= 0 {
        bp.math_layer_info.face_component = face_component;
        return;
    }
    let mut top_face_z = -1e30f32;
    let mut top_face_component = -1;
    let mut bot_face_z = 1e30f32;
    let mut bot_face_component = -1;
    for f in 0..totface {
        let fz = crate::polygon::bm_face_calc_center_bounds(bm, ftable[f]).z;
        if fz > top_face_z {
            top_face_z = fz;
            top_face_component = face_component[f];
        }
        if fz < bot_face_z {
            bot_face_z = fz;
            bot_face_component = face_component[f];
        }
    }
    let c0 = face_component[0];
    swap_face_components(&mut face_component, c0, top_face_component);
    if bot_face_component != top_face_component {
        if bot_face_component == 0 {
            bot_face_component = top_face_component;
        }
        let c1 = face_component[1];
        swap_face_components(&mut face_component, c1, bot_face_component);
    }
    bp.math_layer_info.face_component = face_component;
}

/// `choose_rep_face` — pick best rep face by component/select/matnr/centroid.
fn choose_rep_face(bm: &mut BMesh, bp: &BevelParams, faces: &[usize]) -> usize {
    const VEC_VALUE_LEN: usize = 6;
    let nfaces = faces.len();
    let mut value_vecs = [[0f32; VEC_VALUE_LEN]; 64];
    let mut still_viable = [false; 64];
    let mut num_viable = 0;
    for f in 0..nfaces.min(64) {
        let bmf = faces[f];
        if bmf == BM_NIL {
            continue;
        }
        still_viable[f] = true;
        num_viable += 1;
        let bmf_index = bm.face(bmf).head.index as usize;
        let mut vi = 0;
        value_vecs[f][vi] = if bp.math_layer_info.face_component.is_empty() {
            0.0
        } else {
            bp.math_layer_info.face_component[bmf_index] as f32
        };
        vi += 1;
        value_vecs[f][vi] = if bm.face(bmf).head.hflag & BM_ELEM_SELECT != 0 { 0.0 } else { 1.0 };
        vi += 1;
        value_vecs[f][vi] = if bm.face(bmf).mat_nr >= 0 { bm.face(bmf).mat_nr as f32 } else { 0.0 };
        vi += 1;
        let cent = crate::polygon::bm_face_calc_center_bounds(bm, bmf);
        value_vecs[f][vi] = cent.z;
        value_vecs[f][vi + 1] = cent.x;
        value_vecs[f][vi + 2] = cent.y;
    }
    let mut best_f: i32 = -1;
    let mut value_index = 0;
    while num_viable > 1 && value_index < VEC_VALUE_LEN {
        for f in 0..nfaces.min(64) {
            if !still_viable[f] || f as i32 == best_f {
                continue;
            }
            if best_f == -1 {
                best_f = f as i32;
                continue;
            }
            if value_vecs[f][value_index] < value_vecs[best_f as usize][value_index] {
                best_f = f as i32;
                for i in (0..f).rev() {
                    if still_viable[i] {
                        still_viable[i] = false;
                        num_viable -= 1;
                    }
                }
            } else if value_vecs[f][value_index] > value_vecs[best_f as usize][value_index] {
                still_viable[f] = false;
                num_viable -= 1;
            }
        }
        value_index += 1;
    }
    if best_f == -1 {
        best_f = 0;
    }
    faces[best_f as usize]
}

/* ===== offset / profile math (C ~1644-2850) ===== */

/// `slide_dist` — point `d` from `v` along edge `e`'s direction toward other vert.
fn slide_dist(bm: &BMesh, e: EHRef, bp: &BevelParams, v: usize, d: f32) -> Vec3 {
    let eh = bp.eh(e);
    let other = if bm.edge(eh.e).v1 == v { bm.edge(eh.e).v2 } else { bm.edge(eh.e).v1 };
    let mut dir = bm.vert(v).co - bm.vert(other).co;
    let len = dir.normalize_and_get_length();
    let mut d = d;
    if d > len {
        d = len - 50.0 * BEVEL_EPSILON_D;
    }
    bm.vert(v).co - dir * d
}

/// `is_outside_edge` — `co` outside `e`'s edge segment (beyond endpoints)?
/// C also takes `ret_closer_v` but never assigns it; callers guard with it,
/// so we return the closer endpoint index for parity with intent.
fn is_outside_edge(bm: &BMesh, eh: &EdgeHalf, co: Vec3) -> (bool, usize) {
    let l1 = bm.vert(bm.edge(eh.e).v1).co;
    let mut u = bm.vert(bm.edge(eh.e).v2).co - l1;
    let h = co - l1;
    let lenu = u.normalize_and_get_length();
    let lambda = u.dot(h);
    let d1 = (co - l1).length_squared();
    let d2 = (co - bm.vert(bm.edge(eh.e).v2).co).length_squared();
    let closer = if d1 <= d2 { bm.edge(eh.e).v1 } else { bm.edge(eh.e).v2 };
    (
        lambda <= -BEVEL_EPSILON_BIG * lenu
            || lambda >= (1.0 + BEVEL_EPSILON_BIG) * lenu,
        closer,
    )
}

/// `edges_angle_kind` — ANGLE_SMALLER/STRAIGHT/LARGER between e1,e2 at `v`.
fn edges_angle_kind(bm: &BMesh, bp: &BevelParams, e1: EHRef, e2: EHRef, v: usize) -> i32 {
    let eh1 = bp.eh(e1);
    let eh2 = bp.eh(e2);
    let v1 = other_vert(bm, eh1.e, v);
    let v2 = other_vert(bm, eh2.e, v);
    let mut dir1 = bm.vert(v).co - bm.vert(v1).co;
    let mut dir2 = bm.vert(v).co - bm.vert(v2).co;
    dir1.normalize();
    dir2.normalize();
    if nearly_parallel_normalized(dir1, dir2) {
        return ANGLE_STRAIGHT;
    }
    let mut cross = dir1.cross(dir2);
    cross.normalize();
    let no = if eh1.fnext != BM_NIL {
        bm.face(eh1.fnext).no
    } else if eh2.fprev != BM_NIL {
        bm.face(eh2.fprev).no
    } else {
        bm.vert(v).no
    };
    if cross.dot(no) < 0.0 {
        ANGLE_LARGER
    } else {
        ANGLE_SMALLER
    }
}

#[inline]
fn other_vert(bm: &BMesh, e: usize, v: usize) -> usize {
    let ed = bm.edge(e);
    if ed.v1 == v { ed.v2 } else { ed.v1 }
}

/// `point_between_edges` — is `co`'s direction inside (dir1,dir2) around f?
fn point_between_edges(
    bm: &BMesh,
    bp: &BevelParams,
    co: Vec3,
    v: usize,
    f: usize,
    e1: EHRef,
    e2: EHRef,
) -> bool {
    let v1 = other_vert(bm, bp.eh(e1).e, v);
    let v2 = other_vert(bm, bp.eh(e2).e, v);
    let mut dir1 = bm.vert(v).co - bm.vert(v1).co;
    let mut dir2 = bm.vert(v).co - bm.vert(v2).co;
    let mut dirco = bm.vert(v).co - co;
    dir1.normalize();
    dir2.normalize();
    dirco.normalize();
    let fno = bm.face(f).no;
    let mut ang11 = blenders_math::geom::angle_normalized_v3v3(dir1, dir2);
    let mut ang1co = blenders_math::geom::angle_normalized_v3v3(dir1, dirco);
    if dir1.cross(dir2).dot(fno) < 0.0 {
        ang11 = std::f32::consts::PI * 2.0 - ang11;
    }
    if dir1.cross(dirco).dot(fno) < 0.0 {
        ang1co = std::f32::consts::PI * 2.0 - ang1co;
    }
    ang11 - ang1co > -BEVEL_EPSILON_ANG
}

/// `edge_edge_angle_less_than_180`.
fn edge_edge_angle_less_than_180(bm: &BMesh, e1: usize, e2: usize, f: usize) -> bool {
    debug_assert!(f != BM_NIL);
    let (v, v1, v2);
    let (a, b) = (bm.edge(e1), bm.edge(e2));
    if a.v1 == b.v1 {
        (v, v1, v2) = (a.v1, a.v2, b.v2);
    } else if a.v1 == b.v2 {
        (v, v1, v2) = (a.v1, a.v2, b.v1);
    } else if a.v2 == b.v1 {
        (v, v1, v2) = (a.v2, a.v1, b.v2);
    } else if a.v2 == b.v2 {
        (v, v1, v2) = (a.v2, a.v1, b.v1);
    } else {
        debug_assert!(false);
        return false;
    }
    let dir1 = bm.vert(v1).co - bm.vert(v).co;
    let dir2 = bm.vert(v2).co - bm.vert(v).co;
    dir1.cross(dir2).dot(bm.face(f).no) > 0.0
}

/// `offset_meet_lines_percent_or_absolute` → (l1a,l1b,l2a,l2b).
fn offset_meet_lines_percent_or_absolute(
    bm: &BMesh,
    bp: &BevelParams,
    e1: EHRef,
    e2: EHRef,
    v: usize,
) -> (Vec3, Vec3, Vec3, Vec3) {
    let eh1 = bp.eh(e1);
    let eh2 = bp.eh(e2);
    let v1 = other_vert(bm, eh1.e, v);
    let v2 = other_vert(bm, eh2.e, v);
    let f1 = eh1.fnext;
    let f2 = eh2.fprev;
    let mut no_offsets = f1 == BM_NIL || f2 == BM_NIL;
    let mut e0 = BM_NIL;
    let mut e3 = BM_NIL;
    let mut e4 = BM_NIL;
    let mut e5 = BM_NIL;
    let (mut d0, mut d3, mut d4, mut d5) = (0f32, 0f32, 0f32, 0f32);
    let mut e1_wt = 1.0f32;
    let mut e2_wt = 1.0f32;
    if !no_offsets {
        e0 = bm.lpool[crate::query::bm_face_vert_share_loop(bm, f1, v1)].e;
        e3 = bm.lpool[bm.lpool[crate::query::bm_face_vert_share_loop(bm, f2, v2)].prev].e;
        e4 = bm.lpool[bm.lpool[crate::query::bm_face_vert_share_loop(bm, f1, v)].prev].e;
        e5 = bm.lpool[crate::query::bm_face_vert_share_loop(bm, f2, v)].e;
        no_offsets = !edge_edge_angle_less_than_180(bm, e0, eh1.e, f1)
            || !edge_edge_angle_less_than_180(bm, eh1.e, e4, f1)
            || !edge_edge_angle_less_than_180(bm, eh2.e, e3, f2)
            || !edge_edge_angle_less_than_180(bm, e5, eh2.e, f1);
        if !no_offsets {
            if bp.offset_type == BEVEL_AMT_ABSOLUTE {
                d0 = bp.offset;
                d3 = bp.offset;
                d4 = bp.offset;
                d5 = bp.offset;
            } else {
                d0 = bp.offset * crate::query::bm_edge_calc_length(bm, e0) / 100.0;
                d3 = bp.offset * crate::query::bm_edge_calc_length(bm, e3) / 100.0;
                d4 = bp.offset * crate::query::bm_edge_calc_length(bm, e4) / 100.0;
                d5 = bp.offset * crate::query::bm_edge_calc_length(bm, e5) / 100.0;
            }
            if bp.use_weights {
                /* edge bevel-weight CustomData — PORT_PENDING → 0.0 like C -1 */
                e1_wt = 0.0;
                e2_wt = 0.0;
            }
        }
    }
    if no_offsets {
        (bm.vert(v).co, bm.vert(v1).co, bm.vert(v).co, bm.vert(v2).co)
    } else {
        (
            slide_dist_vco(bm, e4, v, d4 * e1_wt),
            slide_dist_vco(bm, e0, v1, d0 * e1_wt),
            slide_dist_vco(bm, e5, v, d5 * e2_wt),
            slide_dist_vco(bm, e3, v2, d3 * e2_wt),
        )
    }
}

/// `slide_dist` variant taking a raw edge index.
fn slide_dist_vco(bm: &BMesh, e: usize, v: usize, d: f32) -> Vec3 {
    let other = other_vert(bm, e, v);
    let mut dir = bm.vert(v).co - bm.vert(other).co;
    let len = dir.normalize_and_get_length();
    let mut d = d;
    if d > len {
        d = len - 50.0 * BEVEL_EPSILON_D;
    }
    bm.vert(v).co - dir * d
}

/// `offset_meet` — compute the meet point of e1/e2 offset lines → meetco.
#[allow(clippy::too_many_arguments)]
fn offset_meet(
    bm: &BMesh,
    bp: &BevelParams,
    e1: EHRef,
    e2: EHRef,
    v: usize,
    f_in: usize,
    edges_between: bool,
    e_in_plane: EHRef,
) -> Vec3 {
    let mut meetco = Vec3::ZERO;
    let dir1 = bm.vert(v).co - bm.vert(other_vert(bm, bp.eh(e1).e, v)).co;
    let dir2 = bm.vert(other_vert(bm, bp.eh(e2).e, v)).co - bm.vert(v).co;
    let (mut dir1n, mut dir2p) = (Vec3::ZERO, Vec3::ZERO);
    if edges_between {
        let e1next = bp.eh(e1).next;
        let e2prev = bp.eh(e2).prev;
        dir1n = bm.vert(other_vert(bm, bp.bverts[e1.0].edges[e1next].e, v)).co - bm.vert(v).co;
        dir2p = bm.vert(v).co - bm.vert(other_vert(bm, bp.bverts[e2.0].edges[e2prev].e, v)).co;
    }
    let mut ang = angle_v3v3(dir1, dir2);
    let mut norm_perp1;
    if ang < BEVEL_EPSILON_ANG {
        let norm_v;
        if f_in != BM_NIL {
            norm_v = bm.face(f_in).no;
        } else {
            let mut fcount = 0;
            let mut nv = Vec3::ZERO;
            let mut eloop = e1.1;
            while eloop != e2.1 {
                let fnext = bp.bverts[e1.0].edges[eloop].fnext;
                if fnext != BM_NIL {
                    nv += bm.face(fnext).no;
                    fcount += 1;
                }
                eloop = bp.bverts[e1.0].edges[eloop].next;
            }
            norm_v = if fcount == 0 { bm.vert(v).no } else { nv / fcount as f32 };
        }
        let dsum = dir1 + dir2;
        norm_perp1 = dsum.cross(norm_v);
        norm_perp1.normalize();
        let mut off1a = bm.vert(v).co;
        let d = bp.eh(e1).offset_r.max(bp.eh(e2).offset_l) / (ang / 2.0).cos();
        off1a += norm_perp1 * d;
        meetco = off1a;
    } else if (ang - std::f32::consts::PI).abs() < BEVEL_EPSILON_ANG {
        let d = bp.eh(e1).offset_r.max(bp.eh(e2).offset_l);
        meetco = slide_dist(bm, e2, bp, v, d);
    } else {
        let mut norm_v1;
        let mut norm_v2;
        if f_in != BM_NIL && ang < BEVEL_SMALL_ANG {
            norm_v1 = bm.face(f_in).no;
            norm_v2 = bm.face(f_in).no;
        } else if !edges_between {
            norm_v1 = dir2.cross(dir1);
            norm_v1.normalize();
            let refno = if f_in != BM_NIL { bm.face(f_in).no } else { bm.vert(v).no };
            if norm_v1.dot(refno) < 0.0 {
                norm_v1 = -norm_v1;
            }
            norm_v2 = norm_v1;
        } else {
            norm_v1 = dir1n.cross(dir1);
            norm_v1.normalize();
            let f = bp.eh(e1).fnext;
            let refno = if f != BM_NIL { bm.face(f).no } else { bm.vert(v).no };
            if norm_v1.dot(refno) < 0.0 {
                norm_v1 = -norm_v1;
            }
            norm_v2 = dir2.cross(dir2p);
            norm_v2.normalize();
            let f = bp.eh(e2).fprev;
            let refno = if f != BM_NIL { bm.face(f).no } else { bm.vert(v).no };
            if norm_v2.dot(refno) < 0.0 {
                norm_v2 = -norm_v2;
            }
        }
        norm_perp1 = dir1.cross(norm_v1);
        let mut norm_perp2 = dir2.cross(norm_v2);
        norm_perp1.normalize();
        norm_perp2.normalize();
        let (off1a, off1b, off2a, off2b);
        if bp.offset_type == BEVEL_AMT_PERCENT || bp.offset_type == BEVEL_AMT_ABSOLUTE {
            let r = offset_meet_lines_percent_or_absolute(bm, bp, e1, e2, v);
            off1a = r.0;
            off1b = r.1;
            off2a = r.2;
            off2b = r.3;
        } else {
            let mut a = bm.vert(v).co + norm_perp1 * bp.eh(e1).offset_r;
            let b = a + dir1;
            let mut c = bm.vert(v).co + norm_perp2 * bp.eh(e2).offset_l;
            let d = c + dir2;
            off1a = a;
            off1b = b;
            off2a = c;
            off2b = d;
            let _ = (&mut a, &mut c);
        }
        let (isect_kind, m1, isect2) = blenders_math::geom::isect_line_line(off1a, off1b, off2a, off2b);
        meetco = m1;
        if isect_kind == 0 {
            meetco = off1a;
        } else {
            if bp.eh(e1).offset_r == 0.0 {
                let (outside, closer_v) = is_outside_edge(bm, bp.eh(e1), meetco);
                if outside {
                    meetco = bm.vert(closer_v).co;
                }
            }
            if bp.eh(e2).offset_l == 0.0 {
                let (outside, closer_v) = is_outside_edge(bm, bp.eh(e2), meetco);
                if outside {
                    meetco = bm.vert(closer_v).co;
                }
            }
            if edges_between && bp.eh(e1).offset_r > 0.0 && bp.eh(e2).offset_l > 0.0 {
                if isect_kind == 2 {
                    meetco = (meetco + isect2) * 0.5;
                }
                let mut e = e1.1;
                while e != e2.1 {
                    let fnext = bp.bverts[e1.0].edges[e].fnext;
                    if fnext == BM_NIL {
                        e = bp.bverts[e1.0].edges[e].next;
                        continue;
                    }
                    let plane =
                        blenders_math::geom::plane_from_point_normal(bm.vert(v).co, bm.face(fnext).no);
                    let dropco = blenders_math::geom::closest_to_plane_normalized(plane, meetco);
                    if e_in_plane != EH_NIL {
                        let fip = bp.eh(e_in_plane).fnext;
                        if fip != BM_NIL {
                            ang = angle_v3v3(bm.face(fnext).no, bm.face(fip).no);
                            if ang.abs() < BEVEL_SMALL_ANG
                                || (ang - std::f32::consts::PI).abs() < BEVEL_SMALL_ANG
                            {
                                e = bp.bverts[e1.0].edges[e].next;
                                continue;
                            }
                        }
                    }
                    let enext = bp.bverts[e1.0].edges[e].next;
                    if point_between_edges(bm, bp, dropco, v, fnext, (e1.0, e), (e1.0, enext)) {
                        meetco = dropco;
                        break;
                    }
                    e = bp.bverts[e1.0].edges[e].next;
                }
            }
        }
    }
    meetco
}

const BEVEL_GOOD_ANGLE: f32 = 0.1;

/// `offset_meet_edge` — simple two-edge meet → Option<(meetco, angle)>.
fn offset_meet_edge(bm: &BMesh, bp: &BevelParams, e1: EHRef, e2: EHRef, v: usize) -> Option<(Vec3, f32)> {
    let mut dir1 = bm.vert(other_vert(bm, bp.eh(e1).e, v)).co - bm.vert(v).co;
    let mut dir2 = bm.vert(other_vert(bm, bp.eh(e2).e, v)).co - bm.vert(v).co;
    dir1.normalize();
    dir2.normalize();
    let mut ang = blenders_math::geom::angle_normalized_v3v3(dir1, dir2);
    if ang.abs() < BEVEL_GOOD_ANGLE {
        return None;
    }
    let fno = dir1.cross(dir2);
    if fno.dot(bm.vert(v).no) < 0.0 {
        ang = 2.0 * std::f32::consts::PI - ang;
        return None;
    }
    if (ang - std::f32::consts::PI).abs() < BEVEL_GOOD_ANGLE {
        return None;
    }
    let sinang = ang.sin();
    let mut meetco = bm.vert(v).co;
    if bp.eh(e1).offset_r == 0.0 {
        meetco += dir1 * (bp.eh(e2).offset_l / sinang);
    } else {
        meetco += dir2 * (bp.eh(e1).offset_r / sinang);
    }
    Some((meetco, ang))
}

/// `good_offset_on_edge_between`.
fn good_offset_on_edge_between(bm: &BMesh, bp: &BevelParams, e1: EHRef, e2: EHRef, emid: EHRef, v: usize) -> bool {
    offset_meet_edge(bm, bp, e1, emid, v).is_some()
        && offset_meet_edge(bm, bp, emid, e2, v).is_some()
}

/// `offset_on_edge_between` → (meetco, sinratio) or None.
fn offset_on_edge_between(
    bm: &BMesh,
    bp: &BevelParams,
    e1: EHRef,
    e2: EHRef,
    emid: EHRef,
    v: usize,
) -> Option<(Vec3, f32)> {
    debug_assert!(bp.eh(e1).is_bev && bp.eh(e2).is_bev && !bp.eh(emid).is_bev);
    let r1 = offset_meet_edge(bm, bp, e1, emid, v);
    let r2 = offset_meet_edge(bm, bp, emid, e2, v);
    if bp.offset_type == BEVEL_AMT_PERCENT || bp.offset_type == BEVEL_AMT_ABSOLUTE {
        let v2 = other_vert(bm, bp.eh(emid).e, v);
        let meetco = if bp.offset_type == BEVEL_AMT_PERCENT {
            let wt = if bp.use_weights { 0.0 } else { 1.0 };
            bm.vert(v).co + (bm.vert(v2).co - bm.vert(v).co) * (wt * bp.offset / 100.0)
        } else {
            let mut dir = bm.vert(v2).co - bm.vert(v).co;
            dir.normalize();
            bm.vert(v).co + dir * bp.offset
        };
        let ang1 = r1.map(|r| r.1).unwrap_or(0.0);
        let ang2 = r2.map(|r| r.1).unwrap_or(0.0);
        let sinratio = if ang1 == 0.0 { 1.0 } else { ang2.sin() / ang1.sin() };
        return Some((meetco, sinratio));
    }
    match (r1, r2) {
        (Some((m1, a1)), Some((m2, a2))) => {
            let meetco = (m1 + m2) * 0.5;
            let sinratio = if a1 == 0.0 { 1.0 } else { a2.sin() / a1.sin() };
            Some((meetco, sinratio))
        }
        (Some((m1, _)), None) => Some((m1, 0.0)),
        (None, Some((m2, _))) => Some((m2, 0.0)),
        (None, None) => {
            let meetco = slide_dist(bm, emid, bp, v, bp.eh(e1).offset_r);
            Some((meetco, 0.0))
        }
    }
}

/// `offset_in_plane` — offset point perpendicular to edge within `plane_no`.
fn offset_in_plane(bm: &BMesh, eh: &EdgeHalf, plane_no: Option<Vec3>, left: bool) -> Vec3 {
    let v = if eh.is_rev { bm.edge(eh.e).v2 } else { bm.edge(eh.e).v1 };
    let mut dir = bm.vert(other_vert(bm, eh.e, v)).co - bm.vert(v).co;
    dir.normalize();
    let no = match plane_no {
        Some(n) => n,
        None => {
            let mut n = Vec3::ZERO;
            if dir.x.abs() < dir.y.abs() {
                n.x = 1.0;
            } else {
                n.y = 1.0;
            }
            n
        }
    };
    let mut fdir = if left { dir.cross(no) } else { no.cross(dir) };
    fdir.normalize();
    bm.vert(v).co + fdir * if left { eh.offset_l } else { eh.offset_r }
}

/// `project_to_edge` — closest point on `e`'s line to line co_a-co_b.
fn project_to_edge(bm: &BMesh, e: usize, co_a: Vec3, co_b: Vec3) -> Vec3 {
    let (kind, projco, _other) = blenders_math::geom::isect_line_line(
        bm.vert(bm.edge(e).v1).co,
        bm.vert(bm.edge(e).v2).co,
        co_a,
        co_b,
    );
    if kind == 0 {
        bm.vert(bm.edge(e).v1).co
    } else {
        projco
    }
}

/// `set_profile_params` — fill `bndv.profile` from bound positions.
fn set_profile_params(bm: &BMesh, bp: &mut BevelParams, bndv: usize) {
    let bv = bp.boundverts[bndv].bv;
    let mut do_linear_interp = true;
    let e_idx = bp.boundverts[bndv].ebev;
    let start = bp.boundverts[bndv].nv.co;
    let end = bp.boundverts[bp.boundverts[bndv].next].nv.co;
    if e_idx != BM_NIL {
        let e = &bp.bverts[bv].edges[e_idx];
        do_linear_interp = false;
        let mut pro = bp.boundverts[bndv].profile.clone();
        pro.super_r = bp.pro_super_r;
        pro.proj_dir = bm.vert(bm.edge(e.e).v1).co - bm.vert(bm.edge(e.e).v2).co;
        if e.is_rev {
            pro.proj_dir = -pro.proj_dir;
        }
        pro.proj_dir.normalize();
        pro.middle = project_to_edge(bm, e.e, start, end);
        pro.start = start;
        pro.end = end;
        let mut d1 = pro.middle - start;
        let mut d2 = pro.middle - end;
        d1.normalize();
        d2.normalize();
        pro.plane_no = d1.cross(d2);
        pro.plane_no.normalize();
        if nearly_parallel(d1, d2) {
            pro.middle = bm.vert(bp.bverts[bv].v).co;
            let eprev = &bp.bverts[bv].edges[e.prev];
            let enext = &bp.bverts[bv].edges[e.next];
            if eprev.is_bev && enext.is_bev && bp.bverts[bv].selcount >= 3 {
                let mut d3 = bm.vert(bm.edge(eprev.e).v1).co - bm.vert(bm.edge(eprev.e).v2).co;
                let mut d4 = bm.vert(bm.edge(enext.e).v1).co - bm.vert(bm.edge(enext.e).v2).co;
                d3.normalize();
                d4.normalize();
                if nearly_parallel(d3, d4) {
                    pro.middle = (start + end) * 0.5;
                    do_linear_interp = true;
                } else {
                    let co3 = start + d3;
                    let co4 = end + d4;
                    let (kind, meetco, _) =
                        blenders_math::geom::isect_line_line(start, co3, end, co4);
                    if kind != 0 {
                        pro.middle = meetco;
                    } else {
                        pro.middle = (start + end) * 0.5;
                        do_linear_interp = true;
                    }
                }
            }
            pro.end = end;
            d1 = pro.middle - start;
            d1.normalize();
            d2 = pro.middle - end;
            d2.normalize();
            pro.plane_no = d1.cross(d2);
            pro.plane_no.normalize();
            if nearly_parallel(d1, d2) {
                do_linear_interp = true;
            } else {
                pro.plane_co = bm.vert(bp.bverts[bv].v).co;
                pro.proj_dir = pro.plane_no;
            }
        }
        pro.plane_co = start;
        bp.boundverts[bndv].profile = pro;
    } else if bp.boundverts[bndv].is_arc_start {
        let pro = &mut bp.boundverts[bndv].profile;
        pro.start = start;
        pro.end = end;
        pro.super_r = PRO_CIRCLE_R;
        pro.plane_co = Vec3::ZERO;
        pro.plane_no = Vec3::ZERO;
        pro.proj_dir = Vec3::ZERO;
        do_linear_interp = false;
    } else if bp.affect_type == BEVEL_AFFECT_VERTICES {
        let pro = &mut bp.boundverts[bndv].profile;
        pro.start = start;
        pro.middle = bm.vert(bp.bverts[bv].v).co;
        pro.end = end;
        pro.super_r = bp.pro_super_r;
        pro.plane_co = Vec3::ZERO;
        pro.plane_no = Vec3::ZERO;
        pro.proj_dir = Vec3::ZERO;
        do_linear_interp = false;
    }
    if do_linear_interp {
        let pro = &mut bp.boundverts[bndv].profile;
        pro.super_r = PRO_LINE_R;
        pro.start = start;
        pro.end = end;
        pro.middle = (start + end) * 0.5;
        pro.plane_co = Vec3::ZERO;
        pro.plane_no = Vec3::ZERO;
        pro.proj_dir = Vec3::ZERO;
    }
}

/// `move_profile_plane` — update profile plane to include `bmvert`.
fn move_profile_plane(bp: &mut BevelParams, bm: &BMesh, bndv: usize, bmvert: usize) {
    let pro = bp.boundverts[bndv].profile.clone();
    if pro.proj_dir.length_squared() == 0.0 {
        return;
    }
    let co = bm.vert(bmvert).co;
    let mut d1 = co - pro.start;
    d1.normalize();
    let mut d2 = co - pro.end;
    d2.normalize();
    let mut no = d1.cross(d2);
    let mut no2 = d1.cross(pro.proj_dir);
    let mut no3 = d2.cross(pro.proj_dir);
    if no.normalize_and_get_length() > BEVEL_EPSILON_BIG
        && no2.normalize_and_get_length() > BEVEL_EPSILON_BIG
        && no3.normalize_and_get_length() > BEVEL_EPSILON_BIG
    {
        let dot2 = no.dot(no2);
        let dot3 = no.dot(no3);
        if dot2.abs() < 1.0 - BEVEL_EPSILON_BIG && dot3.abs() < 1.0 - BEVEL_EPSILON_BIG {
            bp.boundverts[bndv].profile.plane_no = no;
        }
    }
    bp.boundverts[bndv].profile.special_params = true;
}

/// `move_weld_profile_planes`.
fn move_weld_profile_planes(bm: &BMesh, bp: &mut BevelParams, bndv1: usize, bndv2: usize) {
    let p1 = bp.boundverts[bndv1].profile.clone();
    let p2 = bp.boundverts[bndv2].profile.clone();
    if p1.proj_dir.length_squared() == 0.0 || p2.proj_dir.length_squared() == 0.0 {
        return;
    }
    let bvv = bp.bverts[bp.boundverts[bndv1].bv].v;
    let d1 = bm.vert(bvv).co - bp.boundverts[bndv1].nv.co;
    let d2 = bm.vert(bvv).co - bp.boundverts[bndv2].nv.co;
    let mut no = d1.cross(d2);
    let l1 = no.normalize_and_get_length();
    let mut no2 = d1.cross(p1.proj_dir);
    let l2 = no2.normalize_and_get_length();
    let mut no3 = d2.cross(p2.proj_dir);
    let l3 = no3.normalize_and_get_length();
    if l1 != 0.0 && (l2 != 0.0 || l3 != 0.0) {
        let dot1 = no.dot(no2).abs();
        let dot2 = no.dot(no3).abs();
        if (dot1 - 1.0).abs() > BEVEL_EPSILON {
            bp.boundverts[bndv1].profile.plane_no = no;
        }
        if (dot2 - 1.0).abs() > BEVEL_EPSILON {
            bp.boundverts[bndv2].profile.plane_no = no;
        }
    }
    bp.boundverts[bndv1].profile.special_params = true;
    bp.boundverts[bndv2].profile.special_params = true;
}

/// `bev_ccw_test` — 1 if `b` follows `a` ccw in `f`, -1 otherwise, 0 if unset.
fn bev_ccw_test(bm: &BMesh, a: usize, b: usize, f: usize) -> i32 {
    if f == BM_NIL {
        return 0;
    }
    let la = crate::query::bm_face_edge_share_loop(bm, f, a);
    let lb = crate::query::bm_face_edge_share_loop(bm, f, b);
    if la == BM_NIL || lb == BM_NIL {
        return 0;
    }
    if bm.lpool[lb].next == la {
        1
    } else {
        -1
    }
}

/// `make_unit_square_map` — matrix mapping unit square to (va,vmid,vb) quad.
fn make_unit_square_map(va: Vec3, vmid: Vec3, vb: Vec3) -> Option<blenders_math::matrix::Mat4> {
    let va_vmid = vmid - va;
    let vb_vmid = vmid - vb;
    if va_vmid.length_squared() == 0.0 || vb_vmid.length_squared() == 0.0 {
        return None;
    }
    if (angle_v3v3(va_vmid, vb_vmid) - std::f32::consts::PI).abs() <= BEVEL_EPSILON_ANG {
        return None;
    }
    let vo = va - vb_vmid;
    let mut vddir = vb_vmid.cross(va_vmid);
    vddir.normalize();
    let vd = vo + vddir;
    let mut m = blenders_math::matrix::Mat4::ZERO;
    let r0 = vmid - va;
    let r1 = vmid - vb;
    let r2 = vmid + vd - va - vb;
    let r3 = va + vb - vmid;
    /* r_mat rows = C's column-major float[4][4] indexing `r_mat[i][*]`.
     * Mat4 here is column-major (m.cols[i] = r_mat[i]). */
    m.cols[0] = blenders_math::vector::Vec4::new(r0.x, r0.y, r0.z, 0.0);
    m.cols[1] = blenders_math::vector::Vec4::new(r1.x, r1.y, r1.z, 0.0);
    m.cols[2] = blenders_math::vector::Vec4::new(r2.x, r2.y, r2.z, 0.0);
    m.cols[3] = blenders_math::vector::Vec4::new(r3.x, r3.y, r3.z, 1.0);
    Some(m)
}

/// `make_unit_cube_map`.
fn make_unit_cube_map(va: Vec3, vb: Vec3, vc: Vec3, vd: Vec3) -> blenders_math::matrix::Mat4 {
    let mut m = blenders_math::matrix::Mat4::ZERO;
    let c0 = (va - vb - vc + vd) * 0.5;
    let c1 = (vb - va - vc + vd) * 0.5;
    let c2 = (vc - va - vb + vd) * 0.5;
    let c3 = (va + vb + vc - vd) * 0.5;
    m.cols[0] = blenders_math::vector::Vec4::new(c0.x, c0.y, c0.z, 0.0);
    m.cols[1] = blenders_math::vector::Vec4::new(c1.x, c1.y, c1.z, 0.0);
    m.cols[2] = blenders_math::vector::Vec4::new(c2.x, c2.y, c2.z, 0.0);
    m.cols[3] = blenders_math::vector::Vec4::new(c3.x, c3.y, c3.z, 1.0);
    m
}

/// `superellipse_co` — superellipse y for x in [0,1], exponent r.
fn superellipse_co(x: f64, r: f32, rbig: bool) -> f64 {
    debug_assert!(r > 0.0);
    let r = r as f64;
    if rbig {
        (1.0 - x.powf(r)).powf(1.0 / r)
    } else {
        1.0 - (1.0 - (1.0 - x).powf(r)).powf(1.0 / r)
    }
}

/// `get_profile_point` — cached profile coord i of nseg.
fn get_profile_point(bp: &BevelParams, pro: &Profile, i: i32, nseg: i32) -> Vec3 {
    if bp.seg == 1 {
        if i == 0 {
            pro.start
        } else {
            pro.end
        }
    } else if nseg == bp.seg {
        debug_assert!(!pro.prof_co.is_empty());
        pro.prof_co[i as usize]
    } else {
        debug_assert!((nseg & (nseg - 1)) == 0 && nseg <= bp.pro_spacing.seg_2);
        let subsample = (bp.pro_spacing.seg_2 / nseg) as usize;
        pro.prof_co_2[i as usize * subsample]
    }
}

/// `calculate_profile_segments` — fill `r_prof_co[0..=ns]`.
#[allow(clippy::too_many_arguments)]
fn calculate_profile_segments(
    profile: &Profile,
    map: &blenders_math::matrix::Mat4,
    use_map: bool,
    reversed: bool,
    ns: i32,
    xvals: &[f64],
    yvals: &[f64],
    r_prof_co: &mut [Vec3],
) {
    for k in 0..=ns as usize {
        let co = if k == 0 {
            profile.start
        } else if k == ns as usize {
            profile.end
        } else if use_map {
            let p = Vec3::new(
                if reversed { yvals[ns as usize - k] } else { xvals[k] } as f32,
                if reversed { xvals[ns as usize - k] } else { yvals[k] } as f32,
                0.0,
            );
            map.transform_point(p)
        } else {
            profile.start + (profile.end - profile.start) * (k as f32 / ns as f32)
        };
        r_prof_co[k] = if profile.proj_dir.length_squared() != 0.0 {
            let co2 = co + profile.proj_dir;
            blenders_math::geom::isect_line_plane(co, co2, profile.plane_co, profile.plane_no)
                .unwrap_or(co)
        } else {
            co
        };
    }
}

/// `calculate_profile` — fill `pro.prof_co`/`prof_co_2`.
fn calculate_profile(bp: &BevelParams, bndv: &mut BoundVert, reversed: bool, miter: bool) {
    let pro_spacing = if miter { &bp.pro_spacing_miter } else { &bp.pro_spacing };
    if bp.seg == 1 {
        return;
    }
    let need_2 = bp.seg != bp.pro_spacing.seg_2;
    if bndv.profile.prof_co.is_empty() {
        bndv.profile.prof_co = vec![Vec3::ZERO; bp.seg as usize + 1];
        bndv.profile.prof_co_2 = if need_2 {
            vec![Vec3::ZERO; bp.pro_spacing.seg_2 as usize + 1]
        } else {
            bndv.profile.prof_co.clone()
        };
    }
    let use_map;
    let mut map = blenders_math::matrix::Mat4::IDENTITY;
    if bp.profile_type == BEVEL_PROFILE_SUPERELLIPSE && bndv.profile.super_r == PRO_LINE_R {
        use_map = false;
    } else {
        match make_unit_square_map(
            bndv.profile.start,
            bndv.profile.middle,
            bndv.profile.end,
        ) {
            Some(m) => {
                map = m;
                use_map = true;
            }
            None => {
                use_map = false;
            }
        }
    }
    if bp.vmesh_method == BEVEL_VMESH_CUTOFF && use_map {
        let bottom = map.transform_point(Vec3::ZERO);
        let top = map.transform_point(Vec3::new(1.0, 1.0, 0.0));
        bndv.profile.height = (bottom - top).length();
    }
    let (pro_super_seg, seg_2) = (bp.seg, pro_spacing.seg_2);
    let (xv, yv) = (pro_spacing.xvals.clone(), pro_spacing.yvals.clone());
    let mut prof = bndv.profile.clone();
    let mut out = bndv.profile.prof_co.clone();
    calculate_profile_segments(&prof, &map, use_map, reversed, pro_super_seg, &xv, &yv, &mut out);
    prof.prof_co = out.clone();
    bndv.profile.prof_co = out;
    if need_2 {
        let (xv2, yv2) = (pro_spacing.xvals_2.clone(), pro_spacing.yvals_2.clone());
        let mut out2 = vec![Vec3::ZERO; seg_2 as usize + 1];
        calculate_profile_segments(&prof, &map, use_map, reversed, seg_2, &xv2, &yv2, &mut out2);
        bndv.profile.prof_co_2 = out2;
    }
}

/// `snap_to_superellipsoid` — project `co` onto superellipsoid of super_r.
fn snap_to_superellipsoid(co: &mut Vec3, super_r: f32, midline: bool) {
    let r = super_r;
    if r == PRO_CIRCLE_R {
        co.normalize();
        return;
    }
    let a = co.x.max(0.0);
    let b = co.y.max(0.0);
    let c = co.z.max(0.0);
    let mut x = a;
    let mut y = b;
    let mut z = c;
    if r == PRO_SQUARE_R || r == PRO_SQUARE_IN_R {
        debug_assert!(z.abs() < BEVEL_EPSILON);
        z = 0.0;
        x = x.min(1.0);
        y = y.min(1.0);
        if r == PRO_SQUARE_R {
            let dx = 1.0 - x;
            let dy = 1.0 - y;
            if dx < dy {
                x = 1.0;
                if midline {
                    y = 1.0;
                }
            } else {
                y = 1.0;
                if midline {
                    x = 1.0;
                }
            }
        } else if x < y {
            x = 0.0;
            if midline {
                y = 0.0;
            }
        } else {
            y = 0.0;
            if midline {
                x = 0.0;
            }
        }
    } else {
        let rinv = 1.0 / r;
        if a == 0.0 {
            if b == 0.0 {
                x = 0.0;
                y = 0.0;
                z = c.powf(rinv);
            } else {
                x = 0.0;
                y = (1.0 / (1.0 + (c / b).powf(r))).powf(rinv);
                z = c * y / b;
            }
        } else {
            x = (1.0 / (1.0 + (b / a).powf(r) + (c / a).powf(r))).powf(rinv);
            y = b * x / a;
            z = c * x / a;
        }
    }
    co.x = x;
    co.y = y;
    co.z = z;
}

/* ===== seam / sharp propagation (C ~2851-3250) ===== */

/// `HASNOT_SEAMSHARP` — edge lacks the flag (seam absent / sharp absent).
fn hasnot_seamsharp(bm: &BMesh, eh: &EdgeHalf, flag: u8) -> bool {
    const SEAM: u8 = crate::BM_ELEM_SEAM;
    const SMOOTH: u8 = crate::BM_ELEM_SMOOTH;
    (flag == SEAM && bm.edge(eh.e).head.hflag & SEAM == 0)
        || (flag == SMOOTH && bm.edge(eh.e).head.hflag & SMOOTH != 0)
}

/// `check_edge_data_seam_sharp_edges` — record run lengths on boundverts.
fn check_edge_data_seam_sharp_edges(bm: &BMesh, bp: &mut BevelParams, bvi: usize, flag: u8) {
    const SEAM: u8 = crate::BM_ELEM_SEAM;
    const SMOOTH: u8 = crate::BM_ELEM_SMOOTH;
    let mut e = 0usize;
    let efirst0 = 0usize;
    loop {
        if !hasnot_seamsharp(bm, &bp.bverts[bvi].edges[e], flag) {
            break;
        }
        e = bp.bverts[bvi].edges[e].next;
        if e == efirst0 {
            break;
        }
    }
    if hasnot_seamsharp(bm, &bp.bverts[bvi].edges[e], flag) {
        return;
    }
    let efirst = e;
    loop {
        let mut flag_count = 0i32;
        let mut ne = bp.bverts[bvi].edges[e].next;
        while hasnot_seamsharp(bm, &bp.bverts[bvi].edges[ne], flag) && ne != efirst {
            if bp.bverts[bvi].edges[ne].is_bev {
                flag_count += 1;
            }
            ne = bp.bverts[bvi].edges[ne].next;
        }
        if ne == e || (ne == efirst && hasnot_seamsharp(bm, &bp.bverts[bvi].edges[efirst], flag)) {
            break;
        }
        let rightv = bp.bverts[bvi].edges[e].rightv;
        if rightv != BM_NIL {
            if flag == SEAM {
                bp.boundverts[rightv].seam_len = flag_count;
            } else if flag == SMOOTH {
                bp.boundverts[rightv].sharp_len = flag_count;
            }
        }
        e = ne;
        if e == efirst {
            break;
        }
    }
}

/// `bevel_extend_edge_data_ex` — extend seam/sharp flags onto bevel edges.
fn bevel_extend_edge_data_ex(bm: &mut BMesh, bp: &mut BevelParams, bvi: usize, flag: u8) {
    const SEAM: u8 = crate::BM_ELEM_SEAM;
    const SMOOTH: u8 = crate::BM_ELEM_SMOOTH;
    debug_assert!(flag == SEAM || flag == SMOOTH);
    let vm_kind;
    let (vm_seg, vm_count, boundstart);
    {
        let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
        vm_kind = vm.mesh_kind;
        vm_seg = vm.seg;
        vm_count = vm.count;
        boundstart = vm.boundstart;
    }
    let _ = vm_kind;
    let mut bcur = boundstart;
    let mut start = bcur;
    loop {
        let extend_len = if flag == SEAM {
            bp.boundverts[bcur].seam_len
        } else {
            bp.boundverts[bcur].sharp_len
        };
        if extend_len != 0 {
            if bp.boundverts[boundstart].seam_len == 0 && start == boundstart {
                start = bcur;
            }
            let idx_end = bp.boundverts[bcur].index + extend_len;
            for i in bp.boundverts[bcur].index..idx_end {
                let ni = (i % vm_count) as usize;
                let vm_ref = bp.bverts[bvi].vmesh.as_ref().unwrap();
                let mut v1 = vm_ref.mesh[mesh_vert(vm_ref, ni as i32, 0, 0)].v;
                for k in 1..vm_seg {
                    let vm_ref = bp.bverts[bvi].vmesh.as_ref().unwrap();
                    let v2 = vm_ref.mesh[mesh_vert(vm_ref, ni as i32, 0, k)].v;
                    let mut e = bm.vert(v1).e;
                    while e != BM_NIL && bm.edge(e).v1 != v2 && bm.edge(e).v2 != v2 {
                        e = crate::structure::bmesh_disk_edge_next(bm, e, v1);
                    }
                    if e != BM_NIL {
                        if flag == SEAM {
                            bm.epool[e].head.hflag |= SEAM;
                        } else {
                            bm.epool[e].head.hflag &= !SMOOTH;
                        }
                    }
                    v1 = v2;
                }
                let vm_ref = bp.bverts[bvi].vmesh.as_ref().unwrap();
                let v3 = vm_ref.mesh[mesh_vert(vm_ref, ((i + 1) % vm_count) as i32, 0, 0)].v;
                let mut e = bm.vert(v1).e;
                while e != BM_NIL && bm.edge(e).v1 != v3 && bm.edge(e).v2 != v3 {
                    e = crate::structure::bmesh_disk_edge_next(bm, e, v1);
                }
                if e != BM_NIL {
                    if flag == SEAM {
                        bm.epool[e].head.hflag |= SEAM;
                    } else {
                        bm.epool[e].head.hflag &= !SMOOTH;
                    }
                }
                bcur = bp.boundverts[bcur].next;
            }
        } else {
            bcur = bp.boundverts[bcur].next;
        }
        if bcur == start {
            break;
        }
    }
}

/// `bevel_extend_edge_data`.
fn bevel_extend_edge_data(bm: &mut BMesh, bp: &mut BevelParams, bvi: usize) {
    let (kind, selcount) = {
        let bv = &bp.bverts[bvi];
        (bv.vmesh.as_ref().map(|v| v.mesh_kind).unwrap_or_default(), bv.selcount)
    };
    if kind == MeshKind::TriFan || selcount < 2 {
        return;
    }
    bevel_extend_edge_data_ex(bm, bp, bvi, crate::BM_ELEM_SEAM);
    bevel_extend_edge_data_ex(bm, bp, bvi, crate::BM_ELEM_SMOOTH);
}

/// `bevel_edges_sharp_boundary` — un-smooth edges between recon and bevel faces.
fn bevel_edges_sharp_boundary(bm: &mut BMesh, bp: &BevelParams) {
    const SMOOTH: u8 = crate::BM_ELEM_SMOOTH;
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.face(f).head.hflag & SMOOTH == 0 {
            continue;
        }
        if get_face_kind(bp, f) != F_RECON {
            continue;
        }
        for l in LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
            let lother = bm.lpool[l].radial_next;
            if lother != l && lother != BM_NIL {
                let fother = bm.lpool[lother].f;
                let fkind = get_face_kind(bp, fother);
                if fkind == F_EDGE || fkind == F_VERT {
                    bm.epool[bm.lpool[l].e].head.hflag &= !SMOOTH;
                }
            }
        }
    }
}

/// `bevel_harden_normals` — custom split normals for bevel faces.
/// PORT_PENDING: custom-normal CD layer + lnor space; does the mesh-normal
/// update and sharp boundary pass, skips the clnor write.
fn bevel_harden_normals(bm: &mut BMesh, bp: &BevelParams) {
    if bp.offset == 0.0 || !bp.harden_normals {
        return;
    }
    crate::normals::bm_mesh_normals_update(bm);
    /* no `custom_normal` ldata layer in this port → act as offset == -1 */
    bevel_edges_sharp_boundary(bm, bp);
    crate::normals::bm_lnorspace_update(bm);
    /* clnor write: PORT_PENDING (CustomData). */
}

/// `bevel_set_weighted_normal_face_strength` — PORT_PENDING (CD int layer).
fn bevel_set_weighted_normal_face_strength(_bm: &mut BMesh, _bp: &BevelParams) {
    /* writes to `MOD_WEIGHTEDNORMALS_FACEWEIGHT_CDLAYER_ID`; no pdata layers. */
}

/// `set_bound_vert_seams` — propagate `is_seam` to boundverts, run-length tag.
fn set_bound_vert_seams(bp: &mut BevelParams, bm: &BMesh, bvi: usize, mark_seam: bool, mark_sharp: bool) {
    bp.bverts[bvi].any_seam = false;
    let boundstart = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
    let mut v = boundstart;
    loop {
        bp.boundverts[v].any_seam = false;
        let mut e = bp.boundverts[v].efirst;
        while e != BM_NIL {
            bp.boundverts[v].any_seam |= bp.bverts[bvi].edges[e].is_seam;
            if e == bp.boundverts[v].elast {
                break;
            }
            e = bp.bverts[bvi].edges[e].next;
        }
        let vs = bp.boundverts[v].any_seam;
        bp.bverts[bvi].any_seam |= vs;
        v = bp.boundverts[v].next;
        if v == boundstart {
            break;
        }
    }
    if mark_seam {
        check_edge_data_seam_sharp_edges(bm, bp, bvi, crate::BM_ELEM_SEAM);
    }
    if mark_sharp {
        check_edge_data_seam_sharp_edges(bm, bp, bvi, crate::BM_ELEM_SMOOTH);
    }
}

/// `eh_on_plane` — edge's two faces coplanar (same or opposite normal)?
fn eh_on_plane(bm: &BMesh, eh: &EdgeHalf) -> bool {
    if eh.fprev != BM_NIL && eh.fnext != BM_NIL {
        let dot = bm.face(eh.fprev).no.dot(bm.face(eh.fnext).no);
        if (dot + 1.0).abs() <= BEVEL_EPSILON_BIG || (dot - 1.0).abs() <= BEVEL_EPSILON_BIG {
            return true;
        }
    }
    false
}

/// `calculate_vm_profiles` — set + calculate profiles for all boundverts.
fn calculate_vm_profiles(bm: &BMesh, bp: &mut BevelParams, bvi: usize) {
    let boundstart = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
    let mut bndv = boundstart;
    loop {
        if !bp.boundverts[bndv].profile.special_params {
            set_profile_params(bm, bp, bndv);
        }
        let mut miter_profile = false;
        let mut reverse_profile = false;
        if bp.profile_type == BEVEL_PROFILE_CUSTOM {
            miter_profile =
                bp.boundverts[bndv].is_arc_start || bp.boundverts[bndv].is_patch_start;
            reverse_profile = !bp.boundverts[bndv].is_profile_start && !miter_profile;
        }
        let mut bndv_tmp = bp.boundverts[bndv].clone();
        calculate_profile(bp, &mut bndv_tmp, reverse_profile, miter_profile);
        bp.boundverts[bndv].profile = bndv_tmp.profile;
        bndv = bp.boundverts[bndv].next;
        if bndv == boundstart {
            break;
        }
    }
}

/* ===== boundary construction (C ~3200-3840) ===== */

/// `adjust_bound_vert` — update boundvert pos + profile plane.
fn adjust_bound_vert(bp: &mut BevelParams, bm: &BMesh, bndv: usize, co: Vec3) {
    bp.boundverts[bndv].nv.co = co;
    if !bp.boundverts[bndv].profile.special_params {
        set_profile_params(bm, bp, bndv);
    }
}

/// `build_boundary_vertex_only`.
fn build_boundary_vertex_only(bm: &BMesh, bp: &mut BevelParams, bvi: usize, construct: bool) {
    let efirst = 0usize;
    let mut e = efirst;
    loop {
        let co = slide_dist(bm, (bvi, e), bp, bp.bverts[bvi].v, bp.bverts[bvi].edges[e].offset_l);
        if construct {
            let v = add_new_bound_vert(bp, bvi, co);
            bp.boundverts[v].efirst = e;
            bp.boundverts[v].elast = e;
            bp.bverts[bvi].edges[e].leftv = v;
            bp.bverts[bvi].edges[e].rightv = v;
        } else {
            adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[e].leftv, co);
        }
        e = bp.bverts[bvi].edges[e].next;
        if e == efirst {
            break;
        }
    }
    if construct {
        set_bound_vert_seams(bp, bm, bvi, bp.mark_seam, bp.mark_sharp);
        if bp.affect_vertices_odd && !bp.bverts[bvi].any_seam {
            /* contig_ldata_around_vert → no ldata → false */
            bp.bverts[bvi].any_seam = true;
        }
        let count = bp.bverts[bvi].vmesh.as_ref().unwrap().count;
        let kind = if count == 2 {
            MeshKind::None
        } else if bp.seg == 1 {
            MeshKind::Poly
        } else {
            MeshKind::Adj
        };
        bp.bverts[bvi].vmesh.as_mut().unwrap().mesh_kind = kind;
    }
}

/// `build_boundary_terminal_edge`.
fn build_boundary_terminal_edge(bm: &BMesh, bp: &mut BevelParams, bvi: usize, efirst: EHRef, construct: bool) {
    let bvv = bp.bverts[bvi].v;
    let mut e = efirst.1;
    if bp.bverts[bvi].edgecount == 2 {
        let eh = &bp.bverts[bvi].edges[e];
        let no = if eh.fprev != BM_NIL { Some(bm.face(eh.fprev).no) } else if eh.fnext != BM_NIL { Some(bm.face(eh.fnext).no) } else { None };
        let co = offset_in_plane(bm, eh, no, true);
        if construct {
            let bndv = add_new_bound_vert(bp, bvi, co);
            bp.boundverts[bndv].efirst = e;
            bp.boundverts[bndv].elast = e;
            bp.boundverts[bndv].ebev = e;
            bp.bverts[bvi].edges[e].leftv = bndv;
        } else {
            adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[e].leftv, co);
        }
        let eh = &bp.bverts[bvi].edges[e];
        let no = if eh.fnext != BM_NIL { Some(bm.face(eh.fnext).no) } else if eh.fprev != BM_NIL { Some(bm.face(eh.fprev).no) } else { None };
        let co = offset_in_plane(bm, eh, no, false);
        if construct {
            let bndv = add_new_bound_vert(bp, bvi, co);
            bp.boundverts[bndv].efirst = e;
            bp.boundverts[bndv].elast = e;
            bp.bverts[bvi].edges[e].rightv = bndv;
        } else {
            adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[e].rightv, co);
        }
        let enext = bp.bverts[bvi].edges[e].next;
        let co = slide_dist(bm, (bvi, enext), bp, bvv, bp.bverts[bvi].edges[e].offset_l);
        if construct {
            let bndv = add_new_bound_vert(bp, bvi, co);
            bp.boundverts[bndv].efirst = enext;
            bp.boundverts[bndv].elast = enext;
            bp.bverts[bvi].edges[enext].leftv = bndv;
            bp.bverts[bvi].edges[enext].rightv = bndv;
            set_bound_vert_seams(bp, bm, bvi, bp.mark_seam, bp.mark_sharp);
        } else {
            adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[enext].leftv, co);
        }
    } else {
        let leg_slide =
            bp.offset_type == BEVEL_AMT_PERCENT || bp.offset_type == BEVEL_AMT_ABSOLUTE;
        let eprev = bp.bverts[bvi].edges[e].prev;
        let co = if leg_slide {
            slide_dist(bm, (bvi, eprev), bp, bvv, bp.bverts[bvi].edges[e].offset_l)
        } else {
            offset_meet(bm, bp, (bvi, eprev), (bvi, e), bvv, bp.bverts[bvi].edges[e].fprev, false, EH_NIL)
        };
        if construct {
            let bndv = add_new_bound_vert(bp, bvi, co);
            bp.boundverts[bndv].efirst = eprev;
            bp.boundverts[bndv].elast = e;
            bp.boundverts[bndv].ebev = e;
            bp.bverts[bvi].edges[e].leftv = bndv;
            bp.bverts[bvi].edges[eprev].leftv = bndv;
            bp.bverts[bvi].edges[eprev].rightv = bndv;
        } else {
            adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[e].leftv, co);
        }
        e = bp.bverts[bvi].edges[e].next;
        let eprev = bp.bverts[bvi].edges[e].prev;
        let co = if leg_slide {
            slide_dist(bm, (bvi, e), bp, bvv, bp.bverts[bvi].edges[eprev].offset_r)
        } else {
            offset_meet(bm, bp, (bvi, eprev), (bvi, e), bvv, bp.bverts[bvi].edges[e].fprev, false, EH_NIL)
        };
        if construct {
            let bndv = add_new_bound_vert(bp, bvi, co);
            bp.boundverts[bndv].efirst = eprev;
            bp.boundverts[bndv].elast = e;
            bp.bverts[bvi].edges[e].leftv = bndv;
            bp.bverts[bvi].edges[e].rightv = bndv;
            bp.bverts[bvi].edges[eprev].rightv = bndv;
        } else {
            adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[e].leftv, co);
        }
        let mut d = bp.bverts[bvi].edges[efirst.1].offset_l_spec;
        if bp.profile_type == BEVEL_PROFILE_CUSTOM || bp.profile < 0.25 {
            d *= 2.0f32.sqrt();
        }
        e = bp.bverts[bvi].edges[e].next;
        while bp.bverts[bvi].edges[e].next != efirst.1 {
            let co = slide_dist(bm, (bvi, e), bp, bvv, d);
            if construct {
                let bndv = add_new_bound_vert(bp, bvi, co);
                bp.boundverts[bndv].efirst = e;
                bp.boundverts[bndv].elast = e;
                bp.bverts[bvi].edges[e].leftv = bndv;
                bp.bverts[bvi].edges[e].rightv = bndv;
            } else {
                adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[e].leftv, co);
            }
            e = bp.bverts[bvi].edges[e].next;
        }
    }
    if bp.bverts[bvi].edgecount >= 3 {
        let bndv = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
        debug_assert!(bp.boundverts[bndv].ebev != BM_NIL);
        set_profile_params(bm, bp, bndv);
        move_profile_plane(bp, bm, bndv, bvv);
    }
    if construct {
        set_bound_vert_seams(bp, bm, bvi, bp.mark_seam, bp.mark_sharp);
        let count = bp.bverts[bvi].vmesh.as_ref().unwrap().count;
        let kind = if count == 2 && bp.bverts[bvi].edgecount == 3 {
            MeshKind::None
        } else if count == 3 {
            let mut use_tri_fan = true;
            if bp.profile_type == BEVEL_PROFILE_CUSTOM {
                let bndv = bp.bverts[bvi].edges[efirst.1].leftv;
                let plane = blenders_math::geom::plane_from_point_normal(
                    bp.boundverts[bndv].profile.plane_co,
                    bp.boundverts[bndv].profile.plane_no,
                );
                let bndv = bp.boundverts[bp.bverts[bvi].edges[efirst.1].rightv].next;
                if blenders_math::geom::dist_squared_to_plane(
                    bp.boundverts[bndv].nv.co,
                    plane,
                ) < BEVEL_EPSILON_BIG
                {
                    use_tri_fan = false;
                }
            }
            if use_tri_fan { MeshKind::TriFan } else { MeshKind::Poly }
        } else {
            MeshKind::Poly
        };
        bp.bverts[bvi].vmesh.as_mut().unwrap().mesh_kind = kind;
    }
}

/// `adjust_miter_coords`.
fn adjust_miter_coords(bm: &BMesh, bp: &mut BevelParams, bvi: usize, emiter: EHRef) {
    let miter_outer = bp.miter_outer;
    let v1 = bp.bverts[emiter.0].edges[emiter.1].rightv;
    let (v2, v3) = if miter_outer == BEVEL_MITER_PATCH {
        let v2 = bp.boundverts[v1].next;
        (v2, bp.boundverts[v2].next)
    } else {
        debug_assert!(miter_outer == BEVEL_MITER_ARC);
        (BM_NIL, bp.boundverts[v1].next)
    };
    let v1prev = bp.boundverts[v1].prev;
    let v3next = bp.boundverts[v3].next;
    let co2 = bp.boundverts[v1].nv.co;
    if bp.boundverts[v1].is_arc_start {
        bp.boundverts[v1].profile.middle = co2;
    }
    let vother = other_vert(bm, bp.bverts[bvi].edges[emiter.1].e, bp.bverts[bvi].v);
    let mut edge_dir = bm.vert(bp.bverts[bvi].v).co - bm.vert(vother).co;
    edge_dir.normalize();
    let d = bp.offset / (bp.seg as f32 / 2.0);
    let line_p = co2 + edge_dir * d;
    let co1 = blenders_math::geom::isect_line_plane(
        co2,
        line_p,
        bp.boundverts[v1prev].nv.co,
        edge_dir,
    )
    .unwrap_or(line_p);
    adjust_bound_vert(bp, bm, v1, co1);
    let emiter_other = bp.boundverts[v3].elast;
    let vother = other_vert(bm, bp.bverts[bvi].edges[emiter_other].e, bp.bverts[bvi].v);
    let mut edge_dir = bm.vert(bp.bverts[bvi].v).co - bm.vert(vother).co;
    edge_dir.normalize();
    let line_p = co2 + edge_dir * d;
    let co3 = blenders_math::geom::isect_line_plane(
        co2,
        line_p,
        bp.boundverts[v3next].nv.co,
        edge_dir,
    )
    .unwrap_or(co1);
    adjust_bound_vert(bp, bm, v3, co3);
    let _ = v2;
}

/// `adjust_miter_inner_coords`.
fn adjust_miter_inner_coords(bm: &BMesh, bp: &mut BevelParams, bvi: usize, emiter: EHRef) {
    let vstart = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
    let mut v = vstart;
    loop {
        if bp.boundverts[v].is_arc_start {
            let v3 = bp.boundverts[v].next;
            let e = bp.boundverts[v].efirst;
            if e != emiter.1 {
                let co = bp.boundverts[v].nv.co;
                let vother = other_vert(bm, bp.bverts[bvi].edges[e].e, bp.bverts[bvi].v);
                let mut edge_dir = bm.vert(vother).co - bm.vert(bp.bverts[bvi].v).co;
                edge_dir.normalize();
                let d = bp.offset / (bp.seg as f32 / 2.0);
                let line_p = co + edge_dir * d;
                let conew = blenders_math::geom::isect_line_plane(
                    co,
                    line_p,
                    bp.boundverts[v3].nv.co,
                    edge_dir,
                )
                .unwrap_or(line_p);
                adjust_bound_vert(bp, bm, v, conew);
            }
        } else {
            v = bp.boundverts[v].next;
        }
        if v == vstart {
            break;
        }
    }
}

/// `build_boundary`.
fn build_boundary(bm: &BMesh, bp: &mut BevelParams, bvi: usize, construct: bool) {
    if bp.bverts[bvi].edgecount <= 1 {
        return;
    }
    if bp.affect_type == BEVEL_AFFECT_VERTICES {
        build_boundary_vertex_only(bm, bp, bvi, construct);
        return;
    }
    let efirst = next_bev(bp, bvi, EH_NIL);
    debug_assert!(bp.eh(efirst).is_bev);
    if bp.bverts[bvi].selcount == 1 {
        build_boundary_terminal_edge(bm, bp, bvi, efirst, construct);
        return;
    }
    let miter_outer = if bp.bverts[bvi].selcount >= 3 { bp.miter_outer } else { BEVEL_MITER_SHARP };
    let miter_inner = bp.miter_inner;
    let mut emiter = EH_NIL;
    let bvv = bp.bverts[bvi].v;
    let mut e = efirst.1;
    loop {
        debug_assert!(bp.bverts[bvi].edges[e].is_bev);
        let mut eon = EH_NIL;
        let mut in_plane = 0;
        let mut not_in_plane = 0;
        let mut enip = 0usize;
        let mut eip = 0usize;
        let mut e2 = bp.bverts[bvi].edges[e].next;
        while !bp.bverts[bvi].edges[e2].is_bev {
            if eh_on_plane(bm, &bp.bverts[bvi].edges[e2]) {
                in_plane += 1;
                eip = e2;
            } else {
                not_in_plane += 1;
                enip = e2;
            }
            e2 = bp.bverts[bvi].edges[e2].next;
        }
        let mut r = 0f32;
        let co;
        if in_plane == 0 && not_in_plane == 0 {
            co = offset_meet(bm, bp, (bvi, e), (bvi, e2), bvv, bp.bverts[bvi].edges[e].fnext, false, EH_NIL);
        } else if not_in_plane > 0 {
            if bp.loop_slide
                && not_in_plane == 1
                && good_offset_on_edge_between(bm, bp, (bvi, e), (bvi, e2), (bvi, enip), bvv)
            {
                match offset_on_edge_between(bm, bp, (bvi, e), (bvi, e2), (bvi, enip), bvv) {
                    Some((c, sr)) => {
                        co = c;
                        r = sr;
                        eon = (bvi, enip);
                    }
                    None => {
                        co = offset_meet(bm, bp, (bvi, e), (bvi, e2), bvv, BM_NIL, true, (bvi, eip));
                    }
                }
            } else {
                co = offset_meet(bm, bp, (bvi, e), (bvi, e2), bvv, BM_NIL, true, (bvi, eip));
            }
        } else if bp.loop_slide
            && in_plane == 1
            && good_offset_on_edge_between(bm, bp, (bvi, e), (bvi, e2), (bvi, eip), bvv)
        {
            match offset_on_edge_between(bm, bp, (bvi, e), (bvi, e2), (bvi, eip), bvv) {
                Some((c, sr)) => {
                    co = c;
                    r = sr;
                    eon = (bvi, eip);
                }
                None => {
                    co = offset_meet(bm, bp, (bvi, e), (bvi, e2), bvv, bp.bverts[bvi].edges[e].fnext, false, EH_NIL);
                }
            }
        } else {
            co = offset_meet(bm, bp, (bvi, e), (bvi, e2), bvv, bp.bverts[bvi].edges[e].fnext, false, EH_NIL);
        }
        if construct {
            let v = add_new_bound_vert(bp, bvi, co);
            bp.boundverts[v].efirst = e;
            bp.boundverts[v].elast = e2;
            bp.boundverts[v].ebev = e2;
            bp.boundverts[v].eon = eon.1;
            if eon != EH_NIL {
                bp.boundverts[v].sinratio = r;
            }
            bp.bverts[bvi].edges[e].rightv = v;
            bp.bverts[bvi].edges[e2].leftv = v;
            let mut e3 = bp.bverts[bvi].edges[e].next;
            while e3 != e2 {
                bp.bverts[bvi].edges[e3].leftv = v;
                bp.bverts[bvi].edges[e3].rightv = v;
                e3 = bp.bverts[bvi].edges[e3].next;
            }
            let ang_kind = edges_angle_kind(bm, bp, (bvi, e), (bvi, e2), bvv);
            if (miter_outer != BEVEL_MITER_SHARP && emiter == EH_NIL && ang_kind == ANGLE_LARGER)
                || (miter_inner != BEVEL_MITER_SHARP && ang_kind == ANGLE_SMALLER)
            {
                if ang_kind == ANGLE_LARGER {
                    emiter = (bvi, e);
                }
                let v1 = v;
                bp.boundverts[v1].ebev = BM_NIL;
                let v2 = if ang_kind == ANGLE_LARGER && miter_outer == BEVEL_MITER_PATCH {
                    add_new_bound_vert(bp, bvi, co)
                } else {
                    BM_NIL
                };
                let v3 = add_new_bound_vert(bp, bvi, co);
                bp.boundverts[v3].ebev = e2;
                bp.boundverts[v3].efirst = e2;
                bp.boundverts[v3].elast = e2;
                bp.boundverts[v3].eon = BM_NIL;
                bp.bverts[bvi].edges[e2].leftv = v3;
                if ang_kind == ANGLE_LARGER && miter_outer == BEVEL_MITER_PATCH {
                    bp.boundverts[v1].is_patch_start = true;
                    bp.boundverts[v2].eon = bp.boundverts[v1].eon;
                    bp.boundverts[v2].sinratio = bp.boundverts[v1].sinratio;
                    bp.boundverts[v2].ebev = BM_NIL;
                    bp.boundverts[v1].eon = BM_NIL;
                    bp.boundverts[v1].sinratio = 1.0;
                    bp.boundverts[v1].elast = e;
                    if bp.bverts[bvi].edges[e].next == e2 {
                        bp.boundverts[v2].efirst = BM_NIL;
                        bp.boundverts[v2].elast = BM_NIL;
                    } else {
                        bp.boundverts[v2].efirst = bp.bverts[bvi].edges[e].next;
                        let mut e3 = bp.bverts[bvi].edges[e].next;
                        while e3 != e2 {
                            bp.bverts[bvi].edges[e3].leftv = v2;
                            bp.bverts[bvi].edges[e3].rightv = v2;
                            bp.boundverts[v2].elast = e3;
                            e3 = bp.bverts[bvi].edges[e3].next;
                        }
                    }
                } else {
                    bp.boundverts[v1].is_arc_start = true;
                    bp.boundverts[v1].profile.middle = co;
                    if bp.bverts[bvi].edges[e].next == e2 {
                        bp.boundverts[v1].elast = bp.boundverts[v1].efirst;
                    } else {
                        let between = in_plane + not_in_plane;
                        let bet2 = between / 2;
                        let betodd = (between % 2) == 1;
                        let mut i = 0;
                        let mut e3 = bp.bverts[bvi].edges[e].next;
                        while e3 != e2 {
                            bp.boundverts[v1].elast = e3;
                            if i < bet2 {
                                bp.bverts[bvi].edges[e3].profile_index = 0;
                            } else if betodd && i == bet2 {
                                bp.bverts[bvi].edges[e3].profile_index = bp.seg / 2;
                            } else {
                                bp.bverts[bvi].edges[e3].profile_index = bp.seg;
                            }
                            i += 1;
                            e3 = bp.bverts[bvi].edges[e3].next;
                        }
                    }
                }
            }
        } else {
            let ang_kind = edges_angle_kind(bm, bp, (bvi, e), (bvi, e2), bvv);
            if (miter_outer != BEVEL_MITER_SHARP && emiter == EH_NIL && ang_kind == ANGLE_LARGER)
                || (miter_inner != BEVEL_MITER_SHARP && ang_kind == ANGLE_SMALLER)
            {
                if ang_kind == ANGLE_LARGER {
                    emiter = (bvi, e);
                }
                let v1 = bp.bverts[bvi].edges[e].rightv;
                let (v2, v3) = if ang_kind == ANGLE_LARGER && miter_outer == BEVEL_MITER_PATCH {
                    let v2 = bp.boundverts[v1].next;
                    (v2, bp.boundverts[v2].next)
                } else {
                    (BM_NIL, bp.boundverts[v1].next)
                };
                adjust_bound_vert(bp, bm, v1, co);
                if v2 != BM_NIL {
                    adjust_bound_vert(bp, bm, v2, co);
                }
                adjust_bound_vert(bp, bm, v3, co);
            } else {
                adjust_bound_vert(bp, bm, bp.bverts[bvi].edges[e].rightv, co);
            }
        }
        e = e2;
        if e == efirst.1 {
            break;
        }
    }
    if miter_inner != BEVEL_MITER_SHARP {
        adjust_miter_inner_coords(bm, bp, bvi, emiter);
    }
    if emiter != EH_NIL {
        adjust_miter_coords(bm, bp, bvi, emiter);
    }
    if construct {
        set_bound_vert_seams(bp, bm, bvi, bp.mark_seam, bp.mark_sharp);
        let count = bp.bverts[bvi].vmesh.as_ref().unwrap().count;
        let kind = if count == 2 {
            MeshKind::None
        } else if bp.bverts[bvi].edges[efirst.1].seg == 1 {
            MeshKind::Poly
        } else {
            match bp.vmesh_method {
                BEVEL_VMESH_ADJ => MeshKind::Adj,
                BEVEL_VMESH_CUTOFF => MeshKind::Cutoff,
                _ => MeshKind::Adj,
            }
        };
        bp.bverts[bvi].vmesh.as_mut().unwrap().mesh_kind = kind;
    }
}

/* ===== offset adjustment via least squares (C ~3850-4345) ===== */

/// Tiny dense least-squares solver standing in for `EIG_linear_least_squares_solver`.
struct LinearSolver {
    a: Vec<Vec<f64>>,
    b: Vec<f64>,
    np: usize,
    x: Vec<f64>,
}

impl LinearSolver {
    fn new(nrows: usize, np: usize) -> Self {
        Self {
            a: vec![vec![0.0; np]; nrows],
            b: vec![0.0; nrows],
            np,
            x: vec![0.0; np],
        }
    }
    fn matrix_add(&mut self, row: usize, col: usize, v: f64) {
        self.a[row][col] += v;
    }
    fn rhs_add(&mut self, i: usize, v: f64) {
        self.b[i] += v;
    }
    /// Solve via normal equations + Gaussian elimination w/ partial pivoting.
    fn solve(&mut self) {
        let n = self.np;
        let mut ata = vec![vec![0.0f64; n]; n];
        let mut atb = vec![0.0f64; n];
        for r in 0..self.a.len() {
            for i in 0..n {
                let ari = self.a[r][i];
                if ari == 0.0 {
                    continue;
                }
                for j in 0..n {
                    ata[i][j] += ari * self.a[r][j];
                }
                atb[i] += ari * self.b[r];
            }
        }
        for col in 0..n {
            let mut piv = col;
            for r in col + 1..n {
                if ata[r][col].abs() > ata[piv][col].abs() {
                    piv = r;
                }
            }
            ata.swap(col, piv);
            atb.swap(col, piv);
            if ata[col][col] == 0.0 {
                continue;
            }
            for r in col + 1..n {
                let f = ata[r][col] / ata[col][col];
                if f == 0.0 {
                    continue;
                }
                for c in col..n {
                    ata[r][c] -= f * ata[col][c];
                }
                atb[r] -= f * atb[col];
            }
        }
        for i in (0..n).rev() {
            let mut s = atb[i];
            for j in i + 1..n {
                s -= ata[i][j] * self.x[j];
            }
            self.x[i] = if ata[i][i] != 0.0 { s / ata[i][i] } else { 0.0 };
        }
    }
    fn variable_get(&self, i: usize) -> f64 {
        self.x[i]
    }
}

/// `adjust_the_cycle_or_chain` — LLS over the weld chain/cycle offsets.
fn adjust_the_cycle_or_chain(bp: &mut BevelParams, vstart: usize, iscycle: bool) {
    let mut np = 0usize;
    let mut v = vstart;
    loop {
        np += 1;
        v = bp.boundverts[v].adjchain;
        if v == BM_NIL || v == vstart {
            break;
        }
    }
    let nrows = if iscycle { 3 * np } else { 3 * np - 3 };
    let mut solver = LinearSolver::new(nrows, np);
    v = vstart;
    let mut i = 0usize;
    let weight = BEVEL_MATCH_SPEC_WEIGHT;
    loop {
        let bvi = bp.boundverts[v].bv;
        if iscycle || i < np - 1 {
            let eright = bp.boundverts[v].efirst;
            let eleft = bp.boundverts[v].elast;
            let enextleft = bp.boundverts[bp.boundverts[v].adjchain].elast;
            let (off_r, off_l, sinratio, sinratio_next) = (
                bp.bverts[bvi].edges[eright].offset_r,
                bp.bverts[bvi].edges[enextleft].offset_l,
                bp.boundverts[v].sinratio,
                bp.boundverts[bp.boundverts[v].adjchain].sinratio,
            );
            let _ = eleft;
            solver.matrix_add(i, i, 1.0);
            solver.rhs_add(i, 0.0);
            if iscycle {
                solver.matrix_add(if i > 0 { i - 1 } else { np - 1 }, i, -sinratio as f64);
            } else if i > 0 {
                solver.matrix_add(i - 1, i, -sinratio as f64);
            }
            let mut row = if iscycle { np + 2 * i } else { np - 1 + 2 * i };
            solver.matrix_add(row, i, weight);
            solver.rhs_add(row, weight * off_r as f64);
            row += 1;
            solver.matrix_add(row, if i == np - 1 { 0 } else { i + 1 }, weight * sinratio_next as f64);
            solver.rhs_add(row, weight * off_l as f64);
        } else {
            solver.matrix_add(i - 1, i, -1.0);
        }
        i += 1;
        v = bp.boundverts[v].adjchain;
        if v == BM_NIL || v == vstart {
            break;
        }
    }
    solver.solve();
    v = vstart;
    i = 0;
    loop {
        let bvi = bp.boundverts[v].bv;
        let val = solver.variable_get(i);
        if iscycle || i < np - 1 {
            let eright = bp.boundverts[v].efirst;
            let eleft = bp.boundverts[v].elast;
            bp.bverts[bvi].edges[eright].offset_r = val as f32;
            if iscycle || v != vstart {
                bp.bverts[bvi].edges[eleft].offset_l = (bp.boundverts[v].sinratio as f64 * val) as f32;
            }
        } else {
            let eleft = bp.boundverts[v].elast;
            bp.bverts[bvi].edges[eleft].offset_l = val as f32;
        }
        i += 1;
        v = bp.boundverts[v].adjchain;
        if v == BM_NIL || v == vstart {
            break;
        }
    }
}

/// `next_edgehalf_bev` — most-parallel beveled EdgeHalf along a path.
fn next_edgehalf_bev(bm: &BMesh, bp: &BevelParams, start_edge: EHRef, toward_bv: bool, r_bv: &mut usize) -> EHRef {
    if !toward_bv {
        let (e2, bvo) = find_other_end_edge_half(bm, bp, start_edge);
        *r_bv = bvo;
        return e2;
    }
    let bv = *r_bv;
    if bp.bverts[bv].selcount == 1 {
        return EH_NIL;
    }
    if bp.bverts[bv].selcount == 2 {
        let mut new_edge = bp.eh(start_edge).next;
        while !bp.bverts[bv].edges[new_edge].is_bev {
            new_edge = bp.bverts[bv].edges[new_edge].next;
        }
        return (bv, new_edge);
    }
    let se = bp.eh(start_edge);
    let mut dir_start = if bm.edge(se.e).v1 == bp.bverts[bv].v {
        bm.vert(bm.edge(se.e).v1).co - bm.vert(bm.edge(se.e).v2).co
    } else {
        bm.vert(bm.edge(se.e).v2).co - bm.vert(bm.edge(se.e).v1).co
    };
    dir_start.normalize();
    let mut new_edge = bp.eh(start_edge).next;
    let mut second_best_dot = 0.0f32;
    let mut best_dot = 0.0f32;
    let mut next_edge = EH_NIL;
    while new_edge != start_edge.1 {
        if !bp.bverts[bv].edges[new_edge].is_bev {
            new_edge = bp.bverts[bv].edges[new_edge].next;
            continue;
        }
        let ne = &bp.bverts[bv].edges[new_edge];
        let mut dir_new = if bm.edge(ne.e).v2 == bp.bverts[bv].v {
            bm.vert(bm.edge(ne.e).v1).co - bm.vert(bm.edge(ne.e).v2).co
        } else {
            bm.vert(bm.edge(ne.e).v2).co - bm.vert(bm.edge(ne.e).v1).co
        };
        dir_new.normalize();
        let new_dot = dir_new.dot(dir_start);
        if new_dot > best_dot {
            second_best_dot = best_dot;
            best_dot = new_dot;
            next_edge = (bv, new_edge);
        } else if new_dot > second_best_dot {
            second_best_dot = new_dot;
        }
        new_edge = bp.bverts[bv].edges[new_edge].next;
    }
    if next_edge != EH_NIL
        && blenders_math::base::compare_ff(best_dot, second_best_dot, BEVEL_SMALL_ANG_DOT)
    {
        return EH_NIL;
    }
    next_edge
}

/// `regularize_profile_orientation`.
fn regularize_profile_orientation(bm: &BMesh, bp: &mut BevelParams, bme: usize) {
    let start_bv = find_bevvert(bp, bm.edge(bme).v1);
    if start_bv == BM_NIL {
        return;
    }
    let start_edgehalf = find_edge_half(bp, start_bv, bme);
    if start_edgehalf == EH_NIL
        || !bp.eh(start_edgehalf).is_bev
        || bp.eh(start_edgehalf).visited_rpo
    {
        return;
    }
    let right_highest = bp.boundverts[bp.eh(start_edgehalf).leftv].nv.co.z
        < bp.boundverts[bp.eh(start_edgehalf).rightv].nv.co.z;
    let leftv = bp.eh(start_edgehalf).leftv;
    bp.boundverts[leftv].is_profile_start = right_highest;
    bp.bverts[start_bv].edges[start_edgehalf.1].visited_rpo = true;
    for i in 0..2 {
        let mut edgehalf = start_edgehalf;
        let mut bv = start_bv;
        let mut toward_bv = i == 0;
        edgehalf = next_edgehalf_bev(bm, bp, edgehalf, toward_bv, &mut bv);
        while edgehalf != EH_NIL && !bp.eh(edgehalf).visited_rpo {
            let leftv = bp.eh(edgehalf).leftv;
            bp.boundverts[leftv].is_profile_start = if i == 0 {
                toward_bv ^ right_highest
            } else {
                (!toward_bv) ^ right_highest
            };
            toward_bv = !toward_bv;
            bp.bverts[bv].edges[edgehalf.1].visited_rpo = true;
            edgehalf = next_edgehalf_bev(bm, bp, edgehalf, toward_bv, &mut bv);
        }
    }
}

/// `adjust_offsets` — solve weld chains/cycles, rebuild boundaries.
fn adjust_offsets(bm: &mut BMesh, bp: &mut BevelParams) {
    const TAG: u8 = crate::BM_ELEM_TAG;
    for bmv in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(bmv).head.hflag & TAG == 0 {
            continue;
        }
        let bv = find_bevvert(bp, bmv);
        if bv == BM_NIL {
            continue;
        }
        let boundstart = bp.bverts[bv].vmesh.as_ref().unwrap().boundstart;
        let mut vanchor = boundstart;
        loop {
            if !bp.boundverts[vanchor].visited && bp.boundverts[vanchor].eon != BM_NIL {
                let mut vchainstart = vanchor;
                let mut vchainend = vanchor;
                let mut iscycle = false;
                let mut chainlen = 1usize;
                let mut v = vanchor;
                let mut bvcur = bv;
                while bp.boundverts[v].eon != BM_NIL && !bp.boundverts[v].visited && !iscycle {
                    bp.boundverts[v].visited = true;
                    if bp.boundverts[v].efirst == BM_NIL {
                        break;
                    }
                    let (enext, bvo) = find_other_end_edge_half(bm, bp, (bvcur, bp.boundverts[v].efirst));
                    bvcur = bvo;
                    if enext == EH_NIL {
                        break;
                    }
                    let vnext = bp.bverts[enext.0].edges[enext.1].leftv;
                    bp.boundverts[v].adjchain = vnext;
                    vchainend = vnext;
                    chainlen += 1;
                    if bp.boundverts[vnext].visited {
                        if vnext != vchainstart {
                            break;
                        }
                        adjust_the_cycle_or_chain(bp, vchainstart, true);
                        iscycle = true;
                    }
                    v = vnext;
                }
                if !iscycle {
                    bp.boundverts[v].adjchain = BM_NIL;
                    v = vchainstart;
                    bvcur = bv;
                    loop {
                        bp.boundverts[v].visited = true;
                        if bp.boundverts[v].elast == BM_NIL {
                            break;
                        }
                        let (enext, bvo) =
                            find_other_end_edge_half(bm, bp, (bvcur, bp.boundverts[v].elast));
                        bvcur = bvo;
                        if enext == EH_NIL {
                            break;
                        }
                        let vnext = bp.bverts[enext.0].edges[enext.1].rightv;
                        bp.boundverts[vnext].adjchain = v;
                        chainlen += 1;
                        vchainstart = vnext;
                        v = vnext;
                        if bp.boundverts[v].visited || bp.boundverts[v].eon == BM_NIL {
                            break;
                        }
                    }
                    if chainlen >= 3
                        && bp.boundverts[vchainstart].eon == BM_NIL
                        && bp.boundverts[vchainend].eon == BM_NIL
                    {
                        adjust_the_cycle_or_chain(bp, vchainstart, false);
                    }
                }
            }
            vanchor = bp.boundverts[vanchor].next;
            if vanchor == boundstart {
                break;
            }
        }
    }
    for bmv in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(bmv).head.hflag & TAG != 0 {
            let bv = find_bevvert(bp, bmv);
            if bv != BM_NIL {
                build_boundary(bm, bp, bv, false);
            }
        }
    }
}

/* ===== vmesh construction (C ~4345-5250) ===== */

/// `pipe_test` — boundvert whose two ebev edges form a pipe, else None.
fn pipe_test(bm: &BMesh, bp: &BevelParams, bvi: usize) -> usize {
    let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
    if vm.count < 3 || vm.count > 4 || bp.bverts[bvi].selcount < 3 || bp.bverts[bvi].selcount > 4 {
        return BM_NIL;
    }
    let mut epipe = BM_NIL;
    let mut v1 = vm.boundstart;
    let boundstart = vm.boundstart;
    let mut dir1 = Vec3::ZERO;
    let mut dir3 = Vec3::ZERO;
    let mut v1_found = BM_NIL;
    loop {
        let v2 = bp.boundverts[v1].next;
        let v3 = bp.boundverts[v2].next;
        if bp.boundverts[v1].ebev != BM_NIL
            && bp.boundverts[v2].ebev != BM_NIL
            && bp.boundverts[v3].ebev != BM_NIL
        {
            let e1 = bp.bverts[bvi].edges[bp.boundverts[v1].ebev].e;
            let e3 = bp.bverts[bvi].edges[bp.boundverts[v3].ebev].e;
            dir1 = bm.vert(bp.bverts[bvi].v).co
                - bm.vert(other_vert(bm, e1, bp.bverts[bvi].v)).co;
            dir3 = bm.vert(other_vert(bm, e3, bp.bverts[bvi].v)).co - bm.vert(bp.bverts[bvi].v).co;
            dir1.normalize();
            dir3.normalize();
            if blenders_math::geom::angle_normalized_v3v3(dir1, dir3) < BEVEL_EPSILON_ANG {
                epipe = bp.boundverts[v1].ebev;
                v1_found = v1;
                break;
            }
        }
        v1 = bp.boundverts[v1].next;
        if v1 == boundstart {
            break;
        }
    }
    let _ = epipe;
    if v1_found == BM_NIL {
        return BM_NIL;
    }
    for ei in 0..bp.bverts[bvi].edgecount as usize {
        let fnext = bp.bverts[bvi].edges[ei].fnext;
        if fnext != BM_NIL && dir1.dot(bm.face(fnext).no).abs() > BEVEL_EPSILON_BIG {
            return BM_NIL;
        }
    }
    v1_found
}

/// `new_adj_vmesh` — empty VMesh with the canonical-index size.
fn new_adj_vmesh(count: i32, seg: i32, boundstart: usize) -> VMesh {
    VMesh {
        mesh: vec![
            NewVert {
                v: BM_NIL,
                co: Vec3::ZERO
            };
            (count * (1 + seg / 2) * (1 + seg)) as usize
        ],
        boundstart,
        count,
        seg,
        mesh_kind: MeshKind::Adj,
    }
}

/// `add_new_bound_vert` variant for a standalone VMesh (cube-corner builders).
fn add_bound_vert_vm(bp: &mut BevelParams, vm: &mut VMesh, co: Vec3) -> usize {
    let bndv = BoundVert {
        nv: NewVert { v: BM_NIL, co },
        index: vm.count,
        sinratio: 1.0,
        ..BoundVert::default()
    };
    bp.boundverts.push(bndv);
    let idx = bp.boundverts.len() - 1;
    if vm.count == 0 || vm.boundstart == BM_NIL {
        vm.boundstart = idx;
        bp.boundverts[idx].next = idx;
        bp.boundverts[idx].prev = idx;
    } else {
        let bs = vm.boundstart;
        let prev = bp.boundverts[bs].prev;
        bp.boundverts[idx].next = bs;
        bp.boundverts[idx].prev = prev;
        bp.boundverts[prev].next = idx;
        bp.boundverts[bs].prev = idx;
    }
    vm.count += 1;
    idx
}

/// `mesh_vert_canon` — canonical index for equivalent (i,j,k).
fn mesh_vert_canon(vm: &VMesh, i: i32, j: i32, k: i32) -> usize {
    let n = vm.count;
    let ns = vm.seg;
    let ns2 = ns / 2;
    let odd = ns % 2;
    debug_assert!(0 <= i && i <= n && 0 <= j && j <= ns && 0 <= k && k <= ns);
    if odd == 0 && j == ns2 && k == ns2 {
        return mesh_vert(vm, 0, j, k);
    }
    if j <= ns2 - 1 + odd && k <= ns2 {
        return mesh_vert(vm, i, j, k);
    }
    if k <= ns2 {
        return mesh_vert(vm, (i + n - 1) % n, k, ns - j);
    }
    mesh_vert(vm, (i + 1) % n, ns - k, j)
}

/// `is_canon`.
fn is_canon(vm: &VMesh, i: i32, j: i32, k: i32) -> bool {
    let ns2 = vm.seg / 2;
    if vm.seg % 2 == 1 {
        return j <= ns2 && k <= ns2;
    }
    (j < ns2 && k <= ns2) || (j == ns2 && k == ns2 && i == 0)
}

/// `vmesh_copy_equiv_verts`.
fn vmesh_copy_equiv_verts(vm: &mut VMesh) {
    let n = vm.count;
    let ns = vm.seg;
    let ns2 = ns / 2;
    for i in 0..n {
        for j in 0..=ns2 {
            for k in 0..=ns {
                if is_canon(vm, i, j, k) {
                    continue;
                }
                let c = mesh_vert_canon(vm, i, j, k);
                let idx = mesh_vert(vm, i, j, k);
                vm.mesh[idx].co = vm.mesh[c].co;
                vm.mesh[idx].v = vm.mesh[c].v;
            }
        }
    }
}

/// `vmesh_center`.
fn vmesh_center(vm: &VMesh) -> Vec3 {
    let n = vm.count;
    let ns2 = vm.seg / 2;
    if vm.seg % 2 == 1 {
        let mut cent = Vec3::ZERO;
        for i in 0..n {
            cent += vm.mesh[mesh_vert(vm, i, ns2, ns2)].co;
        }
        cent / n as f32
    } else {
        vm.mesh[mesh_vert(vm, 0, ns2, ns2)].co
    }
}

/// `avg4`.
fn avg4(v0: &NewVert, v1: &NewVert, v2: &NewVert, v3: &NewVert) -> Vec3 {
    (v0.co + v1.co + v2.co + v3.co) * 0.25
}

/// `sabin_gamma`.
fn sabin_gamma(n: i32) -> f32 {
    if n < 3 {
        return 0.0;
    }
    match n {
        3 => return 0.065247584,
        4 => return 0.25,
        5 => return 0.401983447,
        6 => return 0.523423277,
        _ => {}
    }
    let n = n as f64;
    let k = (std::f64::consts::PI / n).cos();
    let k2 = k * k;
    let k4 = k2 * k2;
    let k6 = k4 * k2;
    let y = (3f64.sqrt() * (64.0 * k6 - 144.0 * k4 + 135.0 * k2 - 27.0).sqrt() + 9.0 * k)
        .powf(1.0 / 3.0);
    let x = 0.480749856769136 * y - (0.231120424783545 * (12.0 * k2 - 9.0)) / y;
    ((k * x + 2.0 * k2 - 1.0) / (x * x * (k * x + 1.0))) as f32
}

/// `fill_vmesh_fracs` — cumulative arc-length fractions along boundary i.
fn fill_vmesh_fracs(vm: &VMesh, frac: &mut [f32], i: i32) {
    let ns = vm.seg;
    let mut total = 0.0f32;
    frac[0] = 0.0;
    for k in 0..ns as usize {
        total += (vm.mesh[mesh_vert(vm, i, 0, k as i32)].co
            - vm.mesh[mesh_vert(vm, i, 0, k as i32 + 1)].co)
            .length();
        frac[k + 1] = total;
    }
    if total > 0.0 {
        for k in 1..=ns as usize {
            frac[k] /= total;
        }
    } else {
        frac[ns as usize] = 1.0;
    }
}

/// `fill_profile_fracs`.
fn fill_profile_fracs(bp: &BevelParams, bndv: &BoundVert, frac: &mut [f32], ns: i32) {
    let mut total = 0.0f32;
    frac[0] = 0.0;
    let mut co = bndv.nv.co;
    for k in 0..ns as usize {
        let nextco = get_profile_point(bp, &bndv.profile, k as i32 + 1, ns);
        total += (co - nextco).length();
        frac[k + 1] = total;
        co = nextco;
    }
    if total > 0.0 {
        for k in 1..=ns as usize {
            frac[k] /= total;
        }
    } else {
        frac[ns as usize] = 1.0;
    }
}

/// `interp_range` — segment index containing fraction f + residual.
fn interp_range(frac: &[f32], n: i32, f: f32) -> (i32, f32) {
    for i in 0..n as usize {
        if f <= frac[i + 1] {
            let mut i = i as i32;
            let mut rest = f - frac[i as usize];
            if rest == 0.0 {
                rest = 0.0;
            } else {
                rest /= frac[i as usize + 1] - frac[i as usize];
            }
            if i == n - 1 && rest == 1.0 {
                i = n;
            }
            return (i, rest);
        }
    }
    (n, 0.0)
}

/// `interp_vmesh` — resample an ADJ vmesh to nseg.
fn interp_vmesh(bp: &BevelParams, vm_in: &VMesh, nseg: i32) -> VMesh {
    let n_bndv = vm_in.count;
    let ns_in = vm_in.seg;
    let nseg2 = nseg / 2;
    let odd = nseg % 2;
    let mut vm_out = new_adj_vmesh(n_bndv, nseg, vm_in.boundstart);
    let mut prev_frac = vec![0.0f32; ns_in as usize + 1];
    let mut frac = vec![0.0f32; ns_in as usize + 1];
    let mut new_frac = vec![0.0f32; nseg as usize + 1];
    let mut prev_new_frac = vec![0.0f32; nseg as usize + 1];
    fill_vmesh_fracs(vm_in, &mut prev_frac, n_bndv - 1);
    let mut bndv = vm_in.boundstart;
    let start = bndv;
    fill_profile_fracs(
        bp,
        &bp.boundverts[bp.boundverts[bndv].prev],
        &mut prev_new_frac,
        nseg,
    );
    for i in 0..n_bndv {
        fill_vmesh_fracs(vm_in, &mut frac, i);
        fill_profile_fracs(bp, &bp.boundverts[bndv], &mut new_frac, nseg);
        for j in 0..=(nseg2 - 1 + odd) {
            for k in 0..=nseg2 {
                let mut fraction = new_frac[k as usize];
                let (k_in, restk) = interp_range(&frac, ns_in, fraction);
                fraction = prev_new_frac[(nseg - j) as usize];
                let (k_in_prev, restkprev) = interp_range(&prev_frac, ns_in, fraction);
                let mut j_in = ns_in - k_in_prev;
                let mut restj = -restkprev;
                if restj > -BEVEL_EPSILON {
                    restj = 0.0;
                } else {
                    j_in -= 1;
                    restj = 1.0 + restj;
                }
                let co = if restj < BEVEL_EPSILON && restk < BEVEL_EPSILON {
                    vm_in.mesh[mesh_vert_canon(vm_in, i, j_in, k_in)].co
                } else {
                    let j0inc = if restj < BEVEL_EPSILON || j_in == ns_in { 0 } else { 1 };
                    let k0inc = if restk < BEVEL_EPSILON || k_in == ns_in { 0 } else { 1 };
                    let quad = [
                        vm_in.mesh[mesh_vert_canon(vm_in, i, j_in, k_in)].co,
                        vm_in.mesh[mesh_vert_canon(vm_in, i, j_in, k_in + k0inc)].co,
                        vm_in.mesh[mesh_vert_canon(vm_in, i, j_in + j0inc, k_in + k0inc)].co,
                        vm_in.mesh[mesh_vert_canon(vm_in, i, j_in + j0inc, k_in)].co,
                    ];
                    blenders_math::geom::interp_bilinear_quad_v3(&quad, restk, restj)
                };
                let idx = mesh_vert(&vm_out, i, j, k);
                vm_out.mesh[idx].co = co;
            }
        }
        bndv = bp.boundverts[bndv].next;
        prev_frac.copy_from_slice(&frac);
        prev_new_frac.copy_from_slice(&new_frac);
    }
    let _ = start;
    if odd == 0 {
        let center = vmesh_center(vm_in);
        let idx = mesh_vert(&vm_out, 0, nseg2, nseg2);
        vm_out.mesh[idx].co = center;
    }
    vmesh_copy_equiv_verts(&mut vm_out);
    vm_out
}

/// `cubic_subdiv` — one cubic-subdivision step doubling segments.
fn cubic_subdiv(bp: &BevelParams, vm_in: &VMesh) -> VMesh {
    let n_boundary = vm_in.count;
    let ns_in = vm_in.seg;
    let ns_in2 = ns_in / 2;
    debug_assert!(ns_in % 2 == 0);
    let ns_out = 2 * ns_in;
    let mut vm_out = new_adj_vmesh(n_boundary, ns_out, vm_in.boundstart);
    let mut co;
    for i in 0..n_boundary {
        let idx = mesh_vert(&vm_out, i, 0, 0);
        vm_out.mesh[idx].co = vm_in.mesh[mesh_vert(vm_in, i, 0, 0)].co;
        for k in 1..ns_in {
            co = vm_in.mesh[mesh_vert(vm_in, i, 0, k)].co;
            if bp.profile_type != BEVEL_PROFILE_CUSTOM {
                let co1 = vm_in.mesh[mesh_vert(vm_in, i, 0, k - 1)].co;
                let co2 = vm_in.mesh[mesh_vert(vm_in, i, 0, k + 1)].co;
                let mut acc = co1 + co2;
                acc += co * -2.0;
                co += acc * (-1.0 / 6.0);
            }
            let idx = mesh_vert_canon(&vm_out, i, 0, 2 * k);
            vm_out.mesh[idx].co = co;
        }
    }
    let mut bndv = vm_out.boundstart;
    for i in 0..n_boundary {
        for k in (1..ns_out).step_by(2) {
            co = get_profile_point(bp, &bp.boundverts[bndv].profile, k, ns_out);
            if bp.profile_type != BEVEL_PROFILE_CUSTOM {
                let co1 = vm_out.mesh[mesh_vert_canon(&vm_out, i, 0, k - 1)].co;
                let co2 = vm_out.mesh[mesh_vert_canon(&vm_out, i, 0, k + 1)].co;
                let mut acc = co1 + co2;
                acc += co * -2.0;
                co += acc * (-1.0 / 6.0);
            }
            let idx = mesh_vert_canon(&vm_out, i, 0, k);
            vm_out.mesh[idx].co = co;
        }
        bndv = bp.boundverts[bndv].next;
    }
    vmesh_copy_equiv_verts(&mut vm_out);
    let mut vm_in_mut = vm_in.clone();
    for i in 0..n_boundary {
        for k in 0..ns_in {
            let im = mesh_vert(&vm_in_mut, i, 0, k);
            vm_in_mut.mesh[im].co = vm_out.mesh[mesh_vert(&vm_out, i, 0, 2 * k)].co;
        }
    }
    vmesh_copy_equiv_verts(&mut vm_in_mut);
    let vm_in = &vm_in_mut;
    for i in 0..n_boundary {
        for j in 0..ns_in2 {
            for k in 0..ns_in2 {
                co = avg4(
                    &vm_in.mesh[mesh_vert(vm_in, i, j, k)],
                    &vm_in.mesh[mesh_vert(vm_in, i, j, k + 1)],
                    &vm_in.mesh[mesh_vert(vm_in, i, j + 1, k)],
                    &vm_in.mesh[mesh_vert(vm_in, i, j + 1, k + 1)],
                );
                let idx = mesh_vert(&vm_out, i, 2 * j + 1, 2 * k + 1);
                vm_out.mesh[idx].co = co;
            }
        }
    }
    for i in 0..n_boundary {
        for j in 0..ns_in2 {
            for k in 1..=ns_in2 {
                co = avg4(
                    &vm_in.mesh[mesh_vert(vm_in, i, j, k)],
                    &vm_in.mesh[mesh_vert(vm_in, i, j + 1, k)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j + 1, 2 * k - 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j + 1, 2 * k + 1)],
                );
                let idx = mesh_vert(&vm_out, i, 2 * j + 1, 2 * k);
                vm_out.mesh[idx].co = co;
            }
        }
    }
    for i in 0..n_boundary {
        for j in 1..ns_in2 {
            for k in 0..ns_in2 {
                co = avg4(
                    &vm_in.mesh[mesh_vert(vm_in, i, j, k)],
                    &vm_in.mesh[mesh_vert(vm_in, i, j, k + 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j - 1, 2 * k + 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j + 1, 2 * k + 1)],
                );
                let idx = mesh_vert(&vm_out, i, 2 * j, 2 * k + 1);
                vm_out.mesh[idx].co = co;
            }
        }
    }
    let mut gamma = 0.25f32;
    let mut beta = -gamma;
    for i in 0..n_boundary {
        for j in 1..ns_in2 {
            for k in 1..=ns_in2 {
                let co1 = avg4(
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j, 2 * k - 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j, 2 * k + 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j - 1, 2 * k)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j + 1, 2 * k)],
                );
                let co2 = avg4(
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j - 1, 2 * k - 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j + 1, 2 * k - 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j - 1, 2 * k + 1)],
                    &vm_out.mesh[mesh_vert_canon(&vm_out, i, 2 * j + 1, 2 * k + 1)],
                );
                co = co1;
                co += co2 * beta;
                co += vm_in.mesh[mesh_vert(vm_in, i, j, k)].co * gamma;
                let idx = mesh_vert(&vm_out, i, 2 * j, 2 * k);
                vm_out.mesh[idx].co = co;
            }
        }
    }
    vmesh_copy_equiv_verts(&mut vm_out);
    gamma = sabin_gamma(n_boundary);
    beta = -gamma;
    let mut co1 = Vec3::ZERO;
    let mut co2 = Vec3::ZERO;
    for i in 0..n_boundary {
        co1 += vm_out.mesh[mesh_vert(&vm_out, i, ns_in, ns_in - 1)].co;
        co2 += vm_out.mesh[mesh_vert(&vm_out, i, ns_in - 1, ns_in - 1)].co;
        co2 += vm_out.mesh[mesh_vert(&vm_out, i, ns_in - 1, ns_in + 1)].co;
    }
    co = co1 / n_boundary as f32;
    co += co2 * (beta / (2.0 * n_boundary as f32));
    co += vm_in.mesh[mesh_vert(vm_in, 0, ns_in2, ns_in2)].co * gamma;
    for i in 0..n_boundary {
        let idx = mesh_vert(&vm_out, i, ns_in, ns_in);
        vm_out.mesh[idx].co = co;
    }
    bndv = vm_out.boundstart;
    for i in 0..n_boundary {
        let inext = (i + 1) % n_boundary;
        for k in 0..=ns_out {
            co = get_profile_point(bp, &bp.boundverts[bndv].profile, k, ns_out);
            let idx = mesh_vert(&vm_out, i, 0, k);
            vm_out.mesh[idx].co = co;
            if k >= ns_in && k < ns_out {
                let idx = mesh_vert(&vm_out, inext, ns_out - k, 0);
                vm_out.mesh[idx].co = co;
            }
        }
        bndv = bp.boundverts[bndv].next;
    }
    vm_out
}

/// `make_cube_corner_square`.
fn make_cube_corner_square(bp: &mut BevelParams, nseg: i32) -> VMesh {
    let ns2 = nseg / 2;
    let mut vm = new_adj_vmesh(3, nseg, BM_NIL);
    vm.count = 0;
    for i in 0..3 {
        let mut co = Vec3::ZERO;
        co[i as usize] = 1.0;
        add_bound_vert_vm(bp, &mut vm, co);
    }
    for i in 0..3 {
        for j in 0..=ns2 {
            for k in 0..=ns2 {
                if !is_canon(&vm, i, j, k) {
                    continue;
                }
                let mut co = Vec3::ZERO;
                co[i as usize] = 1.0;
                co[((i + 1) % 3) as usize] = k as f32 * 2.0 / nseg as f32;
                co[((i + 2) % 3) as usize] = j as f32 * 2.0 / nseg as f32;
                let idx = mesh_vert(&vm, i, j, k);
                vm.mesh[idx].co = co;
            }
        }
    }
    vmesh_copy_equiv_verts(&mut vm);
    vm
}

/// `make_cube_corner_square_in`.
fn make_cube_corner_square_in(bp: &mut BevelParams, nseg: i32) -> VMesh {
    let ns2 = nseg / 2;
    let odd = nseg % 2;
    let mut vm = new_adj_vmesh(3, nseg, BM_NIL);
    vm.count = 0;
    for i in 0..3 {
        let mut co = Vec3::ZERO;
        co[i as usize] = 1.0;
        add_bound_vert_vm(bp, &mut vm, co);
    }
    let b = if odd != 0 {
        2.0 / (2.0 * ns2 as f32 + std::f32::consts::SQRT_2)
    } else {
        2.0 / nseg as f32
    };
    for i in 0..3 {
        for k in 0..=ns2 {
            let mut co = Vec3::ZERO;
            co[i as usize] = 1.0 - k as f32 * b;
            co[((i + 1) % 3) as usize] = 0.0;
            co[((i + 2) % 3) as usize] = 0.0;
            let idx = mesh_vert(&vm, i, 0, k);
            vm.mesh[idx].co = co;
            co[((i + 1) % 3) as usize] = 1.0 - k as f32 * b;
            co[((i + 2) % 3) as usize] = 0.0;
            co[i as usize] = 0.0;
            let idx = mesh_vert(&vm, i, 0, nseg - k);
            vm.mesh[idx].co = co;
        }
    }
    vm
}

/// `make_cube_corner_adj_vmesh`.
fn make_cube_corner_adj_vmesh(bp: &mut BevelParams) -> VMesh {
    let nseg = bp.seg;
    let r = bp.pro_super_r;
    if bp.profile_type != BEVEL_PROFILE_CUSTOM {
        if r == PRO_SQUARE_R {
            return make_cube_corner_square(bp, nseg);
        }
        if r == PRO_SQUARE_IN_R {
            return make_cube_corner_square_in(bp, nseg);
        }
    }
    let mut vm0 = new_adj_vmesh(3, 2, BM_NIL);
    vm0.count = 0;
    for i in 0..3 {
        let mut co = Vec3::ZERO;
        co[i as usize] = 1.0;
        add_bound_vert_vm(bp, &mut vm0, co);
    }
    let mut bndv = vm0.boundstart;
    for i in 0..3 {
        let mut coc = Vec3::ZERO;
        coc[i as usize] = 1.0;
        coc[((i + 1) % 3) as usize] = 1.0;
        coc[((i + 2) % 3) as usize] = 0.0;
        bp.boundverts[bndv].profile.super_r = r;
        let s = bp.boundverts[bndv].nv.co;
        let e = bp.boundverts[bp.boundverts[bndv].next].nv.co;
        bp.boundverts[bndv].profile.start = s;
        bp.boundverts[bndv].profile.end = e;
        bp.boundverts[bndv].profile.middle = coc;
        bp.boundverts[bndv].profile.plane_co = s;
        bp.boundverts[bndv].profile.plane_no = s.cross(e);
        bp.boundverts[bndv].profile.proj_dir = bp.boundverts[bndv].profile.plane_no;
        let mut tmp = bp.boundverts[bndv].clone();
        calculate_profile(bp, &mut tmp, false, false);
        bp.boundverts[bndv].profile = tmp.profile;
        let idx = mesh_vert(&vm0, i, 0, 0);
        vm0.mesh[idx].co = bp.boundverts[bndv].profile.start;
        let im = mesh_vert(&vm0, i, 0, 1);
        vm0.mesh[im].co = get_profile_point(bp, &bp.boundverts[bndv].profile, 1, 2);
        bndv = bp.boundverts[bndv].next;
    }
    let mut co = Vec3::splat(0.5773502691896258);
    if nseg > 2 {
        if r > 1.5 {
            co *= 1.4;
        } else if r < 0.75 {
            co *= 0.6;
        }
    }
    let idx = mesh_vert(&vm0, 0, 1, 1);
    vm0.mesh[idx].co = co;
    vmesh_copy_equiv_verts(&mut vm0);
    let mut vm1 = vm0;
    while vm1.seg < nseg && (vm1.seg * 2) <= nseg {
        vm1 = cubic_subdiv(bp, &vm1);
    }
    if vm1.seg != nseg {
        vm1 = interp_vmesh(bp, &vm1, nseg);
    }
    let ns2 = nseg / 2;
    for i in 0..3 {
        for j in 0..=ns2 {
            for k in 0..=nseg {
                let idx = mesh_vert(&vm1, i, j, k);
                snap_to_superellipsoid(&mut vm1.mesh[idx].co, r, false);
            }
        }
    }
    vm1
}

/// `tri_corner_test`.
fn tri_corner_test(bm: &BMesh, bp: &BevelParams, bvi: usize) -> i32 {
    let mut in_plane_e = 0;
    if bp.affect_type == BEVEL_AFFECT_VERTICES || bp.profile_type == BEVEL_PROFILE_CUSTOM {
        return -1;
    }
    if bp.bverts[bvi].vmesh.as_ref().unwrap().count != 3 {
        return 0;
    }
    let offset = bp.bverts[bvi].edges[0].offset_l;
    let mut totang = 0.0f32;
    for i in 0..bp.bverts[bvi].edgecount as usize {
        let e = &bp.bverts[bvi].edges[i];
        let ang = crate::query::bm_edge_calc_face_angle_signed_ex(bm, e.e, 0.0);
        let absang = ang.abs();
        if absang <= std::f32::consts::FRAC_PI_4 {
            in_plane_e += 1;
        } else if absang >= 3.0 * std::f32::consts::FRAC_PI_4 {
            return -1;
        }
        if e.is_bev && !blenders_math::base::compare_ff(e.offset_l, offset, BEVEL_EPSILON) {
            return -1;
        }
        totang += ang;
    }
    if in_plane_e != bp.bverts[bvi].edgecount - 3 {
        return -1;
    }
    let angdiff = (totang.abs() - 3.0 * std::f32::consts::FRAC_PI_2).abs();
    if (bp.pro_super_r == PRO_SQUARE_R && angdiff > std::f32::consts::PI / 16.0)
        || angdiff > std::f32::consts::FRAC_PI_4
    {
        return -1;
    }
    if bp.bverts[bvi].edgecount != 3 || bp.bverts[bvi].selcount != 3 {
        return 0;
    }
    1
}

/// `tri_corner_adj_vmesh`.
fn tri_corner_adj_vmesh(bm: &BMesh, bp: &mut BevelParams, bvi: usize) -> VMesh {
    let boundstart = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
    let co0 = bp.boundverts[boundstart].nv.co;
    let co1 = bp.boundverts[bp.boundverts[boundstart].next].nv.co;
    let co2 = bp.boundverts[bp.boundverts[bp.boundverts[boundstart].next].next].nv.co;
    let mat = make_unit_cube_map(co0, co1, co2, bm.vert(bp.bverts[bvi].v).co);
    let ns = bp.seg;
    let ns2 = ns / 2;
    let mut vm = make_cube_corner_adj_vmesh(bp);
    for i in 0..3 {
        for j in 0..=ns2 {
            for k in 0..=ns {
                let idx = mesh_vert(&vm, i, j, k);
                let v = vm.mesh[idx].co;
                vm.mesh[idx].co = blenders_math::matrix_c::mul_v3_m4v3(mat, v);
            }
        }
    }
    vm
}

/// `adj_vmesh` — main ADJ vmesh: subdivide from seg-2 seed.
fn adj_vmesh(bp: &mut BevelParams, bm: &BMesh, bvi: usize) -> VMesh {
    let n_bndv = bp.bverts[bvi].vmesh.as_ref().unwrap().count;
    if n_bndv == 3
        && tri_corner_test(bm, bp, bvi) != -1
        && bp.pro_super_r != PRO_SQUARE_IN_R
    {
        return tri_corner_adj_vmesh(bm, bp, bvi);
    }
    let nseg = bp.bverts[bvi].vmesh.as_ref().unwrap().seg;
    let boundstart = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
    let mut vm0 = new_adj_vmesh(n_bndv, 2, boundstart);
    let mut bndv = boundstart;
    let mut boundverts_center = Vec3::ZERO;
    for i in 0..n_bndv {
        let im0 = mesh_vert(&vm0, i, 0, 0);
        vm0.mesh[im0].co = bp.boundverts[bndv].nv.co;
        let im = mesh_vert(&vm0, i, 0, 1);
        vm0.mesh[im].co = get_profile_point(bp, &bp.boundverts[bndv].profile, 1, 2);
        boundverts_center += bp.boundverts[bndv].nv.co;
        bndv = bp.boundverts[bndv].next;
    }
    boundverts_center /= n_bndv as f32;
    let original_vertex = bm.vert(bp.bverts[bvi].v).co;
    let negative_fullest = (boundverts_center - original_vertex) + boundverts_center;
    let mut fullness = bp.pro_spacing.fullness as f32;
    let center_direction = original_vertex - boundverts_center;
    if center_direction.length_squared() > BEVEL_EPSILON_SQ {
        if bp.profile_type == BEVEL_PROFILE_CUSTOM {
            fullness *= 2.0;
            let im = mesh_vert(&vm0, 0, 1, 1);
            vm0.mesh[im].co = negative_fullest + center_direction * fullness;
        } else {
            let im = mesh_vert(&vm0, 0, 1, 1);
            vm0.mesh[im].co = boundverts_center + center_direction * fullness;
        }
    } else {
        let im = mesh_vert(&vm0, 0, 1, 1);
        vm0.mesh[im].co = boundverts_center;
    }
    vmesh_copy_equiv_verts(&mut vm0);
    let mut vm1 = vm0;
    loop {
        vm1 = cubic_subdiv(bp, &vm1);
        if vm1.seg >= nseg {
            break;
        }
    }
    if vm1.seg != nseg {
        vm1 = interp_vmesh(bp, &vm1, nseg);
    }
    vm1
}

/// `snap_to_pipe_profile`.
fn snap_to_pipe_profile(bm: &BMesh, bp: &BevelParams, vpipe: usize, midline: bool, co: &mut Vec3) {
    let bvi = bp.boundverts[vpipe].bv;
    let pro = &bp.boundverts[vpipe].profile;
    let e = bp.boundverts[vpipe].ebev;
    if (pro.start - pro.end).length_squared() < BEVEL_EPSILON_D * BEVEL_EPSILON_D {
        *co = pro.start;
        return;
    }
    let ee = bp.bverts[bvi].edges[e].e;
    let edir = bm.vert(bm.edge(ee).v1).co - bm.vert(bm.edge(ee).v2).co;
    let plane = blenders_math::geom::plane_from_point_normal(*co, edir);
    let start_plane = blenders_math::geom::closest_to_plane(plane, pro.start);
    let end_plane = blenders_math::geom::closest_to_plane(plane, pro.end);
    let middle_plane = blenders_math::geom::closest_to_plane(plane, pro.middle);
    let super_r = pro.super_r;
    if let Some(m) = make_unit_square_map(start_plane, middle_plane, end_plane) {
        if let Some(minv) = m.inverted() {
            let mut p = blenders_math::matrix_c::mul_v3_m4v3(minv, *co);
            snap_to_superellipsoid(&mut p, super_r, midline);
            *co = blenders_math::matrix_c::mul_v3_m4v3(m, p);
            return;
        }
    }
    *co = blenders_math::geom::closest_to_line_segment_v3(*co, start_plane, end_plane).0;
}

/// `pipe_adj_vmesh`.
fn pipe_adj_vmesh(bp: &mut BevelParams, bm: &BMesh, bvi: usize, vpipe: usize) -> VMesh {
    let mut vm = adj_vmesh(bp, bm, bvi);
    let n_bndv = bp.bverts[bvi].vmesh.as_ref().unwrap().count;
    let ns = bp.bverts[bvi].vmesh.as_ref().unwrap().seg;
    let half_ns = ns / 2;
    let ipipe1 = bp.boundverts[vpipe].index;
    let ipipe2 = bp.boundverts[bp.boundverts[bp.boundverts[vpipe].next].next].index;
    for i in 0..n_bndv {
        for j in 1..=half_ns {
            for k in 0..=half_ns {
                if !is_canon(&vm, i, j, k) {
                    continue;
                }
                if bp.profile_type == BEVEL_PROFILE_CUSTOM {
                    let (p1, p2, f);
                    if i == ipipe1 || i == ipipe2 {
                        if n_bndv == 3 && i == ipipe1 {
                            let ring = j.max(k);
                            p2 = vm.mesh[mesh_vert(&vm, i, 0, ring)].co;
                            p1 = vm.mesh[mesh_vert(&vm, i, ring, 0)].co;
                            f = if k < j {
                                j.min(k) as f32 / (2.0 * ring as f32)
                            } else {
                                (2.0 * ring as f32 - j as f32) / (2.0 * ring as f32)
                            };
                        } else {
                            p1 = vm.mesh[mesh_vert(&vm, i, 0, k)].co;
                            p2 = vm.mesh[mesh_vert(
                                &vm,
                                if i == ipipe1 { ipipe2 } else { ipipe1 },
                                0,
                                ns - k,
                            )]
                            .co;
                            f = j as f32 / ns as f32;
                        }
                    } else {
                        p1 = vm.mesh[mesh_vert(&vm, i, j, 0)].co;
                        p2 = vm.mesh[mesh_vert(&vm, i, j, ns)].co;
                        f = k as f32 / ns as f32;
                    }
                    let idx = mesh_vert(&vm, i, j, k);
                    vm.mesh[idx].co = p1 + (p2 - p1) * f;
                } else {
                    let even = (ns % 2) == 0;
                    let midline = even
                        && k == half_ns
                        && ((i == 0 && j == half_ns) || i == ipipe1 || i == ipipe2);
                    let idx = mesh_vert(&vm, i, j, k);
                    let mut co = vm.mesh[idx].co;
                    snap_to_pipe_profile(bm, bp, vpipe, midline, &mut co);
                    vm.mesh[idx].co = co;
                }
            }
        }
    }
    vm
}

/* ===== face building for vmeshes (C ~5250-6260) ===== */

/// `get_incident_edges` — two edges of `f` incident to `v` → (e1,e2).
fn get_incident_edges(bm: &BMesh, f: usize, v: usize) -> (usize, usize) {
    let (mut e1, mut e2) = (BM_NIL, BM_NIL);
    if f == BM_NIL {
        return (e1, e2);
    }
    for e in EdgesOfFace::new(bm, f) {
        if bm.edge(e).v1 == v || bm.edge(e).v2 == v {
            if e1 == BM_NIL {
                e1 = e;
            } else if e2 == BM_NIL {
                e2 = e;
            }
        }
    }
    (e1, e2)
}

/// `find_closer_edge`.
fn find_closer_edge(bm: &BMesh, co: Vec3, e1: usize, e2: usize) -> usize {
    debug_assert!(e1 != BM_NIL && e2 != BM_NIL);
    let dsq1 = blenders_math::geom::dist_squared_to_line_segment_v3(
        co,
        bm.vert(bm.edge(e1).v1).co,
        bm.vert(bm.edge(e1).v2).co,
    );
    let dsq2 = blenders_math::geom::dist_squared_to_line_segment_v3(
        co,
        bm.vert(bm.edge(e2).v1).co,
        bm.vert(bm.edge(e2).v2).co,
    );
    if dsq1 < dsq2 {
        e1
    } else {
        e2
    }
}

/// `find_face_internal_boundverts` — up to 3 boundverts inside `f`.
fn find_face_internal_boundverts(bm: &BMesh, bp: &BevelParams, bvi: usize, f: usize) -> [usize; 3] {
    let mut r = [BM_NIL; 3];
    if f == BM_NIL {
        return r;
    }
    let mut n = 0usize;
    let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
    let mut v = vm.boundstart;
    loop {
        if crate::polygon::bm_face_point_inside_test(bm, f, bp.boundverts[v].nv.co) {
            r[n] = v;
            n += 1;
            if n == 3 {
                break;
            }
        }
        v = bp.boundverts[v].next;
        if v == vm.boundstart {
            break;
        }
    }
    r
}

/// `projected_boundary_area`.
fn projected_boundary_area(bm: &mut BMesh, bp: &BevelParams, bvi: usize, f: usize) -> f32 {
    let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
    let axis_mat = blenders_math::geom::axis_dominant_to_m3(bm.face(f).no);
    let (e1, e2) = get_incident_edges(bm, f, bp.bverts[bvi].v);
    debug_assert!(e1 != BM_NIL && e2 != BM_NIL);
    let unsnapped = find_face_internal_boundverts(bm, bp, bvi, f);
    let mut proj_co = Vec::with_capacity(vm.count as usize);
    let mut v = vm.boundstart;
    loop {
        let co = bm.vert(bp.boundverts[v].nv.v).co;
        if unsnapped.contains(&v) {
            let p = axis_mat * co;
            proj_co.push(blenders_math::vector::Vec2::new(p.x, p.y));
        } else {
            let snap1 = blenders_math::geom::closest_to_line_segment_v3(
                co,
                bm.vert(bm.edge(e1).v1).co,
                bm.vert(bm.edge(e1).v2).co,
            )
            .0;
            let snap2 = blenders_math::geom::closest_to_line_segment_v3(
                co,
                bm.vert(bm.edge(e2).v1).co,
                bm.vert(bm.edge(e2).v2).co,
            )
            .0;
            let d1 = (snap1 - co).length_squared();
            let d2 = (snap2 - co).length_squared();
            let p = axis_mat * if d1 <= d2 { snap1 } else { snap2 };
            proj_co.push(blenders_math::vector::Vec2::new(p.x, p.y));
        }
        v = bp.boundverts[v].next;
        if v == vm.boundstart {
            break;
        }
    }
    blenders_math::geom::area_poly_v2(&proj_co)
}

/// `is_bad_uv_poly`.
fn is_bad_uv_poly(bm: &mut BMesh, bp: &BevelParams, bvi: usize, frep: usize) -> bool {
    projected_boundary_area(bm, bp, bvi, frep) < BEVEL_EPSILON_BIG
}

/// `frep_for_center_poly`.
fn frep_for_center_poly(bm: &mut BMesh, bp: &BevelParams, bvi: usize) -> usize {
    let mut fchoices: Vec<usize> = Vec::new();
    let mut any_bmf = BM_NIL;
    let consider_all = bp.bverts[bvi].selcount == 1 || bp.affect_vertices_odd;
    for i in 0..bp.bverts[bvi].edgecount as usize {
        if !bp.bverts[bvi].edges[i].is_bev && !consider_all {
            continue;
        }
        let ftwo = [bp.bverts[bvi].edges[i].fprev, bp.bverts[bvi].edges[i].fnext];
        let bmf = choose_rep_face(bm, bp, &ftwo);
        if bmf != BM_NIL {
            if any_bmf == BM_NIL {
                any_bmf = bmf;
            }
            if !fchoices.contains(&bmf) {
                if bp.math_layer_info.has_math_layers && is_bad_uv_poly(bm, bp, bvi, bmf) {
                    continue;
                }
                fchoices.push(bmf);
            }
        }
    }
    if fchoices.is_empty() {
        return any_bmf;
    }
    choose_rep_face(bm, bp, &fchoices)
}

/// `build_center_ngon`.
fn build_center_ngon(bm: &mut BMesh, bp: &mut BevelParams, bvi: usize, mat_nr: i32) {
    let ns2;
    let boundstart;
    {
        let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
        ns2 = vm.seg / 2;
        boundstart = vm.boundstart;
    }
    let mut frep = BM_NIL;
    let mut frep_e1 = BM_NIL;
    let mut frep_e2 = BM_NIL;
    let mut frep_unsnapped = [BM_NIL; 3];
    if bp.bverts[bvi].any_seam {
        frep = frep_for_center_poly(bm, bp, bvi);
        let (e1, e2) = get_incident_edges(bm, frep, bp.bverts[bvi].v);
        frep_e1 = e1;
        frep_e2 = e2;
        frep_unsnapped = find_face_internal_boundverts(bm, bp, bvi, frep);
    }
    let mut vv = Vec::new();
    let mut vf = Vec::new();
    let mut ve = Vec::new();
    let mut v = boundstart;
    loop {
        let i = bp.boundverts[v].index;
        let mv = {
            let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
            vm.mesh[mesh_vert(vm, i, ns2, ns2)].v
        };
        vv.push(mv);
        if frep != BM_NIL {
            vf.push(frep);
            if frep_unsnapped.contains(&v) {
                ve.push(BM_NIL);
            } else {
                let co = bm.vert(mv).co;
                ve.push(find_closer_edge(bm, co, frep_e1, frep_e2));
            }
        } else {
            vf.push(boundvert_rep_face(bp, v).0);
            ve.push(BM_NIL);
        }
        v = bp.boundverts[v].next;
        if v == boundstart {
            break;
        }
    }
    let f = bev_create_ngon(
        bm,
        bp,
        &vv,
        Some(&vf),
        frep,
        Some(&ve),
        bp.bverts[bvi].v,
        None,
        mat_nr,
        true,
    );
    record_face_kind(bp, f, F_VERT);
}

/// `build_square_in_vmesh`.
fn build_square_in_vmesh(bm: &mut BMesh, bp: &mut BevelParams, bvi: usize, vm1: &VMesh) {
    let (n, ns, ns2, odd);
    {
        let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
        n = vm.count;
        ns = vm.seg;
        ns2 = ns / 2;
        odd = ns % 2;
    }
    for i in 0..n {
        for k in 1..ns {
            let (im, im1);
            {
                let vm = bp.bverts[bvi].vmesh.as_mut().unwrap();
                im = mesh_vert(vm, i, 0, k);
                vm.mesh[im].co = vm1.mesh[mesh_vert(vm1, i, 0, k)].co;
                if i > 0 && k <= ns2 {
                    im1 = mesh_vert(vm, i - 1, 0, ns - k);
                    vm.mesh[im].v = vm.mesh[im1].v;
                    continue;
                } else if i == n - 1 && k > ns2 {
                    im1 = mesh_vert(vm, 0, 0, ns - k);
                    vm.mesh[im].v = vm.mesh[im1].v;
                    continue;
                }
            }
            let bvv = bp.bverts[bvi].v;
            create_mesh_bmvert(bm, bp.bverts[bvi].vmesh.as_mut().unwrap(), i, 0, k, Some(bvv));
        }
    }
    if odd != 0 {
        for i in 0..n {
            let vm = bp.bverts[bvi].vmesh.as_mut().unwrap();
            let im = mesh_vert(vm, i, ns2, ns2);
            let im1 = mesh_vert(vm, i, 0, ns2);
            vm.mesh[im].v = vm.mesh[im1].v;
        }
        build_center_ngon(bm, bp, bvi, bp.mat_nr);
    }
}

/// `closer_v3_v3v3v3`.
fn closer_v3_v3v3v3(a: Vec3, b: Vec3, v: Vec3) -> Vec3 {
    if (a - v).length_squared() <= (b - v).length_squared() {
        a
    } else {
        b
    }
}

/// `square_out_adj_vmesh`.
fn square_out_adj_vmesh(bm: &BMesh, bp: &mut BevelParams, bvi: usize) -> VMesh {
    let n_bndv = bp.bverts[bvi].vmesh.as_ref().unwrap().count;
    let ns = bp.bverts[bvi].vmesh.as_ref().unwrap().seg;
    let ns2 = ns / 2;
    let odd = ns % 2;
    let mut ns2inv = 1.0 / ns2 as f32;
    let boundstart = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
    let mut vm = new_adj_vmesh(n_bndv, ns, boundstart);
    let clstride = (3 * (ns2 + 1)) as usize;
    let mut centerline = vec![0.0f32; clstride * n_bndv as usize];
    let mut cset = vec![false; n_bndv as usize];
    let mut bndv = boundstart;
    let mut i = 0usize;
    let bvco = bm.vert(bp.bverts[bvi].v).co;
    while i < n_bndv as usize {
        let bndco = bp.boundverts[bndv].nv.co;
        let e1 = bp.boundverts[bndv].efirst;
        let e2 = bp.boundverts[bndv].elast;
        let mut ang_kind = ANGLE_STRAIGHT;
        if e1 != BM_NIL && e2 != BM_NIL {
            ang_kind = edges_angle_kind(bm, bp, (bvi, e1), (bvi, e2), bp.bverts[bvi].v);
        }
        if bp.boundverts[bndv].is_patch_start {
            let m = (bp.boundverts[bndv].nv.co
                + bp.boundverts[bp.boundverts[bndv].next].nv.co)
                * 0.5;
            for c in 0..3 {
                centerline[clstride * i + c] = m[c];
            }
            cset[i] = true;
            bndv = bp.boundverts[bndv].next;
            i += 1;
            let m = (bp.boundverts[bndv].nv.co
                + bp.boundverts[bp.boundverts[bndv].next].nv.co)
                * 0.5;
            for c in 0..3 {
                centerline[clstride * i + c] = m[c];
            }
            cset[i] = true;
            bndv = bp.boundverts[bndv].next;
            i += 1;
        } else if bp.boundverts[bndv].is_arc_start {
            let e1a = bp.boundverts[bndv].efirst;
            let _e2a = bp.boundverts[bp.boundverts[bndv].next].efirst;
            let _ = e1a;
            let m = bp.boundverts[bndv].profile.middle;
            for c in 0..3 {
                centerline[clstride * i + c] = m[c];
            }
            bndv = bp.boundverts[bndv].next;
            cset[i] = true;
            i += 1;
        } else if ang_kind == ANGLE_SMALLER {
            let ee1 = bp.bverts[bvi].edges[e1].e;
            let ee2 = bp.bverts[bvi].edges[e2].e;
            let dir1 = bm.vert(bm.edge(ee1).v1).co - bm.vert(bm.edge(ee1).v2).co;
            let dir2 = bm.vert(bm.edge(ee2).v1).co - bm.vert(bm.edge(ee2).v2).co;
            let co1 = bndco + dir1;
            let co2 = bndco + dir2;
            let (ikind, meet1, _) = blenders_math::geom::isect_line_line(
                bm.vert(bm.edge(ee1).v1).co,
                bm.vert(bm.edge(ee1).v2).co,
                bndco,
                co2,
            );
            let (v1set, v1co) = if ikind == 0 { (false, Vec3::ZERO) } else { (true, meet1) };
            let (ikind, meet1, _) = blenders_math::geom::isect_line_line(
                bm.vert(bm.edge(ee2).v1).co,
                bm.vert(bm.edge(ee2).v2).co,
                bndco,
                co1,
            );
            let (v2set, v2co) = if ikind == 0 { (false, Vec3::ZERO) } else { (true, meet1) };
            let iprev = if i == 0 { n_bndv as usize - 1 } else { i - 1 };
            if v2set {
                if cset[i] {
                    let cur = Vec3::new(
                        centerline[clstride * i],
                        centerline[clstride * i + 1],
                        centerline[clstride * i + 2],
                    );
                    let m = closer_v3_v3v3v3(cur, v2co, bvco);
                    for c in 0..3 {
                        centerline[clstride * i + c] = m[c];
                    }
                } else {
                    for c in 0..3 {
                        centerline[clstride * i + c] = v2co[c];
                    }
                    cset[i] = true;
                }
            }
            if v1set {
                if cset[iprev] {
                    let cur = Vec3::new(
                        centerline[clstride * iprev],
                        centerline[clstride * iprev + 1],
                        centerline[clstride * iprev + 2],
                    );
                    let m = closer_v3_v3v3v3(cur, v1co, bvco);
                    for c in 0..3 {
                        centerline[clstride * iprev + c] = m[c];
                    }
                } else {
                    for c in 0..3 {
                        centerline[clstride * iprev + c] = v1co[c];
                    }
                    cset[iprev] = true;
                }
            }
            bndv = bp.boundverts[bndv].next;
            i += 1;
        } else {
            bndv = bp.boundverts[bndv].next;
            i += 1;
        }
    }
    bndv = boundstart;
    for i in 0..n_bndv as usize {
        if !cset[i] {
            let e1 = bp.boundverts[bp.boundverts[bndv].next].efirst;
            let co1 = bp.boundverts[bndv].nv.co;
            let co2 = bp.boundverts[bp.boundverts[bndv].next].nv.co;
            if e1 != BM_NIL {
                let ee = bp.bverts[bvi].edges[e1].e;
                if bp.boundverts[bp.boundverts[bndv].prev].is_arc_start
                    && bp.boundverts[bp.boundverts[bndv].next].is_arc_start
                {
                    let (ikind, meet1, _) = blenders_math::geom::isect_line_line(
                        bm.vert(bm.edge(ee).v1).co,
                        bm.vert(bm.edge(ee).v2).co,
                        co1,
                        co2,
                    );
                    if ikind != 0 {
                        for c in 0..3 {
                            centerline[clstride * i + c] = meet1[c];
                        }
                        cset[i] = true;
                    }
                } else {
                    let src = if bp.boundverts[bp.boundverts[bndv].prev].is_arc_start {
                        co1
                    } else {
                        co2
                    };
                    let snap = blenders_math::geom::closest_to_line_segment_v3(
                        src,
                        bm.vert(bm.edge(ee).v1).co,
                        bm.vert(bm.edge(ee).v2).co,
                    )
                    .0;
                    for c in 0..3 {
                        centerline[clstride * i + c] = snap[c];
                    }
                    cset[i] = true;
                }
            }
            if !cset[i] {
                let m = (co1 + co2) * 0.5;
                for c in 0..3 {
                    centerline[clstride * i + c] = m[c];
                }
                cset[i] = true;
            }
        }
        bndv = bp.boundverts[bndv].next;
    }
    let mut co1;
    let mut co2 = bvco;
    bndv = boundstart;
    for i in 0..n_bndv {
        if odd != 0 {
            let ang = 0.5
                * angle_v3v3(
                    bp.boundverts[bndv].nv.co - bvco,
                    bp.boundverts[bp.boundverts[bndv].next].nv.co - bvco,
                );
            let finalfrac = if ang > BEVEL_SMALL_ANG {
                (0.5 / ang.sin()).min(0.8)
            } else {
                0.8
            };
            ns2inv = 1.0 / (ns2 as f32 + finalfrac);
        }
        let p = clstride * i as usize;
        co1 = Vec3::new(centerline[p], centerline[p + 1], centerline[p + 2]);
        for j in 1..=ns2 {
            let m = co1 + (co2 - co1) * (j as f32 * ns2inv);
            for c in 0..3 {
                centerline[p + 3 * j as usize + c] = m[c];
            }
        }
        bndv = bp.boundverts[bndv].next;
    }
    bndv = boundstart;
    for i in 0..n_bndv {
        co1 = bp.boundverts[bndv].nv.co;
        let pi = clstride * (if i == 0 { n_bndv - 1 } else { i - 1 }) as usize;
        co2 = Vec3::new(centerline[pi], centerline[pi + 1], centerline[pi + 2]);
        for j in 0..ns2 + odd {
            let m = co1 + (co2 - co1) * (j as f32 * ns2inv);
            let idx = mesh_vert(&vm, i, j, 0);
            vm.mesh[idx].co = m;
        }
        let pi = clstride * i as usize;
        co2 = Vec3::new(centerline[pi], centerline[pi + 1], centerline[pi + 2]);
        for k in 1..=ns2 {
            let m = co1 + (co2 - co1) * (k as f32 * ns2inv);
            let idx = mesh_vert(&vm, i, 0, k);
            vm.mesh[idx].co = m;
        }
        bndv = bp.boundverts[bndv].next;
    }
    if odd == 0 {
        let idx = mesh_vert(&vm, 0, ns2, ns2);
        vm.mesh[idx].co = bvco;
    }
    vmesh_copy_equiv_verts(&mut vm);
    bndv = boundstart;
    for i in 0..n_bndv {
        let im1 = if i == 0 { n_bndv - 1 } else { i - 1 };
        for j in 1..ns2 + odd {
            for k in 1..=ns2 {
                let pa = Vec3::new(
                    centerline[clstride * im1 as usize + 3 * k as usize],
                    centerline[clstride * im1 as usize + 3 * k as usize + 1],
                    centerline[clstride * im1 as usize + 3 * k as usize + 2],
                );
                let pb = Vec3::new(
                    centerline[clstride * i as usize + 3 * j as usize],
                    centerline[clstride * i as usize + 3 * j as usize + 1],
                    centerline[clstride * i as usize + 3 * j as usize + 2],
                );
                let (ikind, meet1, meet2) = blenders_math::geom::isect_line_line(
                    vm.mesh[mesh_vert(&vm, i, 0, k)].co,
                    pa,
                    vm.mesh[mesh_vert(&vm, i, j, 0)].co,
                    pb,
                );
                let idx = mesh_vert(&vm, i, j, k);
                if ikind == 0 {
                    vm.mesh[idx].co = vm.mesh[mesh_vert(&vm, i, 0, k)].co
                        + (pa - vm.mesh[mesh_vert(&vm, i, 0, k)].co) * (j as f32 * ns2inv);
                } else if ikind == 1 {
                    vm.mesh[idx].co = meet1;
                } else {
                    vm.mesh[idx].co = (meet1 + meet2) * 0.5;
                }
            }
        }
        bndv = bp.boundverts[bndv].next;
    }
    vmesh_copy_equiv_verts(&mut vm);
    vm
}

/// `snap_edge_for_center_vmesh_vert`.
fn snap_edge_for_center_vmesh_vert(
    i: usize,
    n_bndv: usize,
    eprev: usize,
    enext: usize,
    bndv_rep_faces: &[usize],
    center_frep: usize,
    frep_beats_next: &[bool],
) -> usize {
    let previ = (i + n_bndv - 1) % n_bndv;
    let nexti = (i + 1) % n_bndv;
    if frep_beats_next[previ] && bndv_rep_faces[previ] == center_frep {
        return eprev;
    }
    if !frep_beats_next[i] && bndv_rep_faces[nexti] == center_frep {
        return enext;
    }
    BM_NIL
}

/// `snap_edges_for_vmesh_vert` — fill `r_snap_edges[0..4]`.
#[allow(clippy::too_many_arguments)]
fn snap_edges_for_vmesh_vert(
    i: usize,
    j: i32,
    k: i32,
    ns: i32,
    ns2: i32,
    n_bndv: usize,
    eprev: usize,
    enext: usize,
    enextnext: usize,
    bndv_rep_faces: &[usize],
    center_frep: usize,
    frep_beats_next: &[bool],
    r_snap_edges: &mut [usize; 4],
) {
    debug_assert!(i < n_bndv && 0 <= j && j < ns2 && 0 <= k && k <= ns2);
    for corner in 0..4usize {
        r_snap_edges[corner] = BM_NIL;
        if ns % 2 == 0 {
            continue;
        }
        let previ = (i + n_bndv - 1) % n_bndv;
        let jj = if corner < 2 { j } else { j + 1 };
        let kk = if corner == 0 || corner == 3 { k } else { k + 1 };
        if jj < ns2 && kk < ns2 {
            /* no snap */
        } else if jj < ns2 && kk == ns2 {
            if !frep_beats_next[i] {
                r_snap_edges[corner] = enext;
            }
        } else if jj < ns2 && kk == ns2 + 1 {
            if frep_beats_next[i] {
                r_snap_edges[corner] = enext;
            }
        } else if jj == ns2 && kk < ns2 {
            if frep_beats_next[previ] {
                r_snap_edges[corner] = eprev;
            }
        } else if jj == ns2 && kk == ns2 {
            r_snap_edges[corner] = snap_edge_for_center_vmesh_vert(
                i,
                n_bndv,
                eprev,
                enext,
                bndv_rep_faces,
                center_frep,
                frep_beats_next,
            );
        } else if jj == ns2 && kk == ns2 + 1 {
            let nexti = (i + 1) % n_bndv;
            r_snap_edges[corner] = snap_edge_for_center_vmesh_vert(
                nexti,
                n_bndv,
                enext,
                enextnext,
                bndv_rep_faces,
                center_frep,
                frep_beats_next,
            );
        }
    }
}

/// `bevel_build_rings` — main vmesh fill + quad rings + center ngon.
#[allow(clippy::too_many_arguments)]
fn bevel_build_rings(bm: &mut BMesh, bp: &mut BevelParams, bvi: usize, vpipe: usize) {
    let mat_nr = bp.mat_nr;
    let (n_bndv, ns, ns2, odd, boundstart);
    {
        let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
        n_bndv = vm.count;
        ns = vm.seg;
        ns2 = ns / 2;
        odd = ns % 2;
        boundstart = vm.boundstart;
    }
    debug_assert!(n_bndv >= 3 && ns > 1);
    let vm1 = if bp.pro_super_r == PRO_SQUARE_R
        && bp.bverts[bvi].selcount >= 3
        && odd == 0
        && bp.profile_type != BEVEL_PROFILE_CUSTOM
    {
        square_out_adj_vmesh(bm, bp, bvi)
    } else if vpipe != BM_NIL {
        pipe_adj_vmesh(bp, bm, bvi, vpipe)
    } else if tri_corner_test(bm, bp, bvi) == 1 {
        let vm1 = tri_corner_adj_vmesh(bm, bp, bvi);
        if bp.pro_super_r == PRO_SQUARE_IN_R && bp.profile_type != BEVEL_PROFILE_CUSTOM {
            build_square_in_vmesh(bm, bp, bvi, &vm1);
            return;
        }
        vm1
    } else {
        adj_vmesh(bp, bm, bvi)
    };
    for i in 0..n_bndv {
        for j in 0..=ns2 {
            for k in 0..=ns {
                if j == 0 && (k == 0 || k == ns) {
                    continue;
                }
                let canon;
                {
                    let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
                    canon = is_canon(vm, i, j, k);
                    if !canon {
                        continue;
                    }
                    let im = mesh_vert(vm, i, j, k);
                    let co = vm1.mesh[mesh_vert(&vm1, i, j, k)].co;
                    let _ = im;
                    let _ = co;
                }
                let bvv = bp.bverts[bvi].v;
                {
                    let vm = bp.bverts[bvi].vmesh.as_mut().unwrap();
                    let im = mesh_vert(vm, i, j, k);
                    vm.mesh[im].co = vm1.mesh[mesh_vert(&vm1, i, j, k)].co;
                }
                create_mesh_bmvert(bm, bp.bverts[bvi].vmesh.as_mut().unwrap(), i, j, k, Some(bvv));
            }
        }
    }
    {
        let vm = bp.bverts[bvi].vmesh.as_mut().unwrap();
        vmesh_copy_equiv_verts(vm);
    }
    let mut bndv_rep_faces = vec![BM_NIL; n_bndv as usize];
    let mut bndv = boundstart;
    loop {
        let i = bp.boundverts[bndv].index as usize;
        bndv_rep_faces[i] = boundvert_rep_face(bp, bndv).0;
        bndv = bp.boundverts[bndv].next;
        if bndv == boundstart {
            break;
        }
    }
    let mut center_verts: Vec<usize> = Vec::new();
    let mut center_edge_snaps: Vec<usize> = Vec::new();
    let mut center_face_interps: Vec<usize> = Vec::new();
    let mut frep_beats_next: Vec<bool> = Vec::new();
    let mut center_frep = BM_NIL;
    if odd != 0 && bp.affect_type == BEVEL_AFFECT_EDGES {
        center_verts = vec![BM_NIL; n_bndv as usize];
        center_edge_snaps = vec![BM_NIL; n_bndv as usize];
        center_face_interps = vec![BM_NIL; n_bndv as usize];
        frep_beats_next = vec![false; n_bndv as usize];
        center_frep = frep_for_center_poly(bm, bp, bvi);
        for i in 0..n_bndv as usize {
            center_edge_snaps[i] = BM_NIL;
            let inext = (i + 1) % n_bndv as usize;
            let fchoices = [bndv_rep_faces[i], bndv_rep_faces[inext]];
            let fwinner = choose_rep_face(bm, bp, &fchoices);
            frep_beats_next[i] = fwinner == bndv_rep_faces[i];
        }
    }
    bndv = boundstart;
    loop {
        let i = bp.boundverts[bndv].index as usize;
        let inext = bp.boundverts[bp.boundverts[bndv].next].index as usize;
        let f = bndv_rep_faces[i];
        let f2 = bndv_rep_faces[inext];
        let fc = if odd != 0 && bp.affect_type == BEVEL_AFFECT_EDGES {
            if frep_beats_next[i] { f } else { f2 }
        } else {
            BM_NIL
        };
        let (e, eprev, enext) = if bp.affect_type == BEVEL_AFFECT_VERTICES {
            (
                bp.boundverts[bndv].efirst,
                bp.boundverts[bp.boundverts[bndv].prev].efirst,
                bp.boundverts[bp.boundverts[bndv].next].efirst,
            )
        } else {
            (
                bp.boundverts[bndv].ebev,
                bp.boundverts[bp.boundverts[bndv].prev].ebev,
                bp.boundverts[bp.boundverts[bndv].next].ebev,
            )
        };
        let bme = if e != BM_NIL { bp.bverts[bvi].edges[e].e } else { BM_NIL };
        let bmeprev = if eprev != BM_NIL { bp.bverts[bvi].edges[eprev].e } else { BM_NIL };
        let bmenext = if enext != BM_NIL { bp.bverts[bvi].edges[enext].e } else { BM_NIL };
        for j in 0..ns2 {
            for k in 0..ns2 + odd {
                let (bmv1, bmv2, bmv3, bmv4);
                {
                    let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
                    bmv1 = vm.mesh[mesh_vert(vm, i as i32, j, k)].v;
                    bmv2 = vm.mesh[mesh_vert(vm, i as i32, j, k + 1)].v;
                    bmv3 = vm.mesh[mesh_vert(vm, i as i32, j + 1, k + 1)].v;
                    bmv4 = vm.mesh[mesh_vert(vm, i as i32, j + 1, k)].v;
                }
                let bmvs = [bmv1, bmv2, bmv3, bmv4];
                debug_assert!(bmv1 != BM_NIL && bmv2 != BM_NIL && bmv3 != BM_NIL && bmv4 != BM_NIL);
                let mut fr = [BM_NIL; 4];
                let mut se = [BM_NIL; 4];
                if bp.affect_type == BEVEL_AFFECT_VERTICES {
                    fr = [f2, f2, f2, f2];
                    if j < k {
                        if k == ns2 && j == ns2 - 1 {
                            se[2] = {
                                let en = bp.boundverts[bp.boundverts[bndv].next].efirst;
                                if en != BM_NIL { bp.bverts[bvi].edges[en].e } else { BM_NIL }
                            };
                            se[3] = bme;
                        }
                    } else if j == k {
                        se[0] = bme;
                        se[2] = bme;
                        if e != BM_NIL && !bp.bverts[bvi].edges[e].is_seam {
                            fr[3] = f;
                        }
                    }
                } else {
                    fr = [f, f, f, f];
                    if odd != 0 {
                        let b1 = if eprev != BM_NIL && bp.bverts[bvi].edges[eprev].is_seam {
                            bmeprev
                        } else {
                            BM_NIL
                        };
                        let b2 = if e != BM_NIL && bp.bverts[bvi].edges[e].is_seam {
                            bme
                        } else {
                            BM_NIL
                        };
                        let b3 = if enext != BM_NIL && bp.bverts[bvi].edges[enext].is_seam {
                            bmenext
                        } else {
                            BM_NIL
                        };
                        snap_edges_for_vmesh_vert(
                            i,
                            j,
                            k,
                            ns,
                            ns2,
                            n_bndv as usize,
                            b1,
                            b2,
                            b3,
                            &bndv_rep_faces,
                            center_frep,
                            &frep_beats_next,
                            &mut se,
                        );
                        if k == ns2 {
                            if e == BM_NIL || bp.bverts[bvi].edges[e].is_seam {
                                fr = [fc, fc, fc, fc];
                            } else {
                                fr = [f, f2, f2, f];
                            }
                            if j == ns2 - 1 {
                                center_verts[i] = bmvs[3];
                                center_edge_snaps[i] = se[3];
                                center_face_interps[i] =
                                    if bp.bverts[bvi].any_seam { center_frep } else { f };
                            }
                        }
                    } else {
                        if k == ns2 - 1 {
                            se[1] = bme;
                        }
                        if j == ns2 - 1 && bp.boundverts[bp.boundverts[bndv].prev].ebev != BM_NIL {
                            se[3] = bmeprev;
                        }
                        se[2] = if se[1] != BM_NIL { se[1] } else { se[3] };
                    }
                }
                let r_f = bev_create_ngon(
                    bm,
                    bp,
                    &bmvs,
                    Some(&fr),
                    BM_NIL,
                    Some(&se),
                    bp.bverts[bvi].v,
                    None,
                    mat_nr,
                    true,
                );
                record_face_kind(bp, r_f, F_VERT);
            }
        }
        bndv = bp.boundverts[bndv].next;
        if bndv == boundstart {
            break;
        }
    }
    if odd != 0 {
        if bp.affect_type == BEVEL_AFFECT_EDGES {
            let frep = if bp.bverts[bvi].any_seam {
                frep_for_center_poly(bm, bp, bvi)
            } else {
                BM_NIL
            };
            let cen_f = bev_create_ngon(
                bm,
                bp,
                &center_verts,
                Some(&center_face_interps),
                frep,
                Some(&center_edge_snaps),
                bp.bverts[bvi].v,
                None,
                mat_nr,
                true,
            );
            record_face_kind(bp, cen_f, F_VERT);
        } else {
            build_center_ngon(bm, bp, bvi, mat_nr);
        }
    }
}

/// `bevel_build_cutoff`.
fn bevel_build_cutoff(bm: &mut BMesh, bp: &mut BevelParams, bvi: usize) {
    let n_bndv = bp.bverts[bvi].vmesh.as_ref().unwrap().count;
    let boundstart = bp.bverts[bvi].vmesh.as_ref().unwrap().boundstart;
    let mut bndv = boundstart;
    loop {
        let i = bp.boundverts[bndv].index;
        let mut down_direction = bp.boundverts[bndv]
            .profile
            .plane_no
            .cross(bp.boundverts[bp.boundverts[bndv].prev].profile.plane_no);
        if down_direction.dot(bm.vert(bp.bverts[bvi].v).no) > 0.0 {
            down_direction = -down_direction;
        }
        let length =
            (bp.boundverts[bndv].profile.height / 2f32.sqrt()
                + bp.boundverts[bp.boundverts[bndv].prev].profile.height / 2f32.sqrt())
                / 2.0;
        let new_vert = bp.boundverts[bndv].nv.co + down_direction * length;
        let vm = bp.bverts[bvi].vmesh.as_mut().unwrap();
        let im = mesh_vert(vm, i, 1, 0);
        vm.mesh[im].co = new_vert;
        let im = mesh_vert(vm, bp.boundverts[bp.boundverts[bndv].prev].index, 1, 1);
        vm.mesh[im].co = new_vert;
        bndv = bp.boundverts[bndv].next;
        if bndv == boundstart {
            break;
        }
    }
    let mut build_center_face = true;
    if n_bndv == 3 {
        let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
        let a = vm.mesh[mesh_vert(vm, 0, 1, 0)].co;
        let b = vm.mesh[mesh_vert(vm, 1, 1, 0)].co;
        let c = vm.mesh[mesh_vert(vm, 2, 1, 0)].co;
        build_center_face = (a - b).length_squared() > BEVEL_EPSILON
            && (a - c).length_squared() > BEVEL_EPSILON
            && (b - c).length_squared() > BEVEL_EPSILON;
    }
    let bvv = bp.bverts[bvi].v;
    if build_center_face {
        bndv = boundstart;
        loop {
            let i = bp.boundverts[bndv].index;
            create_mesh_bmvert(bm, bp.bverts[bvi].vmesh.as_mut().unwrap(), i, 1, 0, Some(bvv));
            let vm = bp.bverts[bvi].vmesh.as_mut().unwrap();
            let im0 = mesh_vert(vm, i, 1, 0);
            let im1 = mesh_vert(vm, bp.boundverts[bp.boundverts[bndv].prev].index, 1, 1);
            vm.mesh[im1].v = vm.mesh[im0].v;
            bndv = bp.boundverts[bndv].next;
            if bndv == boundstart {
                break;
            }
        }
    } else {
        create_mesh_bmvert(bm, bp.bverts[bvi].vmesh.as_mut().unwrap(), 0, 1, 0, Some(bvv));
        for i in 1..n_bndv {
            let vm = bp.bverts[bvi].vmesh.as_mut().unwrap();
            let im0 = mesh_vert(vm, 0, 1, 0);
            let im = mesh_vert(vm, i, 1, 0);
            vm.mesh[im].v = vm.mesh[im0].v;
        }
    }
    let ncap = bp.seg + 2 + if build_center_face { 1 } else { 0 };
    let mut face_bmverts = vec![BM_NIL; ncap.max(n_bndv) as usize];
    bndv = boundstart;
    loop {
        let i = bp.boundverts[bndv].index;
        face_bmverts[0] = {
            let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
            vm.mesh[mesh_vert(vm, i, 1, 0)].v
        };
        for k in 0..bp.seg + 1 {
            face_bmverts[k as usize + 1] = {
                let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
                vm.mesh[mesh_vert(vm, i, 0, k)].v
            };
        }
        if build_center_face {
            face_bmverts[bp.seg as usize + 2] = {
                let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
                vm.mesh[mesh_vert(vm, i, 1, 1)].v
            };
        }
        bev_create_ngon(
            bm,
            bp,
            &face_bmverts[..ncap as usize],
            None,
            BM_NIL,
            None,
            bvv,
            None,
            bp.mat_nr,
            true,
        );
        bndv = bp.boundverts[bndv].next;
        if bndv == boundstart {
            break;
        }
    }
    if build_center_face {
        for i in 0..n_bndv as usize {
            face_bmverts[i] = {
                let vm = bp.bverts[bvi].vmesh.as_ref().unwrap();
                vm.mesh[mesh_vert(vm, i as i32, 1, 0)].v
            };
        }
        bev_create_ngon(
            bm,
            bp,
            &face_bmverts[..n_bndv as usize],
            None,
            BM_NIL,
            None,
            bvv,
            None,
            bp.mat_nr,
            true,
        );
    }
}
