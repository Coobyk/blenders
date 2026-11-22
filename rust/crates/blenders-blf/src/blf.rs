//! Port of `intern/blf.cc` — global font table +
//! load/unload/refcount.

/// `BLF_MAX_FONT` verbatim.
pub const BLF_MAX_FONT: usize = 32;
/// `BLF_DEFAULT`/`BLF_MONO`/`BLF_MONO_RENDER` flags verbatim.
pub const BLF_DEFAULT: i32 = 1 << 0;
/// `FontFlags` verbatim.
pub const BLF_BATCH_DRAW: i32 = 1 << 1;
pub const BLF_KERNING_DEFAULT: i32 = 1 << 2;
/// `blf_mono_font`/`blf_mono_font_render` verbatim sentinels.
pub const BLF_NO_FONT: i32 = -1;

/// `FontBLF` verbatim fields.
#[derive(Default)]
pub struct FontBlf {
    /// `filepath`.
    pub filepath: Option<String>,
    /// `mem_name`.
    pub mem_name: Option<String>,
    /// `reference_count`.
    pub reference_count: i32,
    /// `flags`.
    pub flags: i32,
    /// `size`/`aspect`/`pos`/`color`.
    pub size: f32,
    pub aspect: [f32; 3],
    pub pos: [f32; 3],
    pub color: [f32; 4],
}
/// `global_font` table (verbatim — fixed slots).
#[derive(Default)]
pub struct GlobalFonts {
    /// `global_font[]`.
    pub fonts: Vec<Option<FontBlf>>,
    /// `blf_mono_font`/`blf_mono_font_render`.
    pub mono_font: i32,
    pub mono_font_render: i32,
}
impl GlobalFonts {
    /// `BLF_init` (verbatim — MAX slots).
    pub fn new() -> Self {
        Self {
            fonts: vec![None; BLF_MAX_FONT],
            mono_font: BLF_NO_FONT,
            mono_font_render: BLF_NO_FONT,
        }
    }
    /// `blf_get` bounds-check.
    pub fn get(&self, fontid: i32) -> Option<&FontBlf> {
        if fontid >= 0 && (fontid as usize) < BLF_MAX_FONT {
            self.fonts[fontid as usize].as_ref()
        } else {
            None
        }
    }
    /// `blf_search_available` (verbatim — first None).
    pub fn search_available(&self) -> i32 {
        for i in 0..BLF_MAX_FONT {
            if self.fonts[i].is_none() {
                return i as i32;
            }
        }
        -1
    }
    /// `blf_search_by_filepath` (verbatim — path cmp).
    pub fn search_by_filepath(&self, filepath: &str) -> i32 {
        for i in 0..BLF_MAX_FONT {
            if let Some(f) = &self.fonts[i] {
                if let Some(fp) = &f.filepath {
                    if fp == filepath {
                        return i as i32;
                    }
                }
            }
        }
        -1
    }
    /// `blf_search_by_mem_name` (verbatim).
    pub fn search_by_mem_name(&self, mem_name: &str) -> i32 {
        for i in 0..BLF_MAX_FONT {
            if let Some(f) = &self.fonts[i] {
                if let Some(mn) = &f.mem_name {
                    if mn == mem_name {
                        return i as i32;
                    }
                }
            }
        }
        -1
    }
    /// `BLF_is_loaded`/`_mem`/`_id` (verbatim).
    pub fn is_loaded(&self, filepath: &str) -> bool {
        self.search_by_filepath(filepath) >= 0
    }
    pub fn is_loaded_mem(&self, name: &str) -> bool {
        self.search_by_mem_name(name) >= 0
    }
    pub fn is_loaded_id(&self, fontid: i32) -> bool {
        self.get(fontid).is_some()
    }
    /// `BLF_load` (verbatim — existing → ++ref else new slot).
    pub fn load(&mut self, filepath: &str) -> i32 {
        let i = self.search_by_filepath(filepath);
        if i >= 0 {
            if let Some(f) = self.fonts[i as usize].as_mut() {
                f.reference_count += 1;
            }
            return i;
        }
        self.load_unique(filepath)
    }
    /// `BLF_load_unique` (verbatim — slot + refcount 1).
    pub fn load_unique(&mut self, filepath: &str) -> i32 {
        let i = self.search_available();
        if i == -1 {
            return -1;
        }
        let mut font = FontBlf {
            filepath: Some(filepath.to_string()),
            ..Default::default()
        };
        font.reference_count = 1;
        self.fonts[i as usize] = Some(font);
        i
    }
    /// `BLF_unload`/`BLF_unload_id` (verbatim — --ref, free at 0).
    pub fn unload_id(&mut self, fontid: i32) -> bool {
        if fontid < 0 || fontid as usize >= BLF_MAX_FONT {
            return false;
        }
        if let Some(f) = self.fonts[fontid as usize].as_mut() {
            f.reference_count -= 1;
            if f.reference_count <= 0 {
                self.fonts[fontid as usize] = None;
            }
            return true;
        }
        false
    }
    /// `BLF_enable`/`BLF_disable` (verbatim — flags ±).
    pub fn enable(&mut self, fontid: i32, flag: i32) {
        if let Some(f) = self.get_mut(fontid) {
            f.flags |= flag;
        }
    }
    /// `BLF_disable`.
    pub fn disable(&mut self, fontid: i32, flag: i32) {
        if let Some(f) = self.get_mut(fontid) {
            f.flags &= !flag;
        }
    }
    /// `blf_get` mutable.
    pub fn get_mut(&mut self, fontid: i32) -> Option<&mut FontBlf> {
        if fontid >= 0 && (fontid as usize) < BLF_MAX_FONT {
            self.fonts[fontid as usize].as_mut()
        } else {
            None
        }
    }
}
