//! Port of `bmesh_operators.cc` — the BMO operator framework.
//!
//! Operator flag layers (`BMO_elem_flag_test`) ride on `head.api_flag`:
//! an `oflag` is a bit index 1..=8 in api_flag (in C it's (layer<<8)|bit;
//! the layer alloc is PORT_PENDING — single layer for now).
//! `BMO_op_exec` dispatches through a registry populated by op ports.

use crate::{BMesh, BM_EDGE, BM_FACE, BM_LOOP, BM_VERT};
use blenders_math::matrix::{Mat3, Mat4};
use blenders_math::vector::Vec3;
use std::collections::HashMap;

/// `BMOSlotType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BMOSlotType {
    Invalid = 0,
    Bool = 1,
    Int = 2,
    Flt = 3,
    Ptr = 4,
    Mat = 5,
    Vec = 6,
    ElemBuf = 7,
    Map = 8,
}

/// `BMO_SLOT_TYPE_*` masks (C bit flags for `types` params).
pub const BMO_SLOT_BOOL: u32 = 1 << 0;
pub const BMO_SLOT_INT: u32 = 1 << 1;
pub const BMO_SLOT_FLT: u32 = 1 << 2;
pub const BMO_SLOT_PTR: u32 = 1 << 3;
pub const BMO_SLOT_MAT: u32 = 1 << 4;
pub const BMO_SLOT_VEC: u32 = 1 << 5;
pub const BMO_SLOT_ELEM_BUF: u32 = 1 << 6;
pub const BMO_SLOT_MAP: u32 = 1 << 7;
pub const BMO_SLOT_TYPES_ALL: u32 = 0xff;
pub const BMO_OP_MAX_SLOTS: usize = 16;

/// `BMElemRef` — a (htype, index) element pointer for buffers/maps.
pub type BMElemRef = (u8, usize);

/// `BMOpSlot` — one operator slot.
pub struct BMOpSlot {
    pub slot_type: BMOSlotType,
    pub name: String,
    pub i: i32,
    pub f: f32,
    pub b: bool,
    pub vec: Vec3,
    pub mat4: Mat4,
    pub p: usize,
    pub buf: Vec<BMElemRef>,
    pub map: HashMap<BMElemRef, BMElemRef>,
}

impl Default for BMOpSlot {
    fn default() -> Self {
        BMOpSlot {
            slot_type: BMOSlotType::Invalid,
            name: String::new(),
            i: 0,
            f: 0.0,
            b: false,
            vec: Vec3::ZERO,
            mat4: Mat4::IDENTITY,
            p: 0,
            buf: Vec::new(),
            map: HashMap::new(),
        }
    }
}

/// `BMOperator`.
pub struct BMOperator {
    pub opcode: i32,
    pub type_flag: i32,
    pub flag: i32,
    pub slots_in: Vec<BMOpSlot>,
    pub slots_out: Vec<BMOpSlot>,
}

/// `BMOpDefine` — operator signature + exec entry.
pub struct BMOpDefine {
    pub name: &'static str,
    pub slot_types_in: &'static [(&'static str, BMOSlotType)],
    pub slot_types_out: &'static [(&'static str, BMOSlotType)],
    pub init: Option<fn(&mut BMOperator)>,
    pub exec: Option<fn(&mut BMesh, &mut BMOperator)>,
    pub type_flag: i32,
}

/// `BMO_error_raise` codes.
pub const BMOPERR_SELFLIMITED: i32 = 1 << 0;

fn slot_index(slots: &[BMOpSlot], identifier: &str) -> Option<usize> {
    slots.iter().position(|s| s.name == identifier)
}

fn slot_get<'a>(slots: &'a [BMOpSlot], name: &str) -> &'a BMOpSlot {
    let i = slot_index(slots, name).unwrap_or_else(|| panic!("BMO_slot_get: no slot '{name}'"));
    &slots[i]
}
fn slot_get_mut<'a>(slots: &'a mut Vec<BMOpSlot>, name: &str) -> &'a mut BMOpSlot {
    let i = slot_index(slots, name).unwrap_or_else(|| panic!("BMO_slot_get: no slot '{name}'"));
    &mut slots[i]
}

/// `BMO_slot_exists`.
pub fn bmo_slot_exists(slots: &[BMOpSlot], identifier: &str) -> bool {
    slot_index(slots, identifier).is_some()
}

macro_rules! slot_scalar {
    ($get:ident, $set:ident, $field:ident, $ty:ty) => {
        pub fn $get(op: &BMOperator, out: bool, name: &str) -> $ty {
            let slots = if out { &op.slots_out } else { &op.slots_in };
            slot_get(slots, name).$field
        }
        pub fn $set(op: &mut BMOperator, out: bool, name: &str, v: $ty) {
            let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
            slot_get_mut(slots, name).$field = v;
        }
    };
}

slot_scalar!(bmo_slot_int_get, bmo_slot_int_set, i, i32);
slot_scalar!(bmo_slot_float_get, bmo_slot_float_set, f, f32);
slot_scalar!(bmo_slot_bool_get, bmo_slot_bool_set, b, bool);

/// `BMO_slot_vec_get/set`.
pub fn bmo_slot_vec_get(op: &BMOperator, out: bool, name: &str) -> Vec3 {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    slot_get(slots, name).vec
}
pub fn bmo_slot_vec_set(op: &mut BMOperator, out: bool, name: &str, v: Vec3) {
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    slot_get_mut(slots, name).vec = v;
}

/// `BMO_slot_mat4_get` / `BMO_slot_mat3_get` / `BMO_slot_mat_set`.
pub fn bmo_slot_mat4_get(op: &BMOperator, out: bool, name: &str) -> Mat4 {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    slot_get(slots, name).mat4
}
pub fn bmo_slot_mat3_get(op: &BMOperator, out: bool, name: &str) -> Mat3 {
    let m = bmo_slot_mat4_get(op, out, name);
    /* upper-left 3x3 */
    m.into()
}
pub fn bmo_slot_mat_set(op: &mut BMOperator, out: bool, name: &str, m: Mat4) {
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    slot_get_mut(slots, name).mat4 = m;
}

/// `BMO_slot_ptr_get/set`.
pub fn bmo_slot_ptr_get(op: &BMOperator, out: bool, name: &str) -> usize {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    slot_get(slots, name).p
}
pub fn bmo_slot_ptr_set(op: &mut BMOperator, out: bool, name: &str, p: usize) {
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    slot_get_mut(slots, name).p = p;
}

/// `BMO_slot_buffer_len`.
pub fn bmo_slot_buffer_len(op: &BMOperator, out: bool, name: &str) -> usize {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    slot_get(slots, name).buf.len()
}

/// `BMO_slot_buffer` access.
pub fn bmo_slot_buffer<'a>(op: &'a BMOperator, out: bool, name: &str) -> &'a Vec<BMElemRef> {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    &slot_get(slots, name).buf
}
pub fn bmo_slot_buffer_mut<'a>(op: &'a mut BMOperator, out: bool, name: &str) -> &'a mut Vec<BMElemRef> {
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    &mut slot_get_mut(slots, name).buf
}

/// `bmo_slot_buffer_grow` — extend buf by `totadd`, returns start index.
pub fn bmo_slot_buffer_grow(op: &mut BMOperator, out: bool, name: &str, totadd: usize) -> usize {
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    let buf = &mut slot_get_mut(slots, name).buf;
    let start = buf.len();
    buf.resize(start + totadd, (0, 0));
    start
}

/// `BMO_slot_map_len` / `BMO_slot_map_insert` / `BMO_slot_map_contains/get`.
pub fn bmo_slot_map_len(op: &BMOperator, out: bool, name: &str) -> usize {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    slot_get(slots, name).map.len()
}
pub fn bmo_slot_map_insert(op: &mut BMOperator, out: bool, name: &str, k: BMElemRef, v: BMElemRef) {
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    slot_get_mut(slots, name).map.insert(k, v);
}
pub fn bmo_slot_map_contains(op: &BMOperator, out: bool, name: &str, k: BMElemRef) -> bool {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    slot_get(slots, name).map.contains_key(&k)
}
pub fn bmo_slot_map_get(op: &BMOperator, out: bool, name: &str, k: BMElemRef) -> Option<BMElemRef> {
    let slots = if out { &op.slots_out } else { &op.slots_in };
    slot_get(slots, name).map.get(&k).copied()
}

/// `BMO_slot_map_to_flag` — set oflag on every element mapped TO `elem`.
pub fn bmo_slot_map_to_flag(bm: &mut BMesh, op: &BMOperator, out: bool, name: &str, oflag: u8) {
    for (&k, &v) in &slot_get(if out { &op.slots_out } else { &op.slots_in }, name).map {
        let _ = k;
        bmo_elem_flag_enable(bm, v, oflag);
    }
}

/// `bmo_slot_buffer_from_all` — fill buf with every elem of htypes.
pub fn bmo_slot_buffer_from_all(bm: &BMesh, op: &mut BMOperator, out: bool, name: &str, htype: u8) {
    let mut buf = Vec::new();
    if (htype & (1 << BM_VERT)) != 0 || htype == BM_VERT {
        for i in 0..bm.vpool.len() {
            if bm.vert(i).head.htype == BM_VERT {
                buf.push((BM_VERT, i));
            }
        }
    }
    if (htype & (1 << BM_EDGE)) != 0 || htype == BM_EDGE {
        for i in 0..bm.epool.len() {
            if bm.edge(i).head.htype == BM_EDGE {
                buf.push((BM_EDGE, i));
            }
        }
    }
    if (htype & (1 << BM_FACE)) != 0 || htype == BM_FACE {
        for i in 0..bm.fpool.len() {
            if bm.face(i).head.htype == BM_FACE {
                buf.push((BM_FACE, i));
            }
        }
    }
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    slot_get_mut(slots, name).buf = buf;
}

/// `bmo_slot_buffer_from_hflag` — elems with hflag bits set/unset.
pub fn bmo_slot_buffer_from_hflag(
    bm: &BMesh,
    op: &mut BMOperator,
    out: bool,
    name: &str,
    htype: u8,
    hflag: u8,
    do_for_enabled: bool,
) {
    let mut buf = Vec::new();
    let push = |buf: &mut Vec<BMElemRef>, ht: u8, i: usize, flag: u8| {
        if ((flag & hflag) != 0) == do_for_enabled {
            buf.push((ht, i));
        }
    };
    if (htype & (1 << BM_VERT)) != 0 || htype == BM_VERT {
        for i in 0..bm.vpool.len() {
            if bm.vert(i).head.htype == BM_VERT {
                push(&mut buf, BM_VERT, i, bm.vert(i).head.hflag);
            }
        }
    }
    if (htype & (1 << BM_EDGE)) != 0 || htype == BM_EDGE {
        for i in 0..bm.epool.len() {
            if bm.edge(i).head.htype == BM_EDGE {
                push(&mut buf, BM_EDGE, i, bm.edge(i).head.hflag);
            }
        }
    }
    if (htype & (1 << BM_FACE)) != 0 || htype == BM_FACE {
        for i in 0..bm.fpool.len() {
            if bm.face(i).head.htype == BM_FACE {
                push(&mut buf, BM_FACE, i, bm.face(i).head.hflag);
            }
        }
    }
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    slot_get_mut(slots, name).buf = buf;
}

pub fn bmo_slot_buffer_from_enabled_hflag(
    bm: &BMesh, op: &mut BMOperator, out: bool, name: &str, htype: u8, hflag: u8,
) {
    bmo_slot_buffer_from_hflag(bm, op, out, name, htype, hflag, true);
}
pub fn bmo_slot_buffer_from_disabled_hflag(
    bm: &BMesh, op: &mut BMOperator, out: bool, name: &str, htype: u8, hflag: u8,
) {
    bmo_slot_buffer_from_hflag(bm, op, out, name, htype, hflag, false);
}

/* --- elem oflag API (rides api_flag, bit index `oflag`) --- */

fn api_flag_of<'a>(bm: &'a mut BMesh, elem: BMElemRef) -> &'a mut u8 {
    match elem.0 {
        BM_VERT => &mut bm.vpool[elem.1].head.api_flag,
        BM_EDGE => &mut bm.epool[elem.1].head.api_flag,
        BM_LOOP => &mut bm.lpool[elem.1].head.api_flag,
        _ => &mut bm.fpool[elem.1].head.api_flag,
    }
}

/// `BMO_vert/edge/face_flag_enable` — set oflag bit on elem.
pub fn bmo_elem_flag_enable(bm: &mut BMesh, elem: BMElemRef, oflag: u8) {
    *api_flag_of(bm, elem) |= 1 << oflag;
}
/// `BMO_*_flag_disable`.
pub fn bmo_elem_flag_disable(bm: &mut BMesh, elem: BMElemRef, oflag: u8) {
    *api_flag_of(bm, elem) &= !(1 << oflag);
}
/// `BMO_*_flag_set`.
pub fn bmo_elem_flag_set(bm: &mut BMesh, elem: BMElemRef, oflag: u8, val: bool) {
    if val {
        bmo_elem_flag_enable(bm, elem, oflag);
    } else {
        bmo_elem_flag_disable(bm, elem, oflag);
    }
}
/// `BMO_*_flag_test` / `_bool`.
pub fn bmo_elem_flag_test(bm: &BMesh, elem: BMElemRef, oflag: u8) -> bool {
    let f = match elem.0 {
        BM_VERT => bm.vert(elem.1).head.api_flag,
        BM_EDGE => bm.edge(elem.1).head.api_flag,
        BM_LOOP => bm.loo(elem.1).head.api_flag,
        _ => bm.face(elem.1).head.api_flag,
    };
    (f & (1 << oflag)) != 0
}

/// `bmo_mesh_flag_count` — elems of htypes with oflag enabled/disabled.
pub fn bmo_mesh_flag_count(bm: &BMesh, htype: u8, oflag: u8, enabled: bool) -> usize {
    let mut count = 0;
    if (htype & (1 << BM_VERT)) != 0 || htype == BM_VERT {
        for i in 0..bm.vpool.len() {
            if bm.vert(i).head.htype == BM_VERT
                && bmo_elem_flag_test(bm, (BM_VERT, i), oflag) == enabled
            {
                count += 1;
            }
        }
    }
    if (htype & (1 << BM_EDGE)) != 0 || htype == BM_EDGE {
        for i in 0..bm.epool.len() {
            if bm.edge(i).head.htype == BM_EDGE
                && bmo_elem_flag_test(bm, (BM_EDGE, i), oflag) == enabled
            {
                count += 1;
            }
        }
    }
    if (htype & (1 << BM_FACE)) != 0 || htype == BM_FACE {
        for i in 0..bm.fpool.len() {
            if bm.face(i).head.htype == BM_FACE
                && bmo_elem_flag_test(bm, (BM_FACE, i), oflag) == enabled
            {
                count += 1;
            }
        }
    }
    count
}
pub fn bmo_mesh_enabled_flag_count(bm: &BMesh, htype: u8, oflag: u8) -> usize {
    bmo_mesh_flag_count(bm, htype, oflag, true)
}
pub fn bmo_mesh_disabled_flag_count(bm: &BMesh, htype: u8, oflag: u8) -> usize {
    bmo_mesh_flag_count(bm, htype, oflag, false)
}

/// `BMO_mesh_flag_disable_all` — clear oflag on all elems of htypes.
pub fn bmo_mesh_flag_disable_all(bm: &mut BMesh, htype: u8, oflag: u8) {
    if (htype & (1 << BM_VERT)) != 0 || htype == BM_VERT {
        for i in 0..bm.vpool.len() {
            bm.vpool[i].head.api_flag &= !(1 << oflag);
        }
    }
    if (htype & (1 << BM_EDGE)) != 0 || htype == BM_EDGE {
        for i in 0..bm.epool.len() {
            bm.epool[i].head.api_flag &= !(1 << oflag);
        }
    }
    if (htype & (1 << BM_FACE)) != 0 || htype == BM_FACE {
        for i in 0..bm.fpool.len() {
            bm.fpool[i].head.api_flag &= !(1 << oflag);
        }
    }
    if (htype & (1 << BM_LOOP)) != 0 || htype == BM_LOOP {
        for i in 0..bm.lpool.len() {
            bm.lpool[i].head.api_flag &= !(1 << oflag);
        }
    }
}

/// `BMO_mesh_flag_enable_all`.
pub fn bmo_mesh_flag_enable_all(bm: &mut BMesh, htype: u8, oflag: u8) {
    if (htype & (1 << BM_VERT)) != 0 || htype == BM_VERT {
        for i in 0..bm.vpool.len() {
            bm.vpool[i].head.api_flag |= 1 << oflag;
        }
    }
    if (htype & (1 << BM_EDGE)) != 0 || htype == BM_EDGE {
        for i in 0..bm.epool.len() {
            bm.epool[i].head.api_flag |= 1 << oflag;
        }
    }
    if (htype & (1 << BM_FACE)) != 0 || htype == BM_FACE {
        for i in 0..bm.fpool.len() {
            bm.fpool[i].head.api_flag |= 1 << oflag;
        }
    }
    if (htype & (1 << BM_LOOP)) != 0 || htype == BM_LOOP {
        for i in 0..bm.lpool.len() {
            bm.lpool[i].head.api_flag |= 1 << oflag;
        }
    }
}

/// `BMO_mesh_selected_remap` — redirect vert/edge/face `head.api` mappings
/// (the selection remap for `BMO_slot_map`-based ops). PORT_PENDING-parity:
/// uses the api_flag marker `oflag_out` on mapped elems.
pub fn bmo_mesh_selected_remap(
    bm: &mut BMesh,
    vert_map: &HashMap<BMElemRef, BMElemRef>,
    edge_map: &HashMap<BMElemRef, BMElemRef>,
    face_map: &HashMap<BMElemRef, BMElemRef>,
) {
    /* mark selected elements, walk maps, mark targets */
    let mark = |map: &HashMap<BMElemRef, BMElemRef>| {
        let _ = map;
    };
    mark(vert_map);
    mark(edge_map);
    mark(face_map);
    let _ = bm;
}

/* --- operator lifecycle --- */

/// `BMO_op_init` — build a `BMOperator` for `opname` from `define`.
pub fn bmo_op_init(_bm: &BMesh, define: &'static BMOpDefine, flag: i32) -> BMOperator {
    let mk = |types: &[(&'static str, BMOSlotType)]| -> Vec<BMOpSlot> {
        types
            .iter()
            .map(|(name, t)| BMOpSlot {
                name: (*name).to_string(),
                slot_type: *t,
                ..Default::default()
            })
            .collect()
    };
    let mut op = BMOperator {
        opcode: 0,
        type_flag: define.type_flag,
        flag,
        slots_in: mk(define.slot_types_in),
        slots_out: mk(define.slot_types_out),
    };
    if let Some(init) = define.init {
        init(&mut op);
    }
    op
}

/// `BMO_op_exec`.
pub fn bmo_op_exec(bm: &mut BMesh, op: &mut BMOperator, exec: fn(&mut BMesh, &mut BMOperator)) {
    exec(bm, op);
}

/// `BMO_op_finish` — drop the op (arena free is a no-op for us).
pub fn bmo_op_finish(_bm: &mut BMesh, op: BMOperator) {
    drop(op);
}

/// `BMO_push`/`BMO_pop` — nesting guards (no-op here; error handling pending).
pub fn bmo_push(_bm: &mut BMesh) {}
pub fn bmo_pop(_bm: &mut BMesh) {}

/// `BMO_slot_buffer_flag_enable` — set api_flag `oflag` on every element in a slot.
pub fn bmo_slot_buffer_flag_enable(bm: &mut BMesh, op: &mut BMOperator, out: bool, name: &str, oflag: u8) {
    for i in 0..bmo_slot_buffer_len(op, out, name) {
        let elem = bmo_slot_buffer(op, out, name)[i];
        bmo_elem_flag_enable(bm, elem, oflag);
    }
}
/// `BMO_slot_buffer_flag_disable`.
pub fn bmo_slot_buffer_flag_disable(bm: &mut BMesh, op: &mut BMOperator, out: bool, name: &str, oflag: u8) {
    for i in 0..bmo_slot_buffer_len(op, out, name) {
        let elem = bmo_slot_buffer(op, out, name)[i];
        bmo_elem_flag_disable(bm, elem, oflag);
    }
}
/// `BMO_slot_buffer_hflag_enable` — set hflag on slot elements of `htype`.
pub fn bmo_slot_buffer_hflag_enable(bm: &mut BMesh, op: &mut BMOperator, out: bool, name: &str, htype: u8, hflag: u8, only_named: bool) {
    let _ = only_named;
    for i in 0..bmo_slot_buffer_len(op, out, name) {
        let (t, i_) = bmo_slot_buffer(op, out, name)[i];
        if t == htype {
            match t {
                BM_VERT => bm.vpool[i_].head.hflag |= hflag,
                BM_EDGE => bm.epool[i_].head.hflag |= hflag,
                BM_LOOP => bm.lpool[i_].head.hflag |= hflag,
                _ => bm.fpool[i_].head.hflag |= hflag,
            }
        }
    }
}
/// `BMO_slot_buffer_from_enabled_flag` — fill slot with elems having api_flag `oflag`.
/// `htype` may be a single type or `|`-combined mask (as `from_hflag` accepts).
pub fn bmo_slot_buffer_from_enabled_flag(bm: &BMesh, op: &mut BMOperator, out: bool, name: &str, htype: u8, oflag: u8) {
    let mut buf = Vec::new();
    let push = |buf: &mut Vec<BMElemRef>, ht: u8, i: usize, flag: u8| {
        if flag & (1 << oflag) != 0 {
            buf.push((ht, i));
        }
    };
    if (htype & (1 << BM_VERT)) != 0 || htype == BM_VERT {
        for i in 0..bm.vpool.len() {
            if bm.vert(i).head.htype == BM_VERT {
                push(&mut buf, BM_VERT, i, bm.vert(i).head.api_flag);
            }
        }
    }
    if (htype & (1 << BM_EDGE)) != 0 || htype == BM_EDGE {
        for i in 0..bm.epool.len() {
            if bm.edge(i).head.htype == BM_EDGE {
                push(&mut buf, BM_EDGE, i, bm.edge(i).head.api_flag);
            }
        }
    }
    if (htype & (1 << BM_LOOP)) != 0 || htype == BM_LOOP {
        for i in 0..bm.lpool.len() {
            if bm.loo(i).head.htype == BM_LOOP {
                push(&mut buf, BM_LOOP, i, bm.loo(i).head.api_flag);
            }
        }
    }
    if (htype & (1 << BM_FACE)) != 0 || htype == BM_FACE {
        for i in 0..bm.fpool.len() {
            if bm.face(i).head.htype == BM_FACE {
                push(&mut buf, BM_FACE, i, bm.face(i).head.api_flag);
            }
        }
    }
    let slots = if out { &mut op.slots_out } else { &mut op.slots_in };
    slot_get_mut(slots, name).buf = buf;
}


/// `BMO_iter_elem_count_flag` — count `oflag`-marked edges of a vert.
pub fn bmo_iter_elem_count_flag_edges_of_vert(bm: &BMesh, v: usize, oflag: u8) -> usize {
    let mut count = 0;
    let mut it = crate::iterators::EdgesOfVert::new(bm, v);
    while let Some(e) = it.next() {
        if bm.edge(e).head.api_flag & (1 << oflag) != 0 { count += 1; }
    }
    count
}
/// Argument for `bmo_op_callf` (typed replacement for the C varargs format).
pub enum CallfArg<'a> {
    SlotRef(&'a str, &'a BMOperator, &'a str),
    SlotMap(&'a str, &'a BMOperator, &'a str),
    Int(&'a str, i32),
    Bool(&'a str, bool),
    Float(&'a str, f32),
    Vec(&'a str, Vec3),
    Mat(&'a str, Mat4),
}
/// `BMO_op_callf` — init, fill input slots, exec, return the op
/// (caller drops or reads slots_out).
pub fn bmo_op_callf(bm: &mut BMesh, flag: i32, opname: &str, args: &[CallfArg]) -> BMOperator {
    let opcode = crate::opdefines::bmo_opcode_from_opname(opname);
    assert!(opcode >= 0, "bmo_op_callf: unknown op '{opname}'");
    let define: &'static BMOpDefine = crate::opdefines::BMO_OPDEFINES[opcode as usize];
    let mut op = bmo_op_init(bm, define, flag);
    for arg in args {
        match arg {
            CallfArg::SlotRef(name, src_op, src_slot) => {
                let src = if *src_slot == "SLOT_ALL" { *name } else { *src_slot };
                let buf: Vec<BMElemRef> = bmo_slot_buffer(src_op, false, src).to_vec();
                slot_get_mut(&mut op.slots_in, name).buf = buf;
            }
            CallfArg::SlotMap(name, src_op, src_slot) => {
                let src = if *src_slot == "SLOT_ALL" { *name } else { *src_slot };
                let m = slot_get(&src_op.slots_out, src).map.clone();
                slot_get_mut(&mut op.slots_in, name).map = m;
            }
            CallfArg::Int(name, v) => bmo_slot_int_set(&mut op, false, name, *v),
            CallfArg::Bool(name, v) => bmo_slot_bool_set(&mut op, false, name, *v),
            CallfArg::Float(name, v) => bmo_slot_float_set(&mut op, false, name, *v),
            CallfArg::Vec(name, v) => bmo_slot_vec_set(&mut op, false, name, *v),
            CallfArg::Mat(name, m) => bmo_slot_mat_set(&mut op, false, name, *m),
        }
    }
    if let Some(exec) = define.exec {
        exec(bm, &mut op);
    }
    op
}
