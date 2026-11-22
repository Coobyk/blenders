/// Port of `view3d_navigate_view_roll.cc`
/// — viewport roll (`viewroll` op: rotate
/// `viewquat` around forward).
/// `viewroll_exec` core (verbatim).
pub fn view_roll(viewquat: &mut [f32; 4], delta: f32) {
    use blenders_math::quaternion::{AxisAngle, Quaternion};
    use blenders_math::Vec3;
    let rq = Quaternion::from_axis_angle(AxisAngle {
        axis: Vec3::new(0.0, 0.0, 1.0),
        angle: delta,
    });
    let q = rq * Quaternion::new(viewquat[0], viewquat[1], viewquat[2], viewquat[3]);
    *viewquat = [q.w, q.x, q.y, q.z];
}
