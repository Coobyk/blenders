//! Port of `intern/vfontdata_freetype.cc` — `BKE_vfontdata_from_freetypefont`
//! + `BKE_vfontdata_metrics_get_defaults`.
//!
//! `VFontData` is a blenfont-internal struct (forward-declared in
//! `DNA_vfont_types.h`) — `usize` handle. `BLF_*` calls are caller-supplied
//! (PORT_PENDING blenfont).

/// `VFontData_Metrics` (BKE_vfontdata.hh).
#[derive(Clone, Copy, Default)]
pub struct VFontDataMetrics {
    pub scale: f32,
    pub em_ratio: f32,
    pub ascend_ratio: f32,
}

/// `BLF_VFONT_METRICS_*_DEFAULT` (BLF_api.hh).
pub const BLF_VFONT_METRICS_SCALE_DEFAULT: f32 = 1.0;
pub const BLF_VFONT_METRICS_EM_RATIO_DEFAULT: f32 = 1.0;
pub const BLF_VFONT_METRICS_ASCEND_RATIO_DEFAULT: f32 = 0.8;

/// `PackedFile` — (data, size).
pub struct PackedFile<'a> {
    pub data: &'a [u8],
    pub size: usize,
}

/// `BKE_vfontdata_metrics_get_defaults`.
pub fn bke_vfontdata_metrics_get_defaults(metrics: &mut VFontDataMetrics) {
    metrics.scale = BLF_VFONT_METRICS_SCALE_DEFAULT;
    metrics.em_ratio = BLF_VFONT_METRICS_EM_RATIO_DEFAULT;
    metrics.ascend_ratio = BLF_VFONT_METRICS_ASCEND_RATIO_DEFAULT;
}

/// `BKE_vfontdata_from_freetypefont` — `BLF_load_mem` + metrics query +
/// `BLI_str_utf8_invalid_strip`; returns the `VFontData` handle
/// (`0` = load failed, matching `nullptr`).
pub fn bke_vfontdata_from_freetypefont(
    pf: &PackedFile,
    /* `BLF_load_mem("FTVFont", data, size)` → fontid (usize::MAX = -1). */
    blf_load_mem: &mut dyn FnMut(&[u8]) -> usize,
    /* `BLF_display_name_from_id(fontid)` → name. */
    blf_display_name: &mut dyn FnMut(usize) -> String,
    /* `BLF_get_vfont_metrics(fontid, &mut metrics)` → success. */
    blf_get_metrics: &mut dyn FnMut(usize, &mut VFontDataMetrics) -> bool,
    /* `BLF_vfontdata_new(name, metrics)` → handle (allocator PORT_PENDING). */
    alloc: &mut dyn FnMut(&str, &VFontDataMetrics) -> usize,
) -> usize {
    let fontid = blf_load_mem(pf.data);
    if fontid == usize::MAX {
        return 0;
    }
    let name = blf_display_name(fontid);
    /* `BLI_str_utf8_invalid_strip` — replace invalid UTF-8. */
    let clean = String::from_utf8_lossy(name.as_bytes()).into_owned();
    let mut metrics = VFontDataMetrics::default();
    if !blf_get_metrics(fontid, &mut metrics) {
        return 0;
    }
    alloc(&clean, &metrics)
}
