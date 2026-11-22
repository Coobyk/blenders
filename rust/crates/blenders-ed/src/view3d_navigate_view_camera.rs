/// Port of `view3d_navigate_view_camera.cc`
/// — camera-view ops (numpad 0, orbit
/// camera).
/// `view_camera` (verbatim — toggle persp
/// + set viewquat from camera).
pub fn view_camera(is_persp: &mut bool) {
    *is_persp = !*is_persp;
}
