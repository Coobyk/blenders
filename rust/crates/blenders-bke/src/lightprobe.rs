//! Port of `intern/lightprobe.cc` — `BKE_lightprobe_*` + `IDType_ID_LP`.

use blenders_dna::id::{IndexID, NIL, FILTER_ID_IM, FILTER_ID_LP};
use blenders_dna::id_enums::ID_LP;
use blenders_dna::lightprobe_types::*;
use crate::idtype::*;

/// `lightprobe_init_data` — `INIT_DEFAULT_STRUCT_AFTER(probe, id)`:
/// every field after `id` gets its `LightProbe::default()` value.
pub fn lightprobe_init_data(probe: &mut LightProbe) {
    let id = std::mem::take(&mut probe.id);
    *probe = LightProbe::default();
    probe.id = id;
}

/// `lightprobe_foreach_id` — `visibility_grp` walk injected.
pub fn lightprobe_foreach_id(probe: &mut LightProbe, process: &mut dyn FnMut(&mut usize, i32)) {
    const IDWALK_CB_NOP: i32 = 0;
    process(&mut probe.visibility_grp, IDWALK_CB_NOP);
}

/// `lightprobe_blend_write` — `writer.write_id_struct`/`BKE_id_blend_write` injected.
pub fn lightprobe_blend_prepare(probe: &LightProbe, write_id_struct: &mut dyn FnMut(&LightProbe), write_id: &mut dyn FnMut(&blenders_dna::id::ID)) {
    write_id_struct(probe);
    write_id(&probe.id);
}

/// `IDType_ID_LP` — verbatim data table.
pub static IDTYPE_ID_LP: IDTypeInfo = IDTypeInfo::new(
    ID_LP as i16,
    FILTER_ID_LP,
    FILTER_ID_IM,
    IndexID::INDEX_ID_LP as i32,
    core::mem::size_of::<LightProbe>() as u32,
    "LightProbe",
    "lightprobes",
    "LightProbe",
    IDTYPE_FLAGS_APPEND_IS_REUSABLE,
);

/// `BKE_lightprobe_type_set` — per-type defaults.
pub fn bke_lightprobe_type_set(probe: &mut LightProbe, lightprobe_type: i8) {
    probe.type_ = lightprobe_type;
    match probe.type_ as i32 {
        x if x == LIGHTPROBE_TYPE_VOLUME as i32 => {
            probe.distinf = 0.3;
            probe.falloff = 1.0;
            probe.clipsta = 0.01;
        }
        x if x == LIGHTPROBE_TYPE_PLANE as i32 => {
            probe.distinf = 0.1;
            probe.falloff = 0.5;
            probe.clipsta = 0.001;
        }
        x if x == LIGHTPROBE_TYPE_SPHERE as i32 => {
            probe.attenuation_type = LIGHTPROBE_SHAPE_ELLIPSOID as i8;
        }
        _ => debug_assert!(false, "LightProbe type not configured."),
    }
}

/// `BKE_lightprobe_add` — `BKE_id_new` injected.
pub fn bke_lightprobe_add(id_new: &mut dyn FnMut(&str) -> LightProbe, name: &str) -> LightProbe {
    id_new(name)
}

/// `spherical_harmonic_free` — clears L0/L1_a/L1_b/L1_c.
fn spherical_harmonic_free_vecs(v: [&mut Vec<f32>; 4]) {
    for x in v { *x = Vec::new(); }
}

/// `BKE_lightprobe_grid_cache_frame_free`.
pub fn grid_cache_frame_free(cache: &mut LightProbeGridCacheFrame) {
    spherical_harmonic_free_vecs([&mut cache.baking.l0, &mut cache.baking.l1_a, &mut cache.baking.l1_b, &mut cache.baking.l1_c]);
    spherical_harmonic_free_vecs([&mut cache.irradiance.l0, &mut cache.irradiance.l1_a, &mut cache.irradiance.l1_b, &mut cache.irradiance.l1_c]);
    spherical_harmonic_free_vecs([&mut cache.visibility.l0, &mut cache.visibility.l1_a, &mut cache.visibility.l1_b, &mut cache.visibility.l1_c]);
    cache.baking.validity = Vec::new();
    cache.connectivity.validity = Vec::new();
    cache.surfels = NIL;
    cache.surfels_len = 0;
    cache.baking.virtual_offset = Vec::new();
    cache.block_infos = Vec::new();
    cache.block_len = 0;
}

/// `BKE_lightprobe_grid_cache_frame_copy` — deep clone.
pub fn grid_cache_frame_copy(src: &LightProbeGridCacheFrame) -> LightProbeGridCacheFrame {
    src.clone()
}

/// `BKE_lightprobe_cache_create`.
pub fn bke_lightprobe_cache_create() -> LightProbeObjectCache {
    LightProbeObjectCache::default()
}

/// `BKE_lightprobe_cache_copy` — `grid_static_cache` is a
/// `GridCacheFrame` arena idx: `copy_frame` clones it.
pub fn bke_lightprobe_cache_copy(
    src: &LightProbeObjectCache,
    copy_frame: &mut dyn FnMut(usize) -> usize,
) -> LightProbeObjectCache {
    let mut dst = src.clone();
    if src.grid_static_cache != NIL {
        dst.grid_static_cache = copy_frame(src.grid_static_cache);
    }
    dst
}

/// `BKE_lightprobe_cache_free` — `free_frame` injected for the
/// arena-owned frame; shared caches skip it.
pub fn bke_lightprobe_cache_free(cache: &mut LightProbeObjectCache, free_frame: &mut dyn FnMut(usize)) {
    if cache.shared == 0 && cache.grid_static_cache != NIL {
        free_frame(cache.grid_static_cache);
        cache.grid_static_cache = NIL;
    }
}

/// `BKE_lightprobe_grid_cache_frame_sample_count` — overflow-safe
/// multiply (`INT64_MAX` on overflow).
pub fn bke_lightprobe_grid_cache_frame_sample_count(cache: &LightProbeGridCacheFrame) -> i64 {
    if cache.data_layout == LIGHTPROBE_CACHE_ADAPTIVE_RESOLUTION {
        let b = cache.block_size as i64;
        return b.checked_mul(b).and_then(|r| r.checked_mul(b)).and_then(|r| r.checked_mul(cache.block_len as i64)).unwrap_or(i64::MAX);
    }
    /* LIGHTPROBE_CACHE_UNIFORM_GRID */
    (cache.size[0] as i64)
        .checked_mul(cache.size[1] as i64)
        .and_then(|r| r.checked_mul(cache.size[2] as i64))
        .unwrap_or(i64::MAX)
}

/// `lightprobe_grid_cache_frame_blend_read` — layout guard +
/// baking/surfel reset + `block_len=0` on read failure
/// (`BLO_read_array` results injected as `Option<Vec>`).
#[allow(clippy::too_many_arguments)]
pub fn grid_cache_frame_blend_read_prepare(cache: &mut LightProbeGridCacheFrame, ok: bool) {
    if cache.data_layout != LIGHTPROBE_CACHE_ADAPTIVE_RESOLUTION
        && cache.data_layout != LIGHTPROBE_CACHE_UNIFORM_GRID
    {
        *cache = LightProbeGridCacheFrame::default();
        return;
    }
    /* Baking data is not stored. */
    cache.baking = LightProbeBakingData::default();
    cache.surfels = NIL;
    cache.surfels_len = 0;
    if !ok {
        cache.block_len = 0;
    }
}
