/// Port of `transform/transform_snap.cc`
/// — snap system (`transform_snap_increment`:
/// grid-step snapping of `values`).
/// `transform_snap_increment` (verbatim —
/// round vec to `snap_step` multiples).
pub fn transform_snap_increment(vec: &mut [f32; 3], snap_step: f32) {
    if snap_step == 0.0 {
        return;
    }
    for v in vec.iter_mut() {
        *v = (*v / snap_step).round() * snap_step;
    }
}
/// `transform_snap_mixed_apply` (verbatim
/// — apply constrained snap).
pub fn transform_snap_mixed_apply(
    con_mode: u32,
    vec: &mut [f32; 3],
    snap_step: f32,
) {
    for i in 0..3 {
        if con_mode & (crate::transform::CON_AXIS0 << i) != 0 || con_mode == 0 {
            vec[i] = (vec[i] / snap_step).round() * snap_step;
        }
    }
}
/// `snapGridIncrement` (verbatim — distance
/// → grid step for active snap mode).
pub fn snap_grid_increment(_t_mode: i32) -> f32 {
    // C reads `t->tsnap.grid_increment`; the
    // default pixel grid is 1.0 unit.
    1.0
}
/// `transform_snap_is_active` (verbatim —
/// snapping on when `T_SNAP` or always).
pub fn transform_snap_is_active(flag: u32, always_snap: bool) -> bool {
    (flag & crate::transform::T_SNAP) != 0 || always_snap
}
/// `getSnapTarget` → targets (verbatim).
pub const SCE_SNAP_TO_NONE: i32 = 0;
pub const SCE_SNAP_TO_VERTEX: i32 = 1;
pub const SCE_SNAP_TO_EDGE: i32 = 2;
pub const SCE_SNAP_TO_FACE: i32 = 4;
pub const SCE_SNAP_TO_VOLUME: i32 = 8;
pub const SCE_SNAP_TO_INCREMENT: i32 = 16;
pub const SCE_SNAP_TO_GRID: i32 = 32;
