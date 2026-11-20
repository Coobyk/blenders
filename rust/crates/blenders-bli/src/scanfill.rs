//! Port of `scanfill.cc` + `scanfill_utils.cc` — `BLI_scanfill_*`
//! polygon tessellator (scanline fill with hole/loose/isect options).
//!
//! PORT_NOTE: `ScanFillVert`/`ScanFillEdge` pointers → `usize`
//! arena indices (`NONE` = -1 sentinel); the `prev`/`next` links are
//! reused for both the main edge list and the per-vertex scan lists,
//! exactly like upstream.

use std::collections::HashMap;

use blenders_math::geom::{axis_dominant_to_m3_negate, dist_squared_to_line_v2};
use blenders_math::matrix_c::mul_v2_m3v3;
use blenders_math::vector::{cos_v2v2v2, Vec2, Vec3};
use blenders_math::vector_inline::{
    add_newell_cross_v3_v3v3, compare_v2v2, compare_v3v3, len_squared_v2v2, normalize_v3,
};

fn fv2(a: [f32; 2]) -> Vec2 {
    Vec2::new(a[0], a[1])
}
fn fv3(a: [f32; 3]) -> Vec3 {
    Vec3::new(a[0], a[1], a[2])
}

const NONE: i64 = -1;

const SF_EPSILON: f32 = 0.00003;
const SF_EPSILON_SQ: f32 = SF_EPSILON * SF_EPSILON;

const SF_VERT_NEW: u8 = 0;
const SF_VERT_AVAILABLE: u8 = 1;
const SF_VERT_ZERO_LEN: u8 = 2;
const SF_EDGE_NEW: u8 = 0;
const SF_EDGE_INTERNAL: u8 = 2;
const SF_POLY_NEW: u8 = 0;
const SF_POLY_VALID: u8 = 1;

pub const SF_POLY_UNSET: u16 = u16::MAX;

/// `BLI_SCANFILL_CALC_*` flag bits.
pub const SCANFILL_CALC_REMOVE_DOUBLES: i32 = 1 << 1;
pub const SCANFILL_CALC_POLYS: i32 = 1 << 2;
pub const SCANFILL_CALC_HOLES: i32 = 1 << 3;
pub const SCANFILL_CALC_LOOSE: i32 = 1 << 4;

#[derive(Clone, Copy)]
pub struct ScanFillVert {
    pub next: i64,
    pub prev: i64,
    pub co: [f32; 3],
    pub xy: [f32; 2],
    /// `tmp.v`/`tmp.u` union member.
    pub tmp_v: i64,
    pub keyindex: u32,
    pub poly_nr: u16,
    pub edge_count: u8,
    pub f: u8,
    pub user_flag: u32,
}

#[derive(Clone, Copy)]
pub struct ScanFillEdge {
    pub next: i64,
    pub prev: i64,
    pub v1: i64,
    pub v2: i64,
    pub poly_nr: u16,
    pub f: u8,
    pub user_flag: u32,
    /// `tmp.c` union member.
    pub tmp_c: i32,
}

/// `ScanFillContext` — owns the vert/edge/face arenas.
#[derive(Default)]
pub struct ScanFillContext {
    pub verts: Vec<ScanFillVert>,
    pub edges: Vec<ScanFillEdge>,
    pub faces: Vec<[i64; 3]>,
    /// Linked-list heads (index or NONE).
    pub vert_first: i64,
    pub vert_last: i64,
    pub edge_first: i64,
    pub edge_last: i64,
    pub poly_nr: u16,
}

// ---- ListBase ops over the arenas ----
impl ScanFillContext {
    /// `BLI_scanfill_begin`.
    pub fn new() -> Self {
        Self {
            poly_nr: SF_POLY_UNSET,
            vert_first: NONE,
            vert_last: NONE,
            edge_first: NONE,
            edge_last: NONE,
            ..Default::default()
        }
    }

    /// `BLI_scanfill_vert_add` → vert index.
    pub fn vert_add(&mut self, co: [f32; 3]) -> i64 {
        let i = self.verts.len() as i64;
        self.verts.push(ScanFillVert {
            next: NONE,
            prev: self.vert_last,
            co,
            xy: [0.0; 2],
            tmp_v: NONE,
            keyindex: 0,
            poly_nr: self.poly_nr,
            edge_count: 0,
            f: SF_VERT_NEW,
            user_flag: 0,
        });
        if self.vert_last != NONE {
            self.verts[self.vert_last as usize].next = i;
        } else {
            self.vert_first = i;
        }
        self.vert_last = i;
        i
    }

    /// `BLI_scanfill_edge_add` → edge index.
    pub fn edge_add(&mut self, v1: i64, v2: i64) -> i64 {
        let i = self.edges.len() as i64;
        self.edges.push(ScanFillEdge {
            next: NONE,
            prev: self.edge_last,
            v1,
            v2,
            poly_nr: self.poly_nr,
            f: SF_EDGE_NEW,
            user_flag: 0,
            tmp_c: 0,
        });
        if self.edge_last != NONE {
            self.edges[self.edge_last as usize].next = i;
        } else {
            self.edge_first = i;
        }
        self.edge_last = i;
        i
    }

    fn face_add(&mut self, v1: i64, v2: i64, v3: i64) {
        self.faces.push([v1, v2, v3]);
    }

    /// `BLI_remlink` on the edge list.
    fn edge_remlink(&mut self, e: i64) {
        let (p, n) = (self.edges[e as usize].prev, self.edges[e as usize].next);
        if p != NONE {
            self.edges[p as usize].next = n;
        } else {
            self.edge_first = n;
        }
        if n != NONE {
            self.edges[n as usize].prev = p;
        } else {
            self.edge_last = p;
        }
        self.edges[e as usize].prev = NONE;
        self.edges[e as usize].next = NONE;
    }

    /// `BLI_insertlinkbefore` — insert `e` before `at` in the
    /// `(head, last)` pair of indices (scan lists share the link fields).
    fn link_insert_before(head: &mut i64, last: &mut i64, edges: &mut [ScanFillEdge], at: i64, e: i64) {
        let p = edges[at as usize].prev;
        edges[e as usize].prev = p;
        edges[e as usize].next = at;
        edges[at as usize].prev = e;
        if p != NONE {
            edges[p as usize].next = e;
        } else {
            *head = e;
        }
        let _ = last;
    }
    fn link_insert_after(head: &mut i64, last: &mut i64, edges: &mut [ScanFillEdge], at: i64, e: i64) {
        let n = edges[at as usize].next;
        edges[e as usize].next = n;
        edges[e as usize].prev = at;
        edges[at as usize].next = e;
        if n != NONE {
            edges[n as usize].prev = e;
        } else {
            *last = e;
        }
        let _ = head;
    }
    /// `BLI_addtail` onto an arbitrary (head,last) pair.
    fn link_addtail(head: &mut i64, last: &mut i64, edges: &mut [ScanFillEdge], e: i64) {
        edges[e as usize].prev = *last;
        edges[e as usize].next = NONE;
        if *last != NONE {
            edges[*last as usize].next = e;
        } else {
            *head = e;
        }
        *last = e;
    }
    /// `BLI_remlink` from an arbitrary (head,last) pair.
    fn link_remlink(head: &mut i64, last: &mut i64, edges: &mut [ScanFillEdge], e: i64) {
        let (p, n) = (edges[e as usize].prev, edges[e as usize].next);
        if p != NONE {
            edges[p as usize].next = n;
        } else {
            *head = n;
        }
        if n != NONE {
            edges[n as usize].prev = p;
        } else {
            *last = p;
        }
        edges[e as usize].prev = NONE;
        edges[e as usize].next = NONE;
    }
    /// `BLI_remlink` on the vert list.
    fn vert_remlink(&mut self, v: i64) {
        let (p, n) = (self.verts[v as usize].prev, self.verts[v as usize].next);
        if p != NONE {
            self.verts[p as usize].next = n;
        } else {
            self.vert_first = n;
        }
        if n != NONE {
            self.verts[n as usize].prev = p;
        } else {
            self.vert_last = p;
        }
        self.verts[v as usize].prev = NONE;
        self.verts[v as usize].next = NONE;
    }
}

#[derive(Clone, Copy)]
struct PolyFill {
    edges: u32,
    verts: u32,
    min_xy: [f32; 2],
    max_xy: [f32; 2],
    nr: u16,
    f: u8,
}

#[derive(Clone, Copy, Default)]
struct ScanFillVertLink {
    vert: i64,
    edge_first: i64,
    edge_last: i64,
}

fn boundisect(pf2: &PolyFill, pf1: &PolyFill) -> bool {
    if pf1.edges == 0 || pf2.edges == 0 {
        return false;
    }
    !(pf2.max_xy[0] < pf1.min_xy[0]
        || pf2.max_xy[1] < pf1.min_xy[1]
        || pf2.min_xy[0] > pf1.max_xy[0]
        || pf2.min_xy[1] > pf1.max_xy[1])
}

fn fill_target_map_recursive(
    pf_list: &[PolyFill],
    pf_len: usize,
    pf_target: usize,
    pf_test: usize,
    target_map: &mut [usize],
) {
    for b in pf_target + 1..pf_len {
        if target_map[b] != b {
            continue;
        }
        if boundisect(&pf_list[pf_test], &pf_list[b]) {
            target_map[b] = pf_target;
            fill_target_map_recursive(pf_list, pf_len, pf_target, b, target_map);
        }
    }
}

fn mergepolys_simp(ctx: &mut ScanFillContext, pf1: &mut PolyFill, pf2: &mut PolyFill) {
    for v in &mut ctx.verts {
        if v.poly_nr == pf2.nr {
            v.poly_nr = pf1.nr;
        }
    }
    for e in &mut ctx.edges {
        if e.poly_nr == pf2.nr {
            e.poly_nr = pf1.nr;
        }
    }
    pf1.verts += pf2.verts;
    pf1.edges += pf2.edges;
    pf1.max_xy[0] = pf1.max_xy[0].max(pf2.max_xy[0]);
    pf1.max_xy[1] = pf1.max_xy[1].max(pf2.max_xy[1]);
    pf1.min_xy[0] = pf1.min_xy[0].min(pf2.min_xy[0]);
    pf1.min_xy[1] = pf1.min_xy[1].min(pf2.min_xy[1]);
    pf1.f |= pf2.f;
    pf2.verts = 0;
    pf2.edges = 0;
}

fn testedgeside(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> bool {
    let inp = (v2[0] - v1[0]) * (v1[1] - v3[1]) + (v1[1] - v2[1]) * (v1[0] - v3[0]);
    if inp < 0.0 {
        return false;
    }
    if inp == 0.0 && (v1 == v3 || v2 == v3) {
        return false;
    }
    true
}

fn addedgetoscanvert(edges: &mut [ScanFillEdge], sc: &mut ScanFillVertLink, eed: i64, verts: &[ScanFillVert]) -> bool {
    if sc.edge_first == NONE {
        sc.edge_first = eed;
        sc.edge_last = eed;
        edges[eed as usize].prev = NONE;
        edges[eed as usize].next = NONE;
        return true;
    }
    let (x, y) = (verts[edges[eed as usize].v1 as usize].xy[0], verts[edges[eed as usize].v1 as usize].xy[1]);
    let mut fac1 = verts[edges[eed as usize].v2 as usize].xy[1] - y;
    if fac1 == 0.0 {
        fac1 = 1.0e10 * (verts[edges[eed as usize].v2 as usize].xy[0] - x);
    } else {
        fac1 = (x - verts[edges[eed as usize].v2 as usize].xy[0]) / fac1;
    }
    let mut ed = sc.edge_first;
    while ed != NONE {
        if edges[ed as usize].v2 == edges[eed as usize].v2 {
            return false;
        }
        let mut fac = verts[edges[ed as usize].v2 as usize].xy[1] - y;
        if fac == 0.0 {
            fac = 1.0e10 * (verts[edges[ed as usize].v2 as usize].xy[0] - x);
        } else {
            fac = (x - verts[edges[ed as usize].v2 as usize].xy[0]) / fac;
        }
        if fac > fac1 {
            break;
        }
        ed = edges[ed as usize].next;
    }
    if ed != NONE {
        ScanFillContext::link_insert_before(&mut sc.edge_first, &mut sc.edge_last, edges, ed, eed);
    } else {
        ScanFillContext::link_addtail(&mut sc.edge_first, &mut sc.edge_last, edges, eed);
    }
    true
}

/// `addedgetoscanlist` → the sc entry when the edge was already
/// linked (dup), else None.
fn addedgetoscanlist(
    ctx: &mut ScanFillContext,
    scdata: &mut [ScanFillVertLink],
    eed: i64,
) -> Option<usize> {
    {
        let (mut a, mut b) = (ctx.edges[eed as usize].v1, ctx.edges[eed as usize].v2);
        let (ay, by, ax, bx) = (
            ctx.verts[a as usize].xy[1],
            ctx.verts[b as usize].xy[1],
            ctx.verts[a as usize].xy[0],
            ctx.verts[b as usize].xy[0],
        );
        if (ay == by && ax > bx) || ay < by {
            std::mem::swap(&mut a, &mut b);
            ctx.edges[eed as usize].v1 = a;
            ctx.edges[eed as usize].v2 = b;
        }
    }
    let target = ctx.edges[eed as usize].v1;
    // bsearch in scdata sorted by vergscdata (y desc, x asc).
    let idx = scdata
        .binary_search_by(|sc| {
            let a = ctx.verts[sc.vert as usize].xy;
            let b = ctx.verts[target as usize].xy;
            // vergscdata returns >0 when x1 sorts after x2.
            if a[1] < b[1] {
                std::cmp::Ordering::Greater
            } else if a[1] > b[1] {
                std::cmp::Ordering::Less
            } else {
                a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal)
            }
        })
        .ok();
    match idx {
        None => None, // C prints an error and continues
        Some(i) => {
            if !addedgetoscanvert(&mut ctx.edges, &mut scdata[i], eed, &ctx.verts) {
                return Some(i);
            }
            None
        }
    }
}

fn boundinside_ev(edges: &[ScanFillEdge], verts: &[ScanFillVert], eed: i64, eve: i64) -> bool {
    let (a, b) = (edges[eed as usize].v1 as usize, edges[eed as usize].v2 as usize);
    let (va, vb, ve) = (&verts[a].xy, &verts[b].xy, &verts[eve as usize].xy);
    let (minx, maxx) = if va[0] < vb[0] { (va[0], vb[0]) } else { (vb[0], va[0]) };
    if ve[0] < minx || ve[0] > maxx {
        return false;
    }
    let (miny, maxy) = if va[1] < vb[1] { (va[1], vb[1]) } else { (vb[1], va[1]) };
    ve[1] >= miny && ve[1] <= maxy
}

fn testvertexnearedge(ctx: &mut ScanFillContext) {
    let mut vi = ctx.vert_first;
    while vi != NONE {
        let v_next = ctx.verts[vi as usize].next;
        if ctx.verts[vi as usize].edge_count == 1 {
            // Find the one edge containing eve.
            let mut ed1 = ctx.edge_first;
            while ed1 != NONE
                && !(ctx.edges[ed1 as usize].v1 == vi || ctx.edges[ed1 as usize].v2 == vi)
            {
                ed1 = ctx.edges[ed1 as usize].next;
            }
            if ed1 == NONE {
                vi = v_next;
                continue;
            }
            if ctx.edges[ed1 as usize].v1 == vi {
                ctx.edges[ed1 as usize].v1 = ctx.edges[ed1 as usize].v2;
                ctx.edges[ed1 as usize].v2 = vi;
            }
            let mut eed = ctx.edge_first;
            while eed != NONE {
                if eed != ed1
                    && ctx.edges[eed as usize].v1 != vi
                    && ctx.edges[eed as usize].v2 != vi
                    && ctx.verts[vi as usize].poly_nr == ctx.edges[eed as usize].poly_nr
                {
                    let e1 = ctx.edges[eed as usize].v1;
                    let e2 = ctx.edges[eed as usize].v2;
                    if compare_v2v2(ctx.verts[vi as usize].xy, ctx.verts[e1 as usize].xy,
                        SF_EPSILON,
                    ) {
                        ctx.edges[ed1 as usize].v2 = e1;
                        ctx.verts[e1 as usize].edge_count += 1;
                        ctx.verts[vi as usize].edge_count = 0;
                        break;
                    }
                    if compare_v2v2(ctx.verts[vi as usize].xy, ctx.verts[e2 as usize].xy,
                        SF_EPSILON,
                    ) {
                        ctx.edges[ed1 as usize].v2 = e2;
                        ctx.verts[e2 as usize].edge_count += 1;
                        ctx.verts[vi as usize].edge_count = 0;
                        break;
                    }
                    if boundinside_ev(&ctx.edges, &ctx.verts, eed, vi) {
                        let dist = dist_squared_to_line_v2(
                            fv2(ctx.verts[e1 as usize].xy),
                            fv2(ctx.verts[e2 as usize].xy),
                            fv2(ctx.verts[vi as usize].xy),
                        );
                        if dist < SF_EPSILON_SQ {
                            let new_e = ctx.edge_add(e1, vi);
                            let poly = ctx.edges[eed as usize].poly_nr;
                            ctx.edges[new_e as usize].poly_nr = poly;
                            ctx.edge_remlink(new_e);
                            // ed1 now points to the new edge chain.
                            ctx.edges[eed as usize].v1 = vi;
                            ctx.verts[vi as usize].edge_count = 3;
                            break;
                        }
                    }
                }
                eed = ctx.edges[eed as usize].next;
            }
        }
        vi = v_next;
    }
}

/// `splitlist` — move all elements of `temp` into ctx lists, then
/// move those with `poly_nr == nr` back to ctx (they stay); actually
/// C moves ctx lists → temp, then moves `nr` items temp → ctx.
fn splitlist(
    ctx: &mut ScanFillContext,
    tempve: &mut Vec<i64>,
    temped: &mut Vec<i64>,
    nr: u16,
) {
    // Move all ctx verts/edges into temp (ordered).
    let mut v = ctx.vert_first;
    while v != NONE {
        tempve.push(v);
        let n = ctx.verts[v as usize].next;
        ctx.verts[v as usize].prev = NONE;
        ctx.verts[v as usize].next = NONE;
        v = n;
    }
    let mut e = ctx.edge_first;
    while e != NONE {
        temped.push(e);
        let n = ctx.edges[e as usize].next;
        ctx.edges[e as usize].prev = NONE;
        ctx.edges[e as usize].next = NONE;
        e = n;
    }
    ctx.vert_first = NONE;
    ctx.vert_last = NONE;
    ctx.edge_first = NONE;
    ctx.edge_last = NONE;
    // Re-append those matching nr to ctx.
    let mut kept_v = Vec::new();
    for &vi in tempve.iter() {
        if ctx.verts[vi as usize].poly_nr == nr {
            let vi_ = vi;
            ctx.verts[vi_ as usize].prev = ctx.vert_last;
            ctx.verts[vi_ as usize].next = NONE;
            if ctx.vert_last != NONE {
                ctx.verts[ctx.vert_last as usize].next = vi_;
            } else {
                ctx.vert_first = vi_;
            }
            ctx.vert_last = vi_;
        } else {
            kept_v.push(vi);
        }
    }
    *tempve = kept_v;
    let mut kept_e = Vec::new();
    for &ei in temped.iter() {
        if ctx.edges[ei as usize].poly_nr == nr {
            ctx.edges[ei as usize].prev = ctx.edge_last;
            ctx.edges[ei as usize].next = NONE;
            if ctx.edge_last != NONE {
                ctx.edges[ctx.edge_last as usize].next = ei;
            } else {
                ctx.edge_first = ei;
            }
            ctx.edge_last = ei;
        } else {
            kept_e.push(ei);
        }
    }
    *temped = kept_e;
}

/// `scanfill` — triangulate one poly; returns faces added.
fn scanfill(ctx: &mut ScanFillContext, pf: &PolyFill, flag: i32) -> u32 {
    let nr = pf.nr;
    let mut twoconnected = false;

    if flag & SCANFILL_CALC_REMOVE_DOUBLES != 0 {
        for e in 0..ctx.edges.len() {
            let (a, b) = (ctx.edges[e].v1 as usize, ctx.edges[e].v2 as usize);
            if ctx.verts[a].xy == ctx.verts[b].xy {
                if ctx.verts[a].f == SF_VERT_ZERO_LEN && ctx.verts[b].f != SF_VERT_ZERO_LEN {
                    ctx.verts[b].f = SF_VERT_ZERO_LEN;
                    ctx.verts[b].tmp_v = ctx.verts[a].tmp_v;
                } else if ctx.verts[b].f == SF_VERT_ZERO_LEN && ctx.verts[a].f != SF_VERT_ZERO_LEN {
                    ctx.verts[a].f = SF_VERT_ZERO_LEN;
                    ctx.verts[a].tmp_v = ctx.verts[b].tmp_v;
                } else if ctx.verts[b].f == SF_VERT_ZERO_LEN && ctx.verts[a].f == SF_VERT_ZERO_LEN {
                    ctx.verts[a].tmp_v = ctx.verts[b].tmp_v;
                } else {
                    ctx.verts[b].f = SF_VERT_ZERO_LEN;
                    ctx.verts[b].tmp_v = a as i64;
                }
            }
        }
    }

    // Build sorted scdata.
    let mut scdata: Vec<ScanFillVertLink> = Vec::with_capacity(pf.verts as usize);
    {
        let mut v = ctx.vert_first;
        while v != NONE {
            let vv = ctx.verts[v as usize];
            if vv.poly_nr == nr && vv.f != SF_VERT_ZERO_LEN {
                ctx.verts[v as usize].f = SF_VERT_NEW;
                scdata.push(ScanFillVertLink {
                    vert: v,
                    edge_first: NONE,
                    edge_last: NONE,
                });
            }
            v = vv.next;
        }
    }
    scdata.sort_by(|x, y| {
        let a = ctx.verts[x.vert as usize].xy;
        let b = ctx.verts[y.vert as usize].xy;
        // vergscdata: y desc, then x asc.
        b[1].partial_cmp(&a[1]).unwrap().then(a[0].partial_cmp(&b[0]).unwrap())
    });
    let verts = scdata.len();

    if flag & SCANFILL_CALC_REMOVE_DOUBLES != 0 {
        // Move edges with v1==v2 in xy back to the main list.
        let mut e = ctx.edge_first;
        while e != NONE {
            let e_next = ctx.edges[e as usize].next;
            let (a, b) = (ctx.edges[e as usize].v1 as usize, ctx.edges[e as usize].v2 as usize);
            if ctx.verts[a].xy == ctx.verts[b].xy {
                ctx.edge_remlink(e);
            } else {
                ctx.edge_remlink(e);
                addedgetoscanlist(ctx, &mut scdata, e);
            }
            e = e_next;
        }
    } else {
        let mut e = ctx.edge_first;
        while e != NONE {
            let e_next = ctx.edges[e as usize].next;
            ctx.edge_remlink(e);
            if ctx.edges[e as usize].v1 != ctx.edges[e as usize].v2 {
                addedgetoscanlist(ctx, &mut scdata, e);
            }
            e = e_next;
        }
    }

    if pf.f == SF_POLY_NEW {
        twoconnected = true;
    }

    let mut totface = 0u32;
    let maxface = if flag & SCANFILL_CALC_HOLES != 0 {
        2 * verts as u32
    } else {
        verts as u32 - 2
    };

    for a in 0..verts {
        // Remove edges touching count-1 verts.
        {
            let mut ed1 = scdata[a].edge_first;
            while ed1 != NONE {
                let e_next = ctx.edges[ed1 as usize].next;
                let (v1, v2) = (ctx.edges[ed1 as usize].v1, ctx.edges[ed1 as usize].v2);
                if ctx.verts[v1 as usize].edge_count == 1 || ctx.verts[v2 as usize].edge_count == 1 {
                    let mut h = scdata[a].edge_first;
                    let mut l = scdata[a].edge_last;
                    ScanFillContext::link_remlink(&mut h, &mut l, &mut ctx.edges, ed1);
                    scdata[a].edge_first = h;
                    scdata[a].edge_last = l;
                    let mut eh = ctx.edge_first;
                    let mut el = ctx.edge_last;
                    ScanFillContext::link_addtail(&mut eh, &mut el, &mut ctx.edges, ed1);
                    ctx.edge_first = eh;
                    ctx.edge_last = el;
                    if ctx.verts[v1 as usize].edge_count > 1 {
                        ctx.verts[v1 as usize].edge_count -= 1;
                    }
                    if ctx.verts[v2 as usize].edge_count > 1 {
                        ctx.verts[v2 as usize].edge_count -= 1;
                    }
                } else {
                    ctx.verts[v2 as usize].f = SF_VERT_AVAILABLE;
                }
                ed1 = e_next;
            }
        }

        while scdata[a].edge_first != NONE {
            let ed1 = scdata[a].edge_first;
            let ed2 = ctx.edges[ed1 as usize].next;
            if totface >= maxface {
                break;
            }
            if ed2 == NONE {
                scdata[a].edge_first = NONE;
                scdata[a].edge_last = NONE;
                let mut eh = ctx.edge_first;
                let mut el = ctx.edge_last;
                ScanFillContext::link_addtail(&mut eh, &mut el, &mut ctx.edges, ed1);
                ctx.edge_first = eh;
                ctx.edge_last = el;
                ctx.verts[ctx.edges[ed1 as usize].v2 as usize].f = SF_VERT_NEW;
                ctx.verts[ctx.edges[ed1 as usize].v1 as usize].edge_count -= 1;
                ctx.verts[ctx.edges[ed1 as usize].v2 as usize].edge_count -= 1;
            } else {
                let v1 = ctx.edges[ed1 as usize].v2;
                let v2 = ctx.edges[ed1 as usize].v1;
                let v3 = ctx.edges[ed2 as usize].v2;
                if v1 == v2 || v2 == v3 {
                    break;
                }
                let miny = ctx.verts[v1 as usize].xy[1].min(ctx.verts[v3 as usize].xy[1]);
                let mut best_sc: Option<usize> = None;
                let mut angle_best_cos = -1.0f32;
                let mut firsttime = false;
                for b in a + 1..verts {
                    let sv = scdata[b].vert as usize;
                    if ctx.verts[sv].f == SF_VERT_NEW {
                        if ctx.verts[sv].xy[1] <= miny {
                            break;
                        }
                        let sxy = ctx.verts[sv].xy;
                        if testedgeside(ctx.verts[v1 as usize].xy, ctx.verts[v2 as usize].xy, sxy)
                            && testedgeside(ctx.verts[v2 as usize].xy, ctx.verts[v3 as usize].xy, sxy)
                            && testedgeside(ctx.verts[v3 as usize].xy, ctx.verts[v1 as usize].xy, sxy)
                        {
                            if best_sc.is_none() {
                                best_sc = Some(b);
                            } else {
                                if !firsttime {
                                    angle_best_cos = cos_v2v2v2(
                                        fv2(ctx.verts[v2 as usize].xy),
                                        fv2(ctx.verts[v1 as usize].xy),
                                        fv2(ctx.verts[scdata[best_sc.unwrap()].vert as usize].xy),
                                    );
                                    firsttime = true;
                                }
                                let c = cos_v2v2v2(
                                    fv2(ctx.verts[v2 as usize].xy),
                                    fv2(ctx.verts[v1 as usize].xy),
                                    fv2(sxy),
                                );
                                if c > angle_best_cos {
                                    best_sc = Some(b);
                                    angle_best_cos = c;
                                }
                            }
                        }
                    }
                }

                if let Some(bs) = best_sc {
                    let ed3 = ctx.edge_add(v2, scdata[bs].vert);
                    ctx.edge_remlink(ed3);
                    let mut h = scdata[a].edge_first;
                    let mut l = scdata[a].edge_last;
                    ScanFillContext::link_insert_before(&mut h, &mut l, &mut ctx.edges, ed2, ed3);
                    scdata[a].edge_first = h;
                    scdata[a].edge_last = l;
                    ctx.verts[ctx.edges[ed3 as usize].v2 as usize].f = SF_VERT_AVAILABLE;
                    ctx.edges[ed3 as usize].f = SF_EDGE_INTERNAL;
                    ctx.verts[ctx.edges[ed3 as usize].v1 as usize].edge_count += 1;
                    ctx.verts[ctx.edges[ed3 as usize].v2 as usize].edge_count += 1;
                } else {
                    ctx.face_add(v1, v2, v3);
                    totface += 1;
                    {
                        let mut h = scdata[a].edge_first;
                        let mut l = scdata[a].edge_last;
                        ScanFillContext::link_remlink(&mut h, &mut l, &mut ctx.edges, ed1);
                        scdata[a].edge_first = h;
                        scdata[a].edge_last = l;
                    }
                    {
                        let mut eh = ctx.edge_first;
                        let mut el = ctx.edge_last;
                        ScanFillContext::link_addtail(&mut eh, &mut el, &mut ctx.edges, ed1);
                        ctx.edge_first = eh;
                        ctx.edge_last = el;
                    }
                    ctx.verts[ctx.edges[ed1 as usize].v2 as usize].f = SF_VERT_NEW;
                    ctx.verts[ctx.edges[ed1 as usize].v1 as usize].edge_count -= 1;
                    ctx.verts[ctx.edges[ed1 as usize].v2 as usize].edge_count -= 1;
                    if ctx.edges[ed2 as usize].f == SF_EDGE_NEW && twoconnected {
                        let mut h = scdata[a].edge_first;
                        let mut l = scdata[a].edge_last;
                        ScanFillContext::link_remlink(&mut h, &mut l, &mut ctx.edges, ed2);
                        scdata[a].edge_first = h;
                        scdata[a].edge_last = l;
                        let mut eh = ctx.edge_first;
                        let mut el = ctx.edge_last;
                        ScanFillContext::link_addtail(&mut eh, &mut el, &mut ctx.edges, ed2);
                        ctx.edge_first = eh;
                        ctx.edge_last = el;
                        ctx.verts[ctx.edges[ed2 as usize].v2 as usize].f = SF_VERT_NEW;
                        ctx.verts[ctx.edges[ed2 as usize].v1 as usize].edge_count -= 1;
                        ctx.verts[ctx.edges[ed2 as usize].v2 as usize].edge_count -= 1;
                    }
                    let ed3 = ctx.edge_add(v1, v3);
                    ctx.edge_remlink(ed3);
                    ctx.edges[ed3 as usize].f = SF_EDGE_INTERNAL;
                    ctx.verts[v1 as usize].edge_count += 1;
                    ctx.verts[v3 as usize].edge_count += 1;
                    if let Some(sc1) = addedgetoscanlist(ctx, &mut scdata, ed3) {
                        ctx.verts[v1 as usize].edge_count -= 1;
                        ctx.verts[v3 as usize].edge_count -= 1;
                        let mut e3 = scdata[sc1].edge_first;
                        while e3 != NONE {
                            let (a_, b_) = (ctx.edges[e3 as usize].v1, ctx.edges[e3 as usize].v2);
                            if (a_ == v1 && b_ == v3) || (a_ == v3 && b_ == v1) {
                                if twoconnected {
                                    let mut h = scdata[sc1].edge_first;
                                    let mut l = scdata[sc1].edge_last;
                                    ScanFillContext::link_remlink(&mut h, &mut l, &mut ctx.edges, e3);
                                    scdata[sc1].edge_first = h;
                                    scdata[sc1].edge_last = l;
                                    let mut eh = ctx.edge_first;
                                    let mut el = ctx.edge_last;
                                    ScanFillContext::link_addtail(&mut eh, &mut el, &mut ctx.edges, e3);
                                    ctx.edge_first = eh;
                                    ctx.edge_last = el;
                                    ctx.verts[a_ as usize].edge_count -= 1;
                                    ctx.verts[b_ as usize].edge_count -= 1;
                                }
                                break;
                            }
                            e3 = ctx.edges[e3 as usize].next;
                        }
                    }
                }
            }

            // Sweep count<2 edges.
            let mut ed1 = scdata[a].edge_first;
            while ed1 != NONE {
                let e_next = ctx.edges[ed1 as usize].next;
                let (v1, v2) = (ctx.edges[ed1 as usize].v1, ctx.edges[ed1 as usize].v2);
                if ctx.verts[v1 as usize].edge_count < 2 || ctx.verts[v2 as usize].edge_count < 2 {
                    let mut h = scdata[a].edge_first;
                    let mut l = scdata[a].edge_last;
                    ScanFillContext::link_remlink(&mut h, &mut l, &mut ctx.edges, ed1);
                    scdata[a].edge_first = h;
                    scdata[a].edge_last = l;
                    let mut eh = ctx.edge_first;
                    let mut el = ctx.edge_last;
                    ScanFillContext::link_addtail(&mut eh, &mut el, &mut ctx.edges, ed1);
                    ctx.edge_first = eh;
                    ctx.edge_last = el;
                    if ctx.verts[v1 as usize].edge_count > 1 {
                        ctx.verts[v1 as usize].edge_count -= 1;
                    }
                    if ctx.verts[v2 as usize].edge_count > 1 {
                        ctx.verts[v2 as usize].edge_count -= 1;
                    }
                }
                ed1 = e_next;
            }
        }
    }

    totface
}

/// `isect_seg_seg_v2_point` → Some(point) when segments cross
/// (upstream returns 1 for a proper crossing).
fn isect_seg_seg_v2_point(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2], v4: [f32; 2]) -> Option<[f32; 2]> {
    let a1 = v2[0] - v1[0];
    let a2 = v2[1] - v1[1];
    let b1 = v3[0] - v4[0];
    let b2 = v3[1] - v4[1];
    let c1 = v3[0] - v1[0];
    let c2 = v3[1] - v1[1];
    let div = a1 * b2 - a2 * b1;
    if div == 0.0 {
        return None;
    }
    let lambda = (b1 * c2 - b2 * c1) / div;
    let mu = (a1 * c2 - a2 * c1) / div;
    if (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu) {
        return Some([v1[0] + lambda * a1, v1[1] + lambda * a2]);
    }
    None
}

/// `BLI_scanfill_calc_ex` — main entry; returns faces added.
pub fn scanfill_calc_ex(ctx: &mut ScanFillContext, flag: i32, nor_proj: Option<[f32; 3]>) -> u32 {
    // Mark edge verts available.
    for e in 0..ctx.edges.len() {
        let (a, b) = (ctx.edges[e].v1 as usize, ctx.edges[e].v2 as usize);
        ctx.verts[a].f = SF_VERT_AVAILABLE;
        ctx.verts[b].f = SF_VERT_AVAILABLE;
    }
    if !ctx.verts.iter().any(|v| v.f == SF_VERT_AVAILABLE) {
        return 0;
    }

    let mut n = match nor_proj {
        Some(np) => np,
        None => {
            let mut n = [0.0f32; 3];
            let mut v_prev = ctx.verts.last().map(|v| v.co).unwrap_or([0.0; 3]);
            for v in &ctx.verts {
                if !compare_v3v3(v_prev, v.co, SF_EPSILON) {
                    add_newell_cross_v3_v3v3(&mut n, v_prev, v.co);
                    v_prev = v.co;
                }
            }
            n
        }
    };
    if normalize_v3(&mut n) == 0.0 {
        return 0;
    }
    let mat_2d = axis_dominant_to_m3_negate(fv3(n));

    let mut poly: u16 = 0;
    if ctx.poly_nr != SF_POLY_UNSET {
        poly = ctx.poly_nr + 1;
        ctx.poly_nr = SF_POLY_UNSET;
    }

    if flag & SCANFILL_CALC_POLYS != 0 && poly == 0 {
        for vi in 0..ctx.verts.len() {
            let xy = mul_v2_m3v3(mat_2d, fv3(ctx.verts[vi].co));
            ctx.verts[vi].xy = [xy.x, xy.y];
            if ctx.verts[vi].poly_nr == SF_POLY_UNSET {
                ctx.verts[vi].poly_nr = poly;
                let mut ok = true;
                let mut toggle = 0u32;
                while ok {
                    ok = false;
                    toggle += 1;
                    for ei in 0..ctx.edges.len() {
                        let (a, b) = (ctx.edges[ei].v1 as usize, ctx.edges[ei].v2 as usize);
                        if ctx.verts[a].poly_nr == SF_POLY_UNSET && ctx.verts[b].poly_nr == poly {
                            ctx.verts[a].poly_nr = poly;
                            ctx.edges[ei].poly_nr = poly;
                            ok = true;
                        } else if ctx.verts[b].poly_nr == SF_POLY_UNSET && ctx.verts[a].poly_nr == poly {
                            ctx.verts[b].poly_nr = poly;
                            ctx.edges[ei].poly_nr = poly;
                            ok = true;
                        } else if ctx.edges[ei].poly_nr == SF_POLY_UNSET
                            && ctx.verts[a].poly_nr == poly
                            && ctx.verts[b].poly_nr == poly
                        {
                            ctx.edges[ei].poly_nr = poly;
                            ok = true;
                        }
                    }
                }
                poly += 1;
            }
        }
    } else if poly != 0 {
        for v in &mut ctx.verts {
            let xy = mul_v2_m3v3(mat_2d, fv3(v.co));
            v.xy = [xy.x, xy.y];
        }
    } else {
        poly = 1;
        for v in &mut ctx.verts {
            let xy = mul_v2_m3v3(mat_2d, fv3(v.co));
            v.xy = [xy.x, xy.y];
            v.poly_nr = 0;
        }
        for e in &mut ctx.edges {
            e.poly_nr = 0;
        }
    }

    if flag & SCANFILL_CALC_LOOSE != 0 {
        for e in 0..ctx.edges.len() {
            let (a, b) = (ctx.edges[e].v1 as usize, ctx.edges[e].v2 as usize);
            ctx.verts[a].edge_count = ctx.verts[a].edge_count.wrapping_add(1);
            ctx.verts[b].edge_count = ctx.verts[b].edge_count.wrapping_add(1);
            if ctx.verts[a].edge_count > 250 || ctx.verts[b].edge_count > 250 {
                return 0;
            }
        }
        testvertexnearedge(ctx);
        // Iteratively remove degree-1 verts and their edges.
        let mut ok = true;
        let mut toggle = 0u32;
        while ok {
            ok = false;
            toggle += 1;
            let mut e = if toggle & 1 == 1 { ctx.edge_first } else { ctx.edge_last };
            while e != NONE {
                let e_next = if toggle & 1 == 1 {
                    ctx.edges[e as usize].next
                } else {
                    ctx.edges[e as usize].prev
                };
                let (a, b) = (ctx.edges[e as usize].v1, ctx.edges[e as usize].v2);
                if ctx.verts[a as usize].edge_count == 1 {
                    ctx.verts[b as usize].edge_count -= 1;
                    ctx.vert_remlink(a);
                    ctx.edge_remlink(e);
                    ok = true;
                } else if ctx.verts[b as usize].edge_count == 1 {
                    ctx.verts[a as usize].edge_count -= 1;
                    ctx.vert_remlink(b);
                    ctx.edge_remlink(e);
                    ok = true;
                }
                e = e_next;
            }
        }
        if ctx.edge_first == NONE {
            return 0;
        }
    } else {
        for e in 0..ctx.edges.len() {
            let (a, b) = (ctx.edges[e].v1 as usize, ctx.edges[e].v2 as usize);
            ctx.verts[a].edge_count = ctx.verts[a].edge_count.wrapping_add(1);
            ctx.verts[b].edge_count = ctx.verts[b].edge_count.wrapping_add(1);
        }
    }

    // Build PolyFill array.
    let mut pflist: Vec<PolyFill> = (0..poly)
        .map(|a| PolyFill {
            edges: 0,
            verts: 0,
            min_xy: [1.0e20; 2],
            max_xy: [-1.0e20; 2],
            nr: a,
            f: SF_POLY_NEW,
        })
        .collect();
    {
        let mut e = ctx.edge_first;
        while e != NONE {
            pflist[ctx.edges[e as usize].poly_nr as usize].edges += 1;
            e = ctx.edges[e as usize].next;
        }
    }
    {
        let mut v = ctx.vert_first;
        while v != NONE {
            let vv = &ctx.verts[v as usize];
            let pf = &mut pflist[vv.poly_nr as usize];
            pf.verts += 1;
            pf.min_xy[0] = pf.min_xy[0].min(vv.xy[0]);
            pf.min_xy[1] = pf.min_xy[1].min(vv.xy[1]);
            pf.max_xy[0] = pf.max_xy[0].max(vv.xy[0]);
            pf.max_xy[1] = pf.max_xy[1].max(vv.xy[1]);
            if vv.edge_count > 2 {
                pf.f = SF_POLY_VALID;
            }
            v = vv.next;
        }
    }

    if flag & SCANFILL_CALC_HOLES != 0 && poly > 1 {
        let mut target_map: Vec<usize> = (0..poly as usize).collect();
        for a in 0..poly as usize {
            if target_map[a] != a {
                continue;
            }
            fill_target_map_recursive(&pflist, poly as usize, a, a, &mut target_map);
        }
        for a in 0..poly as usize {
            if target_map[a] != a {
                let (dst_i, src_i) = (target_map[a], a);
                // Need &mut two elements — split.
                let src = pflist[src_i];
                let dst = &mut pflist[dst_i];
                let mut src_mut = src;
                mergepolys_simp(ctx, dst, &mut src_mut);
                pflist[src_i] = src_mut;
            }
        }
    }

    // Split per-poly and fill.
    let mut tempve = Vec::new();
    let mut temped = Vec::new();
    let mut totfaces = 0u32;
    for a in 0..poly {
        if pflist[a as usize].edges > 1 {
            splitlist(ctx, &mut tempve, &mut temped, pflist[a as usize].nr);
            totfaces += scanfill(ctx, &pflist[a as usize], flag);
        }
    }
    // Restore leftovers into ctx lists (order preserved).
    for vi in tempve {
        ctx.verts[vi as usize].prev = ctx.vert_last;
        ctx.verts[vi as usize].next = NONE;
        if ctx.vert_last != NONE {
            ctx.verts[ctx.vert_last as usize].next = vi;
        } else {
            ctx.vert_first = vi;
        }
        ctx.vert_last = vi;
    }
    for ei in temped {
        ctx.edges[ei as usize].prev = ctx.edge_last;
        ctx.edges[ei as usize].next = NONE;
        if ctx.edge_last != NONE {
            ctx.edges[ctx.edge_last as usize].next = ei;
        } else {
            ctx.edge_first = ei;
        }
        ctx.edge_last = ei;
    }

    totfaces
}

/// `BLI_scanfill_calc`.
pub fn scanfill_calc(ctx: &mut ScanFillContext, flag: i32) -> u32 {
    scanfill_calc_ex(ctx, flag, None)
}

// ---- scanfill_utils.cc — self-intersection preprocessing ----

const V_ISISECT: u32 = 1;
const E_ISISECT: u32 = 1;
const E_ISDELETE: u32 = 2;

#[derive(Default, Clone, Copy)]
struct PolyInfo {
    edge_first: i64,
    edge_last: i64,
}

struct ScanFillIsect {
    co: [f32; 3],
    v: i64,
}

fn edge_step(poly_info: &[PolyInfo], poly_nr: u16, edges: &[ScanFillEdge], v_prev: i64, v_curr: i64, e_curr: i64) -> i64 {
    let pi = &poly_info[poly_nr as usize];
    let mut eed = if edges[e_curr as usize].next != NONE && e_curr != pi.edge_last {
        edges[e_curr as usize].next
    } else {
        pi.edge_first
    };
    {
        let (a, b) = (edges[eed as usize].v1, edges[eed as usize].v2);
        if (a == v_curr || b == v_curr) && a != v_prev && b != v_prev {
            return eed;
        }
    }
    eed = if edges[e_curr as usize].prev != NONE && e_curr != pi.edge_first {
        edges[e_curr as usize].prev
    } else {
        pi.edge_last
    };
    let (a, b) = (edges[eed as usize].v1, edges[eed as usize].v2);
    if (a == v_curr || b == v_curr) && a != v_prev && b != v_prev {
        return eed;
    }
    NONE
}

/// `scanfill_preprocess_self_isect` — subdivide edges at crossings.
fn scanfill_preprocess_self_isect(
    ctx: &mut ScanFillContext,
    poly_info: &mut [PolyInfo],
    poly_nr: u16,
    filledgebase_first: &mut i64,
    filledgebase_last: &mut i64,
) -> bool {
    let mut isect_hash: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut isects: Vec<ScanFillIsect> = Vec::new();

    {
        let pi = &poly_info[poly_nr as usize];
        let mut eed = pi.edge_first;
        while eed != NONE {
            let mut eed_other = ctx.edges[eed as usize].next;
            while eed_other != NONE {
                let (a, b) = (ctx.edges[eed as usize].v1, ctx.edges[eed as usize].v2);
                let (c, d) = (
                    ctx.edges[eed_other as usize].v1,
                    ctx.edges[eed_other as usize].v2,
                );
                if eed != eed_other && a != c && a != d && b != c && b != d {
                    if let Some(pt) = isect_seg_seg_v2_point(
                        ctx.verts[a as usize].co[0..2].try_into().unwrap(),
                        ctx.verts[b as usize].co[0..2].try_into().unwrap(),
                        ctx.verts[c as usize].co[0..2].try_into().unwrap(),
                        ctx.verts[d as usize].co[0..2].try_into().unwrap(),
                    ) {
                        let mut co = [pt[0], pt[1], ctx.verts[a as usize].co[2]];
                        let _ = &mut co;
                        let v = ctx.vert_add(co);
                        ctx.verts[v as usize].poly_nr = ctx.verts[a as usize].poly_nr;
                        ctx.verts[v as usize].user_flag |= V_ISISECT;
                        let isect_i = isects.len() as i64;
                        isects.push(ScanFillIsect { co, v });
                        isect_hash.entry(eed).or_default().push(isect_i);
                        isect_hash.entry(eed_other).or_default().push(isect_i);
                        ctx.edges[eed as usize].user_flag |= E_ISISECT;
                        ctx.edges[eed_other as usize].user_flag |= E_ISISECT;
                    }
                }
                eed_other = if eed_other == pi.edge_last {
                    NONE
                } else {
                    ctx.edges[eed_other as usize].next
                };
            }
            eed = if eed == pi.edge_last {
                NONE
            } else {
                ctx.edges[eed as usize].next
            };
        }
    }

    if isect_hash.is_empty() {
        return false;
    }

    {
        let mut eed = poly_info[poly_nr as usize].edge_first;
        while eed != NONE {
            if ctx.edges[eed as usize].user_flag & E_ISISECT == 0 {
                eed = if eed == poly_info[poly_nr as usize].edge_last {
                    NONE
                } else {
                    ctx.edges[eed as usize].next
                };
                continue;
            }
            let Some(list) = isect_hash.get(&eed).cloned() else {
                eed = ctx.edges[eed as usize].next;
                continue;
            };
            if poly_info[poly_nr as usize].edge_last == eed {
                poly_info[poly_nr as usize].edge_last = NONE;
            }
            let mut list = list;
            if list.len() > 1 {
                let co2 = ctx.verts[ctx.edges[eed as usize].v2 as usize].co;
                list.sort_by(|&ia, &ib| {
                    let da = len_squared_v2v2(
                        [co2[0], co2[1]],
                        [isects[ia as usize].co[0], isects[ia as usize].co[1]],
                    );
                    let db = len_squared_v2v2(
                        [co2[0], co2[1]],
                        [isects[ib as usize].co[0], isects[ib as usize].co[1]],
                    );
                    // sort_cb: a>b → -1 → descending distance.
                    db.partial_cmp(&da).unwrap()
                });
            }
            // Copy edge after eed, move eed to filledgebase.
            let (v1, v2) = (ctx.edges[eed as usize].v1, ctx.edges[eed as usize].v2);
            let eed_tmp = ctx.edge_add(v1, v2);
            ctx.edge_remlink(eed_tmp);
            let (mut h, mut l) = (ctx.edge_first, ctx.edge_last);
            ScanFillContext::link_insert_after(&mut h, &mut l, &mut ctx.edges, eed, eed_tmp);
            ctx.edge_first = h;
            ctx.edge_last = l;
            ctx.edge_remlink(eed);
            ScanFillContext::link_addtail(filledgebase_first, filledgebase_last, &mut ctx.edges, eed);
            if poly_info[poly_nr as usize].edge_first == eed {
                poly_info[poly_nr as usize].edge_first = eed_tmp;
            }
            let mut eed_cur = eed_tmp;

            for isect_i in list {
                let iv = isects[isect_i as usize].v;
                let eed_subd = ctx.edge_add(iv, ctx.edges[eed_cur as usize].v2);
                ctx.edges[eed_subd as usize].poly_nr = poly_nr;
                ctx.edges[eed_cur as usize].v2 = iv;
                ctx.edge_remlink(eed_subd);
                let (mut h, mut l) = (ctx.edge_first, ctx.edge_last);
                ScanFillContext::link_insert_after(&mut h, &mut l, &mut ctx.edges, eed_cur, eed_subd);
                ctx.edge_first = h;
                ctx.edge_last = l;
                eed_cur = eed_subd;
            }

            if poly_info[poly_nr as usize].edge_last == NONE {
                poly_info[poly_nr as usize].edge_last = eed_cur;
            }
            eed = ctx.edges[eed_cur as usize].next;
        }
    }

    // Mark/delete trailing edges on the isect chain (the inside/
    // E_ISDELETE walk over the poly loop).
    {
        let pi_first = poly_info[poly_nr as usize].edge_first;
        let pi_last = poly_info[poly_nr as usize].edge_last;
        if pi_first != NONE && pi_last != NONE {
            // Find the edge whose v2->co[0] is minimal (choose start).
            let mut min_x = f32::MAX;
            let mut e_curr = pi_last;
            let mut e_next = pi_first;
            let mut eed_prev = pi_last;
            let mut eed = pi_first;
            while eed != NONE {
                if ctx.verts[ctx.edges[eed as usize].v2 as usize].co[0] < min_x {
                    min_x = ctx.verts[ctx.edges[eed as usize].v2 as usize].co[0];
                    e_curr = eed_prev;
                    e_next = eed;
                }
                eed_prev = eed;
                eed = if eed == pi_last {
                    NONE
                } else {
                    ctx.edges[eed as usize].next
                };
            }
            let e_init = e_curr;
            let mut v_prev = ctx.edges[e_curr as usize].v1;
            let mut v_curr = ctx.edges[e_curr as usize].v2;
            let mut inside = false;
            loop {
                let (a, b) = (ctx.edges[e_next as usize].v1, ctx.edges[e_next as usize].v2);
                let v_next = if a == v_curr { b } else { a };
                if inside {
                    ctx.edges[e_next as usize].user_flag |= E_ISDELETE;
                }
                if ctx.verts[v_next as usize].user_flag & V_ISISECT != 0 {
                    inside = !inside;
                }
                v_prev = v_curr;
                v_curr = v_next;
                e_curr = e_next;
                e_next = edge_step(poly_info, poly_nr, &ctx.edges, v_prev, v_curr, e_curr);
                if e_curr == e_init || e_next == NONE {
                    break;
                }
            }
        }
    }

    true
}

/// `BLI_scanfill_calc_self_isect` — find/subdivide crossing edges,
/// moving deleted leftovers into `remverts`/`remedges` index lists.
pub fn scanfill_calc_self_isect(ctx: &mut ScanFillContext) -> (Vec<i64>, Vec<i64>, bool) {
    let mut remverts: Vec<i64> = Vec::new();
    let mut remedges: Vec<i64> = Vec::new();
    if ctx.poly_nr == SF_POLY_UNSET {
        return (remverts, remedges, false);
    }
    let poly_num = ctx.poly_nr as usize + 1;
    let mut poly_info: Vec<PolyInfo> = vec![PolyInfo::default(); poly_num];
    for p in &mut poly_info {
        p.edge_first = NONE;
        p.edge_last = NONE;
    }
    if ctx.poly_nr == 0 {
        poly_info[0].edge_first = ctx.edge_first;
        poly_info[0].edge_last = ctx.edge_last;
    } else {
        let mut poly_nr = 0usize;
        let mut e = ctx.edge_first;
        while e != NONE {
            if poly_info[poly_nr].edge_last != NONE
                && ctx.edges[poly_info[poly_nr].edge_last as usize].poly_nr
                    != ctx.edges[e as usize].poly_nr
            {
                poly_nr += 1;
            }
            if poly_info[poly_nr].edge_first == NONE {
                poly_info[poly_nr].edge_first = e;
                poly_info[poly_nr].edge_last = e;
            } else if ctx.edges[poly_info[poly_nr].edge_last as usize].poly_nr
                == ctx.edges[e as usize].poly_nr
            {
                poly_info[poly_nr].edge_last = e;
            }
            e = ctx.edges[e as usize].next;
        }
    }

    let mut changed = false;
    let mut rem_first = NONE;
    let mut rem_last = NONE;
    for nr in 0..poly_num {
        changed |= scanfill_preprocess_self_isect(
            ctx,
            &mut poly_info,
            nr as u16,
            &mut rem_first,
            &mut rem_last,
        );
    }

    if !changed {
        return (remverts, remedges, false);
    }

    // Move E_ISDELETE edges to rem list.
    {
        let mut e = ctx.edge_first;
        while e != NONE {
            let e_next = ctx.edges[e as usize].next;
            if ctx.edges[e as usize].user_flag & E_ISDELETE != 0 {
                ctx.edge_remlink(e);
                ScanFillContext::link_addtail(&mut rem_first, &mut rem_last, &mut ctx.edges, e);
            }
            e = e_next;
        }
    }
    // Reset flags; collect unreferenced verts.
    {
        for v in &mut ctx.verts {
            v.user_flag = 0;
            v.poly_nr = SF_POLY_UNSET;
        }
        for e in &mut ctx.edges {
            if e.v1 != NONE {
                ctx.verts[e.v1 as usize].user_flag = 1;
            }
            if e.v2 != NONE {
                ctx.verts[e.v2 as usize].user_flag = 1;
            }
            e.poly_nr = SF_POLY_UNSET;
        }
        let mut v = ctx.vert_first;
        while v != NONE {
            let v_next = ctx.verts[v as usize].next;
            if ctx.verts[v as usize].user_flag != 1 {
                ctx.vert_remlink(v);
                remverts.push(v);
            } else {
                ctx.verts[v as usize].user_flag = 0;
            }
            v = v_next;
        }
    }
    // Collect rem edge list into remedges.
    {
        let mut e = rem_first;
        while e != NONE {
            remedges.push(e);
            e = ctx.edges[e as usize].next;
        }
    }
    ctx.poly_nr = SF_POLY_UNSET;
    (remverts, remedges, changed)
}

/// `BLI_scanfill_calc` convenience over a 3D point list + index
/// pairs — builds ctx, fills, returns triangles (point indices).
pub fn scanfill_triangulate(points: &[[f32; 3]], edges: &[[usize; 2]], flag: i32) -> Vec<[usize; 3]> {
    let mut ctx = ScanFillContext::new();
    let mut vmap = Vec::with_capacity(points.len());
    for p in points {
        vmap.push(ctx.vert_add(*p));
    }
    for e in edges {
        ctx.edge_add(vmap[e[0]], vmap[e[1]]);
    }
    scanfill_calc(&mut ctx, flag);
    ctx.faces
        .iter()
        .map(|f| {
            [
                vmap.iter().position(|&x| x == f[0]).unwrap_or(0),
                vmap.iter().position(|&x| x == f[1]).unwrap_or(0),
                vmap.iter().position(|&x| x == f[2]).unwrap_or(0),
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fills_square_and_concave() {
        // Square fan via edge loop.
        let pts = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let edges = [[0, 1], [1, 2], [2, 3], [3, 0]];
        let tris = scanfill_triangulate(&pts, &edges, SCANFILL_CALC_POLYS);
        assert_eq!(tris.len(), 2);
        // Concave arrow.
        let ar = [
            [0.0f32, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 0.5, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let edges = [[0, 1], [1, 2], [2, 3], [3, 4], [4, 0]];
        let tris = scanfill_triangulate(&ar, &edges, SCANFILL_CALC_POLYS);
        assert_eq!(tris.len(), 3);
    }
}
