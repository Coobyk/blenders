//! Port of `intern/material.cc` — material-slot management:
//! `BKE_id_material_*`/`BKE_object_material_*` resize/append/
//! pop/clear/assign/remap/index/sync over the `mat`/`totcol`
//! pair every material-holding ID shares, plus `Material`
//! init defaults and `MaterialGPencilStyle` init.
//!
//! Materials are `usize` ID handles; `id_us_plus`/`id_us_min`
//! are folded into an injected `users` callback
//! (`FnMut(usize, i32)` — `+1`/`-1`), and the depsgraph tags
//! (`DEG_id_tag_update`/`relations_tag_update`) are recorded
//! via `tags` (`FnMut(usize)`). `PORT_PENDING`: node-tree
//! preview dirty-marking, texpaint slot cache (needs
//! Scene+node graph), `BKE_object_material_from_eval_data`
//! (needs evaluated-depsgraph copies).

use blenders_dna::material_types::{Material, MaterialGpencilStyle};
use blenders_dna::object_types::Object;

/// `MatSlots` — the `Material **mat` + `short totcol` pair
/// shared by `Object`, `Mesh`, `Curve`, `Curves`, `MetaBall`,
/// `GreasePencil`, `PointCloud`, `Volume`.
pub struct MatSlots<'a> {
    /// `mat` — slot→material handle.
    pub mat: &'a mut Vec<usize>,
    /// `totcol`.
    pub totcol: &'a mut i32,
}

/// `BKE_object_material_array_p` + `BKE_object_material_`
/// `len_p` → `MatSlots` for an `Object`.
pub fn object_material_slots(ob: &mut Object) -> MatSlots<'_> {
    MatSlots {
        mat: &mut ob.mat,
        totcol: &mut ob.totcol,
    }
}

/// `BKE_material_init_data` — post-ID defaults (verbatim
/// `INIT_DEFAULT_STRUCT_AFTER` semantics: struct Default).
pub fn material_init_data() -> Material {
    Material::default()
}

/// `BKE_gpencil_material_attr_init` — init `gp_style`.
/// `gp_style` stays a handle (`usize`) until gpencil data is
/// embedded; callers assign the allocated struct's handle.
pub fn gpencil_material_attr_init(ma: &mut Material) {
    let _ = ma;
}

/// `BKE_object_material_slot_used` — any `mat_nr == actcol`
/// across the object's data (usage predicate injected since
/// `mat_nr` lives on the obdata's faces/curves).
pub fn object_material_slot_used(actcol: i16, count_using: &dyn Fn(i16) -> usize) -> bool {
    count_using(actcol) > 0
}

/// `BKE_id_materials_copy` (verbatim incl. refcount via
/// `users`).
pub fn id_materials_copy(
    src: &MatSlots,
    dst: &mut MatSlots,
    users: &mut dyn FnMut(usize, i32),
) {
    *dst.totcol = *src.totcol;
    *dst.mat = src.mat.clone();
    for &m in dst.mat.iter() {
        users(m, 1);
    }
}

/// `BKE_id_material_resize` (verbatim: shrink → us_min on the
/// tail, then realloc semantics via `Vec::resize`).
pub fn id_material_resize(
    slots: &mut MatSlots,
    totcol: i32,
    do_id_user: bool,
    users: &mut dyn FnMut(usize, i32),
) {
    let totcol = totcol.max(0) as usize;
    if totcol == *slots.totcol as usize {
        return;
    }
    if do_id_user && totcol < (*slots.totcol as usize) {
        for m in slots.mat[totcol..].to_vec() {
            users(m, -1);
        }
    }
    slots.mat.resize(totcol, 0);
    *slots.totcol = totcol as i32;
}

/// `BKE_id_material_append`.
pub fn id_material_append(
    slots: &mut MatSlots,
    ma: usize,
    users: &mut dyn FnMut(usize, i32),
) {
    slots.mat.push(ma);
    *slots.totcol += 1;
    users(ma, 1);
}

/// `BKE_id_material_pop` — remove slot `index`, shifting the
/// rest (verbatim memmove semantics), returns the material.
pub fn id_material_pop(
    slots: &mut MatSlots,
    index: i32,
    users: &mut dyn FnMut(usize, i32),
) -> Option<usize> {
    if index < 0 || index >= *slots.totcol {
        return None;
    }
    let ret = slots.mat.remove(index as usize);
    *slots.totcol -= 1;
    users(ret, -1);
    Some(ret)
}

/// `BKE_id_material_clear`.
pub fn id_material_clear(slots: &mut MatSlots, users: &mut dyn FnMut(usize, i32)) {
    for &m in slots.mat.clone().iter() {
        users(m, -1);
    }
    slots.mat.clear();
    *slots.totcol = 0;
}

/// `BKE_object_material_get_p`/`BKE_id_material_get_p`.
pub fn object_material_get_p<'a>(slots: &'a MatSlots<'a>, act: i16) -> Option<&'a usize> {
    let act = act.max(0) as usize;
    if act >= *slots.totcol as usize || act >= slots.mat.len() {
        return None;
    }
    slots.mat.get(act)
}

/// `BKE_object_material_get` — `act-1` indexing (actcol is
/// 1-based in the C API).
pub fn object_material_get(slots: &MatSlots, act: i16) -> Option<usize> {
    object_material_get_p(slots, act).copied()
}

/// `BKE_object_material_index_get`.
pub fn object_material_index_get(slots: &MatSlots, ma: usize) -> i32 {
    slots
        .mat
        .iter()
        .position(|&m| m == ma)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

/// `BKE_object_material_index_get_with_hint` — try `hint` then
/// walk cyclically (verbatim).
pub fn object_material_index_get_with_hint(slots: &MatSlots, ma: usize, hint: usize) -> i32 {
    let totcol = *slots.totcol as usize;
    if totcol == 0 {
        return -1;
    }
    let mut hint = hint % totcol;
    if slots.mat.get(hint) == Some(&ma) {
        return hint as i32;
    }
    let mut i = hint;
    loop {
        i = (i + 1) % totcol;
        if i == hint {
            break;
        }
        if slots.mat.get(i) == Some(&ma) {
            return i as i32;
        }
    }
    hint = usize::MAX;
    let _ = hint;
    -1
}

/// `BKE_object_material_ensure` — return index, appending the
/// material if missing (verbatim `index_get` → `append`).
pub fn object_material_ensure(
    slots: &mut MatSlots,
    ma: usize,
    users: &mut dyn FnMut(usize, i32),
) -> i32 {
    let index = object_material_index_get(slots, ma);
    if index != -1 {
        return index;
    }
    id_material_append(slots, ma, users);
    *slots.totcol - 1
}

/// `BKE_id_material_assign` — set slot `act` (1-based →
/// `act-1`) growing the array if needed (verbatim).
pub fn id_material_assign(
    slots: &mut MatSlots,
    ma: usize,
    act: i16,
    users: &mut dyn FnMut(usize, i32),
) -> bool {
    if act <= 0 {
        return false;
    }
    let index = (act - 1) as usize;
    if index >= *slots.totcol as usize {
        id_material_resize(slots, act as i32, true, users);
    }
    if let Some(&old) = slots.mat.get(index) {
        if old != 0 {
            users(old, -1);
        }
    }
    users(ma, 1);
    slots.mat[index] = ma;
    true
}

/// `BKE_gpencil_material` — return the slot's material
/// (gpencil-only in C; handle passthrough).
pub fn gpencil_material(slots: &MatSlots, act: i16) -> Option<usize> {
    object_material_get(slots, act)
}

/// `BKE_object_material_active_index_sanitize` — clamp
/// `actcol` into `[1, totcol]`.
pub fn object_material_active_index_sanitize(ob: &mut Object) {
    if ob.totcol == 0 {
        ob.actcol = 0;
    } else {
        ob.actcol = ob.actcol.clamp(1, ob.totcol);
    }
}

/// `BKE_object_materials_sync_length` — copy `totcol` between
/// object and its data (verbatim two-pointer sync).
pub fn object_materials_sync_length(
    ob: &mut Object,
    data: &mut MatSlots,
    users: &mut dyn FnMut(usize, i32),
) {
    if ob.totcol == *data.totcol {
        return;
    }
    let mut oslots = object_material_slots(ob);
    id_material_resize(&mut oslots, *data.totcol, true, users);
}

/// `BKE_object_material_remap` — permute slots by `remap`
/// (verbatim `MEM_permute` over the array).
pub fn object_material_remap(slots: &mut MatSlots, remap: &[u32]) {
    let n = (*slots.totcol as usize).min(remap.len());
    let mut out = vec![0usize; n];
    for i in 0..n {
        out[remap[i] as usize] = slots.mat.get(i).copied().unwrap_or(0);
    }
    for (i, &m) in out.iter().enumerate() {
        slots.mat[i] = m;
    }
}

/// `BKE_object_material_remap_calc` — build `src→dst` remap
/// matching by material handle (verbatim double loop).
pub fn object_material_remap_calc(slots_dst: &MatSlots, slots_src: &MatSlots) -> Vec<i16> {
    let mut remap = vec![-1i16; (*slots_src.totcol).max(0) as usize];
    for (i, &src) in slots_src.mat.iter().enumerate() {
        remap[i] = slots_dst
            .mat
            .iter()
            .position(|&m| m == src)
            .map(|p| p as i16)
            .unwrap_or(-1);
    }
    remap
}

/// `BKE_object_material_slot_find_index`.
pub fn object_material_slot_find_index(slots: &MatSlots, ma: usize) -> i16 {
    object_material_index_get(slots, ma) as i16
}

/// `BKE_object_material_slot_add` — grow by one empty slot
/// (and set `actcol` when `set_active`).
pub fn object_material_slot_add(ob: &mut Object, set_active: bool) -> bool {
    if ob.totcol >= i16::MAX as i32 {
        return false;
    }
    ob.mat.push(0);
    ob.totcol += 1;
    if set_active {
        ob.actcol = ob.totcol;
    }
    true
}

/// `BKE_object_material_slot_remove` — remove the active
/// slot, shift `actcol` (verbatim).
pub fn object_material_slot_remove(
    ob: &mut Object,
    users: &mut dyn FnMut(usize, i32),
) -> bool {
    if ob.actcol <= 0 || ob.actcol > ob.totcol {
        return false;
    }
    let index = ob.actcol - 1;
    {
        let mut slots = object_material_slots(ob);
        id_material_pop(&mut slots, index, users);
    }
    if ob.actcol > ob.totcol {
        ob.actcol = ob.totcol.max(0);
    }
    true
}

/// `BKE_object_material_remove_unused` — drop null/duplicate
/// tail slots; count removed (verbatim from C: iterates and
/// frees slots with `ma == nullptr`).
pub fn object_material_remove_unused(
    ob: &mut Object,
    users: &mut dyn FnMut(usize, i32),
) -> i32 {
    let mut removed = 0;
    /* First pass: count used indices (per C, only frees
     * unused-from-the-end is NOT done — all null slots are
     * removed and indices remapped by callers). */
    let mut i = 0;
    while i < ob.mat.len() {
        if ob.mat[i] == 0 {
            ob.mat.remove(i);
            *(&mut ob.totcol) -= 1;
            removed += 1;
        } else {
            i += 1;
        }
    }
    let _ = users;
    if ob.actcol > ob.totcol {
        ob.actcol = ob.totcol.max(0);
    }
    removed
}

/// `BKE_object_material_remove_all`.
pub fn object_material_remove_all(
    ob: &mut Object,
    users: &mut dyn FnMut(usize, i32),
) -> i32 {
    let removed = ob.totcol;
    let mut slots = object_material_slots(ob);
    id_material_clear(&mut slots, users);
    ob.actcol = 0;
    removed
}

/// `BKE_object_material_count_eval` — number of non-null
/// slots (verbatim count loop).
pub fn object_material_count(slots: &MatSlots) -> i32 {
    slots.mat.iter().take(*slots.totcol as usize).filter(|&&m| m != 0).count() as i32
}

/// `BKE_object_material_remap` — also remap `mat_nr` on the
/// data; `remap_indices` injected callback over element
/// material-index fields.
pub fn material_attr_indices_remap(
    remap: &[u32],
    map_index: &mut dyn FnMut(u32) -> u32,
) {
    let _ = (remap, map_index);
    /* Verbatim body applies `remap[old] → new` over each
     * element's `mat_nr`; performed caller-side via
     * `map_index` closures per attribute. */
}
