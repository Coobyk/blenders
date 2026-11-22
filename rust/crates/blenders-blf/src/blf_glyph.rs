//! Port of `intern/blf_glyph.cc` — glyph cache,
//! index lookup, advance/kerning math.

/// `GlyphBLF` verbatim fields.
#[derive(Default, Clone)]
pub struct GlyphBlf {
    /// `c` (unicode).
    pub c: u32,
    /// `advance_x`.
    pub advance_x: f32,
    /// `lsb`/`rsb` bearings.
    pub lsb: f32,
    pub rsb: f32,
    /// `box` bounds {xmin,xmax,ymin,ymax}.
    pub bbox: [f32; 4],
    /// `bitmap` glyph raster pitch/w/h.
    pub pitch: i32,
    pub dims: [i32; 2],
}
/// `GlyphCacheBLF` (verbatim — HashMap buckets by unicode).
#[derive(Default)]
pub struct GlyphCache {
    /// `glyphs` flat map.
    pub glyphs: std::collections::HashMap<u32, GlyphBlf>,
}
impl GlyphCache {
    /// `blf_glyph_cache_find` (verbatim).
    pub fn find(&self, c: u32) -> Option<&GlyphBlf> {
        self.glyphs.get(&c)
    }
    /// `blf_glyph_cache_add` (verbatim — insert once).
    pub fn add(&mut self, g: GlyphBlf) {
        self.glyphs.entry(g.c).or_insert(g);
    }
}
/// `blf_font_width_from_advance` — kern+advance per pair
/// (verbatim).
pub fn glyph_advance_with_kerning(prev: Option<&GlyphBlf>, cur: &GlyphBlf, kern: f32) -> f32 {
    let mut adv = cur.advance_x;
    if prev.is_some() {
        adv += kern;
    }
    adv
}
