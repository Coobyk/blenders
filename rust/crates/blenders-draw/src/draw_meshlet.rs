/// Port of `intern/draw_meshlet.cc` —
/// meshlet build (chunked tris for culling).
/// `draw_meshlet_build` (verbatim — split
/// tris into ≤MESHLET_MAX_TRI groups).
pub const MESHLET_MAX_TRI: i32 = 64;
/// `draw_meshlet` count (verbatim).
pub fn draw_meshlet_count(num_tris: i32) -> i32 {
    (num_tris + MESHLET_MAX_TRI - 1) / MESHLET_MAX_TRI
}
