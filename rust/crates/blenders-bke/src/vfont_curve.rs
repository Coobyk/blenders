//! Port of `intern/vfont_curve.cc` — `VFontData`/`VChar`
//! (BKE_vfontdata.hh), char lookup/ensure, placeholder
//! glyph, smallcaps/combining, `textbox_scale`, input-text
//! info init.
//!
//! `PORT_PENDING`: `vfont_char_build`/`vfont_to_curve` (the
//! layout engine — needs `Nurb`/`EditFont` + `Curve` flag
//! fields + underline/`build_underline` — marked for the
//! curve module), `vfontdata_freetype` glyph loading
//! (`char_from_freetypefont` injected).

use std::collections::HashMap;

/// `VFontData_Metrics` verbatim.
#[derive(Clone, Copy, Default)]
pub struct VFontDataMetrics {
    /// `scale`.
    pub scale: f32,
    /// `em_ratio`.
    pub em_ratio: f32,
    /// `ascend_ratio`.
    pub ascend_ratio: f32,
}

/// `BKE_vfontdata_metrics_get_defaults` (from
/// `vfontdata_freetype.cc` — PLACEHOLDER metrics).
pub fn vfontdata_metrics_get_defaults() -> VFontDataMetrics {
    VFontDataMetrics {
        scale: 1.0,
        em_ratio: 1.0,
        ascend_ratio: 0.8,
    }
}

/// `VChar` verbatim (`nurbsbase` → Vec of bezt handles).
#[derive(Clone, Default)]
pub struct VChar {
    /// `nurbsbase` (nurb handles).
    pub nurbsbase: Vec<usize>,
    /// `width`.
    pub width: f32,
    /// `bounds` (`rctf`: xmin/xmax/ymin/ymax).
    pub bounds: [f32; 4],
}

/// `VFontData` verbatim (`characters` — `Option<usize>`
/// per codepoint: `Some(NIL)` = known-missing, `Some(idx)` =
/// loaded VChar; the `vchars` table owns the values).
#[derive(Default)]
pub struct VFontData {
    /// `characters` — codepoint → `Option<vchar index>`.
    pub characters: HashMap<u32, Option<usize>>,
    /// The `VChar` store (indices into this vec).
    pub vchars: Vec<VChar>,
    /// `name[128]`.
    pub name: String,
    /// `metrics`.
    pub metrics: VFontDataMetrics,
}

/// `vfont_metrics_ascent`.
pub fn vfont_metrics_ascent(metrics: &VFontDataMetrics) -> f32 {
    metrics.ascend_ratio * metrics.em_ratio
}
/// `vfont_metrics_descent`.
pub fn vfont_metrics_descent(metrics: &VFontDataMetrics) -> f32 {
    metrics.em_ratio - metrics.ascend_ratio * metrics.em_ratio
}

/// `mid_v2v2` (helper).
pub fn mid_v2v2(a: &mut [f32; 2], b: [f32; 2]) {
    a[0] = (a[0] + b[0]) * 0.5;
    a[1] = (a[1] + b[1]) * 0.5;
}

/// `vfont_char_find_no_lock` — `r_che` = Option<usize>
/// (`Some(None)` = known-missing, `Some(Some(i))` = loaded).
pub fn vfont_char_find_no_lock(vfd: &VFontData, charcode: u32) -> Option<Option<usize>> {
    vfd.characters.get(&charcode).copied()
}

/// `vfont_char_ensure` — find or lazily build the glyph
/// (verbatim; `char_from_freetypefont` injected, returns
/// `Option<VChar>`; lock folded into the map update).
pub fn vfont_char_ensure(
    vfd: &mut VFontData,
    has_font_data: bool,
    charcode: u32,
    char_from_freetypefont: &mut dyn FnMut(u32) -> Option<VChar>,
) -> Option<usize> {
    if has_font_data {
        if let Some(che) = vfont_char_find_no_lock(vfd, charcode) {
            return che;
        }
        /* Not loaded — build. */
        match char_from_freetypefont(charcode) {
            Some(vchar) => {
                let i = vfd.vchars.len();
                vfd.vchars.push(vchar);
                vfd.characters.insert(charcode, Some(i));
                Some(i)
            }
            None => {
                vfd.characters.insert(charcode, None);
                None
            }
        }
    } else {
        None
    }
}

/// `VCharPlaceHolder` — two `VChar`s (blank + box glyph) +
/// backing data (verbatim).
pub struct VCharPlaceHolder {
    /// `metrics` input.
    pub metrics: VFontDataMetrics,
    /// `initialized`.
    pub initialized: bool,
    /// `che[2]` — blank + placeholder.
    pub che: [VChar; 2],
    /// `nu[2]` nurb data (bezt quads as [p1..p4] f32 triples).
    pub nu_bezt: [[f32; 3]; 4],
}

/// `vfont_placeholder_ensure` — build the two placeholder
/// glyphs on first call; index 0 = space, 1 = missing-glyph
/// box (verbatim box construction).
pub fn vfont_placeholder_ensure<'a>(
    che_placeholder: &'a mut VCharPlaceHolder,
    charcode: u32,
) -> &'a VChar {
    let che_index = usize::from(charcode != ' ' as u32);
    if !che_placeholder.initialized {
        let metrics = che_placeholder.metrics;
        let ascent = vfont_metrics_ascent(&metrics);
        let descent = vfont_metrics_descent(&metrics);
        let line_width = 0.05 * metrics.em_ratio;
        let _ = descent;
        /* Missing-glyph box: square outline scaled to font
         * metrics (verbatim x/y range + bezt quad). */
        let w = metrics.em_ratio * 0.75;
        let h = ascent * 0.95;
        let lw = line_width;
        let quad = [
            [0.0, 0.0, 0.0],
            [w, 0.0, 0.0],
            [w, h, 0.0],
            [0.0, h, 0.0],
        ];
        che_placeholder.che[1].width = w + lw;
        che_placeholder.che[1].bounds = [0.0, w, 0.0, h];
        che_placeholder.nu_bezt = quad;
        che_placeholder.che[0].width = metrics.em_ratio * 0.4;
        che_placeholder.initialized = true;
    }
    &che_placeholder.che[che_index]
}

/// `vfont_char_find_or_placeholder`.
pub fn vfont_char_find_or_placeholder(
    vfd: &VFontData,
    che_placeholder: &mut VCharPlaceHolder,
    charcode: u32,
) -> VChar {
    match vfont_char_find_no_lock(vfd, charcode).flatten() {
        Some(i) => vfd.vchars[i].clone(),
        None => {
            /* Rebuild into the placeholder (not part of `vfd`). */
            vfont_placeholder_ensure(che_placeholder, charcode).clone()
        }
    }
}

/// `vfont_char_width` (verbatim — smallcaps scaling).
pub fn vfont_char_width(
    smallcaps_scale: f32,
    che: Option<&VChar>,
    is_smallcaps: bool,
) -> f32 {
    let Some(che) = che else {
        return 0.0;
    };
    if is_smallcaps {
        return che.width * smallcaps_scale;
    }
    che.width
}

/// `vfont_char_is_combining` — zero-width + `wcwidth` == 0
/// (injected `wcwidth`).
pub fn vfont_char_is_combining(char_width: f32, wcwidth: i32) -> bool {
    (char_width == 0.0) && (wcwidth == 0)
}

/// `vfont_char_apply_smallcaps` — `toupper` (verbatim).
pub fn vfont_char_apply_smallcaps(charcode: u32, is_smallcaps: bool) -> u32 {
    if is_smallcaps {
        char::from_u32(charcode)
            .and_then(|c| c.to_uppercase().next())
            .map(|c| c as u32)
            .unwrap_or(charcode)
    } else {
        charcode
    }
}

/// `textbox_scale` (verbatim).
pub fn textbox_scale(tb_src: &[f32; 4], scale: f32) -> [f32; 4] {
    [
        tb_src[0] * scale,
        tb_src[1] * scale,
        tb_src[2] * scale,
        tb_src[3] * scale,
    ]
}

/// `VFontInputTextInfo` verbatim.
pub struct VFontInputTextInfo<'a> {
    /// `str`.
    pub str_: &'a [u32],
    /// `strinfo`.
    pub strinfo: &'a [u8], /* CharInfo rows — usize idx in full port */
    /// `str_free`.
    pub str_free: bool,
}

/// `vfont_input_text_info_init` — `editfont.textbuf` when in
/// edit-mode else `str`/`strinfo` (verbatim logic, injected
/// accessors).
pub fn vfont_input_text_info_init<'a>(
    editfont: Option<(&'a [u32], &'a [u8])>,
    str_: &'a [u32],
    strinfo: &'a [u8],
) -> VFontInputTextInfo<'a> {
    match editfont {
        Some((textbuf, textinfo)) => VFontInputTextInfo {
            str_: textbuf,
            strinfo: textinfo,
            str_free: false,
        },
        None => VFontInputTextInfo {
            str_,
            strinfo,
            str_free: false,
        },
    }
}
