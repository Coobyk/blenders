//! Port of `bmesh_uvselect.cc` — UV selection state & flushing.
//! `cd_loop_uv_offset` becomes a `uv_at` accessor closure (PORT_PENDING: real
//! CustomData UV layers). `BMUVSelectPickParams` → [`PickParams`].

use crate::query;
use crate::query_uv;
use crate::structure::*;
use crate::{
    BMesh, BM_ELEM_HIDDEN, BM_ELEM_SELECT, BM_ELEM_SELECT_UV, BM_ELEM_SELECT_UV_EDGE, BM_ELEM_TAG,
    BM_NIL, BM_SELECT_EDGE, BM_SELECT_FACE, BM_SELECT_VERT,
};
use blenders_math::vector::Vec2;

/// `BMUVSelectPickParams` — `shared` selects co-located UVs; `uv_at` supplies
/// the per-loop UV (PORT_PENDING: CustomData layer offsets).
pub struct PickParams<'a> {
    pub shared: bool,
    pub uv_at: &'a dyn Fn(&BMesh, usize) -> Vec2,
}

#[inline]
fn flag(bm: &BMesh, htype: u8, i: usize) -> u8 {
    match htype {
        crate::BM_VERT => bm.vert(i).head.hflag,
        crate::BM_EDGE => bm.edge(i).head.hflag,
        crate::BM_LOOP => bm.loo(i).head.hflag,
        _ => bm.face(i).head.hflag,
    }
}
#[inline]
fn flag_set(bm: &mut BMesh, htype: u8, i: usize, bit: u8, value: bool) {
    let f = match htype {
        crate::BM_VERT => &mut bm.vpool[i].head.hflag,
        crate::BM_EDGE => &mut bm.epool[i].head.hflag,
        crate::BM_LOOP => &mut bm.lpool[i].head.hflag,
        _ => &mut bm.fpool[i].head.hflag,
    };
    if value {
        *f |= bit;
    } else {
        *f &= !bit;
    }
}
#[inline]
fn flag_on(bm: &BMesh, htype: u8, i: usize, bit: u8) -> bool {
    flag(bm, htype, i) & bit != 0
}

/// All loops on `v` (BM_LOOPS_OF_VERT).
fn loops_of_vert(bm: &BMesh, v: usize) -> Vec<usize> {
    let mut out = Vec::new();
    if bm.vert(v).e == BM_NIL {
        return out;
    }
    let first = bm.vert(v).e;
    let mut e = first;
    loop {
        let mut l = bm.edge(e).l;
        if l != BM_NIL {
            let l_first = l;
            loop {
                if bm.loo(l).v == v {
                    out.push(l);
                }
                l = bm.loo(l).radial_next;
                if l == l_first {
                    break;
                }
            }
        }
        e = bmesh_disk_edge_next(bm, e, v);
        if e == first {
            break;
        }
    }
    out
}

/// `bm_mesh_uvselect_disable_all`.
fn disable_all(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if bm.face(f).head.htype == crate::BM_FACE {
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV | BM_ELEM_SELECT_UV_EDGE, false);
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, false);
        }
    }
}

/* ---- low level ---- */

/// `BM_loop_vert_uvselect_test`.
pub fn bm_loop_vert_uvselect_test(bm: &BMesh, l: usize) -> bool {
    !flag_on(bm, crate::BM_FACE, bm.loo(l).f, BM_ELEM_HIDDEN)
        && flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
}
/// `BM_loop_edge_uvselect_test`.
pub fn bm_loop_edge_uvselect_test(bm: &BMesh, l: usize) -> bool {
    !flag_on(bm, crate::BM_FACE, bm.loo(l).f, BM_ELEM_HIDDEN)
        && flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
}
/// `BM_face_uvselect_test`.
pub fn bm_face_uvselect_test(bm: &BMesh, f: usize) -> bool {
    !flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN)
        && flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV)
}

/* ---- connectivity checks (each radial fan at l->v) ---- */

/// Shared pattern: walk e-disk at l->v, radial fans, apply `check(l_iter)`.
fn vert_fan_check(bm: &BMesh, l: usize, mut check: impl FnMut(&BMesh, usize) -> bool) -> bool {
    let v = bm.loo(l).v;
    debug_assert!(bm.vert(v).e != BM_NIL);
    let first = bm.vert(v).e;
    let mut e = first;
    loop {
        if bm.edge(e).l != BM_NIL {
            let l_first = bm.edge(e).l;
            let mut l_iter = l_first;
            loop {
                if flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                    continue;
                }
                if bm.loo(l_iter).v != v {
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                    continue;
                }
                if l_iter != l && check(bm, l_iter) {
                    return true;
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
        e = bmesh_disk_edge_next(bm, e, v);
        if e == first {
            break;
        }
    }
    false
}

/// `BM_loop_vert_uvselect_check_other_loop_vert` (hflag: SELECT_UV|TAG).
pub fn bm_loop_vert_uvselect_check_other_loop_vert(
    bm: &BMesh,
    l: usize,
    hflag: u8,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) -> bool {
    vert_fan_check(bm, l, |bm, l_iter| {
        flag_on(bm, crate::BM_LOOP, l_iter, hflag)
            && query_uv::bm_loop_uv_share_vert_check(bm, l, l_iter, uv_at)
    })
}
/// `BM_loop_vert_uvselect_check_other_loop_edge` (hflag: SELECT_UV_EDGE|TAG).
pub fn bm_loop_vert_uvselect_check_other_loop_edge(
    bm: &BMesh,
    l: usize,
    hflag: u8,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) -> bool {
    vert_fan_check(bm, l, |bm, l_iter| {
        (flag_on(bm, crate::BM_LOOP, l_iter, hflag)
            || flag_on(bm, crate::BM_LOOP, bm.loo(l_iter).prev, hflag))
            && query_uv::bm_loop_uv_share_vert_check(bm, l, l_iter, uv_at)
    })
}
/// `BM_loop_vert_uvselect_check_other_edge` (hflag: SELECT|TAG).
pub fn bm_loop_vert_uvselect_check_other_edge(
    bm: &BMesh,
    l: usize,
    hflag: u8,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) -> bool {
    vert_fan_check(bm, l, |bm, l_iter| {
        let e_cur = bm.loo(l_iter).e;
        let e_prev = bm.loo(bm.loo(l_iter).prev).e;
        ((!flag_on(bm, crate::BM_EDGE, e_cur, BM_ELEM_HIDDEN)
            && flag_on(bm, crate::BM_EDGE, e_cur, hflag))
            || (!flag_on(bm, crate::BM_EDGE, e_prev, BM_ELEM_HIDDEN)
                && flag_on(bm, crate::BM_EDGE, e_prev, hflag)))
            && query_uv::bm_loop_uv_share_vert_check(bm, l, l_iter, uv_at)
    })
}
/// `BM_loop_vert_uvselect_check_other_face` (hflag: SELECT|SELECT_UV|TAG).
pub fn bm_loop_vert_uvselect_check_other_face(
    bm: &BMesh,
    l: usize,
    hflag: u8,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) -> bool {
    vert_fan_check(bm, l, |bm, l_iter| {
        flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, hflag)
            && query_uv::bm_loop_uv_share_vert_check(bm, l, l_iter, uv_at)
    })
}
/// `BM_loop_edge_uvselect_check_other_loop_edge` (hflag: SELECT|SELECT_UV_EDGE|TAG).
pub fn bm_loop_edge_uvselect_check_other_loop_edge(
    bm: &BMesh,
    l: usize,
    hflag: u8,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) -> bool {
    let mut l_iter = l;
    loop {
        if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN)
            && l_iter != l
            && flag_on(bm, crate::BM_LOOP, l_iter, hflag)
            && query_uv::bm_loop_uv_share_edge_check(bm, l, l_iter, uv_at)
        {
            return true;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    false
}
/// `BM_loop_edge_uvselect_check_other_face` (hflag: SELECT|SELECT_UV).
pub fn bm_loop_edge_uvselect_check_other_face(
    bm: &BMesh,
    l: usize,
    hflag: u8,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) -> bool {
    let mut l_iter = l;
    loop {
        if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN)
            && l_iter != l
            && flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, hflag)
            && query_uv::bm_loop_uv_share_edge_check(bm, l, l_iter, uv_at)
        {
            return true;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    false
}
/// `BM_face_uvselect_check_edges_all`.
pub fn bm_face_uvselect_check_edges_all(bm: &BMesh, f: usize) -> bool {
    if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
        return false;
    }
    let first = bm.face(f).l_first;
    let mut l = first;
    loop {
        if !flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE) {
            return false;
        }
        l = bm.loo(l).next;
        if l == first {
            break;
        }
    }
    true
}

/* ---- setters ---- */

/// `BM_loop_vert_uvselect_set_noflush`.
pub fn bm_loop_vert_uvselect_set_noflush(bm: &mut BMesh, l: usize, select: bool) {
    flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, select);
}
/// `BM_loop_edge_uvselect_set_noflush`.
pub fn bm_loop_edge_uvselect_set_noflush(bm: &mut BMesh, l: usize, select: bool) {
    flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, select);
}
/// `BM_loop_edge_uvselect_set`.
pub fn bm_loop_edge_uvselect_set(bm: &mut BMesh, l: usize, select: bool) {
    bm_loop_edge_uvselect_set_noflush(bm, l, select);
    bm_loop_vert_uvselect_set_noflush(bm, l, select);
    bm_loop_vert_uvselect_set_noflush(bm, bm.loo(l).next, select);
}
/// `BM_face_uvselect_set_noflush`.
pub fn bm_face_uvselect_set_noflush(bm: &mut BMesh, f: usize, select: bool) {
    flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, select);
}
/// `BM_face_uvselect_set`.
pub fn bm_face_uvselect_set(bm: &mut BMesh, f: usize, select: bool) {
    bm_face_uvselect_set_noflush(bm, f, select);
    let first = bm.face(f).l_first;
    let mut l = first;
    loop {
        bm_loop_vert_uvselect_set_noflush(bm, l, select);
        bm_loop_edge_uvselect_set_noflush(bm, l, select);
        l = bm.loo(l).next;
        if l == first {
            break;
        }
    }
}
/// `BM_mesh_uvselect_clear` — invalidate the sync state.
pub fn bm_mesh_uvselect_clear(bm: &mut BMesh) -> bool {
    if !bm.uv_select_sync_valid {
        return false;
    }
    bm.uv_select_sync_valid = false;
    true
}

/* ---- shared setters ---- */

/// `BM_loop_vert_uvselect_set_shared`.
pub fn bm_loop_vert_uvselect_set_shared(
    bm: &mut BMesh,
    l: usize,
    select: bool,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    bm_loop_vert_uvselect_set_noflush(bm, l, select);
    let v = bm.loo(l).v;
    let first = bm.vert(v).e;
    let mut e = first;
    loop {
        if bm.edge(e).l != BM_NIL {
            let l_first = bm.edge(e).l;
            let mut l_iter = l_first;
            loop {
                if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN)
                    && bm.loo(l_iter).v == v
                    && l_iter != l
                    && flag_on(bm, crate::BM_LOOP, l_iter, BM_ELEM_SELECT_UV) != select
                    && query_uv::bm_loop_uv_share_vert_check(bm, l, l_iter, uv_at)
                {
                    bm_loop_vert_uvselect_set_noflush(bm, l_iter, select);
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
        e = bmesh_disk_edge_next(bm, e, v);
        if e == first {
            break;
        }
    }
}
/// `BM_loop_edge_uvselect_set_shared`.
pub fn bm_loop_edge_uvselect_set_shared(
    bm: &mut BMesh,
    l: usize,
    select: bool,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    bm_loop_edge_uvselect_set_noflush(bm, l, select);
    let mut l_iter = bm.loo(l).radial_next;
    if l_iter != l {
        loop {
            if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN)
                && flag_on(bm, crate::BM_LOOP, l_iter, BM_ELEM_SELECT_UV_EDGE) != select
                && query_uv::bm_loop_uv_share_edge_check(bm, l, l_iter, uv_at)
            {
                bm_loop_edge_uvselect_set_noflush(bm, l_iter, select);
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l {
                break;
            }
        }
    }
}
/// `BM_face_uvselect_set_shared`.
pub fn bm_face_uvselect_set_shared(
    bm: &mut BMesh,
    f: usize,
    select: bool,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    bm_face_uvselect_set_noflush(bm, f, select);
    let first = bm.face(f).l_first;
    let mut l = first;
    loop {
        bm_loop_vert_uvselect_set_shared(bm, l, select, uv_at);
        bm_loop_edge_uvselect_set_shared(bm, l, select, uv_at);
        l = bm.loo(l).next;
        if l == first {
            break;
        }
    }
}
/// `BM_mesh_uvselect_set_elem_shared`.
pub fn bm_mesh_uvselect_set_elem_shared(
    bm: &mut BMesh,
    select: bool,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
    loop_verts: &[usize],
    loop_edges: &[usize],
    faces: &[usize],
) {
    for &l in loop_verts {
        bm_loop_vert_uvselect_set_shared(bm, l, select, uv_at);
    }
    for &l in loop_edges {
        bm_loop_edge_uvselect_set_shared(bm, l, select, uv_at);
        if select {
            bm_loop_vert_uvselect_set_shared(bm, l, select, uv_at);
            bm_loop_vert_uvselect_set_shared(bm, bm.loo(l).next, select, uv_at);
        }
    }
    for &f in faces {
        if select {
            bm_face_uvselect_set_shared(bm, f, select, uv_at);
        } else {
            bm_face_uvselect_set_noflush(bm, f, select);
        }
    }
    if !select {
        for &l_edge in loop_edges {
            if flag_on(bm, crate::BM_FACE, bm.loo(l_edge).f, BM_ELEM_HIDDEN) {
                continue;
            }
            for lv in [l_edge, bm.loo(l_edge).next] {
                let l_edge_other = if lv == l_edge {
                    bm.loo(l_edge).prev
                } else {
                    bm.loo(l_edge).next
                };
                if !flag_on(bm, crate::BM_LOOP, l_edge_other, BM_ELEM_SELECT_UV_EDGE)
                    && !bm_loop_vert_uvselect_check_other_loop_edge(
                        bm,
                        lv,
                        BM_ELEM_SELECT_UV_EDGE,
                        uv_at,
                    )
                {
                    bm_loop_vert_uvselect_set_shared(bm, lv, false, uv_at);
                }
            }
        }
        for &f in faces {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                    && !bm_loop_edge_uvselect_check_other_face(bm, l, BM_ELEM_SELECT_UV, uv_at)
                {
                    bm_loop_edge_uvselect_set_shared(bm, l, false, uv_at);
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
        for &f in faces {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                    && !flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                    && !flag_on(bm, crate::BM_LOOP, bm.loo(l).prev, BM_ELEM_SELECT_UV_EDGE)
                    && !bm_loop_vert_uvselect_check_other_loop_edge(
                        bm,
                        l,
                        BM_ELEM_SELECT_UV_EDGE,
                        uv_at,
                    )
                {
                    bm_loop_vert_uvselect_set_shared(bm, l, false, uv_at);
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
    }
}

/* ---- picking ---- */

/// `bm_vert_uvselect_set_pick`.
fn vert_uvselect_set_pick(
    bm: &mut BMesh,
    v: usize,
    select: bool,
    params: &PickParams,
    caller_handles_edge_or_face_mode: bool,
) {
    let _ = params;
    if !caller_handles_edge_or_face_mode {
        debug_assert!(bm.selectmode & BM_SELECT_VERT != 0);
    }
    if flag_on(bm, crate::BM_VERT, v, BM_ELEM_HIDDEN) || bm.vert(v).e == BM_NIL {
        return;
    }
    if select {
        let first = bm.vert(v).e;
        let mut e = first;
        loop {
            if bm.edge(e).l != BM_NIL {
                let l_first = bm.edge(e).l;
                let mut l_iter = l_first;
                loop {
                    if flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l_first {
                            break;
                        }
                        continue;
                    }
                    if v != bm.loo(l_iter).v {
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l_first {
                            break;
                        }
                        continue;
                    }
                    bm_loop_vert_uvselect_set_noflush(bm, l_iter, true);
                    if flag_on(bm, crate::BM_LOOP, bm.loo(l_iter).next, BM_ELEM_SELECT_UV) {
                        bm_loop_edge_uvselect_set_noflush(bm, l_iter, true);
                    }
                    if flag_on(bm, crate::BM_LOOP, bm.loo(l_iter).prev, BM_ELEM_SELECT_UV) {
                        bm_loop_edge_uvselect_set_noflush(bm, bm.loo(l_iter).prev, true);
                    }
                    let lf = bm.loo(l_iter).f;
                    if !flag_on(bm, crate::BM_FACE, lf, BM_ELEM_HIDDEN)
                        && !flag_on(bm, crate::BM_FACE, lf, BM_ELEM_SELECT_UV)
                        && bm_face_uvselect_check_edges_all(bm, lf)
                    {
                        bm_face_uvselect_set_noflush(bm, lf, true);
                    }
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                }
            }
            e = bmesh_disk_edge_next(bm, e, v);
            if e == first {
                break;
            }
        }
    } else {
        let first = bm.vert(v).e;
        let mut e = first;
        loop {
            if bm.edge(e).l != BM_NIL {
                let l_first = bm.edge(e).l;
                let mut l_iter = l_first;
                loop {
                    if flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l_first {
                            break;
                        }
                        continue;
                    }
                    if v != bm.loo(l_iter).v {
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l_first {
                            break;
                        }
                        continue;
                    }
                    bm_loop_vert_uvselect_set_noflush(bm, l_iter, false);
                    bm_loop_edge_uvselect_set_noflush(bm, l_iter, false);
                    bm_loop_edge_uvselect_set_noflush(bm, bm.loo(l_iter).prev, false);
                    let lf = bm.loo(l_iter).f;
                    bm_face_uvselect_set_noflush(bm, lf, false);
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                }
            }
            e = bmesh_disk_edge_next(bm, e, v);
            if e == first {
                break;
            }
        }
    }
}

/// `bm_edge_uvselect_set_pick`.
fn edge_uvselect_set_pick(
    bm: &mut BMesh,
    e: usize,
    select: bool,
    params: &PickParams,
    caller_handles_face_mode: bool,
) {
    if !caller_handles_face_mode {
        debug_assert!(bm.selectmode & (BM_SELECT_VERT | BM_SELECT_EDGE) != 0);
    }
    if flag_on(bm, crate::BM_EDGE, e, BM_ELEM_HIDDEN) {
        return;
    }
    if !select && (bm.selectmode & BM_SELECT_VERT != 0) {
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        vert_uvselect_set_pick(bm, v1, false, params, false);
        vert_uvselect_set_pick(bm, v2, false, params, false);
        return;
    }
    if bm.edge(e).l == BM_NIL {
        return;
    }
    if !params.shared {
        let l_first = bm.edge(e).l;
        if select {
            let mut any_faces_unselected = false;
            let mut l_iter = l_first;
            loop {
                if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                    bm_loop_edge_uvselect_set_noflush(bm, l_iter, true);
                    bm_loop_vert_uvselect_set_noflush(bm, l_iter, true);
                    bm_loop_vert_uvselect_set_noflush(bm, bm.loo(l_iter).next, true);
                    if !any_faces_unselected
                        && !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_SELECT_UV)
                    {
                        any_faces_unselected = true;
                    }
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
            if any_faces_unselected {
                let mut l_iter = l_first;
                loop {
                    let lf = bm.loo(l_iter).f;
                    if !flag_on(bm, crate::BM_FACE, lf, BM_ELEM_HIDDEN)
                        && !flag_on(bm, crate::BM_FACE, lf, BM_ELEM_SELECT_UV)
                        && bm_face_uvselect_check_edges_all(bm, lf)
                    {
                        bm_face_uvselect_set_noflush(bm, lf, true);
                    }
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                }
            }
        } else {
            let mut l_iter = l_first;
            loop {
                if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                    bm_loop_edge_uvselect_set_noflush(bm, l_iter, false);
                    if !flag_on(bm, crate::BM_LOOP, bm.loo(l_iter).prev, BM_ELEM_SELECT_UV_EDGE)
                    {
                        bm_loop_vert_uvselect_set_noflush(bm, l_iter, false);
                    }
                    if !flag_on(bm, crate::BM_LOOP, bm.loo(l_iter).next, BM_ELEM_SELECT_UV_EDGE)
                    {
                        bm_loop_vert_uvselect_set_noflush(bm, bm.loo(l_iter).next, false);
                    }
                    let lf = bm.loo(l_iter).f;
                    bm_face_uvselect_set_noflush(bm, lf, false);
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
        return;
    }
    /* shared UVs */
    if select {
        let mut any_faces_unselected = false;
        let l_first = bm.edge(e).l;
        let mut l_iter = l_first;
        loop {
            if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                bm_loop_edge_uvselect_set_noflush(bm, l_iter, true);
                bm_loop_vert_uvselect_set_shared(bm, l_iter, true, params.uv_at);
                bm_loop_vert_uvselect_set_shared(bm, bm.loo(l_iter).next, true, params.uv_at);
                if !any_faces_unselected
                    && !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_SELECT_UV)
                {
                    any_faces_unselected = true;
                }
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l_first {
                break;
            }
        }
        if any_faces_unselected {
            let mut l_iter = l_first;
            loop {
                let lf = bm.loo(l_iter).f;
                if !flag_on(bm, crate::BM_FACE, lf, BM_ELEM_HIDDEN)
                    && !flag_on(bm, crate::BM_FACE, lf, BM_ELEM_SELECT_UV)
                    && bm_face_uvselect_check_edges_all(bm, lf)
                {
                    bm_face_uvselect_set_noflush(bm, lf, true);
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
    } else {
        let l_first = bm.edge(e).l;
        let mut l_iter = l_first;
        loop {
            if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                bm_loop_edge_uvselect_set_noflush(bm, l_iter, false);
                if !flag_on(bm, crate::BM_LOOP, bm.loo(l_iter).prev, BM_ELEM_SELECT_UV_EDGE) {
                    bm_loop_vert_uvselect_set_noflush(bm, l_iter, false);
                }
                if !flag_on(bm, crate::BM_LOOP, bm.loo(l_iter).next, BM_ELEM_SELECT_UV_EDGE) {
                    bm_loop_vert_uvselect_set_noflush(bm, bm.loo(l_iter).next, false);
                }
                let lf = bm.loo(l_iter).f;
                bm_face_uvselect_set_noflush(bm, lf, false);
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l_first {
                break;
            }
        }
        let mut l_iter = l_first;
        loop {
            if flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
                continue;
            }
            for l_edge_vert in [l_iter, bm.loo(l_iter).next] {
                if flag_on(bm, crate::BM_LOOP, l_edge_vert, BM_ELEM_SELECT_UV) {
                    continue;
                }
                if bm_loop_vert_uvselect_check_other_loop_edge(
                    bm,
                    l_edge_vert,
                    BM_ELEM_SELECT_UV_EDGE,
                    params.uv_at,
                ) {
                    bm_loop_vert_uvselect_set_noflush(bm, l_edge_vert, true);
                } else {
                    bm_loop_vert_uvselect_set_shared(bm, l_edge_vert, false, params.uv_at);
                }
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l_first {
                break;
            }
        }
    }
}

/// `bm_face_uvselect_set_pick`.
fn face_uvselect_set_pick(bm: &mut BMesh, f: usize, select: bool, params: &PickParams) {
    if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
        return;
    }
    if !select {
        if bm.selectmode & BM_SELECT_VERT != 0 {
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                let v = bm.loo(l).v;
                vert_uvselect_set_pick(bm, v, false, params, false);
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            return;
        }
        if bm.selectmode & BM_SELECT_EDGE != 0 {
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                let e = bm.loo(l).e;
                edge_uvselect_set_pick(bm, e, false, params, false);
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            return;
        }
    }
    if !params.shared {
        bm_face_uvselect_set(bm, f, select);
        return;
    }
    if select {
        bm_face_uvselect_set_noflush(bm, f, true);
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            bm_loop_vert_uvselect_set_noflush(bm, l, true);
            bm_loop_edge_uvselect_set_noflush(bm, l, true);
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        let mut l = first;
        loop {
            bm_loop_vert_uvselect_set_shared(bm, l, true, params.uv_at);
            bm_loop_edge_uvselect_set_shared(bm, l, true, params.uv_at);
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    } else {
        bm_face_uvselect_set_noflush(bm, f, false);
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            bm_loop_vert_uvselect_set_noflush(bm, l, false);
            bm_loop_edge_uvselect_set_noflush(bm, l, false);
            if bm_loop_vert_uvselect_check_other_face(bm, l, BM_ELEM_SELECT_UV, params.uv_at) {
                bm_loop_vert_uvselect_set_noflush(bm, l, true);
            } else {
                bm_loop_vert_uvselect_set_shared(bm, l, false, params.uv_at);
            }
            if bm_loop_edge_uvselect_check_other_face(bm, l, BM_ELEM_SELECT_UV, params.uv_at) {
                bm_loop_edge_uvselect_set_noflush(bm, l, true);
            } else {
                bm_loop_edge_uvselect_set_shared(bm, l, false, params.uv_at);
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    }
}

/// `BM_vert_uvselect_set_pick`.
pub fn bm_vert_uvselect_set_pick(bm: &mut BMesh, v: usize, select: bool, params: &PickParams) {
    vert_uvselect_set_pick(bm, v, select, params, false);
}
/// `BM_edge_uvselect_set_pick`.
pub fn bm_edge_uvselect_set_pick(bm: &mut BMesh, e: usize, select: bool, params: &PickParams) {
    edge_uvselect_set_pick(bm, e, select, params, false);
}
/// `BM_face_uvselect_set_pick`.
pub fn bm_face_uvselect_set_pick(bm: &mut BMesh, f: usize, select: bool, params: &PickParams) {
    face_uvselect_set_pick(bm, f, select, params);
}

/// `bm_mesh_uvselect_mode_flush_down_deselect_only`.
fn mode_flush_down_deselect_only(
    bm: &mut BMesh,
    select_mode: i32,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
    shared: bool,
    check_verts: bool,
    check_edges: bool,
) {
    if !(check_verts || check_edges) {
        return;
    }
    let mut do_check = false;
    if select_mode & BM_SELECT_VERT != 0 {
    } else if select_mode & BM_SELECT_EDGE != 0 {
        if check_verts {
            do_check = true;
        }
    } else if select_mode & BM_SELECT_FACE != 0 {
        if check_verts || check_edges {
            do_check = true;
        }
    }
    if !do_check {
        return;
    }
    if select_mode & BM_SELECT_EDGE != 0 {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN)
                || flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV)
            {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                    && !(flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                        || flag_on(bm, crate::BM_LOOP, bm.loo(l).prev, BM_ELEM_SELECT_UV_EDGE))
                {
                    if !shared
                        || !bm_loop_vert_uvselect_check_other_loop_edge(
                            bm,
                            l,
                            BM_ELEM_SELECT_UV_EDGE,
                            uv_at,
                        )
                    {
                        flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, false);
                    }
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
    } else if select_mode & BM_SELECT_FACE != 0 {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN)
                || flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV)
            {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                    && !bm_loop_edge_uvselect_check_other_face(bm, l, BM_ELEM_SELECT_UV, uv_at)
                {
                    flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, false);
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            let mut e_prev_select =
                flag_on(bm, crate::BM_LOOP, bm.loo(first).prev, BM_ELEM_SELECT_UV_EDGE);
            let mut l = first;
            loop {
                let e_iter_select = flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE);
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                    && !(e_prev_select || e_iter_select)
                {
                    if !shared
                        || !bm_loop_vert_uvselect_check_other_face(
                            bm,
                            l,
                            BM_ELEM_SELECT_UV,
                            uv_at,
                        )
                    {
                        flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, false);
                    }
                }
                e_prev_select = e_iter_select;
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
    }
}

/// `BM_mesh_uvselect_set_elem_from_mesh` (VectorList + Span overloads — same logic).
pub fn bm_mesh_uvselect_set_elem_from_mesh(
    bm: &mut BMesh,
    select: bool,
    params: &PickParams,
    verts: &[usize],
    edges: &[usize],
    faces: &[usize],
    caller_is_batch: bool,
) {
    let check_verts = !verts.is_empty();
    let check_edges = !edges.is_empty();
    if !(check_verts || check_edges || !faces.is_empty()) {
        return;
    }
    for &v in verts {
        if caller_is_batch {
            vert_uvselect_set_pick(bm, v, select, params, true);
        } else {
            bm_vert_uvselect_set_pick(bm, v, select, params);
        }
    }
    for &e in edges {
        if caller_is_batch {
            edge_uvselect_set_pick(bm, e, select, params, true);
        } else {
            bm_edge_uvselect_set_pick(bm, e, select, params);
        }
    }
    for &f in faces {
        bm_face_uvselect_set_pick(bm, f, select, params);
    }
    if select {
        bm_mesh_uvselect_mode_flush_only_select(bm);
    }
    mode_flush_down_deselect_only(
        bm,
        bm.selectmode,
        params.uv_at,
        params.shared,
        check_verts,
        check_edges,
    );
}

/* ---- only-select / only-deselect flushing ---- */

/// `BM_mesh_uvselect_flush_from_loop_verts_only_select`.
pub fn bm_mesh_uvselect_flush_from_loop_verts_only_select(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut all_select = true;
        let mut l = first;
        loop {
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                && flag_on(bm, crate::BM_LOOP, bm.loo(l).next, BM_ELEM_SELECT_UV)
            {
                bm_loop_edge_uvselect_set_noflush(bm, l, true);
            } else {
                all_select = false;
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        if all_select {
            bm_face_uvselect_set_noflush(bm, f, true);
        }
    }
}
/// `BM_mesh_uvselect_flush_from_loop_verts_only_deselect`.
pub fn bm_mesh_uvselect_flush_from_loop_verts_only_deselect(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut all_select = true;
        let mut l = first;
        loop {
            if !(flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                && flag_on(bm, crate::BM_LOOP, bm.loo(l).next, BM_ELEM_SELECT_UV))
            {
                bm_loop_edge_uvselect_set_noflush(bm, l, false);
                all_select = false;
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        if !all_select {
            bm_face_uvselect_set_noflush(bm, f, false);
        }
    }
}
/// `BM_mesh_uvselect_flush_from_loop_edges_only_select`.
pub fn bm_mesh_uvselect_flush_from_loop_edges_only_select(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut all_select = true;
        let mut l = first;
        loop {
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE) {
                bm_loop_edge_uvselect_set(bm, l, true);
            } else {
                all_select = false;
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        if all_select {
            bm_face_uvselect_set_noflush(bm, f, true);
        }
    }
}
/// `BM_mesh_uvselect_flush_from_loop_edges_only_deselect`.
pub fn bm_mesh_uvselect_flush_from_loop_edges_only_deselect(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut all_select = true;
        let mut l = first;
        loop {
            bm_loop_vert_uvselect_set_noflush(
                bm,
                l,
                flag_on(bm, crate::BM_LOOP, bm.loo(l).prev, BM_ELEM_SELECT_UV_EDGE)
                    || flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE),
            );
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE) {
            } else {
                bm_loop_edge_uvselect_set_noflush(bm, l, false);
                all_select = false;
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        if !all_select {
            bm_face_uvselect_set_noflush(bm, f, false);
        }
    }
}
/// `BM_mesh_uvselect_flush_from_faces_only_select`.
pub fn bm_mesh_uvselect_flush_from_faces_only_select(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN)
            || !flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV)
        {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            bm_loop_vert_uvselect_set_noflush(bm, l, true);
            bm_loop_edge_uvselect_set_noflush(bm, l, true);
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    }
}
/// `BM_mesh_uvselect_flush_from_faces_only_deselect`.
pub fn bm_mesh_uvselect_flush_from_faces_only_deselect(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN)
            || flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV)
        {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            bm_loop_vert_uvselect_set_noflush(bm, l, false);
            bm_loop_edge_uvselect_set_noflush(bm, l, false);
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    }
}
/// `BM_mesh_uvselect_flush_shared_only_select`.
pub fn bm_mesh_uvselect_flush_shared_only_select(
    bm: &mut BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            if !flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                && bm_loop_vert_uvselect_check_other_loop_vert(bm, l, BM_ELEM_SELECT_UV, uv_at)
            {
                bm_loop_vert_uvselect_set_noflush(bm, l, true);
            }
            if !flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                && bm_loop_edge_uvselect_check_other_loop_edge(
                    bm,
                    l,
                    BM_ELEM_SELECT_UV_EDGE,
                    uv_at,
                )
            {
                bm_loop_edge_uvselect_set_noflush(bm, l, true);
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    }
}
/// `BM_mesh_uvselect_flush_shared_only_deselect`.
pub fn bm_mesh_uvselect_flush_shared_only_deselect(
    bm: &mut BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                && !bm_loop_vert_uvselect_check_other_loop_vert(bm, l, BM_ELEM_SELECT_UV, uv_at)
            {
                bm_loop_vert_uvselect_set_noflush(bm, l, false);
            }
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                && !bm_loop_edge_uvselect_check_other_loop_edge(
                    bm,
                    l,
                    BM_ELEM_SELECT_UV_EDGE,
                    uv_at,
                )
            {
                bm_loop_edge_uvselect_set_noflush(bm, l, false);
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    }
}

/* ---- between-elements flushing ---- */

/// `BM_mesh_uvselect_flush_from_loop_verts`.
pub fn bm_mesh_uvselect_flush_from_loop_verts(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let mut select_all = true;
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            let select = flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                && flag_on(bm, crate::BM_LOOP, bm.loo(l).next, BM_ELEM_SELECT_UV);
            bm_loop_edge_uvselect_set_noflush(bm, l, select);
            if !select {
                select_all = false;
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        bm_face_uvselect_set_noflush(bm, f, select_all);
    }
}
/// `BM_mesh_uvselect_flush_from_loop_edges`.
pub fn bm_mesh_uvselect_flush_from_loop_edges(bm: &mut BMesh, flush_down: bool) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        if flush_down {
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                bm_loop_vert_uvselect_set_noflush(bm, l, false);
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
        bm_face_uvselect_set_noflush(bm, f, false);
    }
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let mut select_all = true;
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE) {
                if flush_down {
                    bm_loop_vert_uvselect_set_noflush(bm, l, true);
                    bm_loop_vert_uvselect_set_noflush(bm, bm.loo(l).next, true);
                }
            } else {
                select_all = false;
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        if select_all {
            bm_face_uvselect_set_noflush(bm, f, true);
        }
    }
}
/// `BM_mesh_uvselect_flush_from_faces`.
pub fn bm_mesh_uvselect_flush_from_faces(bm: &mut BMesh, flush_down: bool) {
    if !flush_down {
        return;
    }
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let select_face = flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV);
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            bm_loop_vert_uvselect_set_noflush(bm, l, select_face);
            bm_loop_edge_uvselect_set_noflush(bm, l, select_face);
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    }
}
/// `BM_mesh_uvselect_flush_from_verts`.
pub fn bm_mesh_uvselect_flush_from_verts(bm: &mut BMesh, select: bool) {
    if select {
        bm_mesh_uvselect_flush_from_loop_verts_only_select(bm);
    } else {
        bm_mesh_uvselect_flush_from_loop_verts_only_deselect(bm);
    }
}

/* ---- mode-aware flushing ---- */

/// `BM_mesh_uvselect_mode_flush_ex`.
pub fn bm_mesh_uvselect_mode_flush_ex(bm: &mut BMesh, selectmode: i32, flush_down: bool) {
    if selectmode & BM_SELECT_VERT != 0 {
        bm_mesh_uvselect_flush_from_loop_verts(bm);
    } else if selectmode & BM_SELECT_EDGE != 0 {
        bm_mesh_uvselect_flush_from_loop_edges(bm, flush_down);
    } else {
        bm_mesh_uvselect_flush_from_faces(bm, flush_down);
    }
}
/// `BM_mesh_uvselect_mode_flush`.
pub fn bm_mesh_uvselect_mode_flush(bm: &mut BMesh) {
    let sm = bm.selectmode;
    bm_mesh_uvselect_mode_flush_ex(bm, sm, false);
}
/// `BM_mesh_uvselect_mode_flush_only_select`.
pub fn bm_mesh_uvselect_mode_flush_only_select(bm: &mut BMesh) {
    if bm.selectmode & BM_SELECT_VERT != 0 {
        bm_mesh_uvselect_flush_from_loop_verts_only_select(bm);
    } else if bm.selectmode & BM_SELECT_EDGE != 0 {
        bm_mesh_uvselect_flush_from_loop_edges_only_select(bm);
    }
}
/// `BM_mesh_uvselect_mode_flush_only_deselect`.
pub fn bm_mesh_uvselect_mode_flush_only_deselect(bm: &mut BMesh) {
    if bm.selectmode & BM_SELECT_VERT != 0 {
        bm_mesh_uvselect_flush_from_loop_verts_only_deselect(bm);
    } else if bm.selectmode & BM_SELECT_EDGE != 0 {
        bm_mesh_uvselect_flush_from_loop_edges_only_deselect(bm);
    } else {
        bm_mesh_uvselect_flush_from_faces_only_deselect(bm);
    }
}
/// `BM_mesh_uvselect_mode_flush_update`.
pub fn bm_mesh_uvselect_mode_flush_update(
    bm: &mut BMesh,
    selectmode_old: i32,
    selectmode_new: i32,
    uv_at: Option<&dyn Fn(&BMesh, usize) -> Vec2>,
) {
    let hob = |m: i32| -> u32 { if m == 0 { 0 } else { m.unsigned_abs().ilog2() } };
    if hob(selectmode_old) >= hob(selectmode_new) {
        if selectmode_old & BM_SELECT_VERT == 0 && selectmode_new & BM_SELECT_VERT != 0 {
            bm_mesh_uvselect_flush_from_loop_verts(bm);
        } else if selectmode_old & BM_SELECT_EDGE == 0 && selectmode_new & BM_SELECT_EDGE != 0 {
            bm_mesh_uvselect_flush_from_loop_edges(bm, false);
        }
        return;
    }
    let mut do_flush_deselect_down = false;
    if selectmode_old & BM_SELECT_VERT != 0 {
        if selectmode_new & BM_SELECT_VERT == 0 {
            do_flush_deselect_down = true;
        }
    } else if selectmode_old & BM_SELECT_EDGE != 0 {
        if selectmode_new & BM_SELECT_EDGE == 0 {
            do_flush_deselect_down = true;
        }
    }
    if !do_flush_deselect_down {
        return;
    }
    /* pass 1: match underlying mesh */
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut select_face = true;
        let mut l = first;
        loop {
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                && !flag_on(bm, crate::BM_VERT, bm.loo(l).v, BM_ELEM_SELECT)
            {
                flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, false);
            }
            if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE) {
                if !flag_on(bm, crate::BM_EDGE, bm.loo(l).e, BM_ELEM_SELECT) {
                    flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, false);
                    select_face = false;
                }
            } else {
                select_face = false;
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        if !select_face {
            flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, false);
        }
    }
    /* pass 2: deselect isolated */
    if let Some(uv_at) = uv_at {
        let check_verts = bm.totvertsel != 0;
        let check_edges = bm.totedgesel != 0;
        mode_flush_down_deselect_only(bm, selectmode_new, uv_at, true, check_verts, check_edges);
    }
}
/// `BM_mesh_uvselect_flush_post_subdivide`.
pub fn bm_mesh_uvselect_flush_post_subdivide(
    bm: &mut BMesh,
    uv_at: Option<&dyn Fn(&BMesh, usize) -> Vec2>,
) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT) {
            bm_face_uvselect_set(bm, f, true);
        }
    }
    let use_edges = bm.selectmode & (BM_SELECT_VERT | BM_SELECT_EDGE) != 0;
    if use_edges {
        let mut edges: Vec<usize> = Vec::new();
        for e in 0..bm.totedge as usize {
            edges.push(e);
        }
        for e in edges {
            if bm.edge(e).l == BM_NIL {
                continue;
            }
            if flag_on(bm, crate::BM_EDGE, e, BM_ELEM_SELECT)
                && !query::bm_edge_is_any_face_flag_test(bm, e, BM_ELEM_SELECT)
            {
                let l_first = bm.edge(e).l;
                let mut l_iter = l_first;
                loop {
                    if !flag_on(bm, crate::BM_FACE, bm.loo(l_iter).f, BM_ELEM_HIDDEN) {
                        bm_loop_edge_uvselect_set(bm, l_iter, true);
                    }
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first {
                        break;
                    }
                }
            }
        }
    }
    /* select shared UVs connected to edge/face */
    if let Some(uv_at) = uv_at {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN)
                || flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV)
            {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if bm.selectmode & BM_SELECT_VERT == 0
                    && !flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                    && bm_loop_edge_uvselect_check_other_loop_edge(
                        bm,
                        l,
                        BM_ELEM_SELECT_UV_EDGE,
                        uv_at,
                    )
                {
                    bm_loop_edge_uvselect_set(bm, l, true);
                }
                if !flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                    && bm_loop_vert_uvselect_check_other_loop_vert(bm, l, BM_ELEM_SELECT_UV, uv_at)
                {
                    bm_loop_vert_uvselect_set_noflush(bm, l, true);
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
    }
    bm_mesh_uvselect_mode_flush_only_select(bm);
}

/* ---- sync from mesh ---- */

/// `bm_mesh_uvselect_flush_from_mesh_sticky_vert_for_vert_mode`.
fn flush_from_mesh_sticky_vert_for_vert_mode(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            let v_select = flag_on(bm, crate::BM_VERT, bm.loo(l).v, BM_ELEM_SELECT);
            let e_select = flag_on(bm, crate::BM_EDGE, bm.loo(l).e, BM_ELEM_SELECT);
            flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, v_select);
            flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, e_select);
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        let fsel = flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT);
        flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, fsel);
    }
    bm.uv_select_sync_valid = true;
}
/// `bm_mesh_uvselect_flush_from_mesh_sticky_vert_for_edge_mode`.
fn flush_from_mesh_sticky_vert_for_edge_mode(bm: &mut BMesh) {
    disable_all(bm);
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            if flag_on(bm, crate::BM_EDGE, bm.loo(l).e, BM_ELEM_SELECT) {
                flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, true);
                for lv in [l, bm.loo(l).next] {
                    if !flag_on(bm, crate::BM_LOOP, lv, BM_ELEM_SELECT_UV) {
                        flag_set(bm, crate::BM_LOOP, lv, BM_ELEM_SELECT_UV, true);
                    }
                }
            }
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT) {
            flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, true);
        }
    }
    bm.uv_select_sync_valid = true;
}
/// `bm_mesh_uvselect_flush_from_mesh_sticky_vert_for_face_mode`.
fn flush_from_mesh_sticky_vert_for_face_mode(bm: &mut BMesh) {
    disable_all(bm);
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT) {
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, true);
                flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, true);
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, true);
        }
    }
    bm.uv_select_sync_valid = true;
}
/// `bm_mesh_uvselect_flush_from_mesh_sticky_location_for_vert_mode`.
fn flush_from_mesh_sticky_location_for_vert_mode(bm: &mut BMesh) {
    flush_from_mesh_sticky_vert_for_vert_mode(bm);
}
/// `bm_mesh_uvselect_flush_from_mesh_sticky_location_for_edge_mode`.
fn flush_from_mesh_sticky_location_for_edge_mode(
    bm: &mut BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut e_prev_select =
            flag_on(bm, crate::BM_EDGE, bm.loo(bm.loo(first).prev).e, BM_ELEM_SELECT);
        let mut l = first;
        loop {
            let e_iter_select = flag_on(bm, crate::BM_EDGE, bm.loo(l).e, BM_ELEM_SELECT);
            let v_iter_select = flag_on(bm, crate::BM_VERT, bm.loo(l).v, BM_ELEM_SELECT)
                && ((e_prev_select || e_iter_select)
                    || bm_loop_vert_uvselect_check_other_edge(bm, l, BM_ELEM_SELECT, uv_at));
            flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, v_iter_select);
            flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, e_iter_select);
            e_prev_select = e_iter_select;
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
        let fsel = flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT);
        flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, fsel);
    }
    bm.uv_select_sync_valid = true;
}
/// `bm_mesh_uvselect_flush_from_mesh_sticky_location_for_face_mode`.
fn flush_from_mesh_sticky_location_for_face_mode(
    bm: &mut BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT) {
            let mut l = first;
            loop {
                flag_set(
                    bm,
                    crate::BM_LOOP,
                    l,
                    BM_ELEM_SELECT_UV | BM_ELEM_SELECT_UV_EDGE,
                    true,
                );
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, true);
        } else {
            let mut l = first;
            loop {
                let v_iter_select = flag_on(bm, crate::BM_VERT, bm.loo(l).v, BM_ELEM_SELECT)
                    && bm_loop_vert_uvselect_check_other_face(bm, l, BM_ELEM_SELECT, uv_at);
                let e_iter_select = flag_on(bm, crate::BM_EDGE, bm.loo(l).e, BM_ELEM_SELECT)
                    && bm_loop_edge_uvselect_check_other_face(bm, l, BM_ELEM_SELECT, uv_at);
                flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV, v_iter_select);
                flag_set(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE, e_iter_select);
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            flag_set(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV, false);
        }
    }
    bm.uv_select_sync_valid = true;
}
/// `BM_mesh_uvselect_sync_from_mesh_sticky_location`.
pub fn bm_mesh_uvselect_sync_from_mesh_sticky_location(
    bm: &mut BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
) {
    if bm.selectmode & BM_SELECT_VERT != 0 {
        flush_from_mesh_sticky_location_for_vert_mode(bm);
    } else if bm.selectmode & BM_SELECT_EDGE != 0 {
        flush_from_mesh_sticky_location_for_edge_mode(bm, uv_at);
    } else {
        flush_from_mesh_sticky_location_for_face_mode(bm, uv_at);
    }
}
/// `BM_mesh_uvselect_sync_from_mesh_sticky_disabled`.
pub fn bm_mesh_uvselect_sync_from_mesh_sticky_disabled(bm: &mut BMesh) {
    flush_from_mesh_sticky_vert_for_vert_mode(bm);
}
/// `BM_mesh_uvselect_sync_from_mesh_sticky_vert`.
pub fn bm_mesh_uvselect_sync_from_mesh_sticky_vert(bm: &mut BMesh) {
    if bm.selectmode & BM_SELECT_VERT != 0 {
        flush_from_mesh_sticky_vert_for_vert_mode(bm);
    } else if bm.selectmode & BM_SELECT_EDGE != 0 {
        flush_from_mesh_sticky_vert_for_edge_mode(bm);
    } else {
        flush_from_mesh_sticky_vert_for_face_mode(bm);
    }
}

/* ---- validation ---- */

/// `UVSelectValidateInfo_*` — error counters per check.
#[derive(Default)]
pub struct ValidateInfo {
    pub sync: [i32; 4],
    pub flush: [i32; 4],
    pub contiguous: [i32; 2],
    pub flush_contiguous: [i32; 3],
}
const TAG: u8 = BM_ELEM_TAG;

fn loop_clear_tag(bm: &mut BMesh) {
    for f in 0..bm.totface as usize {
        let first = bm.face(f).l_first;
        let mut l = first;
        loop {
            flag_set(bm, crate::BM_LOOP, l, TAG, false);
            l = bm.loo(l).next;
            if l == first {
                break;
            }
        }
    }
}

/// `bm_mesh_uvselect_check_viewport_sync`.
fn check_viewport_sync(bm: &BMesh, info_sub: &mut [i32; 4]) -> bool {
    let mut is_valid = true;
    {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                    && !flag_on(bm, crate::BM_VERT, bm.loo(l).v, BM_ELEM_SELECT)
                {
                    info_sub[0] += 1;
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
        if info_sub[0] != 0 {
            is_valid = false;
        }
    }
    {
        for v in 0..bm.totvert as usize {
            if flag_on(bm, crate::BM_VERT, v, BM_ELEM_HIDDEN)
                || !flag_on(bm, crate::BM_VERT, v, BM_ELEM_SELECT)
            {
                continue;
            }
            let any_loop_selected = loops_of_vert(bm, v)
                .iter()
                .any(|&l| flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV));
            if !any_loop_selected {
                info_sub[1] += 1;
            }
        }
        if info_sub[1] != 0 {
            is_valid = false;
        }
    }
    {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                    && !flag_on(bm, crate::BM_EDGE, bm.loo(l).e, BM_ELEM_SELECT)
                {
                    info_sub[2] += 1;
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
        if info_sub[2] != 0 {
            is_valid = false;
        }
    }
    if bm.selectmode & BM_SELECT_VERT == 0 {
        for e in 0..bm.totedge as usize {
            if flag_on(bm, crate::BM_EDGE, e, BM_ELEM_HIDDEN)
                || !flag_on(bm, crate::BM_EDGE, e, BM_ELEM_SELECT)
                || bm.edge(e).l == BM_NIL
            {
                continue;
            }
            let mut any_loop_selected = false;
            let l_first = bm.edge(e).l;
            let mut l = l_first;
            loop {
                if !flag_on(bm, crate::BM_FACE, bm.loo(l).f, BM_ELEM_HIDDEN)
                    && flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                {
                    any_loop_selected = true;
                    break;
                }
                l = bm.loo(l).radial_next;
                if l == l_first {
                    break;
                }
            }
            if !any_loop_selected {
                info_sub[3] += 1;
            }
        }
        if info_sub[3] != 0 {
            is_valid = false;
        }
    }
    is_valid
}

/// `bm_mesh_uvselect_check_flush`.
fn check_flush(bm: &BMesh, info_sub: &mut [i32; 4]) -> bool {
    let mut is_valid = true;
    {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                let v_curr = flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV);
                let v_next = flag_on(bm, crate::BM_LOOP, bm.loo(l).next, BM_ELEM_SELECT_UV);
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE) {
                    if !v_curr || !v_next {
                        info_sub[0] += 1;
                    }
                } else if v_curr && v_next && (bm.selectmode & BM_SELECT_VERT != 0) {
                    info_sub[1] += 1;
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
        if info_sub[0] != 0 || info_sub[1] != 0 {
            is_valid = false;
        }
    }
    {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
                continue;
            }
            let mut uv_vert_select = 0;
            let mut uv_edge_select = 0;
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV) {
                    uv_vert_select += 1;
                }
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE) {
                    uv_edge_select += 1;
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
            let f_len = bm.face(f).len;
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV) {
                if uv_vert_select != f_len {
                    info_sub[0] += 0; /* verts_selected -> flush[0] reused per C ordering below */
                }
            }
            /* C assigns separate counters; fold into slots 2/3 with sign for readability */
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV) {
                if uv_vert_select != f_len {
                    info_sub[2] += 1;
                }
                if uv_edge_select != f_len {
                    info_sub[3] += 1;
                }
            } else {
                if bm.selectmode & BM_SELECT_VERT != 0 {
                    if uv_vert_select == f_len {
                        info_sub[2] += 1;
                    }
                } else if bm.selectmode & BM_SELECT_EDGE != 0 {
                    if uv_edge_select == f_len {
                        info_sub[3] += 1;
                    }
                }
            }
        }
        if info_sub[2] != 0 || info_sub[3] != 0 {
            is_valid = false;
        }
    }
    is_valid
}

/// `bm_mesh_uvselect_check_contiguous`.
fn check_contiguous(
    bm: &mut BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
    info_sub: &mut [i32; 2],
) -> bool {
    const UV_IS_SELECTED: i32 = 1 << 0;
    const UV_IS_UNSELECTED: i32 = 1 << 1;
    let mut is_valid = true;
    /* verts */
    loop_clear_tag(bm);
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l_base = first;
        loop {
            if flag_on(bm, crate::BM_LOOP, l_base, TAG) {
                l_base = bm.loo(l_base).next;
                if l_base == first {
                    break;
                }
                continue;
            }
            flag_set(bm, crate::BM_LOOP, l_base, TAG, true);
            let v = bm.loo(l_base).v;
            let mut select_test = 0;
            for l_other in loops_of_vert(bm, v) {
                if flag_on(bm, crate::BM_FACE, bm.loo(l_other).f, BM_ELEM_HIDDEN)
                    || flag_on(bm, crate::BM_LOOP, l_other, TAG)
                    || !query_uv::bm_loop_uv_share_vert_check(bm, l_base, l_other, uv_at)
                {
                    continue;
                }
                select_test |= if flag_on(bm, crate::BM_LOOP, l_other, BM_ELEM_SELECT_UV) {
                    UV_IS_SELECTED
                } else {
                    UV_IS_UNSELECTED
                };
                if select_test == UV_IS_SELECTED | UV_IS_UNSELECTED {
                    break;
                }
            }
            if select_test == UV_IS_SELECTED | UV_IS_UNSELECTED {
                info_sub[0] += 1;
            }
            l_base = bm.loo(l_base).next;
            if l_base == first {
                break;
            }
        }
    }
    if info_sub[0] != 0 {
        is_valid = false;
    }
    /* edges */
    loop_clear_tag(bm);
    for f in 0..bm.totface as usize {
        if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
            continue;
        }
        let first = bm.face(f).l_first;
        let mut l_base = first;
        loop {
            if flag_on(bm, crate::BM_LOOP, l_base, TAG) {
                l_base = bm.loo(l_base).next;
                if l_base == first {
                    break;
                }
                continue;
            }
            flag_set(bm, crate::BM_LOOP, l_base, TAG, true);
            let mut select_test = 0;
            let mut l_other = bm.loo(l_base).radial_next;
            if l_other != l_base {
                loop {
                    if !flag_on(bm, crate::BM_FACE, bm.loo(l_other).f, BM_ELEM_HIDDEN)
                        && !flag_on(bm, crate::BM_LOOP, l_other, TAG)
                        && query_uv::bm_loop_uv_share_edge_check(bm, l_base, l_other, uv_at)
                    {
                        select_test |= if flag_on(bm, crate::BM_LOOP, l_other, BM_ELEM_SELECT_UV_EDGE)
                        {
                            UV_IS_SELECTED
                        } else {
                            UV_IS_UNSELECTED
                        };
                        if select_test == UV_IS_SELECTED | UV_IS_UNSELECTED {
                            break;
                        }
                    }
                    l_other = bm.loo(l_other).radial_next;
                    if l_other == l_base {
                        break;
                    }
                }
            }
            if select_test == UV_IS_SELECTED | UV_IS_UNSELECTED {
                info_sub[1] += 1;
            }
            l_base = bm.loo(l_base).next;
            if l_base == first {
                break;
            }
        }
    }
    if info_sub[1] != 0 {
        is_valid = false;
    }
    is_valid
}

/// `bm_mesh_uvselect_check_flush_and_contiguous`.
fn check_flush_and_contiguous(
    bm: &BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
    info_sub: &mut [i32; 3],
) -> bool {
    let mut is_valid = true;
    if (bm.selectmode & BM_SELECT_EDGE != 0) && (bm.selectmode & BM_SELECT_VERT == 0) {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN) {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                    && !flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                    && !flag_on(bm, crate::BM_LOOP, bm.loo(l).prev, BM_ELEM_SELECT_UV_EDGE)
                    && !bm_loop_vert_uvselect_check_other_loop_edge(
                        bm,
                        l,
                        BM_ELEM_SELECT_UV_EDGE,
                        uv_at,
                    )
                {
                    info_sub[0] += 1;
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
        if info_sub[0] != 0 {
            is_valid = false;
        }
    }
    if (bm.selectmode & BM_SELECT_FACE != 0)
        && (bm.selectmode & (BM_SELECT_VERT | BM_SELECT_EDGE) == 0)
    {
        for f in 0..bm.totface as usize {
            if flag_on(bm, crate::BM_FACE, f, BM_ELEM_HIDDEN)
                || flag_on(bm, crate::BM_FACE, f, BM_ELEM_SELECT_UV)
            {
                continue;
            }
            let first = bm.face(f).l_first;
            let mut l = first;
            loop {
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV)
                    && !bm_loop_vert_uvselect_check_other_face(bm, l, BM_ELEM_SELECT_UV, uv_at)
                {
                    info_sub[1] += 1;
                }
                if flag_on(bm, crate::BM_LOOP, l, BM_ELEM_SELECT_UV_EDGE)
                    && !bm_loop_edge_uvselect_check_other_face(bm, l, BM_ELEM_SELECT_UV, uv_at)
                {
                    info_sub[2] += 1;
                }
                l = bm.loo(l).next;
                if l == first {
                    break;
                }
            }
        }
        if info_sub[1] != 0 || info_sub[2] != 0 {
            is_valid = false;
        }
    }
    is_valid
}

/// `BM_mesh_uvselect_is_valid`.
pub fn bm_mesh_uvselect_is_valid(
    bm: &mut BMesh,
    uv_at: &dyn Fn(&BMesh, usize) -> Vec2,
    check_sync: bool,
    check_flush_flag: bool,
    check_contiguous_flag: bool,
    info_p: Option<&mut ValidateInfo>,
) -> bool {
    let mut fallback = ValidateInfo::default();
    let info = info_p.unwrap_or(&mut fallback);
    let mut is_valid = true;
    if check_sync {
        debug_assert!(bm.uv_select_sync_valid);
        if !check_viewport_sync(bm, &mut info.sync) {
            is_valid = false;
        }
    }
    if check_flush_flag {
        if !check_flush(bm, &mut info.flush) {
            is_valid = false;
        }
    }
    if check_contiguous_flag {
        if !check_contiguous(bm, uv_at, &mut info.contiguous) {
            is_valid = false;
        }
    }
    if check_flush_flag && check_contiguous_flag {
        if !check_flush_and_contiguous(bm, uv_at, &mut info.flush_contiguous) {
            is_valid = false;
        }
    }
    is_valid
}
