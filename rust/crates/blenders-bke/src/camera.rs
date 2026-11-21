//! Port of `intern/camera.cc` — `CameraParams` viewplane/projection
//! math, camera border/frame computations. ID/scene accesses are
//! plain value parameters.

use blenders_bli::rct::{
    rctf_rotate_expand_around, rctf_size_x, rctf_size_y, rctf_translate, Rctf, Rcti,
};
use blenders_math::matrix::projection;
use blenders_math::matrix::Mat4;
use blenders_math::vector::{Vec2, Vec3};

/// `CAMERA_SENSOR_FIT_AUTO`.
pub const CAMERA_SENSOR_FIT_AUTO: i32 = 0;
/// `CAMERA_SENSOR_FIT_HOR`.
pub const CAMERA_SENSOR_FIT_HOR: i32 = 1;
/// `CAMERA_SENSOR_FIT_VERT`.
pub const CAMERA_SENSOR_FIT_VERT: i32 = 2;
/// `DEFAULT_SENSOR_WIDTH`.
pub const DEFAULT_SENSOR_WIDTH: f32 = 36.0;
/// `DEFAULT_SENSOR_HEIGHT`.
pub const DEFAULT_SENSOR_HEIGHT: f32 = 24.0;
/// `CAM_ORTHO`.
pub const CAM_ORTHO: i32 = 1;
/// `CAM_SHOWPASSEPARTOUT`.
pub const CAM_SHOWPASSEPARTOUT: i32 = 1 << 0;
/// `V3D_SHOW_CAMERA_PASSEPARTOUT`.
pub const V3D_SHOW_CAMERA_PASSEPARTOUT: i32 = 1 << 0;
/// `V3D_RENDER_BORDER`.
pub const V3D_RENDER_BORDER: i32 = 1 << 10;
/// `RV3D_CAMOB`.
pub const RV3D_CAMOB: i32 = 2;
/// `RV3D_FLIP_X`.
pub const RV3D_FLIP_X: i32 = 1 << 4;
/// `R_BORDER`.
pub const R_BORDER: i32 = 0x20000;
/// `OB_CAMERA`.
pub const OB_CAMERA: i16 = 11;

/// `CameraDOFSettings` fields used.
#[derive(Clone, Default)]
pub struct CameraDof {
    /// `focus_object` handle (0 = none).
    pub focus_object: usize,
    /// `focus_subtarget`.
    pub focus_subtarget: String,
    /// `focus_distance`.
    pub focus_distance: f32,
}

/// `Camera` fields used by the ported math.
#[derive(Clone)]
pub struct CameraData {
    /// `lens`.
    pub lens: f32,
    /// `sensor_x`.
    pub sensor_x: f32,
    /// `sensor_y`.
    pub sensor_y: f32,
    /// `sensor_fit`.
    pub sensor_fit: i32,
    /// `shiftx`, `shifty`.
    pub shiftx: f32,
    /// See `shiftx`.
    pub shifty: f32,
    /// `clip_start`, `clip_end`.
    pub clip_start: f32,
    /// See `clip_start`.
    pub clip_end: f32,
    /// `ortho_scale`.
    pub ortho_scale: f32,
    /// `type` (CAM_ORTHO etc).
    pub cam_type: i32,
    /// `flag`.
    pub flag: i32,
    /// `passepartalpha`.
    pub passepartalpha: f32,
    /// `dof`.
    pub dof: CameraDof,
}
impl Default for CameraData {
    fn default() -> Self {
        Self {
            lens: 50.0,
            sensor_x: DEFAULT_SENSOR_WIDTH,
            sensor_y: DEFAULT_SENSOR_HEIGHT,
            sensor_fit: CAMERA_SENSOR_FIT_AUTO,
            shiftx: 0.0,
            shifty: 0.0,
            clip_start: 0.1,
            clip_end: 1000.0,
            ortho_scale: 6.0,
            cam_type: 0,
            flag: 0,
            passepartalpha: 1.0,
            dof: CameraDof::default(),
        }
    }
}

/// `BKE_camera_object_dof_distance` — `ob_to_world` is the camera
/// object's matrix; `focus_world_pos` = focus object's world location
/// (or pose-bone world location when a subtarget is set).
pub fn bke_camera_object_dof_distance(
    ob_type: i16,
    cam: &CameraData,
    ob_to_world: &Mat4,
    focus_world_pos: Option<Vec3>,
) -> f32 {
    if ob_type != OB_CAMERA {
        return 0.0;
    }
    if cam.dof.focus_object != 0 {
        let view_dir = Vec3::new(ob_to_world[2][0], ob_to_world[2][1], ob_to_world[2][2])
            .normalized();
        let fp = focus_world_pos.unwrap_or(Vec3::ZERO);
        let dof_dir = Vec3::new(ob_to_world[3][0], ob_to_world[3][1], ob_to_world[3][2]) - fp;
        return view_dir.dot(dof_dir).abs().max(1e-5);
    }
    cam.dof.focus_distance.max(1e-5)
}

/// `BKE_camera_sensor_size`.
pub fn bke_camera_sensor_size(sensor_fit: i32, sensor_x: f32, sensor_y: f32) -> f32 {
    if sensor_fit == CAMERA_SENSOR_FIT_VERT {
        sensor_y
    } else {
        sensor_x
    }
}

/// `BKE_camera_sensor_fit`.
pub fn bke_camera_sensor_fit(sensor_fit: i32, sizex: f32, sizey: f32) -> i32 {
    if sensor_fit == CAMERA_SENSOR_FIT_AUTO {
        if sizex >= sizey {
            return CAMERA_SENSOR_FIT_HOR;
        }
        return CAMERA_SENSOR_FIT_VERT;
    }
    sensor_fit
}

/// `BKE_camera_frame_size`.
pub fn bke_camera_frame_size(winx: f32, winy: f32, frame_aspect: f32) -> Vec2 {
    let frame_x = winx.min(winy / frame_aspect);
    Vec2::new(frame_x, frame_x * frame_aspect)
}

/// `CameraParams`.
#[derive(Clone)]
pub struct CameraParams {
    /// `is_ortho`.
    pub is_ortho: bool,
    /// `lens`.
    pub lens: f32,
    /// `ortho_scale`.
    pub ortho_scale: f32,
    /// `zoom`.
    pub zoom: f32,
    /// `sensor_x`, `sensor_y`, `sensor_fit`.
    pub sensor_x: f32,
    /// See `sensor_x`.
    pub sensor_y: f32,
    /// See `sensor_x`.
    pub sensor_fit: i32,
    /// `offsetx`, `offsety`.
    pub offsetx: f32,
    /// See `offsetx`.
    pub offsety: f32,
    /// `shiftx`, `shifty`.
    pub shiftx: f32,
    /// See `shiftx`.
    pub shifty: f32,
    /// `clip_start`, `clip_end`.
    pub clip_start: f32,
    /// See `clip_start`.
    pub clip_end: f32,
    /// `use_aspect_override`/`aspect_override`.
    pub use_aspect_override: bool,
    /// See above.
    pub aspect_override: f32,
    /// `roll` (camroll).
    pub roll: f32,
    /// `is_flipped_x`.
    pub is_flipped_x: bool,
    /// `ycor` (out).
    pub ycor: f32,
    /// `viewplane` (out).
    pub viewplane: Rctf,
    /// `viewdx`, `viewdy` (out).
    pub viewdx: f32,
    /// See `viewdx`.
    pub viewdy: f32,
    /// `winmat` (out).
    pub winmat: Mat4,
}
impl Default for CameraParams {
    fn default() -> Self {
        Self {
            is_ortho: false,
            lens: 50.0,
            ortho_scale: 6.0,
            zoom: 1.0,
            sensor_x: DEFAULT_SENSOR_WIDTH,
            sensor_y: DEFAULT_SENSOR_HEIGHT,
            sensor_fit: CAMERA_SENSOR_FIT_AUTO,
            offsetx: 0.0,
            offsety: 0.0,
            shiftx: 0.0,
            shifty: 0.0,
            clip_start: 0.1,
            clip_end: 1000.0,
            use_aspect_override: false,
            aspect_override: 1.0,
            roll: 0.0,
            is_flipped_x: false,
            ycor: 1.0,
            viewplane: Rctf::default(),
            viewdx: 0.0,
            viewdy: 0.0,
            winmat: Mat4::IDENTITY,
        }
    }
}

/// `BKE_camera_params_init`.
pub fn bke_camera_params_init(params: &mut CameraParams) {
    *params = CameraParams::default();
}

/// `BKE_camera_viewplane_offset_transform` — rotate offset by roll;
/// negate x when flipped.
pub fn bke_camera_viewplane_offset_transform(
    roll: f32,
    is_flipped_x: bool,
    offset: Vec2,
) -> Vec2 {
    let (s, c) = roll.sin_cos();
    let mut result = Vec2::new(c * offset.x - s * offset.y, s * offset.x + c * offset.y);
    if is_flipped_x {
        result.x = -result.x;
    }
    result
}

/// `BKE_camera_params_compute_viewplane`.
pub fn bke_camera_params_compute_viewplane(
    params: &mut CameraParams,
    winx: i32,
    winy: i32,
    aspx: f32,
    aspy: f32,
) {
    params.ycor = aspy / aspx;
    let winy_cor = params.ycor * winy as f32;
    let frame_aspect = if params.use_aspect_override {
        params.aspect_override
    } else {
        winy_cor / winx as f32
    };
    let frame_size = bke_camera_frame_size(winx as f32, winy_cor, frame_aspect);

    let mut pixsize = if params.is_ortho {
        params.ortho_scale
    } else {
        let sensor_size =
            bke_camera_sensor_size(params.sensor_fit, params.sensor_x, params.sensor_y);
        (sensor_size * params.clip_start) / params.lens
    };

    let sensor_fit = bke_camera_sensor_fit(params.sensor_fit, 1.0, frame_aspect);
    let viewfac = if sensor_fit == CAMERA_SENSOR_FIT_HOR {
        frame_size.x
    } else {
        frame_size.y
    };
    pixsize /= viewfac;
    pixsize *= params.zoom;

    let mut viewplane = Rctf {
        xmin: -0.5 * winx as f32,
        ymin: -0.5 * params.ycor * winy as f32,
        xmax: 0.5 * winx as f32,
        ymax: 0.5 * params.ycor * winy as f32,
    };
    let dxy = Vec2::new(
        params.shiftx * viewfac + winx as f32 * params.offsetx,
        params.shifty * viewfac + winy as f32 * params.offsety,
    );
    let dxy = bke_camera_viewplane_offset_transform(params.roll, params.is_flipped_x, dxy);
    rctf_translate(&mut viewplane, dxy.x, dxy.y);
    blenders_bli::rct::rctf_mul(&mut viewplane, pixsize);

    params.viewdx = pixsize;
    params.viewdy = params.ycor * pixsize;
    params.viewplane = viewplane;
}

/// `BKE_camera_params_crop_viewplane`.
pub fn bke_camera_params_crop_viewplane(
    viewplane: &mut Rctf,
    winx: i32,
    winy: i32,
    region: &Rcti,
) {
    let pix_size_x = rctf_size_x(viewplane) / winx as f32;
    let pix_size_y = rctf_size_y(viewplane) / winy as f32;
    viewplane.xmin += pix_size_x * region.xmin as f32;
    viewplane.ymin += pix_size_y * region.ymin as f32;
    viewplane.xmax =
        viewplane.xmin + pix_size_x * (region.xmax - region.xmin) as f32;
    viewplane.ymax =
        viewplane.ymin + pix_size_y * (region.ymax - region.ymin) as f32;
}

/// `BKE_camera_params_compute_matrix`.
pub fn bke_camera_params_compute_matrix(params: &mut CameraParams) {
    let vp = params.viewplane;
    params.winmat = if params.is_ortho {
        projection::orthographic(
            vp.xmin,
            vp.xmax,
            vp.ymin,
            vp.ymax,
            params.clip_start,
            params.clip_end,
        )
    } else {
        projection::perspective(
            vp.xmin,
            vp.xmax,
            vp.ymin,
            vp.ymax,
            params.clip_start,
            params.clip_end,
        )
    };
}

/// `BKE_camera_view_border` — camera frame rect in window pixels.
/// `v3d_viewplane` = view3d-computed viewplane; `camera_viewplane` =
/// camera-object-computed viewplane (both computed by the caller with
/// `bke_camera_params_compute_viewplane`).
pub fn bke_camera_view_border(
    rect_view: &Rctf,
    rect_camera: &Rctf,
    winx: i32,
    winy: i32,
) -> Rctf {
    Rctf {
        xmin: ((rect_camera.xmin - rect_view.xmin) / rctf_size_x(rect_view)) * winx as f32,
        xmax: ((rect_camera.xmax - rect_view.xmin) / rctf_size_x(rect_view)) * winx as f32,
        ymin: ((rect_camera.ymin - rect_view.ymin) / rctf_size_y(rect_view)) * winy as f32,
        ymax: ((rect_camera.ymax - rect_view.ymin) / rctf_size_y(rect_view)) * winy as f32,
    }
}

/// `BKE_camera_view_render_border` — camera-view render border;
/// `render_border` is `scene.r.border` (or `v3d->render_border`),
/// `viewborder` the camera frame rect, `camroll` rv3d roll, `flip_x`
/// rv3d flip flag.
pub fn bke_camera_view_render_border(
    is_camera_view: bool,
    use_render_border_flag: bool,
    render_border_mode_set: bool,
    use_camera_frame: bool,
    viewborder: &Rctf,
    render_border: &Rctf,
    winx: i32,
    winy: i32,
    flip_x: bool,
    camroll: f32,
) -> Option<(Rctf, Option<Rctf>)> {
    let mut border;
    if is_camera_view {
        if !render_border_mode_set && !use_camera_frame {
            return None;
        }
        if render_border_mode_set {
            border = Rctf {
                xmin: viewborder.xmin + render_border.xmin * rctf_size_x(viewborder),
                ymin: viewborder.ymin + render_border.ymin * rctf_size_y(viewborder),
                xmax: viewborder.xmin + render_border.xmax * rctf_size_x(viewborder),
                ymax: viewborder.ymin + render_border.ymax * rctf_size_y(viewborder),
            };
        } else {
            border = *viewborder;
        }
        if flip_x {
            std::mem::swap(&mut border.xmin, &mut border.xmax);
            border.xmin = winx as f32 - border.xmin;
            border.xmax = winx as f32 - border.xmax;
        }
    } else {
        if !use_render_border_flag {
            return None;
        }
        border = Rctf {
            xmin: render_border.xmin * winx as f32,
            ymin: render_border.ymin * winy as f32,
            xmax: render_border.xmax * winx as f32,
            ymax: render_border.ymax * winy as f32,
        };
    }
    let mut unrolled = None;
    if is_camera_view && camroll != 0.0 {
        let pivot = Vec2::new(winx as f32 / 2.0, winy as f32 / 2.0);
        let mut out = border;
        rctf_rotate_expand_around(
            &mut out,
            &border,
            pivot,
            if flip_x { -camroll } else { camroll },
        );
        border = out;
        unrolled = Some(out);
    }
    Some((border, unrolled))
}

/// `BKE_camera_view_frame_ex` — the 4 world-space corners of the
/// camera frame. `xsch`,`ysch`,`xasp`,`yasp` = render size/aspect
/// (pass `None` for `scene == nullptr`).
#[allow(clippy::too_many_arguments)]
pub fn bke_camera_view_frame_ex(
    scene_res: Option<(i32, i32, i32, i32)>,
    camera: &CameraData,
    drawsize: f32,
    do_clip: bool,
    scale: [f32; 3],
    r_drawsize: f32,
) -> ([f32; 2], [f32; 2], [[f32; 3]; 4]) {
    let mut r_asp = [1.0f32; 2];
    if let Some((xsch, ysch, xasp, yasp)) = scene_res {
        let aspx = xsch as f32 * xasp as f32;
        let aspy = ysch as f32 * yasp as f32;
        let sensor_fit = bke_camera_sensor_fit(camera.sensor_fit, aspx, aspy);
        if sensor_fit == CAMERA_SENSOR_FIT_HOR {
            r_asp[0] = 1.0;
            r_asp[1] = aspy / aspx;
        } else {
            r_asp[0] = aspx / aspy;
            r_asp[1] = 1.0;
        }
    }
    let facx;
    let facy;
    let depth;
    let mut r_shift = [0.0f32; 2];
    if camera.cam_type == CAM_ORTHO {
        facx = 0.5 * camera.ortho_scale * r_asp[0] * scale[0];
        facy = 0.5 * camera.ortho_scale * r_asp[1] * scale[1];
        r_shift[0] = camera.shiftx * camera.ortho_scale * scale[0];
        r_shift[1] = camera.shifty * camera.ortho_scale * scale[1];
        depth = -drawsize * scale[2];
    } else {
        let half_sensor = 0.5
            * if camera.sensor_fit == CAMERA_SENSOR_FIT_VERT {
                camera.sensor_y
            } else {
                camera.sensor_x
            };
        depth = r_drawsize * camera.lens / (-half_sensor) * scale[2];
        let fac = r_drawsize;
        let scale_x = scale[0];
        let scale_y = scale[1];
        facx = fac * r_asp[0] * scale_x;
        facy = fac * r_asp[1] * scale_y;
        r_shift[0] = camera.shiftx * fac * 2.0 * scale_x;
        r_shift[1] = camera.shifty * fac * 2.0 * scale_y;
    }
    let mut r_vec = [
        [r_shift[0] + facx, r_shift[1] + facy, depth],
        [r_shift[0] + facx, r_shift[1] - facy, depth],
        [r_shift[0] - facx, r_shift[1] - facy, depth],
        [r_shift[0] - facx, r_shift[1] + facy, depth],
    ];
    if do_clip {
        let fac = ((camera.clip_start + 0.1) / -r_vec[0][2]) * scale[2];
        for v in r_vec.iter_mut() {
            if camera.cam_type == CAM_ORTHO {
                v[2] *= fac;
            } else {
                v[0] *= fac;
                v[1] *= fac;
                v[2] *= fac;
            }
        }
    }
    (r_asp, r_shift, r_vec)
}

/// `BKE_camera_view_frame` — `view_frame_ex` with defaults.
pub fn bke_camera_view_frame(
    scene_res: Option<(i32, i32, i32, i32)>,
    camera: &CameraData,
) -> [[f32; 3]; 4] {
    bke_camera_view_frame_ex(scene_res, camera, 1.0, false, [1.0; 3], 1.0).2
}

/// `BKE_camera_multiview_spherical_stereo` — spherical stereo when
/// views_format is STEREO_3D and camera is perspective.
pub fn bke_camera_multiview_spherical_stereo(
    views_format_stereo3d: bool,
    camera: &CameraData,
) -> bool {
    views_format_stereo3d && camera.cam_type != CAM_ORTHO
}
