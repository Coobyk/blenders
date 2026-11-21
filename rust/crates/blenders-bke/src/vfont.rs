//! Port of `intern/vfont.cc` — `VFont` loading/builtin/pack flow and
//! text-selection + clipboard helpers.
//!
//! `PackedFile`/`VFontData` (FreeType parse) are `usize` handles with
//! injected hooks; selection math and the clipboard are fully ported.

use std::sync::Mutex;

/// `FO_BUILTIN_NAME` (BKE_vfont.hh).
pub const FO_BUILTIN_NAME: &str = "<builtin>";

/// Registered builtin font bytes (`BKE_vfont_builtin_register`).
static BUILTIN_FONT: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// `BKE_vfont_is_builtin`.
pub fn bke_vfont_is_builtin(filepath: &str) -> bool {
    filepath == FO_BUILTIN_NAME
}

/// `BKE_vfont_builtin_register`.
pub fn bke_vfont_builtin_register(mem: &[u8]) {
    *BUILTIN_FONT.lock().unwrap() = Some(mem.to_vec());
}

/// `packedfile_new_from_builtin` — fresh copy of the registered
/// builtin bytes (caller wraps into a `PackedFile` via
/// `packedfile_new_from_memory`).
pub fn builtin_font_bytes() -> Option<Vec<u8>> {
    BUILTIN_FONT.lock().unwrap().clone()
}

/// `VFont` (DNA_vfont_types.h) — runtime subset.
#[derive(Default)]
pub struct VFont {
    /// `char filepath[1024]` — relative or absolute path (or builtin).
    pub filepath: String,
    /// `PackedFile *packedfile` — autopacked data (handle).
    pub packedfile: usize,
    /// `PackedFile *temp_pf` — packed copy for loading (handle).
    pub temp_pf: usize,
    /// `VFontData *data` — parsed FreeType data (handle).
    pub data: usize,
    /// ID user count.
    pub id_us: i32,
    /// `vfd->name` — real font name.
    pub name: String,
}

/// `BKE_vfont_data_ensure` — load `vfont->data` from builtin /
/// packed / disk source (all PackedFile ops injected).
pub fn bke_vfont_data_ensure(
    vfont: &mut VFont,
    packedfile_new_from_memory: &dyn Fn(Vec<u8>) -> usize,
    packedfile_new_from_disk: &dyn Fn(&str) -> usize,
    packedfile_duplicate: &dyn Fn(usize) -> usize,
    vfontdata_from_freetypefont: &dyn Fn(usize) -> usize,
    packedfile_free: &dyn Fn(usize),
) {
    let pf = if bke_vfont_is_builtin(&vfont.filepath) {
        match builtin_font_bytes() {
            Some(bytes) => packedfile_new_from_memory(bytes),
            None => {
                /* `CLOG_ERROR`: builtin font not loaded. */
                0
            }
        }
    } else {
        if vfont.packedfile != 0 {
            let pf = vfont.packedfile;
            /* We need to copy a temporary font to memory unless it is
             * already there. */
            if vfont.temp_pf == 0 {
                vfont.temp_pf = packedfile_duplicate(pf);
            }
            pf
        } else {
            let pf = packedfile_new_from_disk(&vfont.filepath);
            if vfont.temp_pf == 0 {
                vfont.temp_pf = packedfile_new_from_disk(&vfont.filepath);
            }
            pf
        }
    };

    /* NOTE(@ideasman42): Don't attempt to find a fallback; failed
     * fonts render placeholder characters instead. */
    if pf != 0 {
        vfont.data = vfontdata_from_freetypefont(pf);
        if pf != vfont.packedfile {
            packedfile_free(pf);
        }
    }
}

/// `BKE_vfont_data_free` — parsed data + temp packed file.
/// (Character/Nurb freeing lives inside `vfontdata_free`.)
pub fn bke_vfont_data_free(
    vfont: &mut VFont,
    vfontdata_free: &dyn Fn(usize),
    packedfile_free: &dyn Fn(usize),
) {
    if vfont.data != 0 {
        vfontdata_free(vfont.data);
        vfont.data = 0;
    }
    if vfont.temp_pf != 0 {
        packedfile_free(vfont.temp_pf); /* Null when the font file can't be found on disk. */
        vfont.temp_pf = 0;
    }
}

/// `BKE_vfont_load` — parse the font, take the ID name from
/// `vfd->name` or the file's basename.
///
/// Returns `(vfont, keep_pf)` where `keep_pf` tells whether
/// `packedfile` was retained (autopack); the caller's
/// `packedfile_free` is invoked inside on a dropped pf.
#[allow(clippy::too_many_arguments)]
pub fn bke_vfont_load(
    filepath: &str,
    autopack: bool,
    packedfile_new_from_memory: &dyn Fn(Vec<u8>) -> usize,
    packedfile_new_from_disk: &dyn Fn(&str) -> usize,
    vfontdata_from_freetypefont: &dyn Fn(usize) -> usize,
    packedfile_free: &dyn Fn(usize),
) -> Option<VFont> {
    let is_builtin = filepath == FO_BUILTIN_NAME;
    let filename = if is_builtin {
        filepath.to_string()
    } else {
        filepath.rsplit('/').next().unwrap_or(filepath).to_string()
    };

    let pf = if is_builtin {
        builtin_font_bytes().map(packedfile_new_from_memory).unwrap_or(0)
    } else {
        packedfile_new_from_disk(filepath)
    };
    if pf == 0 {
        return None;
    }

    let vfd = vfontdata_from_freetypefont(pf);
    if vfd == 0 {
        /* Free the packed file. */
        packedfile_free(pf);
        return None;
    }

    /* `vfd->name` is part of the parsed data — resolved via the same
     * handle in a full port; here the caller stores the font name
     * when known, else the filename. */
    let mut vfont = VFont {
        filepath: filepath.to_string(),
        data: vfd,
        name: filename.clone(),
        ..Default::default()
    };

    /* If auto-pack is on, store the packed file. */
    if !is_builtin && autopack {
        vfont.packedfile = pf;
    }

    /* Do not add FO_BUILTIN_NAME to the temporary list. */
    if filename != FO_BUILTIN_NAME {
        vfont.temp_pf = packedfile_new_from_disk(filepath);
    }

    if vfont.packedfile != pf {
        packedfile_free(pf);
    }
    Some(vfont)
}

/// `BKE_vfont_load_exists_ex` — dedupe by absolute filepath.
/// `fonts` yields `(vfont, abs_filepath)` pairs; returns `Some(index)`
/// when found (user++ applied by caller), else the caller loads.
pub fn bke_vfont_load_exists_ex<'a>(
    filepath_abs: &str,
    fonts: &'a [&'a str],
) -> Option<usize> {
    /* first search an identical filepath */
    fonts.iter().position(|&f| f == filepath_abs)
}

/// `BKE_vfont_packfile_ensure` — skip when already packed (#152638:
/// the original file may be gone).
pub fn bke_vfont_packfile_ensure(
    vfont: &mut VFont,
    packedfile_new_from_disk: &dyn Fn(&str) -> usize,
) {
    if vfont.packedfile != 0 {
        /* Font is already packed and considered unmodified — do not
         * attempt to repack it, since its original file may not be
         * available anymore on the current FS. */
        return;
    }
    vfont.packedfile = packedfile_new_from_disk(&vfont.filepath);
}

/* -------------------------------------------------------------------- */
/* VFont Selection                                                       */

/// `EditFont` selection fields.
#[derive(Default, Clone)]
pub struct EditFontSel {
    pub len: i32,
    pub selstart: i32,
    pub selend: i32,
    pub pos: i32,
}

/// `BKE_vfont_select_get` — returns `direction` (1/-1) and writes
/// `start`/`end`; `0` = no selection.
pub fn bke_vfont_select_get(ef: Option<&EditFontSel>, is_font_object: bool) -> (i32, i32, i32) {
    let Some(ef) = ef else { return (0, 0, 0) };
    if !is_font_object {
        return (0, 0, 0);
    }

    debug_assert!(ef.len >= 0);
    debug_assert!(ef.selstart >= 0 && ef.selstart <= ef.len + 1);
    debug_assert!(ef.selend >= 0 && ef.selend <= ef.len + 1);
    debug_assert!(ef.pos >= 0 && ef.pos <= ef.len);

    if ef.selstart == 0 {
        return (0, 0, 0);
    }

    let (start, end, direction) = if ef.selstart <= ef.selend {
        (ef.selstart - 1, ef.selend - 1, 1)
    } else {
        (ef.selend, ef.selstart - 2, -1)
    };

    if start == end + 1 {
        return (0, 0, 0);
    }

    debug_assert!(start < end + 1);
    (direction, start, end)
}

/// `BKE_vfont_select_clamp`.
pub fn bke_vfont_select_clamp(ef: &mut EditFontSel) {
    ef.pos = ef.pos.min(ef.len);
    ef.selstart = ef.selstart.min(ef.len + 1);
    ef.selend = ef.selend.min(ef.len);
}

/* -------------------------------------------------------------------- */
/* VFont Clipboard                                                       */

/// `CharInfo` — kerning/width per char (subset used for copy/paste).
#[derive(Clone, Copy, Default)]
pub struct CharInfo {
    pub kern: i16,
    pub mat_nr: i16,
    pub flag: i8,
}

struct VfontClipboard {
    text: Vec<u32>,
    info: Vec<CharInfo>,
    len_utf8: usize,
}

static G_VFONT_CLIPBOARD: Mutex<VfontClipboard> = Mutex::new(VfontClipboard {
    text: Vec::new(),
    info: Vec::new(),
    len_utf8: 0,
});

/// `BKE_vfont_clipboard_free`.
pub fn bke_vfont_clipboard_free() {
    let mut cb = G_VFONT_CLIPBOARD.lock().unwrap();
    cb.text = Vec::new();
    cb.info = Vec::new();
    cb.len_utf8 = 0;
}

/// `BKE_vfont_clipboard_set` — replace stored text (utf32) + info.
pub fn bke_vfont_clipboard_set(text_buf: &[u32], info_buf: &[CharInfo]) {
    let mut cb = G_VFONT_CLIPBOARD.lock().unwrap();
    let len_utf8: usize = text_buf
        .iter()
        .map(|&c| char::from_u32(c).map(|ch| ch.len_utf8()).unwrap_or(0))
        .sum();
    cb.text = text_buf.to_vec();
    cb.info = info_buf.to_vec();
    cb.len_utf8 = len_utf8;
}

/// `BKE_vfont_clipboard_get` — `(text, info, len_utf8, len_utf32)`.
pub fn bke_vfont_clipboard_get() -> (Vec<u32>, Vec<CharInfo>, usize, usize) {
    let cb = G_VFONT_CLIPBOARD.lock().unwrap();
    (cb.text.clone(), cb.info.clone(), cb.len_utf8, cb.text.len())
}
