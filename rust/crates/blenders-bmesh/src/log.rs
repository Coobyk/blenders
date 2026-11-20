//! Port of `bmesh_log.cc` — the sculpt undo/redo log.
//! `RangeTreeUInt` → [`RangeTree`]; `ListBaseT<BMLogEntry>` → index-linked
//! `Vec<BMLogEntry>`; CustomData `.sculpt_mask` → optional `mask_of` accessor,
//! `.sculpt_face_set` is PORT_PENDING.

use crate::core;
use crate::{BMesh, BM_NIL};
use blenders_math::vector::Vec3;
use std::collections::{HashMap, HashSet};

/// `RangeTreeUInt` — a set of used u32 ranges; take_any returns the lowest
/// free id, retake marks an id used, release frees it.
#[derive(Default)]
pub struct RangeTree {
    used: HashSet<u32>,
    next_hint: u32,
}
impl RangeTree {
    /// `range_tree_uint_alloc(0, UINT_MAX)`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `range_tree_uint_take_any` — take the lowest free id.
    pub fn take_any(&mut self) -> u32 {
        let mut id = self.next_hint;
        while self.used.contains(&id) {
            id += 1;
        }
        self.used.insert(id);
        self.next_hint = id + 1;
        id
    }
    /// `range_tree_uint_retake` — mark an existing id used.
    pub fn retake(&mut self, id: u32) {
        self.used.insert(id);
    }
    /// `range_tree_uint_release` — free an id.
    pub fn release(&mut self, id: u32) {
        self.used.remove(&id);
        if id < self.next_hint {
            self.next_hint = id;
        }
    }
}

/// `BMLogVert` — a vertex snapshot.
#[derive(Clone, Copy, Default)]
pub struct LogVert {
    pub position: Vec3,
    pub normal: Vec3,
    pub hflag: u8,
    pub mask: f32,
}
/// `BMLogFace` — a triangle snapshot (vert IDs + hflag).
#[derive(Clone, Copy, Default)]
pub struct LogFace {
    pub v_ids: [u32; 3],
    pub hflag: u8,
}

/// `BMLogEntry` — one undo step; prev/next are entry indices.
#[derive(Default)]
pub struct LogEntry {
    pub deleted_verts: HashMap<u32, usize>,
    pub deleted_faces: HashMap<u32, usize>,
    pub added_verts: HashMap<u32, usize>,
    pub added_faces: HashMap<u32, usize>,
    pub modified_verts: HashMap<u32, usize>,
    pub modified_faces: HashMap<u32, usize>,
    pub vert_pool: Vec<LogVert>,
    pub face_pool: Vec<LogFace>,
    pub prev: usize,
    pub next: usize,
    /// `entry->log` — false when freed from the log.
    pub has_log: bool,
}
impl LogEntry {
    fn new() -> Self {
        Self {
            prev: BM_NIL,
            next: BM_NIL,
            has_log: false,
            ..Default::default()
        }
    }
}

/// `BMLog` — the undo log; `elem` keys are (htype, index).
pub struct BMLog {
    pub unused_ids: RangeTree,
    pub id_to_elem: HashMap<u32, (u8, usize)>,
    pub elem_to_id: HashMap<(u8, usize), u32>,
    pub entries: Vec<LogEntry>,
    pub first: usize,
    pub last: usize,
    /// `log->current_entry` — BM_NIL = original mesh (nothing to undo);
    /// == last = nothing to redo.
    pub current_entry: usize,
}

fn bm_log_vert_id_get(log: &BMLog, v: usize) -> u32 {
    log.elem_to_id[&(crate::BM_VERT, v)]
}
fn bm_log_face_id_get(log: &BMLog, f: usize) -> u32 {
    log.elem_to_id[&(crate::BM_FACE, f)]
}
fn bm_log_elem_id_set(log: &mut BMLog, htype: u8, i: usize, id: u32) {
    log.id_to_elem.insert(id, (htype, i));
    log.elem_to_id.insert((htype, i), id);
}
fn bm_log_vert_from_id(log: &BMLog, id: u32) -> usize {
    log.id_to_elem[&id].1
}

/// `bm_log_vert_bmvert_copy` — snapshot a BMVert into a LogVert.
fn log_vert_copy(bm: &BMesh, v: usize, mask: f32) -> LogVert {
    LogVert {
        position: bm.vert(v).co,
        normal: bm.vert(v).no,
        hflag: bm.vert(v).head.hflag,
        mask,
    }
}
/// `bm_log_vert_alloc` — push into current entry's vert_pool.
fn log_vert_alloc(
    bm: &BMesh,
    log: &mut BMLog,
    v: usize,
    mask: f32,
) -> usize {
    let lv = log_vert_copy(bm, v, mask);
    let e = log.current_entry;
    log.entries[e].vert_pool.push(lv);
    log.entries[e].vert_pool.len() - 1
}
/// `bm_log_face_alloc`.
fn log_face_alloc(bm: &BMesh, log: &mut BMLog, f: usize) -> usize {
    debug_assert_eq!(bm.face(f).len, 3);
    let first = bm.face(f).l_first;
    let (l0, l1, l2) = (first, bm.loo(first).next, bm.loo(bm.loo(first).next).next);
    let v_ids = [
        bm_log_vert_id_get(log, bm.loo(l0).v),
        bm_log_vert_id_get(log, bm.loo(l1).v),
        bm_log_vert_id_get(log, bm.loo(l2).v),
    ];
    let lf = LogFace {
        v_ids,
        hflag: bm.face(f).head.hflag,
    };
    let e = log.current_entry;
    log.entries[e].face_pool.push(lf);
    log.entries[e].face_pool.len() - 1
}

/// `bm_log_verts_unmake` — kill verts recorded as added.
fn log_verts_unmake(
    bm: &mut BMesh,
    log: &mut BMLog,
    verts: &HashMap<u32, usize>,
    mask_of: &dyn Fn(&BMesh, usize) -> f32,
) {
    for (&id, &lv_i) in verts {
        let v = bm_log_vert_from_id(log, id);
        /* capture final values before deleting */
        let e = log.current_entry;
        log.entries[e].vert_pool[lv_i] = log_vert_copy(bm, v, mask_of(bm, v));
        core::bm_vert_kill(bm, v);
    }
}
/// `bm_log_faces_unmake` — kill faces recorded as added.
fn log_faces_unmake(bm: &mut BMesh, log: &mut BMLog, faces: &HashMap<u32, usize>) {
    for &id in faces.keys() {
        let f = bm_log_vert_from_id(log, id);
        let mut e_tri = [BM_NIL; 3];
        let mut l = bm.face(f).l_first;
        for e in e_tri.iter_mut() {
            *e = bm.loo(l).e;
            l = bm.loo(l).next;
        }
        core::bm_face_kill(bm, f);
        for e in e_tri {
            if bm.edge(e).l == BM_NIL {
                core::bm_edge_kill(bm, e);
            }
        }
    }
}
/// `bm_log_verts_restore`.
fn log_verts_restore(
    bm: &mut BMesh,
    log: &mut BMLog,
    verts: &HashMap<u32, usize>,
    set_mask: &dyn Fn(&mut BMesh, usize, f32),
) {
    for (&id, &lv_i) in verts {
        let e = log.current_entry;
        let lv = log.entries[e].vert_pool[lv_i];
        let v = core::bm_vert_create(bm, lv.position, None, 0);
        set_mask(bm, v, lv.mask);
        bm.vpool[v].head.hflag = lv.hflag;
        bm.vpool[v].no = lv.normal;
        bm_log_elem_id_set(log, crate::BM_VERT, v, id);
    }
}
/// `bm_log_faces_restore`.
fn log_faces_restore(bm: &mut BMesh, log: &mut BMLog, faces: &HashMap<u32, usize>) {
    /* PORT_PENDING: `.sculpt_face_set` CustomData. */
    for (&id, &lf_i) in faces {
        let e = log.current_entry;
        let lf = log.entries[e].face_pool[lf_i];
        let v: [usize; 3] = [
            bm_log_vert_from_id(log, lf.v_ids[0]),
            bm_log_vert_from_id(log, lf.v_ids[1]),
            bm_log_vert_from_id(log, lf.v_ids[2]),
        ];
        let f = core::bm_face_create_verts(bm, &v, None, 0, false, true);
        bm.fpool[f].head.hflag = lf.hflag;
        bm_log_elem_id_set(log, crate::BM_FACE, f, id);
    }
}
/// `bm_log_vert_values_swap`.
fn log_vert_values_swap(
    bm: &mut BMesh,
    log: &mut BMLog,
    verts: &HashMap<u32, usize>,
    mask_of: &dyn Fn(&BMesh, usize) -> f32,
    set_mask: &dyn Fn(&mut BMesh, usize, f32),
) {
    for (&id, &lv_i) in verts {
        let e = log.current_entry;
        let v = bm_log_vert_from_id(log, id);
        std::mem::swap(&mut bm.vpool[v].co, &mut log.entries[e].vert_pool[lv_i].position);
        std::mem::swap(&mut bm.vpool[v].no, &mut log.entries[e].vert_pool[lv_i].normal);
        std::mem::swap(&mut bm.vpool[v].head.hflag, &mut log.entries[e].vert_pool[lv_i].hflag);
        let mask = log.entries[e].vert_pool[lv_i].mask;
        log.entries[e].vert_pool[lv_i].mask = mask_of(bm, v);
        set_mask(bm, v, mask);
    }
}
/// `bm_log_face_values_swap`.
fn log_face_values_swap(bm: &mut BMesh, log: &mut BMLog, faces: &HashMap<u32, usize>) {
    for (&id, &lf_i) in faces {
        let e = log.current_entry;
        let f = bm_log_vert_from_id(log, id);
        std::mem::swap(&mut bm.fpool[f].head.hflag, &mut log.entries[e].face_pool[lf_i].hflag);
    }
}
/// `bm_log_assign_ids` — assign ids to all current verts+faces.
fn log_assign_ids(bm: &BMesh, log: &mut BMLog) {
    for v in 0..bm.totvert as usize {
        let id = log.unused_ids.take_any();
        bm_log_elem_id_set(log, crate::BM_VERT, v, id);
    }
    for f in 0..bm.totface as usize {
        let id = log.unused_ids.take_any();
        bm_log_elem_id_set(log, crate::BM_FACE, f, id);
    }
}
/// Retake all ids referenced by an entry's maps.
fn entry_retake_ids(log: &mut BMLog, entry: usize) {
    for map in [
        &log.entries[entry].deleted_verts,
        &log.entries[entry].deleted_faces,
        &log.entries[entry].added_verts,
        &log.entries[entry].added_faces,
        &log.entries[entry].modified_verts,
        &log.entries[entry].modified_faces,
    ] {
        for &id in map.keys() {
            log.unused_ids.retake(id);
        }
    }
}

/// `BM_log_create`.
pub fn bm_log_create(bm: &BMesh) -> BMLog {
    let mut log = BMLog {
        unused_ids: RangeTree::new(),
        id_to_elem: HashMap::new(),
        elem_to_id: HashMap::new(),
        entries: Vec::new(),
        first: BM_NIL,
        last: BM_NIL,
        current_entry: BM_NIL,
    };
    log_assign_ids(bm, &mut log);
    log
}
/// `BM_log_cleanup_entry` — mark the entry's ids used and clear its maps.
pub fn bm_log_cleanup_entry(log: &mut BMLog, entry: usize) {
    if log.entries[entry].has_log {
        entry_retake_ids(log, entry);
        log.entries[entry].deleted_verts.clear();
        log.entries[entry].deleted_faces.clear();
        log.entries[entry].added_verts.clear();
        log.entries[entry].added_faces.clear();
        log.entries[entry].modified_verts.clear();
        log.entries[entry].modified_faces.clear();
    }
}
/// `BM_log_from_existing_entries_create` — adopt an entry chain.
/// `entries` is the full chain (first..last in order); `current` is its index.
pub fn bm_log_from_existing_entries_create(
    bm: &BMesh,
    mut entries: Vec<LogEntry>,
    current: usize,
) -> BMLog {
    let mut log = bm_log_create(bm);
    if !entries.is_empty() {
        let n = entries.len();
        for (i, e) in entries.iter_mut().enumerate() {
            e.prev = if i == 0 { BM_NIL } else { i - 1 };
            e.next = if i + 1 == n { BM_NIL } else { i + 1 };
            e.has_log = true;
        }
        log.entries = entries;
        log.first = 0;
        log.last = log.entries.len() - 1;
        log.current_entry = if current != BM_NIL { current } else { BM_NIL };
        for e in 0..log.entries.len() {
            entry_retake_ids(&mut log, e);
        }
    } else {
        log.current_entry = BM_NIL;
    }
    log
}
/// `BM_log_free` — detach entries (their pools/ids live on).
pub fn bm_log_free(log: BMLog) -> Vec<LogEntry> {
    let mut entries = log.entries;
    for e in &mut entries {
        e.has_log = false;
    }
    entries
}
/// `BM_log_entry_add` — append a new entry and make it current.
pub fn bm_log_entry_add(log: &mut BMLog) -> usize {
    let mut entry = LogEntry::new();
    entry.has_log = true;
    entry.prev = log.last;
    let i = log.entries.len();
    if log.last != BM_NIL {
        log.entries[log.last].next = i;
    }
    if log.first == BM_NIL {
        log.first = i;
    }
    log.entries.push(entry);
    log.last = i;
    log.current_entry = i;
    i
}
/// `BM_log_entry_drop` — unlink entry from its chain.
pub fn bm_log_entry_drop(log: &mut BMLog, entry: usize) {
    if !log.entries[entry].has_log {
        /* unlink from detached chain */
        let (prev, next) = (log.entries[entry].prev, log.entries[entry].next);
        if prev != BM_NIL && next != BM_NIL {
            /* C asserts !(prev && next) for detached entries */
        } else if prev != BM_NIL {
            log.entries[prev].next = BM_NIL;
        } else if next != BM_NIL {
            log.entries[next].prev = BM_NIL;
        }
    } else {
        let (prev, next) = (log.entries[entry].prev, log.entries[entry].next);
        if prev != BM_NIL {
            log.entries[prev].next = next;
        } else {
            log.first = next;
        }
        if next != BM_NIL {
            log.entries[next].prev = prev;
        } else {
            log.last = prev;
        }
        if log.current_entry == entry {
            log.current_entry = prev;
        }
    }
}

/// `BM_log_undo`.
pub fn bm_log_undo(
    bm: &mut BMesh,
    log: &mut BMLog,
    mask_of: &dyn Fn(&BMesh, usize) -> f32,
    set_mask: &dyn Fn(&mut BMesh, usize, f32),
) {
    let entry = log.current_entry;
    if entry != BM_NIL {
        log.current_entry = log.entries[entry].prev;
        let (added_f, added_v, deleted_v, deleted_f, mod_v, mod_f) = (
            log.entries[entry].added_faces.clone(),
            log.entries[entry].added_verts.clone(),
            log.entries[entry].deleted_verts.clone(),
            log.entries[entry].deleted_faces.clone(),
            log.entries[entry].modified_verts.clone(),
            log.entries[entry].modified_faces.clone(),
        );
        log_faces_unmake(bm, log, &added_f);
        log_verts_unmake(bm, log, &added_v, mask_of);
        log_verts_restore(bm, log, &deleted_v, set_mask);
        log_faces_restore(bm, log, &deleted_f);
        log_vert_values_swap(bm, log, &mod_v, mask_of, set_mask);
        log_face_values_swap(bm, log, &mod_f);
    }
}
/// `BM_log_redo`.
pub fn bm_log_redo(
    bm: &mut BMesh,
    log: &mut BMLog,
    mask_of: &dyn Fn(&BMesh, usize) -> f32,
    set_mask: &dyn Fn(&mut BMesh, usize, f32),
) {
    let mut entry = log.current_entry;
    if entry == BM_NIL {
        entry = log.first;
    } else if log.entries[entry].next != BM_NIL {
        entry = log.entries[entry].next;
    } else {
        return;
    }
    log.current_entry = entry;
    if entry != BM_NIL {
        let (added_f, added_v, deleted_v, deleted_f, mod_v, mod_f) = (
            log.entries[entry].added_faces.clone(),
            log.entries[entry].added_verts.clone(),
            log.entries[entry].deleted_verts.clone(),
            log.entries[entry].deleted_faces.clone(),
            log.entries[entry].modified_verts.clone(),
            log.entries[entry].modified_faces.clone(),
        );
        log_faces_unmake(bm, log, &deleted_f);
        log_verts_unmake(bm, log, &deleted_v, mask_of);
        log_verts_restore(bm, log, &added_v, set_mask);
        log_faces_restore(bm, log, &added_f);
        log_vert_values_swap(bm, log, &mod_v, mask_of, set_mask);
        log_face_values_swap(bm, log, &mod_f);
    }
}

/// `BM_log_vert_before_modified`.
pub fn bm_log_vert_before_modified(
    bm: &BMesh,
    log: &mut BMLog,
    v: usize,
    mask_of: &dyn Fn(&BMesh, usize) -> f32,
) {
    let entry = log.current_entry;
    let v_id = bm_log_vert_id_get(log, v);
    if log.entries[entry].added_verts.contains_key(&v_id) {
        let i = log.entries[entry].added_verts[&v_id];
        log.entries[entry].vert_pool[i] = log_vert_copy(bm, v, mask_of(bm, v));
    } else if !log.entries[entry].modified_verts.contains_key(&v_id) {
        let i = log_vert_alloc(bm, log, v, mask_of(bm, v));
        log.entries[entry].modified_verts.insert(v_id, i);
    }
}
/// `BM_log_vert_added`.
pub fn bm_log_vert_added(
    bm: &BMesh,
    log: &mut BMLog,
    v: usize,
    mask_of: &dyn Fn(&BMesh, usize) -> f32,
) {
    let v_id = log.unused_ids.take_any();
    bm_log_elem_id_set(log, crate::BM_VERT, v, v_id);
    let i = log_vert_alloc(bm, log, v, mask_of(bm, v));
    let e = log.current_entry;
    log.entries[e].added_verts.insert(v_id, i);
}
/// `BM_log_face_modified`.
pub fn bm_log_face_modified(bm: &BMesh, log: &mut BMLog, f: usize) {
    let f_id = bm_log_face_id_get(log, f);
    let i = log_face_alloc(bm, log, f);
    let e = log.current_entry;
    log.entries[e].modified_faces.insert(f_id, i);
}
/// `BM_log_face_added`.
pub fn bm_log_face_added(bm: &BMesh, log: &mut BMLog, f: usize) {
    debug_assert_eq!(bm.face(f).len, 3);
    let f_id = log.unused_ids.take_any();
    bm_log_elem_id_set(log, crate::BM_FACE, f, f_id);
    let i = log_face_alloc(bm, log, f);
    let e = log.current_entry;
    log.entries[e].added_faces.insert(f_id, i);
}
/// `BM_log_vert_removed`.
pub fn bm_log_vert_removed(
    bm: &BMesh,
    log: &mut BMLog,
    v: usize,
    mask_of: &dyn Fn(&BMesh, usize) -> f32,
) {
    let entry = log.current_entry;
    let v_id = bm_log_vert_id_get(log, v);
    if log.entries[entry].added_verts.remove(&v_id).is_some() {
        log.unused_ids.release(v_id);
    } else {
        let i = log_vert_alloc(bm, log, v, mask_of(bm, v));
        log.entries[entry].deleted_verts.insert(v_id, i);
        if let Some(&i_mod) = log.entries[entry].modified_verts.get(&v_id) {
            log.entries[entry].vert_pool[i] = log.entries[entry].vert_pool[i_mod];
            log.entries[entry].modified_verts.remove(&v_id);
        }
    }
}
/// `BM_log_face_removed`.
pub fn bm_log_face_removed(bm: &BMesh, log: &mut BMLog, f: usize) {
    let entry = log.current_entry;
    let f_id = bm_log_face_id_get(log, f);
    if log.entries[entry].added_faces.remove(&f_id).is_some() {
        log.unused_ids.release(f_id);
    } else {
        let i = log_face_alloc(bm, log, f);
        log.entries[entry].deleted_faces.insert(f_id, i);
        if let Some(&i_mod) = log.entries[entry].modified_faces.get(&f_id) {
            log.entries[entry].face_pool[i] = log.entries[entry].face_pool[i_mod];
            log.entries[entry].modified_faces.remove(&f_id);
        }
    }
}
