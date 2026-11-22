//! Port of `intern/blf_default.cc` — default font
//! id + draw size.

/// `global_font_default`/`global_font_size` verbatim
/// (UI_DEFAULT_TEXT_POINTS=11).
#[derive(Default)]
pub struct BlfDefault {
    /// `global_font_default` (-1 unset).
    pub fontid: i32,
    /// `global_font_size`.
    pub size: f32,
}
impl BlfDefault {
    /// ctor (verbatim — default -1 / 11.0).
    pub fn new() -> Self {
        Self {
            fontid: -1,
            size: 11.0,
        }
    }
    /// `BLF_default_set` (verbatim — -1 or loaded id).
    pub fn set(&mut self, fontid: i32, is_loaded: bool) {
        if fontid == -1 || is_loaded {
            self.fontid = fontid;
        }
    }
    /// `BLF_default_size`.
    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }
}
