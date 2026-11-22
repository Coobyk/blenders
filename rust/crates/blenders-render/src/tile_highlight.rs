//! Port of `intern/tile_highlight.cc` — highlighted
//! render tile set.

/// `TilesHighlight::Tile` (verbatim — tilerect).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Tile {
    /// `rect` = {xmin, xmax, ymin, ymax}.
    pub rect: [i32; 4],
}
impl Tile {
    /// From RenderResult tilerect.
    pub fn from_rect(rect: [i32; 4]) -> Self {
        Self { rect }
    }
    /// From x/y/width/height (`BLI_rcti_init`).
    pub fn from_xywh(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            rect: [x, x + width, y, y + height],
        }
    }
}

/// `TilesHighlight` (verbatim — set + cached flat list).
#[derive(Default)]
pub struct TilesHighlight {
    /// `highlighted_tiles_set_`.
    pub highlighted: std::collections::HashSet<Tile>,
    /// `cached_highlighted_tiles_`.
    cached: Vec<[i32; 4]>,
    /// `did_tiles_change_`.
    did_change: bool,
}
impl TilesHighlight {
    /// `highlight_tile`.
    pub fn highlight(&mut self, tile: Tile) {
        self.highlighted.insert(tile);
        self.did_change = true;
    }
    /// `unhighlight_tile`.
    pub fn unhighlight(&mut self, tile: Tile) {
        self.highlighted.remove(&tile);
        self.did_change = true;
    }
    /// `clear`.
    pub fn clear(&mut self) {
        self.highlighted.clear();
        self.cached = Vec::new();
    }
    /// `get_all_highlighted_tiles` (verbatim — rebuild
    /// flat cache on change).
    pub fn get_all(&mut self) -> &[[i32; 4]] {
        if self.did_change {
            self.cached.clear();
            for tile in &self.highlighted {
                self.cached.push(tile.rect);
            }
            self.did_change = false;
        }
        &self.cached
    }
}
