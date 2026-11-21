//! Port of `bmesh_marking.cc` — selection/hide marking, flush, active elem.

use blenders_math::vector::Vec3;
use crate::iterators::*;
use crate::query::*;
use crate::structure::*;
use crate::{
    BMesh, BMVert, BM_EDGE, BM_FACE, BM_NIL, BM_VERT,
    BM_ELEM_HIDDEN, BM_ELEM_SELECT, BM_SELECT_ALL, BM_SELECT_EDGE, BM_SELECT_FACE,
    BM_SELECT_VERT,
};

/// Recount `totvertsel`/`totedgesel`/`totfacesel` (`recount_totsels`).
pub fn recount_totsels(bm: &mut BMesh) {
    bm.totvertsel = bm
        .vpool
        .iter()
        .filter(|v| v.head.htype == BM_VERT && (v.head.hflag & BM_ELEM_SELECT) != 0)
        .count() as i32;
    bm.totedgesel = bm
        .epool
        .iter()
        .filter(|e| e.head.htype == BM_EDGE && (e.head.hflag & BM_ELEM_SELECT) != 0)
        .count() as i32;
    bm.totfacesel = bm
        .fpool
        .iter()
        .filter(|f| f.head.htype == BM_FACE && (f.head.hflag & BM_ELEM_SELECT) != 0)
        .count() as i32;
}

/// `BM_mesh_select_is_mixed` — some but not all elements selected.
pub fn bm_mesh_select_is_mixed(bm: &BMesh) -> bool {
    let sel = |h: u8| (h & BM_ELEM_SELECT) != 0;
    let mut any = false;
    let mut all = true;
    for v in &bm.vpool {
        if v.head.htype == BM_VERT {
            any |= sel(v.head.hflag);
            all &= sel(v.head.hflag);
        }
    }
    for e in &bm.epool {
        if e.head.htype == BM_EDGE {
            any |= sel(e.head.hflag);
            all &= sel(e.head.hflag);
        }
    }
    for f in &bm.fpool {
        if f.head.htype == BM_FACE {
            any |= sel(f.head.hflag);
            all &= sel(f.head.hflag);
        }
    }
    any && !all
}

fn bm_vert_is_edge_select_any_other(bm: &BMesh, v: usize, e_first: usize) -> bool {
    for e in EdgesOfVert::new(bm, v) {
        if e != e_first && (bm.edge(e).head.hflag & BM_ELEM_SELECT) != 0 {
            return true;
        }
    }
    false
}

fn bm_vert_is_edge_select_any(bm: &BMesh, v: usize) -> bool {
    for e in EdgesOfVert::new(bm, v) {
        if (bm.edge(e).head.hflag & BM_ELEM_SELECT) != 0 {
            return true;
        }
    }
    false
}

fn bm_vert_is_edge_visible_any(bm: &BMesh, v: usize) -> bool {
    for e in EdgesOfVert::new(bm, v) {
        if (bm.edge(e).head.hflag & BM_ELEM_HIDDEN) == 0 {
            return true;
        }
    }
    false
}

fn bm_edge_is_face_select_any_other(bm: &BMesh, l_first: usize, f_except: usize) -> bool {
    let mut l_iter = l_first;
    loop {
        let f = bm.loo(l_iter).f;
        if f != f_except && (bm.face(f).head.hflag & BM_ELEM_SELECT) != 0 {
            return true;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l_first {
            break;
        }
    }
    false
}

fn bm_edge_is_face_select_any(bm: &BMesh, e: usize) -> bool {
    for f in FacesOfEdge::new(bm, e) {
        if (bm.face(f).head.hflag & BM_ELEM_SELECT) != 0 {
            return true;
        }
    }
    false
}

fn bm_edge_is_face_visible_any(bm: &BMesh, e: usize) -> bool {
    for f in FacesOfEdge::new(bm, e) {
        if (bm.face(f).head.hflag & BM_ELEM_HIDDEN) == 0 {
            return true;
        }
    }
    false
}

/// `BM_mesh_select_mode_clean_ex` — remove invalid selections for selectmode.
pub fn bm_mesh_select_mode_clean_ex(bm: &mut BMesh, selectmode: i32) {
    if (selectmode & BM_SELECT_VERT) != 0 {
        /* keep edge/face selection consistent: clear edges/faces whose
         * verts aren't all selected, then re-derive */
        for e in 0..bm.epool.len() {
            if bm.edge(e).head.htype != BM_EDGE {
                continue;
            }
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            let sel = (bm.vert(v1).head.hflag & BM_ELEM_SELECT) != 0
                && (bm.vert(v2).head.hflag & BM_ELEM_SELECT) != 0;
            if !sel {
                bm.epool[e].head.hflag &= !BM_ELEM_SELECT;
            }
        }
        for f in 0..bm.fpool.len() {
            if bm.face(f).head.htype != BM_FACE {
                continue;
            }
            let l_first = bm.face(f).l_first;
            let mut l_iter = l_first;
            let mut all_sel = true;
            loop {
                if (bm.vert(bm.loo(l_iter).v).head.hflag & BM_ELEM_SELECT) == 0 {
                    all_sel = false;
                }
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first {
                    break;
                }
            }
            if !all_sel {
                bm.fpool[f].head.hflag &= !BM_ELEM_SELECT;
            }
        }
    } else {
        /* vertex mode off: edge/face sel implies vert sel */
        for e in 0..bm.epool.len() {
            if bm.edge(e).head.htype != BM_EDGE {
                continue;
            }
            if (bm.edge(e).head.hflag & BM_ELEM_SELECT) != 0 {
                let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                bm.vpool[v1].head.hflag |= BM_ELEM_SELECT;
                bm.vpool[v2].head.hflag |= BM_ELEM_SELECT;
            }
        }
        for f in 0..bm.fpool.len() {
            if bm.face(f).head.htype != BM_FACE {
                continue;
            }
            if (bm.face(f).head.hflag & BM_ELEM_SELECT) != 0 {
                let l_first = bm.face(f).l_first;
                let mut l_iter = l_first;
                loop {
                    let v = bm.loo(l_iter).v;
                    bm.vpool[v].head.hflag |= BM_ELEM_SELECT;
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l_first {
                        break;
                    }
                }
            }
        }
    }
    recount_totsels(bm);
}

/// `BM_mesh_select_mode_clean`.
pub fn bm_mesh_select_mode_clean(bm: &mut BMesh) {
    bm_mesh_select_mode_clean_ex(bm, bm.selectmode);
}

/// `bm_mesh_select_mode_flush_vert_to_edge` — edge sel = both verts sel.
pub fn bm_mesh_select_mode_flush_vert_to_edge(bm: &mut BMesh) {
    for e in 0..bm.epool.len() {
        if bm.edge(e).head.htype != BM_EDGE {
            continue;
        }
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        let sel = (bm.vert(v1).head.hflag & BM_ELEM_SELECT) != 0
            && (bm.vert(v2).head.hflag & BM_ELEM_SELECT) != 0;
        if sel {
            bm.epool[e].head.hflag |= BM_ELEM_SELECT;
        } else {
            bm.epool[e].head.hflag &= !BM_ELEM_SELECT;
        }
    }
}

/// `bm_mesh_select_mode_flush_edge_to_face` — face sel = all edges sel.
pub fn bm_mesh_select_mode_flush_edge_to_face(bm: &mut BMesh) {
    for f in 0..bm.fpool.len() {
        if bm.face(f).head.htype != BM_FACE {
            continue;
        }
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        let mut all_sel = true;
        loop {
            if (bm.edge(bm.loo(l_iter).e).head.hflag & BM_ELEM_SELECT) == 0 {
                all_sel = false;
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        if all_sel {
            bm.fpool[f].head.hflag |= BM_ELEM_SELECT;
        } else {
            bm.fpool[f].head.hflag &= !BM_ELEM_SELECT;
        }
    }
}

/// `bm_mesh_select_mode_flush_edge_to_vert` — vert sel |= edge sel.
pub fn bm_mesh_select_mode_flush_edge_to_vert(bm: &mut BMesh) {
    for e in 0..bm.epool.len() {
        if bm.edge(e).head.htype != BM_EDGE {
            continue;
        }
        if (bm.edge(e).head.hflag & BM_ELEM_SELECT) != 0 {
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            bm.vpool[v1].head.hflag |= BM_ELEM_SELECT;
            bm.vpool[v2].head.hflag |= BM_ELEM_SELECT;
        }
    }
}

/// `bm_mesh_select_mode_flush_face_to_vert_and_edge`.
pub fn bm_mesh_select_mode_flush_face_to_vert_and_edge(bm: &mut BMesh) {
    for f in 0..bm.fpool.len() {
        if bm.face(f).head.htype != BM_FACE {
            continue;
        }
        if (bm.face(f).head.hflag & BM_ELEM_SELECT) != 0 {
            let l_first = bm.face(f).l_first;
            let mut l_iter = l_first;
            loop {
                let (v, e) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                bm.vpool[v].head.hflag |= BM_ELEM_SELECT;
                bm.epool[e].head.hflag |= BM_ELEM_SELECT;
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first {
                    break;
                }
            }
        }
    }
}

/// `BMSelectFlushFlag`.
pub const BM_SELECT_LEN_FLUSH_RECALC_VERT: u8 = 1;
pub const BM_SELECT_LEN_FLUSH_RECALC_EDGE: u8 = 2;
pub const BM_SELECT_LEN_FLUSH_RECALC_FACE: u8 = 4;
pub const BM_SELECT_LEN_FLUSH_RECALC_ALL: u8 =
    BM_SELECT_LEN_FLUSH_RECALC_VERT | BM_SELECT_LEN_FLUSH_RECALC_EDGE | BM_SELECT_LEN_FLUSH_RECALC_FACE;

/// `BM_mesh_select_mode_flush_ex`.
pub fn bm_mesh_select_mode_flush_ex(bm: &mut BMesh, selectmode: i32, flag: u8) {
    if (selectmode & BM_SELECT_VERT) != 0 {
        if (flag & BM_SELECT_LEN_FLUSH_RECALC_EDGE) != 0 {
            bm_mesh_select_mode_flush_vert_to_edge(bm);
        }
        if (flag & BM_SELECT_LEN_FLUSH_RECALC_FACE) != 0 {
            bm_mesh_select_mode_flush_edge_to_face(bm);
        }
    } else if (selectmode & BM_SELECT_EDGE) != 0 {
        if (flag & BM_SELECT_LEN_FLUSH_RECALC_VERT) != 0 {
            bm_mesh_select_mode_flush_edge_to_vert(bm);
        }
        if (flag & BM_SELECT_LEN_FLUSH_RECALC_FACE) != 0 {
            bm_mesh_select_mode_flush_edge_to_face(bm);
        }
    } else if (selectmode & BM_SELECT_FACE) != 0 {
        if (flag & (BM_SELECT_LEN_FLUSH_RECALC_VERT | BM_SELECT_LEN_FLUSH_RECALC_EDGE)) != 0 {
            bm_mesh_select_mode_flush_face_to_vert_and_edge(bm);
        }
    }
    recount_totsels(bm);
}

/// `BM_mesh_select_mode_flush`.
pub fn bm_mesh_select_mode_flush(bm: &mut BMesh) {
    bm_mesh_select_mode_flush_ex(bm, bm.selectmode, BM_SELECT_LEN_FLUSH_RECALC_ALL);
}

/// `BM_mesh_select_flush_from_verts` — propagate vert sel to edges/faces.
pub fn bm_mesh_select_flush_from_verts(bm: &mut BMesh, select: bool) {
    for e in 0..bm.epool.len() {
        if bm.edge(e).head.htype != BM_EDGE {
            continue;
        }
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        if (bm.vert(v1).head.hflag & BM_ELEM_SELECT) != 0
            && (bm.vert(v2).head.hflag & BM_ELEM_SELECT) != 0
        {
            if select {
                bm.epool[e].head.hflag |= BM_ELEM_SELECT;
            } else {
                bm.epool[e].head.hflag &= !BM_ELEM_SELECT;
            }
        }
    }
    for f in 0..bm.fpool.len() {
        if bm.face(f).head.htype != BM_FACE {
            continue;
        }
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        let mut all_sel = true;
        loop {
            if (bm.edge(bm.loo(l_iter).e).head.hflag & BM_ELEM_SELECT) == 0 {
                all_sel = false;
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        if all_sel {
            if select {
                bm.fpool[f].head.hflag |= BM_ELEM_SELECT;
            } else {
                bm.fpool[f].head.hflag &= !BM_ELEM_SELECT;
            }
        }
    }
    recount_totsels(bm);
}

/// `BM_vert_select_set` — set vert select + flush totals.
pub fn bm_vert_select_set(bm: &mut BMesh, v: usize, select: bool) {
    debug_assert_eq!(bm.vert(v).head.htype, BM_VERT);
    if ((bm.vert(v).head.hflag & BM_ELEM_SELECT) != 0) != select {
        if select {
            bm.vpool[v].head.hflag |= BM_ELEM_SELECT;
            bm.totvertsel += 1;
        } else {
            bm.vpool[v].head.hflag &= !BM_ELEM_SELECT;
            bm.totvertsel -= 1;
        }
    }
}

/// `BM_edge_select_set` — select edge, flush verts + radial faces sel.
pub fn bm_edge_select_set(bm: &mut BMesh, e: usize, select: bool) {
    debug_assert_eq!(bm.edge(e).head.htype, BM_EDGE);
    if ((bm.edge(e).head.hflag & BM_ELEM_SELECT) != 0) == select {
        return;
    }
    if select {
        bm.epool[e].head.hflag |= BM_ELEM_SELECT;
        bm.totedgesel += 1;
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bm_vert_select_set(bm, v1, true);
        bm_vert_select_set(bm, v2, true);
        for f in FacesOfEdge::new(bm, e).collect::<Vec<_>>() {
            bm_face_select_set(bm, f, true);
        }
    } else {
        bm.epool[e].head.hflag &= !BM_ELEM_SELECT;
        bm.totedgesel -= 1;
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        /* deselect verts only when no other selected edge remains */
        if !bm_vert_is_edge_select_any_other(bm, v1, e) {
            bm_vert_select_set(bm, v1, false);
        }
        if !bm_vert_is_edge_select_any_other(bm, v2, e) {
            bm_vert_select_set(bm, v2, false);
        }
        let l_first = bm.edge(e).l;
        if l_first != BM_NIL {
            let faces: Vec<usize> = FacesOfEdge::new(bm, e).collect();
            for f in faces {
                if (bm.face(f).head.hflag & BM_ELEM_SELECT) != 0 {
                    /* deselect face if any of its other edges are unselected... */
                    let mut still_sel = true;
                    let lf = bm.face(f).l_first;
                    let mut l_iter = lf;
                    loop {
                        let le = bm.loo(l_iter).e;
                        if le != e && (bm.edge(le).head.hflag & BM_ELEM_SELECT) == 0 {
                            still_sel = false;
                        }
                        l_iter = bm.loo(l_iter).next;
                        if l_iter == lf {
                            break;
                        }
                    }
                    if !still_sel {
                        bm_face_select_set(bm, f, false);
                    }
                }
            }
        }
    }
}

/// `BM_face_select_set` — select face, flush edges + verts.
pub fn bm_face_select_set(bm: &mut BMesh, f: usize, select: bool) {
    debug_assert_eq!(bm.face(f).head.htype, BM_FACE);
    if ((bm.face(f).head.hflag & BM_ELEM_SELECT) != 0) == select {
        return;
    }
    if select {
        bm.fpool[f].head.hflag |= BM_ELEM_SELECT;
        bm.totfacesel += 1;
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        let mut ev: Vec<(usize, usize)> = Vec::new();
        loop {
            ev.push((bm.loo(l_iter).e, bm.loo(l_iter).v));
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        for (e, v) in ev {
            bm_edge_select_set_noflush(bm, e, true);
            bm_vert_select_set(bm, v, true);
        }
    } else {
        bm.fpool[f].head.hflag &= !BM_ELEM_SELECT;
        bm.totfacesel -= 1;
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        let mut ev: Vec<(usize, usize)> = Vec::new();
        loop {
            ev.push((bm.loo(l_iter).e, bm.loo(l_iter).v));
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
        for (e, _v) in &ev {
            let e = *e;
            if !bm_edge_is_face_select_any_other(bm, bm.edge(e).l.max(0).min(usize::MAX), f) && !bm_edge_is_face_select_any(bm, e) {
                /* edge still selected only if no other selected face uses it */
                let mut other_sel = false;
                for fo in FacesOfEdge::new(bm, e) {
                    if fo != f && (bm.face(fo).head.hflag & BM_ELEM_SELECT) != 0 {
                        other_sel = true;
                    }
                }
                if !other_sel {
                    bm_edge_select_set_noflush(bm, e, false);
                }
            }
        }
        for (_e, v) in ev {
            let v = v;
            if !bm_vert_is_edge_select_any(bm, v) {
                bm_vert_select_set(bm, v, false);
            }
        }
    }
}

/// `BM_edge_select_set_noflush` — select edge without vert/face flush.
pub fn bm_edge_select_set_noflush(bm: &mut BMesh, e: usize, select: bool) {
    debug_assert_eq!(bm.edge(e).head.htype, BM_EDGE);
    if ((bm.edge(e).head.hflag & BM_ELEM_SELECT) != 0) != select {
        if select {
            bm.epool[e].head.hflag |= BM_ELEM_SELECT;
            bm.totedgesel += 1;
        } else {
            bm.epool[e].head.hflag &= !BM_ELEM_SELECT;
            bm.totedgesel -= 1;
        }
    }
}

/// `BM_face_select_set_noflush`.
pub fn bm_face_select_set_noflush(bm: &mut BMesh, f: usize, select: bool) {
    debug_assert_eq!(bm.face(f).head.htype, BM_FACE);
    if ((bm.face(f).head.hflag & BM_ELEM_SELECT) != 0) != select {
        if select {
            bm.fpool[f].head.hflag |= BM_ELEM_SELECT;
            bm.totfacesel += 1;
        } else {
            bm.fpool[f].head.hflag &= !BM_ELEM_SELECT;
            bm.totfacesel -= 1;
        }
    }
}

/// `BM_vert_select_set_noflush`.
pub fn bm_vert_select_set_noflush(bm: &mut BMesh, v: usize, select: bool) {
    bm_vert_select_set(bm, v, select);
}

/// `BM_mesh_select_mode_set` — change selectmode, cleaning selection.
pub fn bm_mesh_select_mode_set(bm: &mut BMesh, selectmode: i32) {
    if bm.selectmode != selectmode {
        bm.selectmode = selectmode;
        bm_mesh_select_mode_clean(bm);
    }
}

/// `bm_mesh_flag_count` — count elements with hflag state `val`.
fn bm_mesh_flag_count(bm: &BMesh, htype: u8, hflag: u8, val: bool) -> i32 {
    let mut count = 0;
    if (htype & BM_VERT) != 0 {
        for v in &bm.vpool {
            if v.head.htype == BM_VERT && ((v.head.hflag & hflag) != 0) == val {
                count += 1;
            }
        }
    }
    if (htype & BM_EDGE) != 0 {
        for e in &bm.epool {
            if e.head.htype == BM_EDGE && ((e.head.hflag & hflag) != 0) == val {
                count += 1;
            }
        }
    }
    if (htype & BM_FACE) != 0 {
        for f in &bm.fpool {
            if f.head.htype == BM_FACE && ((f.head.hflag & hflag) != 0) == val {
                count += 1;
            }
        }
    }
    count
}

/// `BM_mesh_elem_hflag_count_enabled`.
pub fn bm_mesh_elem_hflag_count_enabled(bm: &BMesh, htype: u8, hflag: u8) -> i32 {
    bm_mesh_flag_count(bm, htype, hflag, true)
}

/// `BM_mesh_elem_hflag_count_disabled`.
pub fn bm_mesh_elem_hflag_count_disabled(bm: &BMesh, htype: u8, hflag: u8) -> i32 {
    bm_mesh_flag_count(bm, htype, hflag, false)
}

/// `BM_elem_select_set` — generic select by htype.
pub fn bm_elem_select_set(bm: &mut BMesh, htype: u8, ele: usize, select: bool) {
    match htype {
        BM_VERT => bm_vert_select_set(bm, ele, select),
        BM_EDGE => bm_edge_select_set(bm, ele, select),
        BM_FACE => bm_face_select_set(bm, ele, select),
        _ => debug_assert!(false),
    }
}

/// `BM_mesh_active_face_set`.
pub fn bm_mesh_active_face_set(bm: &mut BMesh, f: usize) {
    bm.act_face = f;
}

/// `BM_mesh_active_face_index_get`.
pub fn bm_mesh_active_face_index_get(bm: &BMesh, is_sloppy: bool, is_selected: bool) -> i32 {
    let f = bm_mesh_active_face_get(bm, is_sloppy, is_selected);
    if f == BM_NIL {
        -1
    } else {
        bm.face(f).head.index
    }
}

/// `BM_mesh_active_edge_index_get`.
pub fn bm_mesh_active_edge_index_get(bm: &BMesh) -> i32 {
    if bm.act_edge == BM_NIL {
        -1
    } else {
        bm.edge(bm.act_edge).head.index
    }
}

/// `BM_mesh_active_vert_index_get`.
pub fn bm_mesh_active_vert_index_get(bm: &BMesh) -> i32 {
    if bm.act_vert == BM_NIL {
        -1
    } else {
        bm.vert(bm.act_vert).head.index
    }
}

/// `BM_mesh_active_face_get` — active face, or sloppy fallback (last selected).
pub fn bm_mesh_active_face_get(bm: &BMesh, is_sloppy: bool, is_selected: bool) -> usize {
    if bm.act_face != BM_NIL
        && bm.face(bm.act_face).head.htype == BM_FACE
        && (!is_selected || (bm.face(bm.act_face).head.hflag & BM_ELEM_SELECT) != 0)
    {
        return bm.act_face;
    }
    if !is_sloppy {
        return BM_NIL;
    }
    /* sloppy: last face in pool that qualifies (C uses edit-selection history) */
    for f in (0..bm.fpool.len()).rev() {
        if bm.face(f).head.htype == BM_FACE
            && (!is_selected || (bm.face(f).head.hflag & BM_ELEM_SELECT) != 0)
            && (bm.face(f).head.hflag & BM_ELEM_HIDDEN) == 0
        {
            return f;
        }
    }
    BM_NIL
}

/// `BM_mesh_active_edge_get`.
pub fn bm_mesh_active_edge_get(bm: &BMesh) -> usize {
    bm.act_edge
}

/// `BM_mesh_active_vert_get`.
pub fn bm_mesh_active_vert_get(bm: &BMesh) -> usize {
    bm.act_vert
}

/// `BM_mesh_active_elem_get` — the active element of the dominant selectmode.
pub fn bm_mesh_active_elem_get(bm: &BMesh) -> (u8, usize) {
    if bm.act_face != BM_NIL && (bm.selectmode & BM_SELECT_FACE) != 0 {
        return (BM_FACE, bm.act_face);
    }
    if bm.act_edge != BM_NIL && (bm.selectmode & BM_SELECT_EDGE) != 0 {
        return (BM_EDGE, bm.act_edge);
    }
    if bm.act_vert != BM_NIL {
        return (BM_VERT, bm.act_vert);
    }
    (0, BM_NIL)
}

/// `BMEditSelection` — active-element record (htype, ele).
#[derive(Clone, Copy)]
pub struct BMEditSelection {
    pub htype: u8,
    pub ele: usize,
}

/// `BM_editselection_center`.
pub fn bm_editselection_center(bm: &BMesh, ese: &BMEditSelection) -> Vec3 {
    match ese.htype {
        BM_VERT => bm.vert(ese.ele).co,
        BM_EDGE => (bm.vert(bm.edge(ese.ele).v1).co + bm.vert(bm.edge(ese.ele).v2).co) / 2.0,
        BM_FACE => bm_face_calc_center_median(bm, ese.ele),
        _ => Vec3::new(0.0, 0.0, 0.0),
    }
}

/// `BM_face_calc_center_median` — average of corner coords.
pub fn bm_face_calc_center_median(bm: &BMesh, f: usize) -> Vec3 {
    let mut cent = Vec3::new(0.0, 0.0, 0.0);
    let mut n = 0i32;
    for l in LoopsOfFace::new(bm, f) {
        cent = cent + bm.vert(bm.loo(l).v).co;
        n += 1;
    }
    if n > 0 {
        cent / n as f32
    } else {
        cent
    }
}

/// `BM_editselection_normal`.
pub fn bm_editselection_normal(bm: &BMesh, ese: &BMEditSelection) -> Vec3 {
    match ese.htype {
        BM_VERT => bm.vert(ese.ele).no,
        BM_EDGE => {
            let e = bm.edge(ese.ele);
            let mut r_normal = bm.vert(e.v1).no + bm.vert(e.v2).no;
            let plane = bm.vert(e.v2).co - bm.vert(e.v1).co;
            let vec = r_normal.cross(plane);
            r_normal = plane.cross(vec);
            r_normal.normalize();
            r_normal
        }
        BM_FACE => bm.face(ese.ele).no,
        _ => Vec3::new(0.0, 0.0, 0.0),
    }
}

/// `BM_mesh_elem_hflag_disable_all` — clear `hflag` on all elems of `htype`
/// (or all types when htype==BM_ALL... here: match C semantics: htype masks).
pub fn bm_mesh_elem_hflag_disable_all(bm: &mut BMesh, htype: u8, hflag: u8) {
    if htype & (1 << crate::BM_VERT) != 0 || htype == crate::BM_VERT {
        for v in 0..bm.totvert as usize { bm.vpool[v].head.hflag &= !hflag; }
    }
    if htype & (1 << crate::BM_EDGE) != 0 || htype == crate::BM_EDGE {
        for e in 0..bm.totedge as usize { bm.epool[e].head.hflag &= !hflag; }
    }
    if htype & (1 << crate::BM_LOOP) != 0 || htype == crate::BM_LOOP {
        for l in 0..bm.totloop as usize { bm.lpool[l].head.hflag &= !hflag; }
    }
    if htype & (1 << crate::BM_FACE) != 0 || htype == crate::BM_FACE {
        for f in 0..bm.totface as usize { bm.fpool[f].head.hflag &= !hflag; }
    }
}

/// `BM_elem_flag_merge_into` — merge hflag of two elems into a third.
pub fn bm_elem_flag_merge_into(bm: &mut BMesh, elem: (u8, usize), a: (u8, usize), b: (u8, usize)) {
    let fa = crate::hflag_of(bm, a);
    let fb = crate::hflag_of(bm, b);
    match elem.0 {
        crate::BM_VERT => bm.vpool[elem.1].head.hflag |= fa | fb,
        crate::BM_EDGE => bm.epool[elem.1].head.hflag |= fa | fb,
        crate::BM_LOOP => bm.lpool[elem.1].head.hflag |= fa | fb,
        _ => bm.fpool[elem.1].head.hflag |= fa | fb,
    }
}

/// `BM_mesh_elem_hflag_enable_all` — enable hflag on all elems of htype.
pub fn bm_mesh_elem_hflag_enable_all(bm: &mut BMesh, htype: u8, hflag: u8) {
    if htype & (1 << crate::BM_VERT) != 0 {
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            bm.vpool[v].head.hflag |= hflag;
        }
    }
    if htype & (1 << crate::BM_EDGE) != 0 {
        for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            bm.epool[e].head.hflag |= hflag;
        }
    }
    if htype & (1 << crate::BM_FACE) != 0 {
        for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
            bm.fpool[f].head.hflag |= hflag;
        }
    }
}
