//! Port of `xr/intern/wm_xr_draw.cc` — pose→matrix
//! conversion and per-view view/proj matrix creation.

/// `GHOST_XrPose` verbatim.
#[derive(Clone, Copy, Default)]
pub struct GhostXrPose {
    /// `orientation_quat` (w,x,y,z).
    pub orientation_quat: [f32; 4],
    /// `position`.
    pub position: [f32; 3],
}
/// `GHOST_XrFov` verbatim.
#[derive(Clone, Copy, Default)]
pub struct GhostXrFov {
    /// `angle_left`/`angle_right`/`angle_up`/`angle_down`.
    pub angle_left: f32,
    pub angle_right: f32,
    pub angle_up: f32,
    pub angle_down: f32,
}
/// `XR_SESSION_*` flags verbatim.
pub const XR_SESSION_USE_POSITION_TRACKING: i32 = 1 << 0;
pub const XR_SESSION_USE_ABSOLUTE_TRACKING: i32 = 1 << 1;
/// `XR_CONTROLLER_DRAW_*` verbatim.
pub const XR_CONTROLLER_DRAW_DARK_RAY: i32 = 0;
pub const XR_CONTROLLER_DRAW_LIGHT_RAY: i32 = 1;
/// `XR_BASE_POSE_*` verbatim.
pub const XR_BASE_POSE_SCENE_CAMERA: i32 = 0;
pub const XR_BASE_POSE_OBJECT: i32 = 1;
pub const XR_BASE_POSE_CUSTOM: i32 = 2;

/// `quat_to_mat4` (normalized w,x,y,z).
pub fn quat_to_mat4(q: [f32; 4]) -> [[f32; 4]; 4] {
    let q0 = q[0] * 2.0;
    let q1 = q[1] * 2.0;
    let q2 = q[2] * 2.0;
    let q3 = q[3] * 2.0;
    let qda = q0 * q[1];
    let qdb = q0 * q[2];
    let qdc = q0 * q[3];
    let qaa = q1 * q[1];
    let qab = q1 * q[2];
    let qac = q1 * q[3];
    let qbb = q2 * q[2];
    let qbc = q2 * q[3];
    let qcc = q3 * q[3];
    [
        [1.0 - (qbb + qcc), qda + qbc, qdb - qac, 0.0],
        [qda - qbc, 1.0 - (qaa + qcc), qdc + qab, 0.0],
        [qdb + qac, qdc - qab, 1.0 - (qaa + qbb), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// `invert_qt_qt_normalized`.
pub fn invert_qt(q: [f32; 4]) -> [f32; 4] {
    [q[0], -q[1], -q[2], -q[3]]
}
fn translate_m4(m: &mut [[f32; 4]; 4], x: f32, y: f32, z: f32) {
    m[3][0] += m[0][0] * x + m[1][0] * y + m[2][0] * z;
    m[3][1] += m[0][1] * x + m[1][1] * y + m[2][1] * z;
    m[3][2] += m[0][2] * x + m[1][2] * y + m[2][2] * z;
}
pub fn mul_m4(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut o = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            o[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    o
}

/// `wm_xr_pose_to_mat` (verbatim — quat mat + pos row).
pub fn wm_xr_pose_to_mat(pose: &GhostXrPose) -> [[f32; 4]; 4] {
    let mut m = quat_to_mat4(pose.orientation_quat);
    m[3] = [pose.position[0], pose.position[1], pose.position[2], 1.0];
    m
}

/// `wm_xr_pose_scale_to_mat` (verbatim — scale rot rows).
pub fn wm_xr_pose_scale_to_mat(pose: &GhostXrPose, scale: f32) -> [[f32; 4]; 4] {
    let mut m = wm_xr_pose_to_mat(pose);
    assert!(scale > 0.0);
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] *= scale;
        }
    }
    m
}

/// `wm_xr_pose_to_imat` (verbatim — inv quat + translate).
pub fn wm_xr_pose_to_imat(pose: &GhostXrPose) -> [[f32; 4]; 4] {
    let iquat = invert_qt(pose.orientation_quat);
    let mut m = quat_to_mat4(iquat);
    translate_m4(&mut m, -pose.position[0], -pose.position[1], -pose.position[2]);
    m
}

/// `wm_xr_pose_scale_to_imat` (verbatim — inv + 1/scale).
pub fn wm_xr_pose_scale_to_imat(pose: &GhostXrPose, scale: f32) -> [[f32; 4]; 4] {
    let iquat = invert_qt(pose.orientation_quat);
    let mut m = quat_to_mat4(iquat);
    assert!(scale > 0.0);
    let s = 1.0 / scale;
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] *= s;
        }
    }
    translate_m4(&mut m, -pose.position[0], -pose.position[1], -pose.position[2]);
    m
}

/// `wm_xr_draw_matrices_create` (verbatim — eye inv ∓
/// tracking offsets × base inv × nav inv; fov proj).
pub fn wm_xr_draw_matrices_create(
    eye_pose: &GhostXrPose,
    local_pose_position: [f32; 3],
    eye_position_ofs: [f32; 3],
    base_pose: &GhostXrPose,
    base_scale: f32,
    nav_pose: &GhostXrPose,
    viewer_scale: f32,
    use_position_tracking: bool,
    use_absolute_tracking: bool,
    fov: &GhostXrFov,
    clip_start: f32,
    clip_end: f32,
) -> ([[f32; 4]; 4], [[f32; 4]; 4]) {
    let mut eye = *eye_pose;
    if !use_position_tracking {
        for i in 0..3 {
            eye.position[i] -= local_pose_position[i];
        }
    }
    if !use_absolute_tracking {
        for i in 0..3 {
            eye.position[i] -= eye_position_ofs[i];
        }
    }
    let eye_inv = wm_xr_pose_to_imat(&eye);
    let base_inv = wm_xr_pose_scale_to_imat(base_pose, base_scale);
    let nav_inv = wm_xr_pose_scale_to_imat(nav_pose, viewer_scale);
    let m = mul_m4(&eye_inv, &base_inv);
    let viewmat = mul_m4(&m, &nav_inv);
    /* `perspective_m4_fov` (frustum corners). */
    let near = clip_start;
    let far = clip_end;
    let left = near * fov.angle_left.tan();
    let right = near * fov.angle_right.tan();
    let top = near * fov.angle_up.tan();
    let bottom = near * fov.angle_down.tan();
    let mut projmat = [[0.0f32; 4]; 4];
    projmat[0][0] = 2.0 * near / (right - left);
    projmat[1][1] = 2.0 * near / (top - bottom);
    projmat[2][0] = (right + left) / (right - left);
    projmat[2][1] = (top + bottom) / (top - bottom);
    projmat[2][2] = -(far + near) / (far - near);
    projmat[2][3] = -1.0;
    projmat[3][2] = -(2.0 * far * near) / (far - near);
    (viewmat, projmat)
}
