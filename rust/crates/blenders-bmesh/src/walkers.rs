//! Port of `bmesh_walkers_impl.cc` + `bmesh_walkers.cc` — `BMWalker` types.
//!
//! `BMWalker` becomes a kind-tagged struct holding a state stack and two
//! visited sets (visit_set / visit_set_alt). Masks read `head.api_flag`
//! (BMO flag layers ride api_flag until operator flags are ported).
//! UV/data-region walkers (`BMW_LOOPDATA_*`, `BMW_UV_VERTEX`, `BMW_EDGELOOP`
//! UV variants) are PORT_PENDING (customdata).

use crate::delete::bm_edge_is_boundary;
use crate::query::*;
use crate::structure::*;
use crate::{BMesh, BM_EDGE, BM_ELEM_HIDDEN, BM_ELEM_SEAM, BM_ELEM_SMOOTH, BM_FACE, BM_LOOP,
            BM_NIL, BM_VERT};
use std::collections::HashSet;

/// `BMW_FLAG_*` walker flags.
pub const BMW_FLAG_TEST_HIDDEN: u32 = 1 << 0;

/// `BMW_DELIMIT_*`.
pub const BMW_DELIMIT_EDGE_MARK_SEAM: u32 = 1 << 0;
pub const BMW_DELIMIT_EDGE_MARK_SHARP: u32 = 1 << 1;
pub const BMW_DELIMIT_FACE_MARK_MATERIAL: u32 = 1 << 2;
pub const BMW_DELIMIT_EDGE_LOOP_NGONS: u32 = 1 << 3;
pub const BMW_DELIMIT_EDGE_RING_NGONS: u32 = 1 << 4;

/// `BMW` walker types.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BMWType {
    VertShell,
    LoopShell,
    LoopShellWire,
    FaceShell,
    ConnectedVertex,
    Islandbound,
    Island,
    IslandManifold,
    EdgeLoop,
    FaceLoop,
    EdgeRing,
    Edgeboundary,
    NonManifoldedge,
    UVEdge,
}

#[derive(Clone, Copy)]
enum WalkState {
    Edge(usize),                       // shell walkers: current edge
    Loop(usize),                       // loop-shell walkers
    WireElem(u8, usize),               // loop-shell-wire: loop or edge elem
    Vert(usize),
    Islandbound { base: usize, curloop: usize, lastv: usize },
    Island { cur: usize },
    EdgeLoop {
        cur: usize, lastv: usize, is_boundary: bool, is_single: bool, f_hub: usize,
    },
    FaceLoop { l: usize, no_calc: bool },
    Edgering { l: usize, wireedge: usize, no_calc: bool },
    NonManifoldEdge {
        start: usize, startv: usize, cur: usize, lastv: usize, face_count: i32,
    },
}

/// `BMWalker`.
pub struct BMWalker {
    pub kind: BMWType,
    pub flag: u32,
    pub delimit: u32,
    pub mask_vert: u8,
    pub mask_edge: u8,
    pub mask_face: u8,
    stack: Vec<WalkState>,
    visit_set: HashSet<(u8, usize)>,
    visit_set_alt: HashSet<(u8, usize)>,
    /// `walker->layer` loop data for `BMW_UVEDGE` — PORT_PENDING CustomData:
    /// set a UV accessor before walking (`BM_ELEM_CD_GET_FLOAT_P` equivalent).
    pub uv_at: Option<std::sync::Arc<dyn Fn(&BMesh, usize) -> blenders_math::vector::Vec2>>,
}

fn hflag_of(bm: &BMesh, htype: u8, i: usize) -> u8 {
    match htype {
        BM_VERT => bm.vert(i).head.hflag,
        BM_EDGE => bm.edge(i).head.hflag,
        BM_LOOP => bm.loo(i).head.hflag,
        _ => bm.face(i).head.hflag,
    }
}
fn api_of(bm: &BMesh, htype: u8, i: usize) -> u8 {
    match htype {
        BM_VERT => bm.vert(i).head.api_flag,
        BM_EDGE => bm.edge(i).head.api_flag,
        BM_LOOP => bm.loo(i).head.api_flag,
        _ => bm.face(i).head.api_flag,
    }
}

impl BMWalker {
    pub fn new(kind: BMWType, flag: u32, delimit: u32) -> Self {
        BMWalker {
            kind,
            flag,
            delimit,
            mask_vert: 0,
            mask_edge: 0,
            mask_face: 0,
            stack: Vec::new(),
            visit_set: HashSet::new(),
            visit_set_alt: HashSet::new(),
            uv_at: None,
        }
    }

    pub fn current(&self) -> Option<WalkState> {
        self.stack.last().copied()
    }

    fn mask_check(&self, bm: &BMesh, htype: u8, i: usize) -> bool {
        if i == BM_NIL {
            return false;
        }
        if (self.flag & BMW_FLAG_TEST_HIDDEN) != 0 && (hflag_of(bm, htype, i) & BM_ELEM_HIDDEN) != 0
        {
            return false;
        }
        let mask = match htype {
            BM_VERT => self.mask_vert,
            BM_EDGE => self.mask_edge,
            _ => self.mask_face,
        };
        mask == 0 || (api_of(bm, htype, i) & mask) != 0
    }

    /* --- visit helpers -------------------------------------------------- */

    fn visit_edge(&mut self, bm: &BMesh, e: usize) {
        if self.visit_set.contains(&(BM_EDGE, e)) {
            return;
        }
        if !self.mask_check(bm, BM_EDGE, e) {
            return;
        }
        self.stack.push(WalkState::Edge(e));
        self.visit_set.insert((BM_EDGE, e));
    }

    fn visit_loop(&mut self, bm: &BMesh, l: usize) {
        if self.visit_set.contains(&(BM_LOOP, l)) {
            return;
        }
        if !self.mask_check(bm, BM_FACE, bm.loo(l).f) {
            return;
        }
        self.stack.push(WalkState::Loop(l));
        self.visit_set.insert((BM_LOOP, l));
    }

    fn visit_edge_wire(&mut self, bm: &BMesh, e: usize) {
        debug_assert!(self.edge_is_wire(bm, e));
        if self.visit_set_alt.contains(&(BM_EDGE, e)) {
            return;
        }
        if !self.mask_check(bm, BM_EDGE, e) {
            return;
        }
        self.stack.push(WalkState::WireElem(BM_EDGE, e));
        self.visit_set_alt.insert((BM_EDGE, e));
    }

    fn visit_vert(&mut self, bm: &BMesh, v: usize) {
        if self.visit_set.contains(&(BM_VERT, v)) {
            return;
        }
        if !self.mask_check(bm, BM_VERT, v) {
            return;
        }
        self.stack.push(WalkState::Vert(v));
        self.visit_set.insert((BM_VERT, v));
    }

    fn visit_vert_wire(&mut self, bm: &BMesh, v: usize, e_from: usize) {
        if self.visit_set_alt.contains(&(BM_VERT, v)) {
            return;
        }
        if !self.mask_check(bm, BM_VERT, v) {
            return;
        }
        let mut e = bm.vert(v).e;
        if e == BM_NIL {
            self.visit_set_alt.insert((BM_VERT, v));
            return;
        }
        let e_first = e;
        loop {
            if self.edge_is_wire(bm, e) && e != e_from {
                let v_other = if bm.edge(e).v1 == v {
                    bm.edge(e).v2
                } else {
                    bm.edge(e).v1
                };
                self.visit_edge_wire(bm, e);
                /* check for stepping onto a non-wire vert's loops */
                for l in crate::iterators::LoopsOfVert::new(bm, v_other) {
                    self.visit_loop(bm, l);
                }
            }
            e = bmesh_disk_edge_next(bm, e, v);
            if e == e_first || e == BM_NIL {
                break;
            }
        }
        self.visit_set_alt.insert((BM_VERT, v));
    }

    fn edge_is_wire(&self, bm: &BMesh, e: usize) -> bool {
        if (self.flag & BMW_FLAG_TEST_HIDDEN) != 0 {
            if bm.edge(e).l == BM_NIL {
                return true;
            }
            /* wire if all faces hidden */
            let l0 = bm.edge(e).l;
            let mut l = l0;
            loop {
                if (bm.face(bm.loo(l).f).head.hflag & BM_ELEM_HIDDEN) == 0 {
                    return false;
                }
                l = bm.loo(l).radial_next;
                if l == l0 {
                    break;
                }
            }
            true
        } else {
            bm.edge(e).l == BM_NIL
        }
    }

    /// `BMW_begin` — seed the walk from an element (htype + index).
    pub fn begin(&mut self, bm: &mut BMesh, htype: u8, i: usize) {
        match self.kind {
            BMWType::VertShell | BMWType::FaceShell => {
                let e = if htype == BM_EDGE {
                    i
                } else if htype == BM_VERT {
                    bm.vert(i).e
                } else {
                    BM_NIL
                };
                if e != BM_NIL {
                    self.visit_edge(bm, e);
                }
            }
            BMWType::LoopShell | BMWType::LoopShellWire => match htype {
                BM_LOOP => {
                    self.visit_loop(bm, i);
                    if self.kind == BMWType::LoopShellWire {
                        self.visit_vert_wire(bm, bm.loo(i).v, BM_NIL);
                    }
                }
                BM_VERT => {
                    if self.kind == BMWType::LoopShellWire {
                        self.visit_vert_wire(bm, i, BM_NIL);
                    }
                    for l in crate::iterators::LoopsOfVert::new(bm, i) {
                        self.visit_loop(bm, l);
                    }
                }
                BM_EDGE => {
                    if self.mask_check(bm, BM_EDGE, i) {
                        if self.kind == BMWType::LoopShellWire {
                            self.visit_vert_wire(bm, bm.edge(i).v1, BM_NIL);
                            self.visit_vert_wire(bm, bm.edge(i).v2, BM_NIL);
                        }
                    }
                    let l0 = bm.edge(i).l;
                    if l0 != BM_NIL {
                        let mut l = l0;
                        loop {
                            self.visit_loop(bm, l);
                            self.visit_loop(bm, bm.loo(l).next);
                            l = bm.loo(l).radial_next;
                            if l == l0 {
                                break;
                            }
                        }
                    }
                }
                BM_FACE => {
                    let l0 = bm.face(i).l_first;
                    let mut l = l0;
                    loop {
                        self.visit_loop(bm, l);
                        l = bm.loo(l).next;
                        if l == l0 {
                            break;
                        }
                    }
                }
                _ => {}
            },
            BMWType::ConnectedVertex => {
                if htype == BM_VERT {
                    self.visit_vert(bm, i);
                } else if htype == BM_EDGE {
                    self.visit_vert(bm, bm.edge(i).v1);
                }
            }
            BMWType::Islandbound => {
                if htype == BM_LOOP {
                    self.stack.push(WalkState::Islandbound {
                        base: i,
                        curloop: i,
                        lastv: bm.loo(i).v,
                    });
                    self.visit_set.insert((BM_LOOP, i));
                }
            }
            BMWType::Island | BMWType::IslandManifold => {
                if htype == BM_FACE && self.mask_check(bm, BM_FACE, i) {
                    self.stack.push(WalkState::Island { cur: i });
                    self.visit_set.insert((BM_FACE, i));
                }
            }
            BMWType::EdgeLoop => self.edgeloop_begin(bm, i),
            BMWType::FaceLoop => self.faceloop_begin(bm, i),
            BMWType::EdgeRing => self.edgering_begin(bm, i),
            BMWType::Edgeboundary => {
                /* `bmw_EdgeboundaryWalker_begin`: data is a boundary edge. */
                if htype == BM_EDGE && !self.visit_set.contains(&(BM_EDGE, i)) {
                    debug_assert!(crate::delete::bm_edge_is_boundary(bm, i));
                    self.stack.push(WalkState::Edge(i));
                    self.visit_set.insert((BM_EDGE, i));
                }
            }
            BMWType::NonManifoldedge => {
                if htype == BM_EDGE && !self.visit_set.contains(&(BM_EDGE, i)) {
                    self.stack.push(WalkState::NonManifoldEdge {
                        start: i,
                        cur: i,
                        startv: bm.edge(i).v1,
                        lastv: bm.edge(i).v1,
                        face_count: crate::query::bm_edge_face_count(bm, i),
                    });
                    self.visit_set.insert((BM_EDGE, i));
                }
            }
            BMWType::UVEdge => {
                if htype == BM_LOOP && !self.visit_set.contains(&(BM_LOOP, i)) {
                    self.stack.push(WalkState::Loop(i));
                    self.visit_set.insert((BM_LOOP, i));
                }
            }
        }
    }

    /// `BMW_current_yield` — element the top state yields (htype, index).
    pub fn current_yield(&self, bm: &BMesh) -> Option<(u8, usize)> {
        match self.stack.last()? {
            WalkState::Edge(e) => Some((BM_EDGE, *e)),
            WalkState::Loop(l) => Some((BM_LOOP, *l)),
            WalkState::WireElem(ht, i) => Some((*ht, *i)),
            WalkState::Vert(v) => Some((BM_VERT, *v)),
            WalkState::Islandbound { curloop, .. } => Some((BM_LOOP, *curloop)),
            WalkState::Island { cur } => Some((BM_FACE, *cur)),
            WalkState::EdgeLoop { cur, .. } => Some((BM_EDGE, *cur)),
            WalkState::FaceLoop { l, .. } => Some((BM_FACE, bm.loo(*l).f)),
            WalkState::NonManifoldEdge { cur, .. } => Some((BM_EDGE, *cur)),
            WalkState::Edgering { l, wireedge, .. } => {
                if *wireedge != BM_NIL {
                    Some((BM_EDGE, *wireedge))
                } else {
                    Some((BM_EDGE, bm.loo(*l).e))
                }
            }
        }
    }

    /// `BMW_step` — pop the top state, expand from it, return its yield.
    pub fn step(&mut self, bm: &mut BMesh) -> Option<(u8, usize)> {
        let st = self.stack.pop()?;
        match st {
            WalkState::Edge(e) => match self.kind {
                BMWType::VertShell => {
                    for i in 0..2 {
                        let v = if i == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                        let edges: Vec<usize> =
                            crate::iterators::EdgesOfVert::new(bm, v).collect();
                        for e2 in edges {
                            self.visit_edge(bm, e2);
                        }
                    }
                    Some((BM_EDGE, e))
                }
                BMWType::FaceShell => {
                    if bm.edge(e).l != BM_NIL {
                        let l0 = bm.edge(e).l;
                        let mut l_iter = l0;
                        loop {
                            let f = bm.loo(l_iter).f;
                            let l_first = bm.face(f).l_first;
                            let mut l = l_first;
                            loop {
                                let e2 = bm.loo(l).e;
                                if e2 != e {
                                    self.visit_edge(bm, e2);
                                }
                                l = bm.loo(l).next;
                                if l == l_first {
                                    break;
                                }
                            }
                            l_iter = bm.loo(l_iter).radial_next;
                            if l_iter == l0 {
                                break;
                            }
                        }
                    }
                    Some((BM_EDGE, e))
                }
                BMWType::Edgeboundary => {
                    if self.mask_check(bm, BM_EDGE, e) {
                        for i in 0..2 {
                            let v = if i == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                            for e_other in
                                crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>()
                            {
                                if e_other != e
                                    && crate::delete::bm_edge_is_boundary(bm, e_other)
                                    && !self.visit_set.contains(&(BM_EDGE, e_other))
                                    && self.mask_check(bm, BM_EDGE, e_other)
                                {
                                    self.stack.push(WalkState::Edge(e_other));
                                    self.visit_set.insert((BM_EDGE, e_other));
                                }
                            }
                        }
                    }
                    Some((BM_EDGE, e))
                }
                _ => Some((BM_EDGE, e)),
            },
            WalkState::Loop(l) => {
                if self.kind == BMWType::UVEdge {
                    self.uvedge_step_impl(bm, l);
                } else {
                    self.loopshell_step_impl(bm, l);
                    if self.kind == BMWType::LoopShellWire {
                        self.visit_vert_wire(bm, bm.loo(l).v, BM_NIL);
                    }
                }
                Some((BM_LOOP, l))
            },
            WalkState::NonManifoldEdge {
                start,
                startv,
                cur,
                lastv,
                face_count,
            } => {
                /* `bmw_NonManifoldedgeWalker_step`. */
                let mut l_cur = BM_NIL;
                let mut v_found = BM_NIL;
                'pass: for pass in 0..2 {
                    let e = if pass == 0 { cur } else { start };
                    let v = if pass == 0 {
                        crate::structure::bm_edge_other_vert(bm, e, lastv)
                    } else {
                        startv
                    };
                    v_found = v;
                    let l0 = bm.edge(e).l;
                    if l0 == BM_NIL {
                        continue;
                    }
                    let mut l = l0;
                    loop {
                        let l_next = bmw_nonmanifold_find_next(bm, l, v, face_count);
                        if l_next != BM_NIL
                            && !self.visit_set.contains(&(BM_EDGE, bm.loo(l_next).e))
                        {
                            if l_cur == BM_NIL {
                                l_cur = l_next;
                            } else if bm.loo(l_cur).e != bm.loo(l_next).e {
                                l_cur = BM_NIL;
                                break;
                            }
                        }
                        l = bm.loo(l).radial_next;
                        if l == l0 {
                            break;
                        }
                    }
                    if l_cur != BM_NIL {
                        break 'pass;
                    }
                }
                if l_cur != BM_NIL {
                    debug_assert!(!self.visit_set.contains(&(BM_EDGE, bm.loo(l_cur).e)));
                    self.stack.push(WalkState::NonManifoldEdge {
                        start,
                        startv,
                        cur: bm.loo(l_cur).e,
                        lastv: v_found,
                        face_count,
                    });
                    self.visit_set.insert((BM_EDGE, bm.loo(l_cur).e));
                }
                Some((BM_EDGE, cur))
            }
            WalkState::WireElem(ht, i) => {
                if ht == BM_LOOP {
                    self.loopshell_step_impl(bm, i);
                    self.visit_vert_wire(bm, bm.loo(i).v, BM_NIL);
                } else {
                    self.visit_vert_wire(bm, bm.edge(i).v1, i);
                    self.visit_vert_wire(bm, bm.edge(i).v2, i);
                }
                Some((ht, i))
            }
            WalkState::Vert(v) => {
                let edges: Vec<usize> = crate::iterators::EdgesOfVert::new(bm, v).collect();
                for e in edges {
                    let v2 = if bm.edge(e).v1 == v {
                        bm.edge(e).v2
                    } else {
                        bm.edge(e).v1
                    };
                    self.visit_vert(bm, v2);
                }
                Some((BM_VERT, v))
            }
            WalkState::Islandbound { curloop, lastv, .. } => {
                self.islandbound_step(bm, curloop, lastv)
            }
            WalkState::Island { cur } => {
                self.island_step_impl(bm, cur, self.kind == BMWType::IslandManifold);
                Some((BM_FACE, cur))
            }
            WalkState::EdgeLoop {
                cur,
                lastv,
                is_boundary,
                is_single,
                f_hub,
            } => {
                self.edgeloop_step(bm, cur, lastv, is_boundary, is_single, f_hub);
                Some((BM_EDGE, cur))
            }
            WalkState::FaceLoop { l, no_calc } => {
                self.faceloop_step(bm, l, no_calc);
                Some((BM_FACE, bm.loo(l).f))
            }
            WalkState::Edgering { l, wireedge, no_calc } => {
                if wireedge != BM_NIL {
                    return Some((BM_EDGE, wireedge));
                }
                self.edgering_step(bm, l, no_calc);
                Some((BM_EDGE, bm.loo(l).e))
            }
        }
    }

    /* --- per-kind stepping ---------------------------------------------- */

    fn loopshell_step_impl(&mut self, bm: &BMesh, l: usize) {
        self.visit_loop(bm, bm.loo(l).next);
        self.visit_loop(bm, bm.loo(l).prev);
        let e_pair = [bm.loo(l).e, bm.loo(bm.loo(l).prev).e];
        for e in e_pair {
            if !self.mask_check(bm, BM_EDGE, e) {
                continue;
            }
            let l0 = bm.edge(e).l;
            if l0 == BM_NIL {
                continue;
            }
            let mut l_iter = l0;
            loop {
                let l_radial = if bm.loo(l_iter).v == bm.loo(l).v {
                    l_iter
                } else {
                    bm.loo(l_iter).next
                };
                if l != l_radial {
                    self.visit_loop(bm, l_radial);
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l0 {
                    break;
                }
            }
        }
    }

    fn islandbound_step(&mut self, bm: &BMesh, curloop: usize, lastv: usize) -> Option<(u8, usize)> {
        let mut l = curloop;
        let mut e = bm.loo(l).e;
        let v = if bm.edge(e).v1 == lastv {
            bm.edge(e).v2
        } else {
            bm.edge(e).v1
        };
        loop {
            l = bm_loop_other_edge_loop(bm, l, v);
            if bm_loop_is_manifold(bm, l) {
                l = bm.loo(l).radial_next;
                e = bm.loo(l).e;
                if !self.mask_check(bm, BM_FACE, bm.loo(l).f) {
                    l = bm.loo(l).radial_next;
                    break;
                }
            } else {
                e = bm.loo(l).e;
                break;
            }
        }
        let _ = e;
        if l == curloop {
            return None;
        }
        if self.visit_set.contains(&(BM_LOOP, l)) {
            return Some((BM_LOOP, curloop));
        }
        self.visit_set.insert((BM_LOOP, l));
        self.stack.push(WalkState::Islandbound {
            base: curloop,
            curloop: l,
            lastv: v,
        });
        Some((BM_LOOP, curloop))
    }

    fn island_step_impl(&mut self, bm: &BMesh, f: usize, only_manifold: bool) {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let e = bm.loo(l_iter).e;
            if !self.mask_check(bm, BM_EDGE, e) {
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first {
                    break;
                }
                continue;
            }
            let mut l_radial_iter = bm.loo(l_iter).radial_next;
            if only_manifold && l_radial_iter != l_iter {
                let mut face_count = 1;
                loop {
                    if self.mask_check(bm, BM_FACE, bm.loo(l_radial_iter).f) {
                        face_count += 1;
                        if face_count == 3 {
                            break;
                        }
                    }
                    l_radial_iter = bm.loo(l_radial_iter).radial_next;
                    if l_radial_iter == l_iter {
                        break;
                    }
                }
                if face_count != 2 {
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l_first {
                        break;
                    }
                    continue;
                }
            }
            /* walk radial: push unvisited manifold faces */
            let l0 = bm.loo(l_iter).radial_next;
            let mut l_r = l0;
            loop {
                let f2 = bm.loo(l_r).f;
                if !self.visit_set.contains(&(BM_FACE, f2))
                    && self.mask_check(bm, BM_FACE, f2)
                {
                    self.stack.push(WalkState::Island { cur: f2 });
                    self.visit_set.insert((BM_FACE, f2));
                }
                l_r = bm.loo(l_r).radial_next;
                if l_r == l_iter {
                    break;
                }
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }

    /* --- EdgeLoop walker ------------------------------------------------- */

    fn edgeloop_begin(&mut self, bm: &mut BMesh, e: usize) {
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        let vert_edge_count = [
            bm_vert_edge_count_nonwire(bm, v1),
            bm_vert_edge_count_nonwire(bm, v2),
        ];
        let vert_face_count = [bm_vert_face_count(bm, v1), bm_vert_face_count(bm, v2)];
        let is_boundary = bm_edge_is_boundary(bm, e);
        let is_single = is_boundary && bm_edge_is_single(bm, e);
        let mut f_hub = BM_NIL;
        if !is_boundary
            && ((vert_edge_count[0] == 3 && vert_face_count[0] == 3)
                || (vert_edge_count[1] == 3 && vert_face_count[1] == 3))
        {
            /* pick the biggest face as hub (only 5+ sides) */
            let mut f_best = BM_NIL;
            for f in crate::iterators::FacesOfEdge::new(bm, e) {
                if f_best == BM_NIL || bm.face(f_best).len < bm.face(f).len {
                    f_best = f;
                }
            }
            if f_best != BM_NIL && bm.face(f_best).len > 4 {
                f_hub = f_best;
            }
        }
        self.stack.push(WalkState::EdgeLoop {
            cur: e,
            lastv: v1,
            is_boundary,
            is_single,
            f_hub,
        });
        self.visit_set.insert((BM_EDGE, e));

        /* Rewind: walk until stack drains, keep last yielded state. */
        let mut owalk: Option<WalkState> = None;
        while let Some(s) = self.stack.last().copied() {
            owalk = Some(s);
            self.step(bm);
        }
        if let Some(WalkState::EdgeLoop {
            cur, lastv, is_boundary, is_single, f_hub,
        }) = owalk
        {
            let lastv_new = if bm.edge(cur).v1 == lastv {
                bm.edge(cur).v2
            } else {
                bm.edge(cur).v1
            };
            self.stack.push(WalkState::EdgeLoop {
                cur,
                lastv: lastv_new,
                is_boundary,
                is_single,
                f_hub,
            });
            self.visit_set.clear();
            self.visit_set.insert((BM_EDGE, cur));
        }
    }

    fn edgeloop_step(
        &mut self,
        bm: &mut BMesh,
        e: usize,
        lastv: usize,
        is_boundary: bool,
        is_single: bool,
        f_hub: usize,
    ) {
        let mut l = bm.edge(e).l;
        if f_hub != BM_NIL {
            /* INTERIOR NGON EDGE */
            let v = if bm.edge(e).v1 == lastv {
                bm.edge(e).v2
            } else {
                bm.edge(e).v1
            };
            if bm_vert_edge_count_nonwire(bm, v) == 3 {
                let lf = bm_face_other_vert_loop(bm, f_hub, lastv, v);
                let nexte = bmesh_disk_edge_exists(bm, v, bm.loo(lf).v);
                if nexte != BM_NIL
                    && self.mask_check(bm, BM_EDGE, nexte)
                    && !self.visit_set.contains(&(BM_EDGE, nexte))
                    && !bm_edge_is_boundary(bm, nexte)
                {
                    self.stack.push(WalkState::EdgeLoop {
                        cur: nexte,
                        lastv: v,
                        is_boundary,
                        is_single,
                        f_hub,
                    });
                    self.visit_set.insert((BM_EDGE, nexte));
                }
            }
            return;
        }
        if l == BM_NIL {
            /* WIRE EDGE */
            for i in 0..2 {
                let v = if i == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                let edges: Vec<usize> = crate::iterators::EdgesOfVert::new(bm, v).collect();
                for nexte in edges {
                    if bm.edge(nexte).l == BM_NIL
                        && self.mask_check(bm, BM_EDGE, nexte)
                        && !self.visit_set.contains(&(BM_EDGE, nexte))
                    {
                        self.stack.push(WalkState::EdgeLoop {
                            cur: nexte,
                            lastv: v,
                            is_boundary,
                            is_single,
                            f_hub,
                        });
                        self.visit_set.insert((BM_EDGE, nexte));
                    }
                }
            }
            return;
        }
        let v = if bm.edge(e).v1 == lastv {
            bm.edge(e).v2
        } else {
            bm.edge(e).v1
        };
        if !is_boundary {
            /* NORMAL EDGE WITH FACES */
            let vert_edge_tot = bm_vert_edge_count_nonwire(bm, v);
            if vert_edge_tot == 4 || vert_edge_tot == 2 {
                let i_opposite = vert_edge_tot / 2;
                let mut i = 0;
                loop {
                    l = bm_loop_other_edge_loop(bm, l, v);
                    if bm_edge_is_manifold(bm, bm.loo(l).e) {
                        l = bm.loo(l).radial_next;
                    } else {
                        l = BM_NIL;
                        break;
                    }
                    i += 1;
                    if i == i_opposite {
                        break;
                    }
                }
            } else {
                l = BM_NIL;
            }
        } else {
            /* BOUNDARY EDGE */
            let mut has_corner_delimit = false;
            if bm_vert_edge_count_nonwire(bm, v) == 2 {
                /* corner where two boundary edges meet */
                let e2 = bmesh_disk_edge_next(bm, e, v);
                if e2 != BM_NIL && bm_edge_is_boundary(bm, e2) {
                    has_corner_delimit = true;
                }
            }
            if !has_corner_delimit {
                loop {
                    l = bm_loop_other_edge_loop(bm, l, v);
                    if bm_edge_is_manifold(bm, bm.loo(l).e) {
                        l = bm.loo(l).radial_next;
                    } else if bm_edge_is_boundary(bm, bm.loo(l).e) {
                        break;
                    } else {
                        l = BM_NIL;
                        break;
                    }
                }
            } else {
                l = BM_NIL;
            }
        }
        if l != BM_NIL && self.edgeloop_delimit_mark_check(bm, v, e, l) {
            l = BM_NIL;
        }
        if l != BM_NIL && (self.delimit & BMW_DELIMIT_EDGE_LOOP_NGONS) != 0 {
            if is_single != bm_edge_is_single(bm, bm.loo(l).e) {
                l = BM_NIL;
            }
        }
        if l != BM_NIL && l != bm.edge(e).l {
            let nexte = bm.loo(l).e;
            if self.mask_check(bm, BM_EDGE, nexte)
                && !self.visit_set.contains(&(BM_EDGE, nexte))
            {
                self.stack.push(WalkState::EdgeLoop {
                    cur: nexte,
                    lastv: v,
                    is_boundary,
                    is_single,
                    f_hub,
                });
                self.visit_set.insert((BM_EDGE, nexte));
            }
        }
    }

    fn edgeloop_delimit_mark_check(&self, bm: &BMesh, v: usize, e: usize, l: usize) -> bool {
        if (self.delimit & BMW_DELIMIT_EDGE_MARK_SEAM) != 0
            && self.edgeloop_delimit_by_mark(bm, v, e, l, |f| {
                (f & BM_ELEM_SEAM) != 0
            })
        {
            return true;
        }
        if (self.delimit & BMW_DELIMIT_EDGE_MARK_SHARP) != 0
            && self.edgeloop_delimit_by_mark(bm, v, e, l, |f| {
                (f & BM_ELEM_SMOOTH) == 0
            })
        {
            return true;
        }
        false
    }

    fn edgeloop_delimit_by_mark(
        &self,
        bm: &BMesh,
        v: usize,
        e: usize,
        l: usize,
        mark: impl Fn(u8) -> bool,
    ) -> bool {
        let le = bm.loo(l).e;
        if mark(bm.edge(e).head.hflag) {
            if !mark(bm.edge(le).head.hflag)
                && !((self.flag & BMW_FLAG_TEST_HIDDEN) != 0
                    && (bm.edge(le).head.hflag & BM_ELEM_HIDDEN) != 0)
                && bm.edge(le).l != BM_NIL
            {
                return true;
            }
        } else {
            for e_other in crate::iterators::EdgesOfVert::new(bm, v) {
                if mark(bm.edge(e_other).head.hflag)
                    && !((self.flag & BMW_FLAG_TEST_HIDDEN) != 0
                        && (bm.edge(e_other).head.hflag & BM_ELEM_HIDDEN) != 0)
                    && bm.edge(e_other).l != BM_NIL
                {
                    return true;
                }
            }
        }
        false
    }

    /* --- FaceLoop walker ------------------------------------------------ */

    fn faceloop_include_face(&self, bm: &BMesh, l: usize) -> bool {
        if bm.face(bm.loo(l).f).len != 4 {
            return false;
        }
        if !self.mask_check(bm, BM_FACE, bm.loo(l).f) {
            return false;
        }
        if self.visit_set.contains(&(BM_FACE, bm.loo(l).f))
            && self.visit_set_alt.contains(&(BM_EDGE, bm.loo(l).e))
        {
            return false;
        }
        true
    }

    fn faceloop_edge_begins_loop(&self, bm: &BMesh, e: usize) -> bool {
        if self.edge_is_wire(bm, e) {
            return false;
        }
        if bm_edge_is_boundary(bm, e) && !self.faceloop_include_face(bm, bm.edge(e).l) {
            return false;
        }
        bm_edge_is_manifold(bm, e)
    }

    fn faceloop_delimit_check(&self, bm: &BMesh, f_a: usize, e: usize, f_b: usize) -> bool {
        if (self.delimit & (BMW_DELIMIT_EDGE_MARK_SEAM | BMW_DELIMIT_EDGE_MARK_SHARP)) != 0 {
            if (self.delimit & BMW_DELIMIT_EDGE_MARK_SEAM) != 0
                && (bm.edge(e).head.hflag & BM_ELEM_SEAM) != 0
            {
                return true;
            }
            if (self.delimit & BMW_DELIMIT_EDGE_MARK_SHARP) != 0
                && (bm.edge(e).head.hflag & BM_ELEM_SMOOTH) == 0
            {
                return true;
            }
        }
        if (self.delimit & BMW_DELIMIT_FACE_MARK_MATERIAL) != 0
            && bm.face(f_a).mat_nr != bm.face(f_b).mat_nr
        {
            return true;
        }
        false
    }

    fn faceloop_begin(&mut self, bm: &mut BMesh, e: usize) {
        if !self.faceloop_edge_begins_loop(bm, e) {
            return;
        }
        let l = bm.edge(e).l;
        self.stack.push(WalkState::FaceLoop { l, no_calc: false });
        self.visit_set.insert((BM_FACE, bm.loo(l).f));

        /* Rewind. */
        let mut owalk: Option<WalkState> = None;
        while let Some(s) = self.stack.last().copied() {
            owalk = Some(s);
            self.step(bm);
        }
        if let Some(WalkState::FaceLoop { l, .. }) = owalk {
            self.stack.push(WalkState::FaceLoop { l, no_calc: false });
            self.visit_set_alt.clear();
            self.visit_set_alt.insert((BM_EDGE, bm.loo(l).e));
            self.visit_set.clear();
            self.visit_set.insert((BM_FACE, bm.loo(l).f));
            /* start on a delimiting edge: add both sides */
            let l_rad = bm.loo(l).radial_next;
            if l_rad != l
                && self.faceloop_delimit_check(
                    bm,
                    bm.loo(l).f,
                    bm.loo(l).e,
                    bm.loo(l_rad).f,
                )
            {
                self.stack.push(WalkState::FaceLoop {
                    l: l_rad,
                    no_calc: false,
                });
                self.visit_set.insert((BM_FACE, bm.loo(l_rad).f));
            }
        }
    }

    fn faceloop_step(&mut self, bm: &BMesh, l_cur: usize, no_calc: bool) {
        let f = bm.loo(l_cur).f;
        if no_calc {
            return;
        }
        let mut l = bm.loo(l_cur).radial_next;
        if !self.faceloop_include_face(bm, l) {
            /* go to the opposite side of the quad */
            let mut l2 = bm.loo(bm.loo(l_cur).next).next;
            if !bm_edge_is_manifold(bm, bm.loo(l2).e) {
                l2 = bm.loo(bm.loo(l_cur).prev).prev;
            }
            l = bm.loo(l2).radial_next;
        }
        if self.faceloop_include_face(bm, l) {
            let l_e = bm.loo(l).e;
            let l_f = bm.loo(l).f;
            let stop = bm.face(l_f).len != 4 || self.faceloop_delimit_check(bm, f, l_e, l_f);
            self.stack.push(WalkState::FaceLoop {
                l: if stop { l_cur } else { l },
                no_calc: stop,
            });
            self.visit_set_alt.insert((BM_EDGE, l_e));
            self.visit_set.insert((BM_FACE, l_f));
        }
    }

    /* --- EdgeRing walker -------------------------------------------------- */

    fn edgering_delimit_check(&self, bm: &BMesh, e: usize) -> bool {
        if !bm_edge_is_manifold(bm, e) {
            return false;
        }
        if (self.delimit & BMW_DELIMIT_EDGE_MARK_SEAM) != 0
            && (bm.edge(e).head.hflag & BM_ELEM_SEAM) != 0
        {
            return true;
        }
        if (self.delimit & BMW_DELIMIT_EDGE_MARK_SHARP) != 0
            && (bm.edge(e).head.hflag & BM_ELEM_SMOOTH) == 0
        {
            return true;
        }
        if (self.delimit & BMW_DELIMIT_FACE_MARK_MATERIAL) != 0 {
            let l = bm.edge(e).l;
            let lr = bm.loo(l).radial_next;
            if bm.face(bm.loo(l).f).mat_nr != bm.face(bm.loo(lr).f).mat_nr {
                return true;
            }
        }
        false
    }

    fn edgering_begin(&mut self, bm: &mut BMesh, e: usize) {
        let l = bm.edge(e).l;
        if l == BM_NIL {
            self.stack.push(WalkState::Edgering {
                l: BM_NIL,
                wireedge: e,
                no_calc: false,
            });
            return;
        }
        if self.edgering_delimit_check(bm, e) {
            self.stack.push(WalkState::Edgering {
                l: BM_NIL,
                wireedge: e,
                no_calc: false,
            });
            return;
        }
        self.stack.push(WalkState::Edgering {
            l,
            wireedge: BM_NIL,
            no_calc: false,
        });
        self.visit_set.insert((BM_EDGE, bm.loo(l).e));

        /* Rewind. */
        let mut owalk: Option<WalkState> = None;
        let mut hit_no_calc = false;
        while let Some(s) = self.stack.last().copied() {
            owalk = Some(s);
            if let WalkState::Edgering { no_calc, .. } = s {
                hit_no_calc = no_calc;
            }
            self.step(bm);
        }
        if let Some(WalkState::Edgering { l, .. }) = owalk {
            let delimit_ngon = (self.delimit & BMW_DELIMIT_EDGE_RING_NGONS) != 0;
            let mut l_start = l;
            let f_len = bm.face(bm.loo(l_start).f).len;
            if if delimit_ngon { f_len != 4 } else { f_len % 2 != 0 } {
                l_start = bm.loo(l_start).radial_next;
            }
            self.stack.push(WalkState::Edgering {
                l: l_start,
                wireedge: BM_NIL,
                no_calc: false,
            });
            self.visit_set.clear();
            self.visit_set.insert((BM_EDGE, bm.loo(l_start).e));
            let l_rad = bm.loo(l_start).radial_next;
            if l_rad != l_start {
                let f_len2 = bm.face(bm.loo(l_start).f).len;
                if hit_no_calc
                    || (if delimit_ngon {
                        f_len2 != 4
                    } else {
                        f_len2 % 2 != 0
                    })
                {
                    self.stack.push(WalkState::Edgering {
                        l: l_rad,
                        wireedge: BM_NIL,
                        no_calc: false,
                    });
                }
            }
        }
    }

    fn edgering_step(&mut self, bm: &BMesh, l_cur: usize, no_calc: bool) {
        /* edge ring: cross the face to the loop two steps away (quad ring) */
        if no_calc {
            return;
        }
        let mut l = bm.loo(bm.loo(l_cur).next).next;
        let e = bm.loo(l).e;
        if self.edgering_delimit_check(bm, e) {
            /* mark terminating edge as visited then stop */
            if !self.visit_set.contains(&(BM_EDGE, e)) {
                self.stack.push(WalkState::Edgering {
                    l,
                    wireedge: BM_NIL,
                    no_calc: true,
                });
                self.visit_set.insert((BM_EDGE, e));
            }
            return;
        }
        if bm_edge_is_manifold(bm, e) {
            l = bm.loo(l).radial_next;
            let e2 = bm.loo(l).e;
            if !self.visit_set.contains(&(BM_EDGE, e2)) {
                let f_len = bm.face(bm.loo(l).f).len;
                let delimit_ngon = (self.delimit & BMW_DELIMIT_EDGE_RING_NGONS) != 0;
                let stop = if delimit_ngon {
                    f_len != 4
                } else {
                    f_len % 2 != 0
                };
                self.stack.push(WalkState::Edgering {
                    l,
                    wireedge: BM_NIL,
                    no_calc: stop,
                });
                self.visit_set.insert((BM_EDGE, e2));
            }
        }
    }
}

/// `BMW_walk` convenience — drain the whole walk collecting yields.
pub fn bmw_walk_all(bm: &mut BMesh, walker: &mut BMWalker) -> Vec<(u8, usize)> {
    let mut out = Vec::new();
    while let Some(elem) = walker.step(bm) {
        out.push(elem);
    }
    out
}

/// `bm_loop_is_manifold` — is this loop's edge 2-manifold?
fn bm_loop_is_manifold(bm: &BMesh, l: usize) -> bool {
    bm_edge_is_manifold(bm, bm.loo(l).e)
}

/// `bm_edge_is_single` — boundary edge on an ngon with a boundary neighbor.
fn bm_edge_is_single(bm: &BMesh, e: usize) -> bool {
    bm_edge_is_boundary(bm, e)
        && bm.face(bm.loo(bm.edge(e).l).f).len > 4
        && (bm_edge_is_boundary(bm, bm.loo(bm.loo(bm.edge(e).l).next).e)
            || bm_edge_is_boundary(bm, bm.loo(bm.loo(bm.edge(e).l).prev).e))
}

/// `bmw_NonManifoldLoop_find_next_around_vertex`.
fn bmw_nonmanifold_find_next(bm: &BMesh, l: usize, v: usize, face_count: i32) -> usize {
    debug_assert!(!bm_loop_is_manifold(bm, l));
    let mut l = l;
    loop {
        l = crate::query::bm_loop_other_edge_loop(bm, l, v);
        if bm_loop_is_manifold(bm, l) {
            l = bm.loo(l).radial_next;
        } else if crate::query::bm_edge_face_count(bm, bm.loo(l).e) == face_count {
            return l;
        } else {
            break;
        }
    }
    BM_NIL
}

impl BMWalker {
    /// `bmw_UVEdgeWalker_step` — walk along UV-equal radial loops
    /// (`CustomData_data_equals` on the loop UV layer → `uv_at` accessor;
    /// PORT_PENDING when `uv_at` is unset: compares verts instead).
    fn uvedge_step_impl(&mut self, bm: &mut BMesh, l: usize) {
        if !self.mask_check(bm, BM_EDGE, bm.loo(l).e) {
            return;
        }
        for i in 0..2 {
            let l_pivot = if i == 1 { bm.loo(l).next } else { l };
            for l_radial_first in
                crate::iterators::LoopsOfVert::new(bm, bm.loo(l_pivot).v).collect::<Vec<_>>()
            {
                let mut l_radial = l_radial_first;
                loop {
                    if !self.visit_set.contains(&(BM_LOOP, l_radial)) {
                        if bm.loo(l_radial).v != bm.loo(l_pivot).v
                            && !self.mask_check(bm, BM_EDGE, bm.loo(l_radial).e)
                        {
                            /* skip */
                        } else {
                            let l_other = if bm.loo(l_radial).v != bm.loo(l_pivot).v {
                                bm.loo(l_radial).next
                            } else {
                                l_radial
                            };
                            let equal = if let Some(uv_at) = &self.uv_at {
                                uv_at(bm, l_pivot) == uv_at(bm, l_other)
                            } else {
                                /* PORT_PENDING CustomData equality. */
                                true
                            };
                            if equal {
                                self.stack.push(WalkState::Loop(l_radial));
                                self.visit_set.insert((BM_LOOP, l_radial));
                            }
                        }
                    }
                    l_radial = bm.loo(l_radial).radial_next;
                    if l_radial == l_radial_first {
                        break;
                    }
                }
                if self.mask_edge != 0 {
                    let l_boundary = bm.loo(l_radial_first).prev;
                    if crate::delete::bm_edge_is_boundary(bm, bm.loo(l_boundary).e)
                        && self.mask_check(bm, BM_EDGE, bm.loo(l_boundary).e)
                        && !self.visit_set.contains(&(BM_LOOP, l_boundary))
                    {
                        self.visit_set.insert((BM_LOOP, l_boundary));
                        self.stack.push(WalkState::Loop(l_boundary));
                    }
                }
            }
        }
    }
}
