//! Port of `bmo_connect_pair.cc` — connect a vertex pair across many faces.
//! Heap of path states (PathLinkState) walks edges/verts intersecting the
//! pair's plane; links share a pool → Vec arena with usize "pointers".

use crate::operators::*;
use crate::{BMesh, BM_EDGE, BM_FACE, BM_NIL, BM_VERT};
use blenders_bli::heap_simple::HeapSimple;
use blenders_math::matrix::Mat3;
use blenders_math::vector::{ortho_v3, project_plane_normalized, Vec3};

const CONNECT_EPS_BASIS: f32 = 0.00001;
const VERT_OUT: u8 = 1;
const VERT_EXCLUDE: u8 = 2;
const FACE_EXCLUDE: u8 = 2;
const ELE_TOUCHED: u8 = 4;

type BMElemRef2 = (u8, usize);

/// `dot_m3_v3_row_x`.
fn dot_m3_v3_row_x(m: &Mat3, v: Vec3) -> f32 {
    m.cols[0].x * v.x + m.cols[1].x * v.y + m.cols[2].x * v.z
}

/// `PathLink` — node of the path chain (indexes into `link_pool`).
#[derive(Clone)]
struct PathLink {
    next: Option<usize>,
    ele: BMElemRef2,      /* edge or vert */
    ele_from: Option<BMElemRef2>, /* edge or face we came from */
}

/// `PathLinkState`.
#[derive(Clone)]
struct PathLinkState {
    link_last: Option<usize>,
    dist: f32,
    co_prev: Vec3,
}

struct PathContext {
    matrix: Mat3,
    axis_sep: f32,
    connect_eps: f32,
    v_pair: [usize; 2],
    link_pool: Vec<PathLink>,
}

/// `MinDistDir` — best hit in both directions (see #43792).
struct MinDistDir {
    dist_min: [f32; 2],
    dir: Vec3,
}
impl MinDistDir {
    fn new() -> Self {
        MinDistDir { dist_min: [f32::MAX, f32::MAX], dir: Vec3::ZERO }
    }
    fn test(&self, dist_dir: Vec3, dist_sq: f32) -> i32 {
        if self.dist_min[0] == f32::MAX {
            return 0;
        }
        if dist_dir.dot(self.dir) > 0.0 {
            if dist_sq < self.dist_min[0] {
                return 0;
            }
        } else if dist_sq < self.dist_min[1] {
            return 1;
        }
        -1
    }
    fn update(&mut self, dist_dir: Vec3) {
        if self.dist_min[0] == f32::MAX {
            self.dir = dist_dir;
        }
    }
}

fn state_isect_co_pair(pc: &PathContext, co_a: Vec3, co_b: Vec3) -> bool {
    let diff_a = dot_m3_v3_row_x(&pc.matrix, co_a) - pc.axis_sep;
    let diff_b = dot_m3_v3_row_x(&pc.matrix, co_b) - pc.axis_sep;
    let test_a = if diff_a.abs() < pc.connect_eps { 0 } else if diff_a < 0.0 { -1 } else { 1 };
    let test_b = if diff_b.abs() < pc.connect_eps { 0 } else if diff_b < 0.0 { -1 } else { 1 };
    test_a != 0 && test_b != 0 && test_a != test_b
}
fn state_isect_co_exact(pc: &PathContext, co: Vec3) -> bool {
    (dot_m3_v3_row_x(&pc.matrix, co) - pc.axis_sep).abs() <= pc.connect_eps
}
fn state_calc_co_pair_fac(pc: &PathContext, co_a: Vec3, co_b: Vec3) -> f32 {
    let diff_a = (dot_m3_v3_row_x(&pc.matrix, co_a) - pc.axis_sep).abs();
    let diff_b = (dot_m3_v3_row_x(&pc.matrix, co_b) - pc.axis_sep).abs();
    let tot = diff_a + diff_b;
    if tot > f32::EPSILON { diff_a / tot } else { 0.5 }
}
fn state_calc_co_pair(pc: &PathContext, co_a: Vec3, co_b: Vec3) -> Vec3 {
    co_a.lerp(co_b, state_calc_co_pair_fac(pc, co_a, co_b))
}

struct Ctx<'a> {
    bm: &'a mut BMesh,
    pc: PathContext,
    states_arena: Vec<PathLinkState>,
    heap: HeapSimple<usize>,
}

fn ele_touched(bm: &BMesh, ele: BMElemRef2) -> bool {
    match ele.0 {
        BM_VERT => bm.vert(ele.1).head.api_flag & ELE_TOUCHED != 0,
        BM_EDGE => bm.edge(ele.1).head.api_flag & ELE_TOUCHED != 0,
        _ => bm.face(ele.1).head.api_flag & ELE_TOUCHED != 0,
    }
}
fn ele_touch_mark(bm: &mut BMesh, ele: BMElemRef2) {
    match ele.0 {
        BM_VERT => bm.vpool[ele.1].head.api_flag |= ELE_TOUCHED,
        BM_EDGE => bm.epool[ele.1].head.api_flag |= ELE_TOUCHED,
        _ => bm.fpool[ele.1].head.api_flag |= ELE_TOUCHED,
    }
}
fn face_walk_test(bm: &BMesh, f: usize) -> bool {
    bm.face(f).head.api_flag & FACE_EXCLUDE == 0
}
fn vert_walk_test(bm: &BMesh, v: usize) -> bool {
    bm.vert(v).head.api_flag & VERT_EXCLUDE == 0
}

fn state_link_add(ctx: &mut Ctx, si: usize, ele: BMElemRef2, ele_from: Option<BMElemRef2>) {
    ele_touch_mark(ctx.bm, ele);
    /* track distance */
    let co = match ele.0 {
        BM_VERT => ctx.bm.vert(ele.1).co,
        BM_EDGE => {
            let e = ctx.bm.edge(ele.1);
            state_calc_co_pair(&ctx.pc, ctx.bm.vert(e.v1).co, ctx.bm.vert(e.v2).co)
        }
        _ => Vec3::ZERO,
    };
    {
        let st = &mut ctx.states_arena[si];
        if ele_from.is_some() {
            st.dist += (st.co_prev - co).length();
        }
        st.co_prev = co;
    }
    ctx.pc.link_pool.push(PathLink {
        next: ctx.states_arena[si].link_last,
        ele,
        ele_from,
    });
    ctx.states_arena[si].link_last = Some(ctx.pc.link_pool.len() - 1);
}

/// `state_link_add_test` — dupe the state when a branch already added a link.
fn state_link_add_test(
    ctx: &mut Ctx,
    si: usize,
    orig_link_last: Option<usize>,
    ele: BMElemRef2,
    ele_from: Option<BMElemRef2>,
) -> usize {
    let is_new = orig_link_last != ctx.states_arena[si].link_last;
    let si = if is_new {
        let dup = ctx.states_arena[si].clone();
        ctx.states_arena.push(dup);
        ctx.states_arena.len() - 1
    } else {
        si
    };
    state_link_add(ctx, si, ele, ele_from);
    if is_new {
        let d = ctx.states_arena[si].dist;
        ctx.heap.insert(d, si);
    }
    si
}

/* walk around the face edges */
fn state_step_face_edges(
    ctx: &mut Ctx,
    mut si: usize,
    orig_co_prev: Vec3,
    orig_link_last: Option<usize>,
    mut l_iter: usize,
    l_last: usize,
    mddir: &mut MinDistDir,
) -> usize {
    let mut l_iter_best: [usize; 2] = [BM_NIL, BM_NIL];
    loop {
        let (v_a, v_b) = (ctx.bm.loo(l_iter).v, ctx.bm.loo(ctx.bm.loo(l_iter).next).v);
        let e_i = ctx.bm.loo(l_iter).e;
        let f_i = ctx.bm.loo(l_iter).f;
        if state_isect_co_pair(&ctx.pc, ctx.bm.vert(v_a).co, ctx.bm.vert(v_b).co) {
            let co_isect = state_calc_co_pair(&ctx.pc, ctx.bm.vert(v_a).co, ctx.bm.vert(v_b).co);
            let dist_dir = co_isect - orig_co_prev;
            let dist_test = dist_dir.length_squared();
            let index = mddir.test(dist_dir, dist_test);
            if index != -1 {
                if face_walk_test(ctx.bm, f_i) && !ele_touched(ctx.bm, (BM_EDGE, e_i)) {
                    mddir.update(dist_dir);
                    mddir.dist_min[index as usize] = dist_test;
                    l_iter_best[index as usize] = l_iter;
                }
            }
        }
        l_iter = ctx.bm.loo(l_iter).next;
        if l_iter == l_last {
            break;
        }
    }
    for li in l_iter_best {
        if li != BM_NIL {
            let ele = (BM_EDGE, ctx.bm.loo(li).e);
            let from = (BM_FACE, ctx.bm.loo(li).f);
            si = state_link_add_test(ctx, si, orig_link_last, ele, Some(from));
        }
    }
    si
}

/* walk around the face verts */
fn state_step_face_verts(
    ctx: &mut Ctx,
    mut si: usize,
    orig_co_prev: Vec3,
    orig_link_last: Option<usize>,
    mut l_iter: usize,
    l_last: usize,
    mddir: &mut MinDistDir,
) -> usize {
    let mut l_iter_best: [usize; 2] = [BM_NIL, BM_NIL];
    loop {
        let v_i = ctx.bm.loo(l_iter).v;
        let f_i = ctx.bm.loo(l_iter).f;
        if state_isect_co_exact(&ctx.pc, ctx.bm.vert(v_i).co) {
            let dist_dir = ctx.bm.vert(v_i).co - orig_co_prev;
            let dist_test = dist_dir.length_squared();
            let index = mddir.test(dist_dir, dist_test);
            if index != -1 {
                if face_walk_test(ctx.bm, f_i) && !ele_touched(ctx.bm, (BM_VERT, v_i)) {
                    mddir.update(dist_dir);
                    mddir.dist_min[index as usize] = dist_test;
                    l_iter_best[index as usize] = l_iter;
                }
            }
        }
        l_iter = ctx.bm.loo(l_iter).next;
        if l_iter == l_last {
            break;
        }
    }
    for li in l_iter_best {
        if li != BM_NIL {
            let ele = (BM_VERT, ctx.bm.loo(li).v);
            let from = (BM_FACE, ctx.bm.loo(li).f);
            si = state_link_add_test(ctx, si, orig_link_last, ele, Some(from));
        }
    }
    si
}

/// `state_step` — true if new links were added.
fn state_step(ctx: &mut Ctx, si: usize) -> bool {
    let orig = ctx.states_arena[si].clone();
    let link = ctx.pc.link_pool[orig.link_last.unwrap()].clone();
    let (orig_co_prev, orig_link_last) = (orig.co_prev, orig.link_last);
    let mut si = si;

    if link.ele.0 == BM_EDGE {
        let e = link.ele.1;
        for l_start in crate::iterators::LoopsOfEdge::new(ctx.bm, e).collect::<Vec<_>>() {
            let f = ctx.bm.loo(l_start).f;
            if Some((BM_FACE, f)) != link.ele_from && face_walk_test(ctx.bm, f) {
                let mut mddir = MinDistDir::new();
                let (n, n2) = (ctx.bm.loo(l_start).next, ctx.bm.loo(ctx.bm.loo(l_start).next).next);
                si = state_step_face_edges(ctx, si, orig_co_prev, orig_link_last, n, l_start, &mut mddir);
                si = state_step_face_verts(ctx, si, orig_co_prev, orig_link_last, n2, l_start, &mut mddir);
            }
        }
    } else if link.ele.0 == BM_VERT {
        let v = link.ele.1;
        /* Vert loops. */
        for l_start in crate::iterators::LoopsOfVert::new(ctx.bm, v).collect::<Vec<_>>() {
            let f = ctx.bm.loo(l_start).f;
            if Some((BM_FACE, f)) != link.ele_from && face_walk_test(ctx.bm, f) {
                let mut mddir = MinDistDir::new();
                let (n, p) = (ctx.bm.loo(l_start).next, ctx.bm.loo(l_start).prev);
                let n2 = ctx.bm.loo(n).next;
                si = state_step_face_edges(ctx, si, orig_co_prev, orig_link_last, n, p, &mut mddir);
                if ctx.bm.face(f).len > 3 {
                    /* Adjacent verts are handled in vert edges below. */
                    si = state_step_face_verts(ctx, si, orig_co_prev, orig_link_last, n2, p, &mut mddir);
                }
            }
        }
        /* Vert edges. */
        for e in crate::iterators::EdgesOfVert::new(ctx.bm, v).collect::<Vec<_>>() {
            let v_other = crate::structure::bm_edge_other_vert(ctx.bm, e, v);
            if Some((BM_EDGE, e)) != link.ele_from && vert_walk_test(ctx.bm, v_other) {
                if state_isect_co_exact(&ctx.pc, ctx.bm.vert(v_other).co) {
                    if !ele_touched(ctx.bm, (BM_VERT, v_other)) {
                        si = state_link_add_test(
                            ctx, si, orig_link_last, (BM_VERT, v_other), Some((BM_EDGE, e)));
                    }
                }
            }
        }
    }
    orig_link_last != ctx.states_arena[si].link_last
}

/// `bm_vert_pair_to_matrix` — orientation matrix from 2 vertices.
fn bm_vert_pair_to_matrix(bm: &BMesh, v_pair: [usize; 2]) -> Mat3 {
    let eps = 1e-8f32;
    let basis_dir = (bm.vert(v_pair[0]).co - bm.vert(v_pair[1]).co).normalized();
    /* align both normals to the direction before combining */
    let mut basis_nor_a = project_plane_normalized(bm.vert(v_pair[0]).no, basis_dir);
    let mut basis_nor_b = project_plane_normalized(bm.vert(v_pair[1]).no, basis_dir);
    if basis_nor_a.dot(basis_nor_b) < 0.0 {
        basis_nor_b = -basis_nor_b;
    }
    let _ = &mut basis_nor_a;
    let mut basis_nor = basis_nor_a + basis_nor_b;
    basis_nor.normalize();
    let mut basis_tmp = basis_dir.cross(basis_nor);

    /* Try get the axis from surrounding faces, fallback to ortho_v3. */
    if basis_tmp.length() < eps {
        /* vertex normals are directly opposite — find loop with lowest angle. */
        let mut axis_pair = [(Vec3::ZERO, -f32::MAX); 2];
        for i in 0..2 {
            for l in crate::iterators::LoopsOfVert::new(bm, v_pair[i]) {
                let f_no = bm.face(ctx_face(bm, l)).no;
                let proj = project_plane_normalized(basis_dir, f_no).normalized();
                if proj.length() > eps {
                    let angle_cos_test = proj.dot(basis_dir);
                    if angle_cos_test > axis_pair[i].1 {
                        axis_pair[i] = (proj, angle_cos_test);
                    }
                }
            }
        }
        basis_nor = if axis_pair[0].1 < axis_pair[1].1 { axis_pair[1].0 } else { axis_pair[0].0 };
        basis_nor = project_plane_normalized(basis_nor, basis_dir);
        basis_tmp = basis_dir.cross(basis_nor);
        if basis_tmp.length() < eps {
            basis_nor = ortho_v3(basis_dir);
            basis_tmp = basis_dir.cross(basis_nor).normalized();
        }
    }

    let m = Mat3 { cols: [basis_tmp, basis_dir, basis_nor] };
    m.inverted().unwrap_or(Mat3::IDENTITY)
}
fn ctx_face(bm: &BMesh, l: usize) -> usize { bm.loo(l).f }

/// `bmo_connect_vert_pair_exec`.
pub fn bmo_connect_vert_pair_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let verts: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "verts")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_VERT).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    if verts.len() != 2 {
        return;
    }
    let v_pair = [verts[0], verts[1]];

    bmo_slot_buffer_flag_enable(bm, op, false, "faces_exclude", FACE_EXCLUDE);
    bmo_slot_buffer_flag_enable(bm, op, false, "verts_exclude", VERT_EXCLUDE);

    /* Scale CONNECT_EPS_BASIS by pair extents (#113111). */
    let connect_eps = CONNECT_EPS_BASIS
        * bm.vert(v_pair[0])
            .co
            .length_squared()
            .max(bm.vert(v_pair[1]).co.length_squared())
            .sqrt()
            .max(1.0);

    let matrix = bm_vert_pair_to_matrix(bm, v_pair);
    let axis_sep = dot_m3_v3_row_x(&matrix, bm.vert(v_pair[0]).co);

    let mut ctx = Ctx {
        bm,
        pc: PathContext {
            matrix,
            axis_sep,
            connect_eps,
            v_pair,
            link_pool: Vec::new(),
        },
        states_arena: Vec::new(),
        heap: HeapSimple::new(),
    };

    /* add first vertex */
    ctx.states_arena.push(PathLinkState {
        link_last: None,
        dist: 0.0,
        co_prev: Vec3::ZERO,
    });
    state_link_add(&mut ctx, 0, (BM_VERT, v_pair[0]), None);
    ctx.heap.insert(ctx.states_arena[0].dist, 0);

    let mut state_best: Option<usize> = None;
    while !ctx.heap.is_empty() {
        let si = ctx.heap.pop_min().unwrap();
        let last = ctx.states_arena[si].link_last.unwrap();
        let ele = ctx.pc.link_pool[last].ele;
        if ele == (BM_VERT, v_pair[1]) {
            state_best = Some(si);
            break;
        } else if state_step(&mut ctx, si) {
            let d = ctx.states_arena[si].dist;
            ctx.heap.insert(d, si);
        }
        /* dead end: drop state (arena keeps memory) */
    }

    if let Some(si) = state_best {
        let bm: &mut BMesh = ctx.bm;
        let mut link_i = ctx.states_arena[si].link_last;
        while let Some(li) = link_i {
            let link = ctx.pc.link_pool[li].clone();
            if link.ele.0 == BM_EDGE {
                let e = link.ele.1;
                let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                let e_fac = state_calc_co_pair_fac(
                    &ctx.pc,
                    bm.vert(v1).co,
                    bm.vert(v2).co,
                );
                let v_new = crate::mods::bm_edge_split(bm, e, v1, None, e_fac);
                /* Face normals are used by connect_verts — recalc. */
                if bm.edge(e).l != BM_NIL {
                    let l_first = bm.edge(e).l;
                    let mut l_iter = l_first;
                    loop {
                        let f = bm.loo(l_iter).f;
                        crate::normals::bm_face_calc_normals(bm, f);
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l_first {
                            break;
                        }
                    }
                }
                bm.vpool[v_new].head.api_flag |= VERT_OUT;
            } else if link.ele.0 == BM_VERT {
                bm.vpool[link.ele.1].head.api_flag |= VERT_OUT;
            }
            link_i = link.next;
        }
    }
    ctx.bm.vpool[v_pair[0]].head.api_flag |= VERT_OUT;
    ctx.bm.vpool[v_pair[1]].head.api_flag |= VERT_OUT;

    let bm = ctx.bm;
    if state_best.is_some() {
        /* connect_verts verts=VERT_OUT faces_exclude=<slot> check_degenerate=true */
        let mut op_sub = bmo_op_init(bm, &crate::opdefines::OPDEF_CONNECT_VERTS, op.flag);
        if let Some(si2) = op_sub.slots_in.iter().position(|s| s.name == "verts") {
            op_sub.slots_in[si2].buf = crate::iterators::VertsOfMesh::new(bm)
                .filter(|&v| bm.vert(v).head.api_flag & VERT_OUT != 0)
                .map(|v| (BM_VERT, v))
                .collect();
        }
        if let (Some(dst), Some(src)) = (
            op_sub.slots_in.iter().position(|s| s.name == "faces_exclude"),
            op.slots_in.iter().find(|s| s.name == "faces_exclude"),
        ) {
            op_sub.slots_in[dst].buf = src.buf.clone();
        }
        if let Some(si2) = op_sub.slots_in.iter().position(|s| s.name == "check_degenerate") {
            op_sub.slots_in[si2].b = true;
        }
        bmo_op_exec(bm, &mut op_sub, crate::bmo::bmo_connect::bmo_connect_verts_exec);
        /* copy edges.out */
        if let (Some(dst), Some(src)) = (
            op.slots_out.iter().position(|s| s.name == "edges.out"),
            op_sub.slots_out.iter().find(|s| s.name == "edges.out"),
        ) {
            op.slots_out[dst].buf = src.buf.clone();
        }
        bmo_op_finish(bm, op_sub);
    }
}
