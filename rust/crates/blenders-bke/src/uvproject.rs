//! Port of `intern/uvproject.cc` — `BKE_uvproject_*` camera/view UV
//! projection for the UVProject modifier.

use blenders_dna::camera_types::{Camera, CAM_PANO, CAM_PERSP, CAM_CUSTOM};
use blenders_math::matrix::Mat4;
use blenders_math::matrix_c::{invert_m4, mul_m4_v4, mul_v3_m4v3, normalize_m4};
use blenders_math::quaternion::focallength_to_fov;
use blenders_math::vector::{Vec2, Vec3, Vec4};

/// `ProjCameraInfo` (uvproject.h).
#[derive(Clone)]
pub struct ProjCameraInfo {
    pub camangle: f32,
    pub camsize: f32,
    pub xasp: f32,
    pub yasp: f32,
    pub shiftx: f32,
    pub shifty: f32,
    pub rotmat: Mat4,
    pub caminv: Mat4,
    pub do_persp: bool,
    pub do_pano: bool,
    pub do_rotmat: bool,
}

/// `BKE_uvproject_from_camera`.
pub fn bke_uvproject_from_camera(source: Vec3, uci: &ProjCameraInfo) -> Vec2 {
    let mut pv4 = Vec4::new(source.x, source.y, source.z, 1.0);
    if uci.do_rotmat {
        mul_m4_v4(uci.rotmat, &mut pv4);
    }
    mul_m4_v4(uci.caminv, &mut pv4);
    let mut target = Vec2::ZERO;
    if uci.do_pano {
        let angle = pv4.x.atan2(-pv4.z) / (std::f32::consts::PI * 2.0); /* angle around the camera */
        if !uci.do_persp {
            target.x = angle; /* No correct method here, just map to 0-1. */
            target.y = pv4.y / uci.camsize;
        } else {
            /* 2D position from the camera */
            let vec2d = Vec2::new(pv4.x, pv4.z);
            target.x = angle * (std::f32::consts::PI / uci.camangle);
            target.y = pv4.y / (vec2d.length() * (uci.camsize * 2.0));
        }
    } else {
        if pv4.z == 0.0 {
            pv4.z = 0.00001; /* don't allow div by 0 */
        }
        if !uci.do_persp {
            target.x = pv4.x / uci.camsize;
            target.y = pv4.y / uci.camsize;
        } else {
            target.x = (-pv4.x * ((1.0 / uci.camsize) / pv4.z)) / 2.0;
            target.y = (-pv4.y * ((1.0 / uci.camsize) / pv4.z)) / 2.0;
        }
    }
    target.x *= uci.xasp;
    target.y *= uci.yasp;
    target.x += uci.shiftx;
    target.y += uci.shifty;
    target
}

/// `BKE_uvproject_from_view`.
pub fn bke_uvproject_from_view(source: Vec3, persmat: Mat4, rotmat: Mat4, winx: f32, winy: f32) -> Vec2 {
    let mut pv4 = Vec4::new(source.x, source.y, source.z, 1.0);
    mul_m4_v4(rotmat, &mut pv4);
    mul_m4_v4(persmat, &mut pv4);
    let mut target = Vec2::ZERO;
    if pv4.w.abs() > 0.00001 {
        /* avoid division by zero */
        target.x = winx / 2.0 + (winx / 2.0) * pv4.x / pv4.w;
        target.y = winy / 2.0 + (winy / 2.0) * pv4.y / pv4.w;
    } else {
        target.x = winx / 2.0 + (winx / 2.0) * pv4.x;
        target.y = winy / 2.0 + (winy / 2.0) * pv4.y;
    }
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut wx = winx;
    let mut wy = winy;
    if wx > wy {
        y = (wx - wy) / 2.0;
        wy = wx;
    } else {
        x = (wy - wx) / 2.0;
        wx = wy;
    }
    Vec2::new((x + target.x) / wx, (y + target.y) / wy)
}

/// `BKE_uvproject_camera_info` — returns `None` on non-invertible matrix.
/// `ob_data` is the object's `Camera` pointer payload, `object_to_world` its
/// matrix (Object isn't fully ported — callers pass the pieces).
pub fn bke_uvproject_camera_info(
    camera: &Camera,
    object_to_world: Mat4,
    rotmat: Option<Mat4>,
    winx: f32,
    winy: f32,
) -> Option<ProjCameraInfo> {
    let mut uci = ProjCameraInfo {
        camangle: 0.0,
        camsize: 0.0,
        xasp: 0.0,
        yasp: 0.0,
        shiftx: 0.0,
        shifty: 0.0,
        rotmat: Mat4::IDENTITY,
        caminv: Mat4::IDENTITY,
        do_persp: false,
        do_pano: false,
        do_rotmat: false,
    };
    uci.do_pano = camera.type_ == CAM_PANO;
    uci.do_persp = camera.type_ == CAM_PERSP || camera.type_ == CAM_CUSTOM;
    uci.camangle = focallength_to_fov(camera.lens, camera.sensor_x) / 2.0;
    uci.camsize = if uci.do_persp { uci.camangle.tan() } else { camera.ortho_scale };
    uci.caminv = object_to_world;
    normalize_m4(&mut uci.caminv);
    if !invert_m4(&mut uci.caminv) {
        return None;
    }
    if let Some(rm) = rotmat {
        uci.rotmat = rm;
        uci.do_rotmat = true;
    }
    if winx > winy {
        uci.xasp = 1.0;
        uci.yasp = winx / winy;
    } else {
        uci.xasp = winy / winx;
        uci.yasp = 1.0;
    }
    uci.shiftx = 0.5 - camera.shiftx * uci.xasp;
    uci.shifty = 0.5 - camera.shifty * uci.yasp;
    Some(uci)
}

/// `BKE_uvproject_from_view_ortho`.
pub fn bke_uvproject_from_view_ortho(source: Vec3, rotmat: Mat4) -> Vec2 {
    let pv = mul_v3_m4v3(rotmat, source);
    Vec2::new(-pv.x, pv.z)
}

/// `BKE_uvproject_camera_info_scale`.
pub fn bke_uvproject_camera_info_scale(uci: &mut ProjCameraInfo, scale_x: f32, scale_y: f32) {
    uci.xasp *= scale_x;
    uci.yasp *= scale_y;
}
