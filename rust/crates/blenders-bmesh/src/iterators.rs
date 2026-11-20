//! Port of `bmesh_iterators.cc` — element iterators.
//!
//! The C `BMIter` state machine is replaced by Rust iterator structs (each
//! holding the walk state) plus collect-into-`Vec` helpers matching
//! `BM_iter_as_array`. `BM_ITER_*` types: `BM_VERTS_OF_MESH`,
//! `BM_EDGES_OF_MESH`, `BM_FACES_OF_MESH`, `BM_LOOPS_OF_FACE`,
//! `BM_EDGES_OF_VERT`, `BM_FACES_OF_VERT`, `BM_LOOPS_OF_VERT`,
//! `BM_LOOPS_OF_EDGE`, `BM_LOOPS_OF_LOOP`, `BM_FACES_OF_EDGE`,
//! `BM_VERTS_OF_EDGE`, `BM_VERTS_OF_FACE`, `BM_EDGES_OF_FACE`.

use crate::structure::*;
use crate::{BMesh, BM_EDGE, BM_FACE, BM_LOOP, BM_NIL, BM_VERT};

/// `BMIter` iterator types (`BM_ITYPE_*` from bmesh_iterators.hh).
pub const BM_VERTS_OF_MESH: u8 = 1;
pub const BM_EDGES_OF_MESH: u8 = 2;
pub const BM_FACES_OF_MESH: u8 = 3;
pub const BM_LOOPS_OF_FACE: u8 = 4;
pub const BM_EDGES_OF_VERT: u8 = 5;
pub const BM_FACES_OF_VERT: u8 = 6;
pub const BM_LOOPS_OF_VERT: u8 = 7;
pub const BM_LOOPS_OF_EDGE: u8 = 8;
pub const BM_LOOPS_OF_LOOP: u8 = 9;
pub const BM_FACES_OF_EDGE: u8 = 10;
pub const BM_VERTS_OF_EDGE: u8 = 11;
pub const BM_VERTS_OF_FACE: u8 = 12;
pub const BM_EDGES_OF_FACE: u8 = 13;

/// `BM_iter_mesh_count`.
pub fn bm_iter_mesh_count(itype: u8, bm: &BMesh) -> i32 {
    match itype {
        BM_VERTS_OF_MESH => bm.totvert,
        BM_EDGES_OF_MESH => bm.totedge,
        BM_FACES_OF_MESH => bm.totface,
        _ => 0,
    }
}

/// Iterate live verts (skipping killed slots: `htype==0`).
pub struct VertsOfMesh<'a> {
    bm: &'a BMesh,
    i: usize,
}
impl<'a> VertsOfMesh<'a> {
    pub fn new(bm: &'a BMesh) -> Self {
        VertsOfMesh { bm, i: 0 }
    }
}
impl<'a> Iterator for VertsOfMesh<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.i < self.bm.vpool.len() {
            let i = self.i;
            self.i += 1;
            if self.bm.vpool[i].head.htype == BM_VERT {
                return Some(i);
            }
        }
        None
    }
}

/// Iterate live edges.
pub struct EdgesOfMesh<'a> {
    bm: &'a BMesh,
    i: usize,
}
impl<'a> EdgesOfMesh<'a> {
    pub fn new(bm: &'a BMesh) -> Self {
        EdgesOfMesh { bm, i: 0 }
    }
}
impl<'a> Iterator for EdgesOfMesh<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.i < self.bm.epool.len() {
            let i = self.i;
            self.i += 1;
            if self.bm.epool[i].head.htype == BM_EDGE {
                return Some(i);
            }
        }
        None
    }
}

/// Iterate live faces.
pub struct FacesOfMesh<'a> {
    bm: &'a BMesh,
    i: usize,
}
impl<'a> FacesOfMesh<'a> {
    pub fn new(bm: &'a BMesh) -> Self {
        FacesOfMesh { bm, i: 0 }
    }
}
impl<'a> Iterator for FacesOfMesh<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.i < self.bm.fpool.len() {
            let i = self.i;
            self.i += 1;
            if self.bm.fpool[i].head.htype == BM_FACE {
                return Some(i);
            }
        }
        None
    }
}

/// `bmiter__loop_of_face` — loops of face f (cycle order).
pub struct LoopsOfFace<'a> {
    bm: &'a BMesh,
    first: usize,
    cur: usize,
    done: bool,
}
impl<'a> LoopsOfFace<'a> {
    pub fn new(bm: &'a BMesh, f: usize) -> Self {
        let first = bm.face(f).l_first;
        LoopsOfFace {
            bm,
            first,
            cur: first,
            done: first == BM_NIL,
        }
    }
}
impl<'a> Iterator for LoopsOfFace<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.done {
            return None;
        }
        let r = self.cur;
        self.cur = self.bm.loo(self.cur).next;
        if self.cur == self.first {
            self.done = true;
        }
        Some(r)
    }
}

/// `bmiter__edge_of_vert` — disk cycle edges around v.
pub struct EdgesOfVert<'a> {
    bm: &'a BMesh,
    v: usize,
    first: usize,
    cur: usize,
    done: bool,
}
impl<'a> EdgesOfVert<'a> {
    pub fn new(bm: &'a BMesh, v: usize) -> Self {
        let first = bm.vert(v).e;
        EdgesOfVert {
            bm,
            v,
            first,
            cur: first,
            done: first == BM_NIL,
        }
    }
}
impl<'a> Iterator for EdgesOfVert<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.done {
            return None;
        }
        let r = self.cur;
        self.cur = bmesh_disk_edge_next(self.bm, self.cur, self.v);
        if self.cur == self.first {
            self.done = true;
        }
        Some(r)
    }
}

/// `bmiter__face_of_vert` — faces incident on vert via radial loops.
pub struct FacesOfVert<'a> {
    bm: &'a BMesh,
    e_iter: usize,
    e_first: usize,
    l_iter: usize,
    l_first: usize,
    v: usize,
    done: bool,
    first_face: usize,
}
impl<'a> FacesOfVert<'a> {
    pub fn new(bm: &'a BMesh, v: usize) -> Self {
        let e = bm.vert(v).e;
        let (e_iter, l_iter, l_first, first_face) = if e != BM_NIL {
            let l = bmesh_disk_facevert_find_first(bm, e, v);
            if l != BM_NIL {
                (e, l, l, bm.loo(l).f)
            } else {
                (BM_NIL, BM_NIL, BM_NIL, BM_NIL)
            }
        } else {
            (BM_NIL, BM_NIL, BM_NIL, BM_NIL)
        };
        FacesOfVert {
            bm,
            e_iter,
            e_first: e_iter,
            l_iter,
            l_first,
            v,
            done: e_iter == BM_NIL,
            first_face,
        }
    }
}
impl<'a> Iterator for FacesOfVert<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.done {
            return None;
        }
        let r = self.bm.loo(self.l_iter).f;
        /* advance radial loop, then disk edges */
        let mut l_next = bmesh_radial_faceloop_find_next(self.bm, self.l_iter, self.v);
        if l_next == self.l_iter || self.bm.loo(l_next).f == self.first_face {
            /* move to next edge in disk */
            let e_next = bmesh_disk_edge_next(self.bm, self.e_iter, self.v);
            if e_next == self.e_first {
                self.done = true;
            } else {
                self.e_iter = e_next;
                l_next = bmesh_disk_facevert_find_first(self.bm, e_next, self.v);
                if l_next == BM_NIL {
                    /* skip edges without face loops */
                    loop {
                        if self.e_iter == self.e_first {
                            self.done = true;
                            break;
                        }
                        self.e_iter = bmesh_disk_edge_next(self.bm, self.e_iter, self.v);
                        let lf = bmesh_disk_facevert_find_first(self.bm, self.e_iter, self.v);
                        if lf != BM_NIL {
                            self.l_iter = lf;
                            self.l_first = lf;
                            self.first_face = self.bm.loo(lf).f;
                            break;
                        }
                    }
                } else {
                    self.l_iter = l_next;
                    self.l_first = l_next;
                    self.first_face = self.bm.loo(l_next).f;
                }
            }
        } else {
            self.l_iter = l_next;
        }
        Some(r)
    }
}

/// `bmiter__loop_of_vert` — loops of vert v.
pub struct LoopsOfVert<'a> {
    bm: &'a BMesh,
    e_iter: usize,
    e_first: usize,
    l_iter: usize,
    l_first: usize,
    v: usize,
    done: bool,
}
impl<'a> LoopsOfVert<'a> {
    pub fn new(bm: &'a BMesh, v: usize) -> Self {
        let e = bm.vert(v).e;
        let (e_iter, l_iter, l_first) = if e != BM_NIL {
            let l = bmesh_disk_facevert_find_first(bm, e, v);
            (e, l, l)
        } else {
            (BM_NIL, BM_NIL, BM_NIL)
        };
        LoopsOfVert {
            bm,
            e_iter,
            e_first: e_iter,
            l_iter,
            l_first,
            v,
            done: l_iter == BM_NIL,
        }
    }
}
impl<'a> Iterator for LoopsOfVert<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.done {
            return None;
        }
        let r = self.l_iter;
        let l_next = bmesh_radial_faceloop_find_next(self.bm, self.l_iter, self.v);
        if l_next != self.l_iter {
            self.l_iter = l_next;
        } else {
            /* advance disk edge */
            loop {
                self.e_iter = bmesh_disk_edge_next(self.bm, self.e_iter, self.v);
                if self.e_iter == self.e_first {
                    self.done = true;
                    break;
                }
                let lf = bmesh_disk_facevert_find_first(self.bm, self.e_iter, self.v);
                if lf != BM_NIL {
                    self.l_iter = lf;
                    self.l_first = lf;
                    break;
                }
            }
        }
        Some(r)
    }
}

/// `bmiter__loop_of_edge` — radial loops of edge e.
pub struct LoopsOfEdge<'a> {
    bm: &'a BMesh,
    first: usize,
    cur: usize,
    done: bool,
}
impl<'a> LoopsOfEdge<'a> {
    pub fn new(bm: &'a BMesh, e: usize) -> Self {
        let first = bm.edge(e).l;
        LoopsOfEdge {
            bm,
            first,
            cur: first,
            done: first == BM_NIL,
        }
    }
}
impl<'a> Iterator for LoopsOfEdge<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.done {
            return None;
        }
        let r = self.cur;
        self.cur = self.bm.loo(self.cur).radial_next;
        if self.cur == self.first {
            self.done = true;
        }
        Some(r)
    }
}

/// `bmiter__loop_of_loop` — loops sharing l's edge in l's face neighborhood.
pub struct LoopsOfLoop<'a> {
    bm: &'a BMesh,
    first: usize,
    cur: usize,
    done: bool,
}
impl<'a> LoopsOfLoop<'a> {
    pub fn new(bm: &'a BMesh, l: usize) -> Self {
        let e = bm.loo(l).e;
        let first = if e != BM_NIL { bm.edge(e).l } else { BM_NIL };
        LoopsOfLoop {
            bm,
            first,
            cur: first,
            done: first == BM_NIL,
        }
    }
}
impl<'a> Iterator for LoopsOfLoop<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.done {
            return None;
        }
        let r = self.cur;
        self.cur = self.bm.loo(self.cur).radial_next;
        if self.cur == self.first {
            self.done = true;
        }
        Some(r)
    }
}

/// `bmiter__face_of_edge` — faces of edge e via radial.
pub struct FacesOfEdge<'a> {
    bm: &'a BMesh,
    first: usize,
    cur: usize,
    done: bool,
}
impl<'a> FacesOfEdge<'a> {
    pub fn new(bm: &'a BMesh, e: usize) -> Self {
        let first = bm.edge(e).l;
        FacesOfEdge {
            bm,
            first,
            cur: first,
            done: first == BM_NIL,
        }
    }
}
impl<'a> Iterator for FacesOfEdge<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.done {
            return None;
        }
        let r = self.bm.loo(self.cur).f;
        self.cur = self.bm.loo(self.cur).radial_next;
        if self.cur == self.first {
            self.done = true;
        }
        Some(r)
    }
}

/// `bmiter__vert_of_edge` — v1 then v2.
pub struct VertsOfEdge {
    i: usize,
}
impl VertsOfEdge {
    pub fn new() -> Self {
        VertsOfEdge { i: 0 }
    }
    pub fn next(&mut self, bm: &BMesh, e: usize) -> Option<usize> {
        let r = match self.i {
            0 => bm.edge(e).v1,
            1 => bm.edge(e).v2,
            _ => return None,
        };
        self.i += 1;
        Some(r)
    }
}

/// `bmiter__vert_of_face` — verts of face (loop cycle).
pub struct VertsOfFace<'a> {
    inner: LoopsOfFace<'a>,
}
impl<'a> VertsOfFace<'a> {
    pub fn new(bm: &'a BMesh, f: usize) -> Self {
        VertsOfFace {
            inner: LoopsOfFace::new(bm, f),
        }
    }
}
impl<'a> Iterator for VertsOfFace<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        self.inner.next().map(|l| self.inner.bm.loo(l).v)
    }
}

/// `bmiter__edge_of_face` — edges of face (loop cycle).
pub struct EdgesOfFace<'a> {
    inner: LoopsOfFace<'a>,
}
impl<'a> EdgesOfFace<'a> {
    pub fn new(bm: &'a BMesh, f: usize) -> Self {
        EdgesOfFace {
            inner: LoopsOfFace::new(bm, f),
        }
    }
}
impl<'a> Iterator for EdgesOfFace<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        self.inner.next().map(|l| self.inner.bm.loo(l).e)
    }
}

/// `BM_iter_elem_count_flag` — count elements with hflag set/unset.
pub fn bm_iter_elem_count_flag(itype: u8, bm: &BMesh, hflag: u8, value: bool) -> i32 {
    let mut count = 0;
    match itype {
        BM_VERTS_OF_MESH => {
            for v in &bm.vpool {
                if v.head.htype == BM_VERT && ((v.head.hflag & hflag) != 0) == value {
                    count += 1;
                }
            }
        }
        BM_EDGES_OF_MESH => {
            for e in &bm.epool {
                if e.head.htype == BM_EDGE && ((e.head.hflag & hflag) != 0) == value {
                    count += 1;
                }
            }
        }
        BM_FACES_OF_MESH => {
            for f in &bm.fpool {
                if f.head.htype == BM_FACE && ((f.head.hflag & hflag) != 0) == value {
                    count += 1;
                }
            }
        }
        _ => debug_assert!(false),
    }
    count
}

/// `BM_iter_mesh_count_flag`.
pub fn bm_iter_mesh_count_flag(itype: u8, bm: &BMesh, hflag: u8, value: bool) -> i32 {
    bm_iter_elem_count_flag(itype, bm, hflag, value)
}

/// `BM_iter_as_array` — collect iterator results into a Vec.
pub fn bm_iter_as_array<T: Iterator<Item = usize>>(iter: T) -> Vec<usize> {
    iter.collect()
}

