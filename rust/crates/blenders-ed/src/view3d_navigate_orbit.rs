/// Port of `view3d_navigate_view_orbit.cc`
/// — viewport orbit (`vieworbit` op:
/// rotate `viewquat` around view axis).
/// `vieworbit_exec` core (verbatim —
/// delta → quat).
pub fn view_orbit(viewquat: &mut [f32; 4], dx: f32, dy: f32) {
    use blenders_math::quaternion::{AxisAngle, Quaternion};
    use blenders_math::Vec3;
    // Rotate around view Z (yaw) then view X (pitch).
    let zquat = Quaternion::from_axis_angle(AxisAngle {
        axis: Vec3::new(0.0, 0.0, 1.0),
        angle: -dx,
    });
    let xquat = Quaternion::from_axis_angle(AxisAngle {
        axis: Vec3::new(1.0, 0.0, 0.0),
        angle: -dy,
    });
    let cur = Quaternion::new(viewquat[0], viewquat[1], viewquat[2], viewquat[3]);
    let q = xquat * cur * zquat;
    *viewquat = [q.w, q.x, q.y, q.z];
}
