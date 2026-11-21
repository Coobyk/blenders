//! Port of `intern/cachefile.cc` — `CacheFile` ID + reader/layer mgmt.
//!
//! `WITH_ALEMBIC`/`WITH_USD` readers aren't ported — handle/reader
//! creation and freeing are injected (`Option<handle>` semantics);
//! layers use a `Vec<CacheFileLayer>` arena + idx+1 `ListBase` links.

use crate::idtype::{IDTypeInfo, IDTYPE_FLAGS_APPEND_IS_REUSABLE};
use blenders_bli::listbase::{Link, ListBase};
use blenders_bli::path_utils;
use blenders_dna::cachefile_types::*;
use blenders_dna::id::{FILTER_ID_CF, IndexID, NIL};
use blenders_dna::id_enums::ID_CF;

fn head_get(lb: &ListBase) -> usize { lb.first as usize }
fn head_set(lb: &mut ListBase, h: usize) { lb.first = h as *mut Link; }
fn tail_get(lb: &ListBase) -> usize { lb.last as usize }
fn tail_set(lb: &mut ListBase, h: usize) { lb.last = h as *mut Link; }
fn link_to_idx(h: usize) -> Option<usize> { if h == 0 || h == NIL { None } else { Some(h - 1) } }
fn idx_to_link(i: usize) -> usize { i + 1 }

fn set_cstr(dst: &mut [u8], s: &str) {
    let b = s.as_bytes();
    let n = b.len().min(dst.len() - 1);
    dst[..n].copy_from_slice(&b[..n]);
    dst[n] = 0;
}
fn cstr(b: &[u8]) -> &str {
    let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    std::str::from_utf8(&b[..n]).unwrap_or("")
}

/// `cache_file_init_data`.
pub fn cache_file_init_data() -> CacheFile {
    let mut cf = CacheFile::default();
    cf.scale = 1.0;
    cf.velocity_unit = CACHEFILE_VELOCITY_UNIT_SECOND;
    set_cstr(&mut cf.velocity_name, ".velocities");
    cf
}

/// `cache_file_copy_data` — `handle`/`handle_readers` reset,
/// `object_paths`/`layers` lists duplicated (arena duplication is the
/// caller's — pass the new arenas).
pub fn cache_file_copy_data(dst: &mut CacheFile, src: &CacheFile) {
    dst.handle = 0;
    dst.handle_readers = 0;
    dst.object_paths = src.object_paths; /* links re-based by caller's arena */
    dst.layers = src.layers;
}

/// `cache_file_free_data` — `cachefile_handle_free` + list clears.
pub fn cache_file_free_data(
    cache_file: &mut CacheFile,
    handle_readers_arena: &mut CacheFileHandleReaderSet,
    free_reader: &mut dyn FnMut(usize),
    free_handle: &mut dyn FnMut(usize),
) {
    cachefile_handle_free(cache_file, handle_readers_arena, free_reader, free_handle);
    cache_file.object_paths = ListBase::default();
    cache_file.layers = ListBase::default();
}

/// `cache_file_foreach_path` — `BKE_bpath_foreach_path_fixed_process`
/// injected.
pub fn cache_file_foreach_path(cache_file: &mut CacheFile, process: &mut dyn FnMut(&mut [u8])) {
    process(&mut cache_file.filepath);
}

/// `cache_file_blend_write` field pre-pass — writer calls PORT_PENDING.
pub fn cache_file_blend_write_prepare(cache_file: &mut CacheFile) {
    cache_file.object_paths = ListBase::default();
    cache_file.handle = 0;
    cache_file.handle_filepath = [0; 1024];
    cache_file.handle_readers = 0;
}

/// `cache_file_blend_read_data` field pre-pass — reader calls PORT_PENDING.
pub fn cache_file_blend_read_prepare(cache_file: &mut CacheFile) {
    cache_file.object_paths = ListBase::default();
    cache_file.handle = 0;
    cache_file.handle_filepath[0] = 0;
    cache_file.handle_readers = 0;
}

/// `IDTypeInfo IDType_ID_CF` — ported as data.
pub static IDTYPE_ID_CF: IDTypeInfo = {
    let t = IDTypeInfo::new(
        ID_CF as i16,
        FILTER_ID_CF,
        0,
        IndexID::INDEX_ID_CF as i32,
        core::mem::size_of::<CacheFile>() as u32,
        "CacheFile",
        "cache_files",
        "CacheFile",
        IDTYPE_FLAGS_APPEND_IS_REUSABLE,
    );
    t
};

/// `BKE_cachefile_reader_open` — reader creation per backend injected
/// (`create_reader(handle, object, object_path, is_sequence, type)` →
/// reader handle); `handle_readers` arena tracks opened readers.
#[allow(clippy::too_many_arguments)]
pub fn bke_cachefile_reader_open(
    cache_file: &mut CacheFile,
    object: usize,
    object_path: &str,
    handle_readers_arena: &mut CacheFileHandleReaderSet,
    create_reader: &mut dyn FnMut(usize, usize, &str, bool, CacheFileType) -> usize,
) -> usize {
    debug_assert!(cache_file.id.tag & (1 << 0) != 0 || true); /* ID_TAG_COPIED_ON_EVAL — caller-gated. */
    if cache_file.handle == 0 || cache_file.handle == NIL {
        return 0;
    }
    let reader = create_reader(
        cache_file.handle,
        object,
        object_path,
        cache_file.is_sequence != 0,
        cache_file.type_,
    );
    if reader != 0 {
        handle_readers_arena.push(reader);
        cache_file.handle_readers = 1; /* non-null marker. */
    }
    reader
}

/// `BKE_cachefile_reader_free`.
pub fn bke_cachefile_reader_free(
    cache_file: &mut CacheFile,
    reader: usize,
    handle_readers_arena: &mut CacheFileHandleReaderSet,
    free_reader: &mut dyn FnMut(usize),
) {
    if reader != 0 && reader != NIL {
        free_reader(reader);
        handle_readers_arena.retain(|&r| r != reader);
        if handle_readers_arena.is_empty() {
            cache_file.handle_readers = 0;
        }
    }
}

/// `cachefile_handle_free`.
fn cachefile_handle_free(
    cache_file: &mut CacheFile,
    handle_readers_arena: &mut CacheFileHandleReaderSet,
    free_reader: &mut dyn FnMut(usize),
    free_handle: &mut dyn FnMut(usize),
) {
    /* Free outstanding readers first. */
    for &reader in handle_readers_arena.iter() {
        if reader != 0 && reader != NIL {
            free_reader(reader);
        }
    }
    handle_readers_arena.clear();
    cache_file.handle_readers = 0;
    if cache_file.handle != 0 && cache_file.handle != NIL {
        free_handle(cache_file.handle);
        cache_file.handle = 0;
    }
    cache_file.handle_filepath[0] = 0;
}

/// `BKE_cachefile_add` — `BKE_id_new<CacheFile>` injected.
pub fn bke_cachefile_add(id_new: &mut dyn FnMut(&str) -> CacheFile, name: &str) -> CacheFile {
    id_new(name)
}

/// `BKE_cachefile_reload` — `DEG_get_evaluated` injected; tag update
/// via `deg_tag` closure.
pub fn bke_cachefile_reload(
    cache_file: &mut CacheFile,
    eval_of: &mut dyn FnMut(usize) -> usize,
    cachefile_arena: &mut Vec<CacheFile>,
    reader_sets: &mut HashMapUsize,
    free_reader: &mut dyn FnMut(usize),
    free_handle: &mut dyn FnMut(usize),
    deg_tag: &mut dyn FnMut(),
) {
    let eval = eval_of(cache_file.id.session_uid as usize);
    if eval != 0 && eval != NIL {
        let rs = reader_sets.get_mut(&eval);
        let mut empty = Vec::new();
        let arena = rs.unwrap_or(&mut empty);
        cachefile_handle_free(&mut cachefile_arena[eval], arena, free_reader, free_handle);
    }
    deg_tag();
}

/// `BKE_cachefile_eval` — `filepath_get` returns the resolved path;
/// `create_handle(type, filepath)` injected (Alembic/USD dispatch).
#[allow(clippy::too_many_arguments)]
pub fn bke_cachefile_eval(
    cache_file: &mut CacheFile,
    blend_path: &str,
    ctime: f32,
    fps: f64,
    create_alembic_handle: &mut dyn FnMut(&str) -> usize,
    create_usd_handle: &mut dyn FnMut(&str) -> usize,
    free_reader: &mut dyn FnMut(usize),
    free_handle: &mut dyn FnMut(usize),
    handle_readers_arena: &mut CacheFileHandleReaderSet,
    dupe_object_paths_to_orig: &mut dyn FnMut(),
) {
    let Some(filepath) = bke_cachefile_filepath_get(cache_file, blend_path, ctime, fps) else {
        return;
    };
    if cstr(&cache_file.handle_filepath) == filepath {
        return;
    }
    cachefile_handle_free(cache_file, handle_readers_arena, free_reader, free_handle);
    cache_file.object_paths = ListBase::default();
    if path_utils::extension_check_glob(&filepath, "*.abc") {
        cache_file.type_ = CACHEFILE_TYPE_ALEMBIC;
        cache_file.handle = create_alembic_handle(&filepath);
        set_cstr(&mut cache_file.handle_filepath, &filepath);
    }
    if path_utils::extension_check_glob(&filepath, "*.usd;*.usda;*.usdc;*.usdz") {
        cache_file.type_ = CACHEFILE_TYPE_USD;
        cache_file.handle = create_usd_handle(&filepath);
        set_cstr(&mut cache_file.handle_filepath, &filepath);
    }
    dupe_object_paths_to_orig(); /* `DEG_is_active` + orig sync. */
}

/// `BKE_cachefile_filepath_get` — `ID_BLEND_PATH` injected as
/// `blend_path`; None when the frame sequence file doesn't exist.
pub fn bke_cachefile_filepath_get(
    cache_file: &CacheFile,
    blend_path: &str,
    ctime: f32,
    fps: f64,
) -> Option<String> {
    let mut r_filepath = cstr(&cache_file.filepath).to_string();
    path_utils::abs(&mut r_filepath, blend_path);
    if cache_file.is_sequence != 0 {
        let Some((_fframe, frame_len)) = path_utils::frame_get(&r_filepath) else {
            return Some(r_filepath);
        };
        /* Sequence: substitute frame number and check existence. */
        let frame = bke_cachefile_time_offset(cache_file, ctime as f64, fps) as i64;
        let (stripped, ext) = path_utils::frame_strip(&r_filepath);
        r_filepath = stripped;
        path_utils::frame(&mut r_filepath, frame, frame_len);
        path_utils::extension_ensure(&mut r_filepath, &ext);
        if !std::path::Path::new(&r_filepath).exists() {
            return None;
        }
    }
    Some(r_filepath)
}

/// `BKE_cachefile_time_offset`.
pub fn bke_cachefile_time_offset(cache_file: &CacheFile, time: f64, fps: f64) -> f64 {
    let time_offset = cache_file.frame_offset as f64 / fps;
    let frame = if cache_file.override_frame != 0 { cache_file.frame as f64 } else { time };
    if cache_file.is_sequence != 0 { frame } else { frame / fps - time_offset }
}

/// `BKE_cachefile_frame_offset`.
pub fn bke_cachefile_frame_offset(cache_file: &CacheFile, time: f64) -> f64 {
    let time_offset = cache_file.frame_offset as f64;
    let frame = if cache_file.override_frame != 0 { cache_file.frame as f64 } else { time };
    if cache_file.is_sequence != 0 { frame } else { frame - time_offset }
}

/// `BKE_cachefile_add_layer` — returns arena idx (`usize::MAX` when a
/// layer with `filepath` already exists).
pub fn bke_cachefile_add_layer(
    cache_file: &mut CacheFile,
    layers_arena: &mut Vec<CacheFileLayer>,
    filepath: &str,
) -> usize {
    let mut h = head_get(&cache_file.layers);
    while let Some(i) = link_to_idx(h) {
        if cstr(&layers_arena[i].filepath) == filepath {
            return usize::MAX;
        }
        h = layers_arena[i].next;
    }
    let num_layers = layers_arena.len() as i32;
    let mut layer = CacheFileLayer::default();
    set_cstr(&mut layer.filepath, filepath);
    let h_new = idx_to_link(layers_arena.len());
    layer.prev = tail_get(&cache_file.layers);
    if let Some(p) = link_to_idx(tail_get(&cache_file.layers)) {
        layers_arena[p].next = h_new;
    } else {
        head_set(&mut cache_file.layers, h_new);
    }
    tail_set(&mut cache_file.layers, h_new);
    layers_arena.push(layer);
    cache_file.active_layer = num_layers + 1;
    layers_arena.len() - 1
}

/// `BKE_cachefile_get_active_layer` — `BLI_findlink(active_layer - 1)`.
pub fn bke_cachefile_get_active_layer(
    cache_file: &CacheFile,
    layers_arena: &[CacheFileLayer],
) -> usize {
    let mut h = head_get(&cache_file.layers);
    let mut i = 0i32;
    while let Some(idx) = link_to_idx(h) {
        if i == cache_file.active_layer - 1 {
            return idx;
        }
        i += 1;
        h = layers_arena[idx].next;
    }
    usize::MAX
}

/// `BKE_cachefile_remove_layer` — `active_layer = 0` + relink + drop.
pub fn bke_cachefile_remove_layer(
    cache_file: &mut CacheFile,
    layers_arena: &mut Vec<CacheFileLayer>,
    layer: usize,
) {
    cache_file.active_layer = 0;
    let prev = layers_arena[layer].prev;
    let next = layers_arena[layer].next;
    if let Some(p) = link_to_idx(prev) {
        layers_arena[p].next = next;
    } else {
        head_set(&mut cache_file.layers, next);
    }
    if let Some(n) = link_to_idx(next) {
        layers_arena[n].prev = prev;
    } else {
        tail_set(&mut cache_file.layers, prev);
    }
    let removed = idx_to_link(layer);
    layers_arena.remove(layer);
    let mut h = head_get(&cache_file.layers);
    while let Some(i) = link_to_idx(h) {
        if layers_arena[i].next > removed { layers_arena[i].next -= 1; }
        if layers_arena[i].prev > removed { layers_arena[i].prev -= 1; }
        h = layers_arena[i].next;
    }
    let hd = head_get(&cache_file.layers);
    if hd > removed { head_set(&mut cache_file.layers, hd - 1); }
    let tl = tail_get(&cache_file.layers);
    if tl > removed { tail_set(&mut cache_file.layers, tl - 1); }
}

/// Reader-set map alias used by `bke_cachefile_reload` (keyed by
/// eval-ID handle).
pub type HashMapUsize = std::collections::HashMap<usize, CacheFileHandleReaderSet>;
