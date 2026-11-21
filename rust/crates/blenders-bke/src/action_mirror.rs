//! Port of `intern/action_mirror.cc` — `BKE_action_flip_with_pose`
//! (flip an action's pose-bone F-Curves across the X axis).
//!
//! `Object`/`bPoseChannel`/`FCurve`/`BezTriple` are `usize`/DNA handles;
//! FCurve evaluation + keyed-frame collection are injected closures
//! (`evaluate_fcurve_only_curve`, `BKE_fcurves_calc_keyed_frames` —
//! PORT_PENDING animrig).

use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

/// `FCurve_KeyCache` — per-channel cached eval + bezt lookup.
#[derive(Default)]
pub struct FCurveKeyCache {
    /// `FCurve *fcurve` — usize::MAX = ignore.
    pub fcurve: usize,
    /// `fcurve_eval` — evaluated values per `keyed_frames`.
    pub fcurve_eval: Vec<f32>,
    /// `bezt_array` — `BezTriple` handle per keyed frame (usize::MAX = none).
    pub bezt_array: Vec<usize>,
}

/// All transformation channels the flip touches (`FCURVE_CHANNEL_LEN`).
pub struct FCurvePChanCache {
    pub loc: [FCurveKeyCache; 3],
    pub eul: [FCurveKeyCache; 3],
    pub quat: [FCurveKeyCache; 4],
    pub rot_axis: [FCurveKeyCache; 3],
    pub rot_angle: FCurveKeyCache,
    pub scale: [FCurveKeyCache; 3],
    pub rotmode: FCurveKeyCache,
    pub bbone_curveinx: FCurveKeyCache,
    pub bbone_curveoutz: FCurveKeyCache,
    pub bbone_curveinx_alt: FCurveKeyCache,
    pub bbone_curveouty: FCurveKeyCache,
    pub bbone_scaleinx: FCurveKeyCache,
    pub bbone_scaleiny: FCurveKeyCache,
    pub bbone_scaleinz: FCurveKeyCache,
    pub bbone_scaleoutx: FCurveKeyCache,
    pub bbone_scaleouty: FCurveKeyCache,
    pub bbone_scaleoutz: FCurveKeyCache,
    pub bbone_easeinx: FCurveKeyCache,
    pub bbone_easeiny: FCurveKeyCache,
    pub bbone_easeoutx: FCurveKeyCache,
    pub bbone_easeouty: FCurveKeyCache,
    pub bbone_rollinx: FCurveKeyCache,
    pub bbone_rolliny: FCurveKeyCache,
    pub bbone_rolloutx: FCurveKeyCache,
    pub bbone_rollouty: FCurveKeyCache,
}

impl Default for FCurvePChanCache {
    fn default() -> Self {
        Self {
            loc: Default::default(),
            eul: Default::default(),
            quat: Default::default(),
            rot_axis: Default::default(),
            rot_angle: Default::default(),
            scale: Default::default(),
            rotmode: Default::default(),
            bbone_curveinx: Default::default(),
            bbone_curveoutz: Default::default(),
            bbone_curveinx_alt: Default::default(),
            bbone_curveouty: Default::default(),
            bbone_scaleinx: Default::default(),
            bbone_scaleiny: Default::default(),
            bbone_scaleinz: Default::default(),
            bbone_scaleoutx: Default::default(),
            bbone_scaleouty: Default::default(),
            bbone_scaleoutz: Default::default(),
            bbone_easeinx: Default::default(),
            bbone_easeiny: Default::default(),
            bbone_easeoutx: Default::default(),
            bbone_easeouty: Default::default(),
            bbone_rollinx: Default::default(),
            bbone_rolliny: Default::default(),
            bbone_rolloutx: Default::default(),
            bbone_rollouty: Default::default(),
        }
    }
}

/// `action_flip_pchan_cache_init` — fill `fcurve_eval` + `bezt_array` for
/// `keyed_frames`. `eval_fcurve` = `evaluate_fcurve_only_curve`
/// (PORT_PENDING); `bezt_at` maps a frame to its `BezTriple` handle or none.
pub fn action_flip_pchan_cache_init(
    fkc: &mut FCurveKeyCache,
    keyed_frames: &[f32],
    eval_fcurve: &mut dyn FnMut(usize, f32) -> f32,
    bezt_at: &mut dyn FnMut(usize, i32) -> usize,
) {
    debug_assert!(fkc.fcurve != usize::MAX);
    fkc.fcurve_eval = keyed_frames
        .iter()
        .map(|&evaltime| eval_fcurve(fkc.fcurve, evaltime))
        .collect();
    fkc.bezt_array = keyed_frames
        .iter()
        .map(|&f| bezt_at(fkc.fcurve, (f + 0.5).floor() as i32))
        .collect();
}

/// `BKE_pchan_to_mat4` subset — build the channel matrix from
/// loc/eul-or-quat-or-axisangle/scale. `rotmode` uses
/// `ROT_MODE_QUAT = 0`, euler orders 1-6, `ROT_MODE_AXISANGLE = -1`.
pub fn bke_pchan_to_mat4(
    loc: [f32; 3],
    eul: [f32; 3],
    quat: [f32; 4],
    rot_axis: [f32; 3],
    rot_angle: f32,
    scale: [f32; 3],
    rotmode: i32,
) -> Mat4 {
    let m = if rotmode == -1 {
        /* axis-angle → mat4 */
        blenders_math::quaternion::axis_angle_to_mat4(
            Vec3::new(rot_axis[0], rot_axis[1], rot_axis[2]), rot_angle)
    } else if rotmode == 0 {
        /* quat → mat4 */
        blenders_math::quaternion::Quaternion::new(quat[0], quat[1], quat[2], quat[3]).to_mat4()
    } else {
        /* euler (order stored in rotmode) → mat4 */
        let order = match rotmode {
            2 => blenders_math::euler::EulerOrder::Xzy,
            3 => blenders_math::euler::EulerOrder::Yxz,
            4 => blenders_math::euler::EulerOrder::Yzx,
            5 => blenders_math::euler::EulerOrder::Zxy,
            6 => blenders_math::euler::EulerOrder::Zyx,
            _ => blenders_math::euler::EulerOrder::Xyz,
        };
        Mat4::from(blenders_math::euler::Euler3::new(
            Vec3::new(eul[0], eul[1], eul[2]), order).to_mat3())
    };
    let mut m = m;
    /* apply scale columns then set location */
    for c in 0..3 {
        m[c] = m[c] * scale[c];
    }
    m[3] = blenders_math::vector::Vec4::new(loc[0], loc[1], loc[2], 1.0);
    m
}

/// `action_flip_pchan`'s per-frame transform — the full chain:
/// read cached values → `BKE_pchan_to_mat4` → pose space → X flip → back to
/// bone space (flipped bone's arm_mat_inv) → X-ortho fix.
///
/// `arm_mat`/`arm_mat_inv` = rest matrices of the source/flipped bone.
pub fn action_flip_frame_transform(
    chan_mat: Mat4,
    arm_mat: Mat4,
    arm_mat_inv: Mat4,
    arm_x_col: Vec3,
    pchan_flip_exists: bool,
) -> Mat4 {
    let flip_mtx = {
        let mut m = Mat4::IDENTITY;
        m[0][0] = -1.0;
        m
    };
    let mut chan_mat = arm_mat * chan_mat;
    chan_mat = chan_mat * flip_mtx;
    chan_mat = flip_mtx * chan_mat;
    chan_mat = arm_mat_inv * chan_mat;

    let is_x_axis_orthogonal = !pchan_flip_exists
        && arm_x_col.dot(Vec3::new(1.0, 0.0, 0.0)).abs() <= 1e-6;
    if is_x_axis_orthogonal {
        let extra_mat = Mat4::from_cols(
            blenders_math::vector::Vec4::new(-1.0, 0.0, 0.0, 0.0),
            blenders_math::vector::Vec4::new(0.0, 1.0, 0.0, 0.0),
            blenders_math::vector::Vec4::new(0.0, 0.0, -1.0, 0.0),
            blenders_math::vector::Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        chan_mat = extra_mat * chan_mat;
    }
    chan_mat
}
