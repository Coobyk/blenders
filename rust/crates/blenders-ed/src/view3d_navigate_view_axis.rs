/// Port of `view3d_navigate_view_axis.cc`
/// — snap view to axis (front/back/top/
/// left/right + diagonal).
/// `view_axis` (verbatim — axis → viewquat).
pub fn view_axis(view_axis: i32) -> [f32; 4] {
    use blenders_math::quaternion::{AxisAngle, Quaternion};
    use blenders_math::Vec3;
    let q = match view_axis {
        // RV3D_VIEW_FRONT
        0 => Quaternion::from_axis_angle(AxisAngle {
            axis: Vec3::new(1.0, 0.0, 0.0),
            angle: 0.0,
        }),
        // RV3D_VIEW_BACK
        1 => Quaternion::from_axis_angle(AxisAngle {
            axis: Vec3::new(0.0, 0.0, 1.0),
            angle: std::f32::consts::PI,
        }),
        // RV3D_VIEW_TOP
        2 => Quaternion::from_axis_angle(AxisAngle {
            axis: Vec3::new(1.0, 0.0, 0.0),
            angle: std::f32::consts::FRAC_PI_2,
        }),
        // RV3D_VIEW_BOTTOM
        3 => Quaternion::from_axis_angle(AxisAngle {
            axis: Vec3::new(1.0, 0.0, 0.0),
            angle: -std::f32::consts::FRAC_PI_2,
        }),
        // RV3D_VIEW_LEFT
        4 => Quaternion::from_axis_angle(AxisAngle {
            axis: Vec3::new(0.0, 0.0, 1.0),
            angle: std::f32::consts::FRAC_PI_2,
        }),
        // RV3D_VIEW_RIGHT
        5 => Quaternion::from_axis_angle(AxisAngle {
            axis: Vec3::new(0.0, 0.0, 1.0),
            angle: -std::f32::consts::FRAC_PI_2,
        }),
        _ => Quaternion::new(1.0, 0.0, 0.0, 0.0),
    };
    [q.w, q.x, q.y, q.z]
}
