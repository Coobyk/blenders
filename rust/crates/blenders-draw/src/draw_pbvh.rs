/// Port of `intern/draw_pbvh.cc` — PBVH
/// → GPU extract (sculpt nodes, masks,
/// face-sets).
/// `DRW_pbvh` (verbatim — per-node batch).
pub struct DrwPbvh {
    /// `nodes` (pbvh node batches).
    pub nodes: Vec<usize>,
}
