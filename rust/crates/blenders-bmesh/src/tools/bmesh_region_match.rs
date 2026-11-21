//! Port of `bmesh_region_match.cc` — find matching topologies of a
//! face-region elsewhere in the mesh (`BM_mesh_region_match`).
//! `USE_WALKER_REUSE`, `USE_PIVOT_SEARCH`, `USE_PIVOT_FASTMATCH` are all
//! enabled (as in C); GHash/mempool/LinkNode → HashMap/HashSet/Vec.

use crate::iterators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_ELEM_TAG, BM_FACE, BM_VERT};
use std::collections::{HashMap, HashSet};

type UidInt = usize;
type SuidInt = isize;

const PRIME_VERT_INIT: UidInt = 100003;

/// `UIDFaceStepItem`.
#[derive(Default)]
struct UidFaceStepItem {
    uid: UidInt,
    list: Vec<usize>, /* faces */
}

/// `UIDFaceStep`.
#[derive(Default)]
struct UidFaceStep {
    faces: Vec<usize>,
    items: Vec<UidFaceStepItem>,
}

/// `UIDWalk`.
struct UidWalk {
    faces_step: std::collections::VecDeque<UidFaceStep>,
    verts_uid: HashMap<usize, UidInt>, /* BMVert idx -> uid */
    faces_uid: HashMap<usize, UidInt>, /* BMFace idx -> uid */
    use_face_isolate: bool,
    pass: UidInt,
    /* cache */
    verts_uid_pass: HashMap<usize, UidInt>,
    faces_step_next: HashSet<usize>,
    faces_from_uid: HashMap<UidInt, usize>, /* uid -> index into fstep.items */
}

impl UidWalk {
    /// `bm_uidwalk_init` / `bm_uidwalk_clear`.
    fn new() -> Self {
        UidWalk {
            faces_step: Default::default(),
            verts_uid: HashMap::new(),
            faces_uid: HashMap::new(),
            use_face_isolate: false,
            pass: 1,
            verts_uid_pass: HashMap::new(),
            faces_step_next: HashSet::new(),
            faces_from_uid: HashMap::new(),
        }
    }
    fn clear(&mut self) {
        self.faces_step.clear();
        self.verts_uid.clear();
        self.faces_uid.clear();
        self.verts_uid_pass.clear();
        self.faces_step_next.clear();
        self.faces_from_uid.clear();
        self.use_face_isolate = false;
        self.pass = 1;
    }
    /// `bm_uidwalk_face_test`.
    #[inline]
    fn face_test(&self, bm: &BMesh, f: usize) -> bool {
        if self.use_face_isolate {
            bm.face(f).head.hflag & BM_ELEM_TAG != 0
        } else {
            true
        }
    }
}

/// `bm_uidwalk_calc_vert_uid`.
fn bm_uidwalk_calc_vert_uid(bm: &BMesh, uidwalk: &UidWalk, v: usize) -> UidInt {
    const PRIME_VERT_SMALL: UidInt = 7;
    const PRIME_VERT_MID: UidInt = 43;
    const PRIME_VERT_LARGE: UidInt = 1031;
    const PRIME_FACE_SMALL: UidInt = 13;
    const PRIME_FACE_MID: UidInt = 53;
    let mut uid = uidwalk.pass.wrapping_mul(PRIME_VERT_LARGE);
    {
        let mut tot = 0usize;
        for e in EdgesOfVert::new(bm, v) {
            let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
            if let Some(&uid_other) = uidwalk.verts_uid.get(&v_other) {
                uid ^= uid_other.wrapping_mul(PRIME_VERT_SMALL);
                tot += 1;
            }
        }
        uid ^= tot.wrapping_mul(PRIME_VERT_MID);
    }
    {
        let mut tot = 0usize;
        for f in FacesOfVert::new(bm, v) {
            if let Some(&uid_other) = uidwalk.faces_uid.get(&f) {
                uid ^= uid_other.wrapping_mul(PRIME_FACE_SMALL);
                tot += 1;
            }
        }
        uid ^= tot.wrapping_mul(PRIME_FACE_MID);
    }
    uid
}

/// `bm_uidwalk_calc_face_uid`.
fn bm_uidwalk_calc_face_uid(bm: &BMesh, uidwalk: &UidWalk, f: usize) -> UidInt {
    const PRIME_VERT_SMALL: UidInt = 11;
    const PRIME_FACE_SMALL: UidInt = 17;
    const PRIME_FACE_LARGE: UidInt = 1013;
    let mut uid = uidwalk
        .pass
        .wrapping_mul(bm.face(f).len as usize)
        .wrapping_mul(PRIME_FACE_LARGE);
    {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if let Some(&uid_other) = uidwalk.verts_uid.get(&bm.loo(l_iter).v) {
                uid ^= uid_other.wrapping_mul(PRIME_VERT_SMALL);
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if bm.loo(l_iter).radial_next != l_iter {
                let mut l_iter_radial = bm.loo(l_iter).radial_next;
                loop {
                    if let Some(&uid_other) =
                        uidwalk.faces_uid.get(&bm.loo(l_iter_radial).f)
                    {
                        uid ^= uid_other.wrapping_mul(PRIME_FACE_SMALL);
                    }
                    l_iter_radial = bm.loo(l_iter_radial).radial_next;
                    if l_iter_radial == l_iter {
                        break;
                    }
                }
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    uid
}

/// `bm_uidwalk_rehash`.
fn bm_uidwalk_rehash(bm: &BMesh, uidwalk: &mut UidWalk) {
    let keys: Vec<usize> = uidwalk.verts_uid.keys().copied().collect();
    let mut uid_store = Vec::with_capacity(keys.len());
    for &v in &keys {
        uid_store.push(bm_uidwalk_calc_vert_uid(bm, uidwalk, v));
    }
    for (i, &v) in keys.iter().enumerate() {
        *uidwalk.verts_uid.get_mut(&v).unwrap() = uid_store[i];
    }
    let keys: Vec<usize> = uidwalk.faces_uid.keys().copied().collect();
    let mut uid_store = Vec::with_capacity(keys.len());
    for &f in &keys {
        uid_store.push(bm_uidwalk_calc_face_uid(bm, uidwalk, f));
    }
    for (i, &f) in keys.iter().enumerate() {
        *uidwalk.faces_uid.get_mut(&f).unwrap() = uid_store[i];
    }
}

/// `bm_uidwalk_rehash_facelinks`.
fn bm_uidwalk_rehash_facelinks(
    bm: &BMesh,
    uidwalk: &mut UidWalk,
    faces: &[usize],
    is_init: bool,
) {
    let mut uid_store = Vec::with_capacity(faces.len());
    for &f in faces {
        uid_store.push(bm_uidwalk_calc_face_uid(bm, uidwalk, f));
    }
    for (i, &f) in faces.iter().enumerate() {
        if is_init {
            uidwalk.faces_uid.insert(f, uid_store[i]);
        } else if let Some(p) = uidwalk.faces_uid.get_mut(&f) {
            *p = uid_store[i];
        }
    }
}

/// `bm_vert_is_uid_connect`.
fn bm_vert_is_uid_connect(bm: &BMesh, uidwalk: &UidWalk, v: usize) -> bool {
    for e in EdgesOfVert::new(bm, v) {
        let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
        if uidwalk.verts_uid.contains_key(&v_other) {
            return true;
        }
    }
    false
}

/// `bm_uidwalk_pass_add`.
fn bm_uidwalk_pass_add(
    bm: &BMesh,
    uidwalk: &mut UidWalk,
    faces_pass: Vec<usize>,
) {
    bm_uidwalk_rehash_facelinks(bm, uidwalk, &faces_pass, true);
    debug_assert!(uidwalk.verts_uid_pass.is_empty());
    debug_assert!(uidwalk.faces_step_next.is_empty());
    let mut fstep = UidFaceStep::default();
    for &f in &faces_pass {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let v = bm.loo(l_iter).v;
            if !uidwalk.verts_uid.contains_key(&v)
                && !uidwalk.verts_uid_pass.contains_key(&v)
                && bm_vert_is_uid_connect(bm, uidwalk, v)
            {
                let uid = bm_uidwalk_calc_vert_uid(bm, uidwalk, v);
                uidwalk.verts_uid_pass.insert(v, uid);
            }
            if bm.loo(l_iter).radial_next != l_iter {
                let mut l_iter_radial = bm.loo(l_iter).radial_next;
                loop {
                    let fr = bm.loo(l_iter_radial).f;
                    if !uidwalk.faces_uid.contains_key(&fr)
                        && !uidwalk.faces_step_next.contains(&fr)
                        && uidwalk.face_test(bm, fr)
                    {
                        uidwalk.faces_step_next.insert(fr);
                        fstep.faces.push(fr);
                    }
                    l_iter_radial = bm.loo(l_iter_radial).radial_next;
                    if l_iter_radial == l_iter {
                        break;
                    }
                }
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    uidwalk.faces_step.push_front(fstep);
    for (v, uid) in uidwalk.verts_uid_pass.drain() {
        uidwalk.verts_uid.insert(v, uid);
    }
    bm_uidwalk_rehash_facelinks(bm, uidwalk, &faces_pass, false);
    uidwalk.pass += 1;
    uidwalk.faces_step_next.clear();
}

/// `bm_uidwalk_init_from_edge`.
fn bm_uidwalk_init_from_edge(bm: &mut BMesh, uidwalk: &mut UidWalk, e: usize) -> u32 {
    let mut l_iter = bm.edge(e).l;
    let e_l_first = l_iter;
    let mut f_arr: Vec<usize> = Vec::new();
    loop {
        let f = bm.loo(l_iter).f;
        if uidwalk.face_test(bm, f) {
            f_arr.push(f);
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == e_l_first {
            break;
        }
    }
    f_arr.sort_by_key(|&f| bm.face(f).len);
    uidwalk.verts_uid.insert(bm.edge(e).v1, PRIME_VERT_INIT);
    uidwalk.verts_uid.insert(bm.edge(e).v2, PRIME_VERT_INIT);
    let mut fstep_num = 0u32;
    let mut i = 0usize;
    while i < f_arr.len() {
        let mut faces_pass = Vec::new();
        let f_len = bm.face(f_arr[i]).len;
        loop {
            faces_pass.push(f_arr[i]);
            i += 1;
            if i >= f_arr.len() || f_len != bm.face(f_arr[i]).len {
                break;
            }
        }
        bm_uidwalk_pass_add(bm, uidwalk, faces_pass);
        fstep_num += 1;
    }
    fstep_num
}

/// `bm_uidwalk_facestep_begin` — split `fstep.faces` by uid into sorted
/// `fstep.items`; returns true if any unvisited face was found.
fn bm_uidwalk_facestep_begin(
    bm: &BMesh,
    uidwalk_faces_uid: &HashMap<usize, UidInt>,
    uidwalk_pass: UidInt,
    uidwalk_verts_uid: &HashMap<usize, UidInt>,
    faces_from_uid: &mut HashMap<UidInt, usize>,
    fstep: &mut UidFaceStep,
) -> bool {
    debug_assert!(faces_from_uid.is_empty());
    debug_assert!(fstep.items.is_empty());
    let uidwalk_view = UidWalk {
        verts_uid: uidwalk_verts_uid.clone(),
        faces_uid: uidwalk_faces_uid.clone(),
        pass: uidwalk_pass,
        ..UidWalk::new()
    };
    let mut ok = false;
    let mut faces_new = Vec::new();
    for &f in &fstep.faces {
        if !uidwalk_faces_uid.contains_key(&f) {
            let uid = bm_uidwalk_calc_face_uid(bm, &uidwalk_view, f);
            ok = true;
            let item_idx = match faces_from_uid.get(&uid) {
                Some(&i) => i,
                None => {
                    fstep.items.push(UidFaceStepItem {
                        uid,
                        list: Vec::new(),
                    });
                    let i = fstep.items.len() - 1;
                    faces_from_uid.insert(uid, i);
                    i
                }
            };
            fstep.items[item_idx].list.push(f);
            faces_new.push(f);
        }
    }
    fstep.faces = faces_new;
    faces_from_uid.clear();
    fstep.items.sort_by_key(|it| it.uid);
    ok
}

/// `bm_uidwalk_facestep_end`.
fn bm_uidwalk_facestep_end(fstep: &mut UidFaceStep) {
    fstep.items.clear();
}

/// `bm_mesh_region_match_pair` — walk from `e_src`/`e_dst`, comparing
/// face-step uid chains. Returns the matching face region (if any).
fn bm_mesh_region_match_pair(
    w_src: &mut UidWalk,
    w_dst: &mut UidWalk,
    bm: &mut BMesh,
    e_src: usize,
    e_dst: usize,
    faces_src_region_len: usize,
) -> Option<Vec<usize>> {
    debug_assert!(e_src != e_dst);
    let mut found = false;
    w_src.use_face_isolate = true;
    if bm_uidwalk_init_from_edge(bm, w_src, e_src)
        != bm_uidwalk_init_from_edge(bm, w_dst, e_dst)
    {
        w_src.clear();
        w_dst.clear();
        return None;
    }
    'outer: loop {
        let mut ok = false;
        debug_assert!(w_src.faces_step.len() == w_dst.faces_step.len());
        let mut i = 0usize;
        while i < w_src.faces_step.len() && i < w_dst.faces_step.len() {
            if w_src.faces_step[i].faces.is_empty() {
                w_src.faces_step.remove(i);
                w_dst.faces_step.remove(i);
                continue;
            }
            let (beg_src, beg_dst) = {
                let a = bm_uidwalk_facestep_begin(
                    bm,
                    &w_src.faces_uid,
                    w_src.pass,
                    &w_src.verts_uid,
                    &mut w_src.faces_from_uid,
                    &mut w_src.faces_step[i],
                );
                let b = bm_uidwalk_facestep_begin(
                    bm,
                    &w_dst.faces_uid,
                    w_dst.pass,
                    &w_dst.verts_uid,
                    &mut w_dst.faces_from_uid,
                    &mut w_dst.faces_step[i],
                );
                (a, b)
            };
            if beg_src && beg_dst {
                let mut si = 0usize;
                let mut di = 0usize;
                while si < w_src.faces_step[i].items.len()
                    && di < w_dst.faces_step[i].items.len()
                {
                    while di < w_dst.faces_step[i].items.len()
                        && w_dst.faces_step[i].items[di].uid
                            < w_src.faces_step[i].items[si].uid
                    {
                        di += 1;
                    }
                    if di >= w_dst.faces_step[i].items.len()
                        || w_src.faces_step[i].items[si].uid
                            != w_dst.faces_step[i].items[di].uid
                        || w_src.faces_step[i].items[si].list.len()
                            > w_dst.faces_step[i].items[di].list.len()
                    {
                        ok = false;
                        break;
                    }
                    if w_src.faces_step[i].items[si].list.len()
                        == w_dst.faces_step[i].items[di].list.len()
                    {
                        let list_src =
                            std::mem::take(&mut w_src.faces_step[i].items[si].list);
                        let list_dst =
                            std::mem::take(&mut w_dst.faces_step[i].items[di].list);
                        bm_uidwalk_pass_add(bm, w_src, list_src);
                        bm_uidwalk_pass_add(bm, w_dst, list_dst);
                        ok = true;
                    }
                    si += 1;
                    di += 1;
                }
            }
            bm_uidwalk_facestep_end(&mut w_src.faces_step[i]);
            bm_uidwalk_facestep_end(&mut w_dst.faces_step[i]);
            i += 1;
        }
        if !ok {
            break 'outer;
        }
        found = w_dst.faces_uid.len() == faces_src_region_len;
        if found {
            break 'outer;
        }
        bm_uidwalk_rehash(bm, w_src);
        bm_uidwalk_rehash(bm, w_dst);
    }
    let result = if found {
        Some(w_dst.faces_uid.keys().copied().collect())
    } else {
        None
    };
    w_src.clear();
    w_dst.clear();
    result
}

/// `bm_face_array_visit` — tag as visited, avoid re-use.
fn bm_face_array_visit(
    bm: &mut BMesh,
    faces: &[usize],
    visit_faces: bool,
) -> usize {
    let mut verts_len = 0usize;
    for &f in faces {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if bm.vert(bm.loo(l_iter).v).head.hflag & BM_ELEM_TAG == 0 {
                verts_len += 1;
            }
            let e = bm.loo(l_iter).e;
            bm.epool[e].head.hflag |= BM_ELEM_TAG;
            let v = bm.loo(l_iter).v;
            bm.vpool[v].head.hflag |= BM_ELEM_TAG;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        if visit_faces {
            bm.fpool[f].head.hflag |= BM_ELEM_TAG;
        }
    }
    verts_len
}

/* USE_PIVOT_SEARCH */

#[inline]
fn abs_intptr(a: isize) -> isize {
    if a < 0 {
        -a
    } else {
        a
    }
}

/// `bm_edge_is_region_boundary` — any face in the radial not tagged?
fn bm_edge_is_region_boundary(bm: &BMesh, e: usize) -> bool {
    let l = bm.edge(e).l;
    if bm.loo(l).radial_next != l {
        let mut l_iter = l;
        loop {
            if bm.face(bm.loo(l_iter).f).head.hflag & BM_ELEM_TAG == 0 {
                return true;
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l {
                break;
            }
        }
        false
    } else {
        true
    }
}

/// `bm_face_region_pivot_edge_use_best`.
fn bm_face_region_pivot_edge_use_best(
    bm: &BMesh,
    gh: &HashMap<usize, SuidInt>,
    e_test: usize,
    r_e_pivot_best: &mut usize,
    e_pivot_best_id: &mut [SuidInt; 2],
) {
    let mut e_pivot_test_id = [
        *gh.get(&bm.edge(e_test).v1).unwrap_or(&0),
        *gh.get(&bm.edge(e_test).v2).unwrap_or(&0),
    ];
    if e_pivot_test_id[0] > e_pivot_test_id[1] {
        e_pivot_test_id.swap(0, 1);
    }
    if *r_e_pivot_best == BM_NIL
        || (if e_pivot_best_id[0] != e_pivot_test_id[0] {
            e_pivot_best_id[0] < e_pivot_test_id[0]
        } else {
            e_pivot_best_id[1] < e_pivot_test_id[1]
        })
    {
        *r_e_pivot_best = e_test;
        e_pivot_best_id[0] = e_pivot_test_id[0];
        e_pivot_best_id[1] = e_pivot_test_id[1];
    }
}

/// `bm_face_region_vert_boundary_id`.
fn bm_face_region_vert_boundary_id(bm: &BMesh, v: usize) -> SuidInt {
    const PRIME_VERT_SMALL_A: SuidInt = 7;
    const PRIME_VERT_SMALL_B: SuidInt = 13;
    const PRIME_VERT_MID_A: SuidInt = 103;
    const PRIME_VERT_MID_B: SuidInt = 131;
    let mut tot = 0isize;
    let mut id: SuidInt = PRIME_VERT_MID_A;
    for l in LoopsOfVert::new(bm, v) {
        let is_boundary_vert = bm_edge_is_region_boundary(bm, bm.loo(l).e)
            || bm_edge_is_region_boundary(bm, bm.loo(bm.loo(l).prev).e);
        id ^= (bm.face(bm.loo(l).f).len as isize)
            * if is_boundary_vert {
                PRIME_VERT_SMALL_A
            } else {
                PRIME_VERT_SMALL_B
            };
        tot += 1;
    }
    id ^= tot * PRIME_VERT_MID_B;
    if id != 0 {
        abs_intptr(id)
    } else {
        1
    }
}

/// `bm_face_region_vert_pass_id`.
fn bm_face_region_vert_pass_id(
    bm: &BMesh,
    gh: &HashMap<usize, SuidInt>,
    v: usize,
) -> SuidInt {
    const PRIME_VERT_MID_A: SuidInt = 23;
    const PRIME_VERT_MID_B: SuidInt = 31;
    let mut tot: SuidInt = 0;
    let mut v_sum_face_len: SuidInt = 0;
    let mut v_sum_id: SuidInt = 0;
    let id_min = isize::MIN + 1;
    for e in EdgesOfVert::new(bm, v) {
        if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
            let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
            if bm.vert(v_other).head.hflag & BM_ELEM_TAG != 0 {
                let v_other_id = *gh.get(&v_other).unwrap_or(&0);
                if v_other_id > 0 {
                    v_sum_id += v_other_id;
                    tot += 1;
                    let l_first = bm.edge(e).l;
                    let mut l_iter = l_first;
                    loop {
                        if bm.face(bm.loo(l_iter).f).head.hflag & BM_ELEM_TAG != 0 {
                            v_sum_face_len += bm.face(bm.loo(l_iter).f).len as isize;
                        }
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l_first {
                            break;
                        }
                    }
                }
            }
        }
    }
    let mut id = tot * PRIME_VERT_MID_A;
    id ^= v_sum_face_len * PRIME_VERT_MID_B;
    id ^= v_sum_id;
    id = if id == 0 {
        1
    } else if id < id_min {
        id_min
    } else {
        id
    };
    abs_intptr(id)
}

/// `bm_face_region_pivot_edge_find` — returns `(e_pivot, depth)`.
fn bm_face_region_pivot_edge_find(
    bm: &mut BMesh,
    faces_region: &[usize],
    verts_region_len: usize,
) -> (usize, usize) {
    let mut gh: HashMap<usize, SuidInt> = HashMap::new();
    let mut e_pivot = BM_NIL;
    let mut e_pivot_fallback = BM_NIL;
    let mut pass: SuidInt = 0;
    let mut vert_queue_used = 0usize;
    let mut vert_queue_prev: Vec<usize> = Vec::new();
    let mut vert_queue_next: Vec<usize> = Vec::new();
    for &f in faces_region {
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let e = bm.loo(l_iter).e;
            if bm_edge_is_region_boundary(bm, e) {
                for j in 0..2 {
                    let v = if j == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                    if !gh.contains_key(&v) {
                        let v_id = bm_face_region_vert_boundary_id(bm, v);
                        gh.insert(v, v_id);
                        vert_queue_prev.push(v);
                        vert_queue_used += 1;
                    }
                }
            } else {
                e_pivot_fallback = e;
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
    while !vert_queue_prev.is_empty() {
        while let Some(v) = vert_queue_prev.pop() {
            for e in EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
                    let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
                    if bm.vert(v_other).head.hflag & BM_ELEM_TAG != 0
                        && !gh.contains_key(&v_other)
                    {
                        let v_id_other =
                            -bm_face_region_vert_pass_id(bm, &gh, v_other);
                        gh.insert(v_other, v_id_other);
                        vert_queue_next.push(v_other);
                        vert_queue_used += 1;
                    }
                }
            }
        }
        for &v in &vert_queue_next {
            let v_id = gh.get_mut(&v).unwrap();
            *v_id = -*v_id;
            debug_assert!(*v_id > 0);
        }
        std::mem::swap(&mut vert_queue_prev, &mut vert_queue_next);
        vert_queue_next.clear();
        pass += 1;
        if vert_queue_used == verts_region_len {
            break;
        }
    }
    /* NOTE: C iterates the LinkStack (last-in-first-out) — Vec order is the
     * reverse; only affects *which* best edge is picked on ties. */
    if vert_queue_prev.len() >= 2 {
        let mut e_pivot_best = BM_NIL;
        let mut e_pivot_best_id = [0isize; 2];
        for &v in &vert_queue_prev {
            bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
        }
        for &v in &vert_queue_prev {
            bm.vpool[v].head.hflag |= BM_ELEM_TAG;
            for e_test in EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                if bm.edge(e_test).head.hflag & BM_ELEM_TAG != 0 {
                    let v_other = crate::structure::bm_edge_other_vert(bm, e_test, v);
                    if bm.vert(v_other).head.hflag & BM_ELEM_TAG == 0 {
                        bm_face_region_pivot_edge_use_best(
                            bm,
                            &gh,
                            e_test,
                            &mut e_pivot_best,
                            &mut e_pivot_best_id,
                        );
                    }
                }
            }
        }
        e_pivot = e_pivot_best;
    }
    if e_pivot == BM_NIL && !vert_queue_prev.is_empty() {
        let mut e_pivot_best = BM_NIL;
        let mut e_pivot_best_id = [0isize; 2];
        debug_assert!(pass != 0);
        pass -= 1;
        for &v in &vert_queue_prev {
            for e_test in EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                if bm.edge(e_test).head.hflag & BM_ELEM_TAG != 0 {
                    let v_other = crate::structure::bm_edge_other_vert(bm, e_test, v);
                    if bm.vert(v_other).head.hflag & BM_ELEM_TAG != 0 {
                        bm_face_region_pivot_edge_use_best(
                            bm,
                            &gh,
                            e_test,
                            &mut e_pivot_best,
                            &mut e_pivot_best_id,
                        );
                    }
                }
            }
        }
        e_pivot = e_pivot_best;
    }
    if e_pivot == BM_NIL {
        e_pivot = e_pivot_fallback;
        pass = 0;
    }
    (e_pivot, pass.max(0) as usize)
}

/* USE_PIVOT_FASTMATCH */
type UidFashMatch = usize;

/// `bm_vert_fasthash_single`.
fn bm_vert_fasthash_single(bm: &BMesh, v: usize) -> UidFashMatch {
    const PRIME_EDGE: UidFashMatch = 7;
    const PRIME_FACE: UidFashMatch = 31;
    const PRIME_LOOP: UidFashMatch = 61;
    let mut e_num: UidFashMatch = 0;
    let mut f_num: UidFashMatch = 0;
    let mut l_num: UidFashMatch = 0;
    for e in EdgesOfVert::new(bm, v) {
        if bm.edge(e).l != BM_NIL {
            let l_first = bm.edge(e).l;
            let mut l_iter = l_first;
            e_num += 1;
            loop {
                f_num += 1;
                l_num += bm.face(bm.loo(l_iter).f).len as usize;
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
    }
    (e_num * PRIME_EDGE) ^ (f_num * PRIME_FACE).wrapping_mul(l_num * PRIME_LOOP)
}

/// `bm_vert_fasthash_create`.
fn bm_vert_fasthash_create(bm: &mut BMesh, depth: usize) -> Vec<UidFashMatch> {
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT);
    let mut id_prev = vec![0usize; bm.totvert as usize];
    let mut id_curr = vec![0usize; bm.totvert as usize];
    for v in VertsOfMesh::new(bm) {
        id_prev[bm.vert(v).head.index as usize] = bm_vert_fasthash_single(bm, v);
    }
    for _ in 0..depth {
        id_curr.copy_from_slice(&id_prev);
        for e in EdgesOfMesh::new(bm) {
            if bm.edge(e).l != BM_NIL {
                let i1 = bm.vert(bm.edge(e).v1).head.index as usize;
                let i2 = bm.vert(bm.edge(e).v2).head.index as usize;
                id_curr[i1] = id_curr[i1].wrapping_add(id_prev[i2]);
                id_curr[i2] = id_curr[i2].wrapping_add(id_prev[i1]);
            }
        }
        std::mem::swap(&mut id_prev, &mut id_curr);
    }
    id_prev
}

/// `bm_vert_fasthash_edge_order`.
fn bm_vert_fasthash_edge_order(
    bm: &BMesh,
    fm: &[UidFashMatch],
    e: usize,
) -> [UidFashMatch; 2] {
    let mut e_fm = [
        fm[bm.vert(bm.edge(e).v1).head.index as usize],
        fm[bm.vert(bm.edge(e).v2).head.index as usize],
    ];
    if e_fm[0] > e_fm[1] {
        e_fm.swap(0, 1);
    }
    e_fm
}
/// `bm_vert_fasthash_edge_is_match`.
fn bm_vert_fasthash_edge_is_match(
    bm: &BMesh,
    fm: &[UidFashMatch],
    e_a: usize,
    e_b: usize,
) -> bool {
    let e_a_fm = bm_vert_fasthash_edge_order(bm, fm, e_a);
    let e_b_fm = bm_vert_fasthash_edge_order(bm, fm, e_b);
    e_a_fm[0] == e_b_fm[0] && e_a_fm[1] == e_b_fm[1]
}

/// `BM_mesh_region_match` — returns a list of matched face regions
/// (each a list of face indices).
pub fn bm_mesh_region_match(
    bm: &mut BMesh,
    faces_region: &[usize],
) -> Vec<Vec<usize>> {
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_VERT | BM_EDGE | BM_FACE, BM_ELEM_TAG);
    let verts_region_len = bm_face_array_visit(bm, faces_region, true);
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT | BM_FACE);
    let (e_src, depth) =
        bm_face_region_pivot_edge_find(bm, faces_region, verts_region_len);
    let mut r_face_regions = Vec::new();
    if e_src == BM_NIL {
        return r_face_regions;
    }
    let fm = if depth > 0 {
        Some(bm_vert_fasthash_create(bm, depth))
    } else {
        None
    };
    /* USE_WALKER_REUSE */
    let mut w_src = UidWalk::new();
    let mut w_dst = UidWalk::new();
    for e_dst in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e_dst).head.hflag & BM_ELEM_TAG != 0 || bm.edge(e_dst).l == BM_NIL {
            continue;
        }
        if let Some(fm) = &fm {
            if !bm_vert_fasthash_edge_is_match(bm, fm, e_src, e_dst) {
                continue;
            }
        }
        if let Some(faces_result) = bm_mesh_region_match_pair(
            &mut w_src,
            &mut w_dst,
            bm,
            e_src,
            e_dst,
            faces_region.len(),
        ) {
            bm_face_array_visit(bm, &faces_result, false);
            r_face_regions.push(faces_result);
        }
    }
    r_face_regions
}
