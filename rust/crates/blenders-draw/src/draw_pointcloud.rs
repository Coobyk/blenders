/// Port of `intern/draw_pointcloud.cc` —
/// point cloud → GPU points prim.
/// `drw_pointcloud` (verbatim — positions
/// pass-through).
pub fn drw_pointcloud(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    positions.to_vec()
}
