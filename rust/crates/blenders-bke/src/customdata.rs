//! Port of `intern/customdata.cc` — the `CustomData` layer
//! table: typemap, layer index lookups, active/render
//! selection, flags, name uniqueness, layer add/remove,
//! mask helpers.
//!
//! `PORT_PENDING`: `CustomData_merge(_layout)` per-type
//! copy/alloc (needs `LayerTypeInfo` fn table for copy/
//! interp — injected `layer_copy`), `CustomData_external`
//! file ops, `alloc_`/`free_`/`set_`/`ensure_layers_are_
//! mutable` (ImplicitSharingInfo handles), `interpolate`
//! (interp fns per type), `CustomData_validate_versions`.

use blenders_dna::customdata_types::{CustomData, CustomDataLayer};

/// `MAX_CUSTOMDATA_LAYER_NAME` verbatim.
pub const MAX_CUSTOMDATA_LAYER_NAME: usize = 64;

/// `CustomData_update_typemap` (verbatim).
pub fn customdata_update_typemap(data: &mut CustomData) {
    let mut lasttype = -1i32;
    for t in data.typemap.iter_mut() {
        *t = -1;
    }
    for i in 0..data.totlayer as usize {
        let ty = data.layers[i].type_ as i32;
        if ty != lasttype {
            data.typemap[ty as usize] = i as i32;
            lasttype = ty;
        }
    }
}

/// `CustomData_get_layer_index__notypemap` (verbatim).
pub fn customdata_get_layer_index_notypemap(data: &CustomData, type_: i32) -> i32 {
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ {
            return i as i32;
        }
    }
    -1
}

/// `CustomData_get_layer_index` (verbatim — the typemap
/// hit; `-1` = none).
pub fn customdata_get_layer_index(data: &CustomData, type_: i32) -> i32 {
    data.typemap[type_ as usize]
}

/// `CustomData_get_layer_index_n` (verbatim).
pub fn customdata_get_layer_index_n(data: &CustomData, type_: i32, n: i32) -> i32 {
    debug_assert!(n >= 0);
    let i = customdata_get_layer_index(data, type_);
    if i != -1 {
        let cand = i + n;
        if (cand as usize) < data.totlayer as usize
            && data.layers[cand as usize].type_ as i32 == type_
        {
            return cand;
        }
        return -1;
    }
    -1
}

fn layer_name(layer: &CustomDataLayer) -> String {
    let end = layer
        .name
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(layer.name.len());
    String::from_utf8_lossy(&layer.name[..end]).into_owned()
}

/// `CustomData_get_named_layer_index` (verbatim).
pub fn customdata_get_named_layer_index(data: &CustomData, type_: i32, name: &str) -> i32 {
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ && layer_name(&data.layers[i]) == name {
            return i as i32;
        }
    }
    -1
}

/// `CustomData_get_named_layer_index_notype` (verbatim).
pub fn customdata_get_named_layer_index_notype(data: &CustomData, name: &str) -> i32 {
    for i in 0..data.totlayer as usize {
        if layer_name(&data.layers[i]) == name {
            return i as i32;
        }
    }
    -1
}

/// `CustomData_get_active_layer_index` (verbatim).
pub fn customdata_get_active_layer_index(data: &CustomData, type_: i32) -> i32 {
    let li = data.typemap[type_ as usize];
    if li != -1 {
        li + data.layers[li as usize].active
    } else {
        -1
    }
}

/// `CustomData_get_render_layer_index` (verbatim).
pub fn customdata_get_render_layer_index(data: &CustomData, type_: i32) -> i32 {
    let li = data.typemap[type_ as usize];
    if li != -1 {
        li + data.layers[li as usize].active_rnd
    } else {
        -1
    }
}

/// `CustomData_get_named_layer` (verbatim).
pub fn customdata_get_named_layer(data: &CustomData, type_: i32, name: &str) -> i32 {
    let ni = customdata_get_named_layer_index(data, type_, name);
    let li = data.typemap[type_ as usize];
    if ni != -1 {
        ni - li
    } else {
        -1
    }
}

/// `CustomData_get_active_layer` (verbatim).
pub fn customdata_get_active_layer(data: &CustomData, type_: i32) -> i32 {
    let li = data.typemap[type_ as usize];
    if li != -1 {
        data.layers[li as usize].active
    } else {
        -1
    }
}

/// `CustomData_get_render_layer` (verbatim).
pub fn customdata_get_render_layer(data: &CustomData, type_: i32) -> i32 {
    let li = data.typemap[type_ as usize];
    if li != -1 {
        data.layers[li as usize].active_rnd
    } else {
        -1
    }
}

/// `CustomData_number_of_layers` (verbatim — count of a
/// type in the table).
pub fn customdata_number_of_layers(data: &CustomData, type_: i32) -> i32 {
    data.layers[..data.totlayer as usize]
        .iter()
        .filter(|l| l.type_ as i32 == type_)
        .count() as i32
}

/// `CustomData_get_active_layer_name`.
pub fn customdata_get_active_layer_name(data: &CustomData, type_: i32) -> Option<String> {
    let li = customdata_get_active_layer_index(data, type_);
    if li < 0 {
        return None;
    }
    Some(layer_name(&data.layers[li as usize]))
}

/// `CustomData_get_render_layer_name`.
pub fn customdata_get_render_layer_name(data: &CustomData, type_: i32) -> Option<String> {
    let li = customdata_get_render_layer_index(data, type_);
    if li < 0 {
        return None;
    }
    Some(layer_name(&data.layers[li as usize]))
}

/// `CustomData_set_layer_active` (verbatim).
pub fn customdata_set_layer_active(data: &mut CustomData, type_: i32, n: i32) {
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ {
            data.layers[i].active = n;
        }
    }
}
/// `CustomData_set_layer_render` (verbatim).
pub fn customdata_set_layer_render(data: &mut CustomData, type_: i32, n: i32) {
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ {
            data.layers[i].active_rnd = n;
        }
    }
}
/// `CustomData_set_layer_active_index` (verbatim).
pub fn customdata_set_layer_active_index(data: &mut CustomData, type_: i32, n: i32) {
    let layer_index = n - data.typemap[type_ as usize];
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ {
            data.layers[i].active = layer_index;
        }
    }
}
/// `CustomData_set_layer_render_index` (verbatim).
pub fn customdata_set_layer_render_index(data: &mut CustomData, type_: i32, n: i32) {
    let layer_index = n - data.typemap[type_ as usize];
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ {
            data.layers[i].active_rnd = layer_index;
        }
    }
}

/// `CustomData_set_layer_flag` — OR `flag` into all layers
/// of `type_` (verbatim).
pub fn customdata_set_layer_flag(data: &mut CustomData, type_: i32, flag: i32) {
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ {
            data.layers[i].flag |= flag;
        }
    }
}

/// `CustomData_unset_layer_flag` (verbatim).
pub fn customdata_unset_layer_flag(data: &mut CustomData, type_: i32, flag: i32) {
    for i in 0..data.totlayer as usize {
        if data.layers[i].type_ as i32 == type_ {
            data.layers[i].flag &= !flag;
        }
    }
}

/// `CustomData_reset` — empty the table (verbatim).
pub fn customdata_reset(data: &mut CustomData) {
    data.layers.clear();
    data.totlayer = 0;
    data.maxlayer = 0;
    for t in data.typemap.iter_mut() {
        *t = -1;
    }
    data.totsize = 0;
    data.pool = 0;
    data.external = None;
}

/// `CustomData_merge` — add `source` layers not already
/// present in `dest` by name (verbatim merge loop; the
/// per-element copy is `layer_copy(&mut [u8],
/// &[u8], totelem)` on raw `data` blobs injected).
pub fn customdata_merge(
    source: &CustomData,
    dest: &mut CustomData,
    mask: u64,
    layer_copy: &mut dyn FnMut(&CustomDataLayer, &CustomDataLayer),
) -> bool {
    let mut updated = false;
    for i in 0..source.totlayer as usize {
        let sl = &source.layers[i];
        let ty = sl.type_ as i32;
        if ty >= 64 || ((mask >> ty) & 1) == 0 {
            continue;
        }
        let name = layer_name(sl);
        if customdata_get_named_layer_index(dest, ty, &name) != -1 {
            continue;
        }
        let mut nl = sl.clone();
        nl.data = sl.data; /* copy handle — blob sharing
                            * resolved by `layer_copy` */
        layer_copy(sl, &nl);
        dest.layers.push(nl);
        updated = true;
    }
    if updated {
        dest.totlayer = dest.layers.len() as i32;
        dest.maxlayer = dest.totlayer;
        customdata_update_typemap(dest);
    }
    updated
}

/// `CustomData_layertype_is_singleton` — the type table
/// (verbatim list of singleton types).
pub fn customdata_layertype_is_singleton(type_: i32) -> bool {
    matches!(
        type_,
        /* CD_MVERT */ 0
        | /* CD_MSTICKY */ 1
        | /* CD_MDEORMVERT */ 2
        | /* CD_MEDGE */ 3
        | /* CD_MLOOP */ 5
        | /* CD_MPOLY */ 6
        | /* CD_MTEXPOLY */ 11
        | /* CD_MLOOPUV */ 16
        | /* CD_MLOOPTANGENT */ 17
        | /* CD_CUSTOMLOOPNORMAL */ 18
        | /* CD_SCULPT_FACE_SETS */ 20
        | /* CD_ORIGINDEX_MLOOP */ 25
        | /* CD_SHAPEKEY */ 27
        | /* CD_CLOTH_ORCO */ 33
        | /* CD_PROP_INT8 */ 34
        | /* CD_CREASE */ 43
        | /* CD_PROP_STRING */ 46
    )
}

/// `CD_FLAG_NOCOPY`/`CD_FLAG_NOFREE`/`CD_FLAG_TEMPORARY`
/// (DNA_customdata_types `eCustomDataLayer_Flag`).
pub const CD_FLAG_NOCOPY: i32 = 1 << 0;
/// `CD_FLAG_NOFREE`.
pub const CD_FLAG_NOFREE: i32 = 1 << 1;
/// `CD_FLAG_TEMPORARY`.
pub const CD_FLAG_TEMPORARY: i32 = 1 << 2;
/// `CD_FLAG_ELEM_*` handled in DNA.
pub const CD_FLAG_EXTERNAL: i32 = 1 << 3;
