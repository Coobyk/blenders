/// Port of `space_view3d/view3d_project.cc`
/// — `ED_view3d_project_*` (world → window
/// coords) + `ED_view3d_win_to_*` (window
/// → world rays/vectors).
/// `eV3DProjStatus` (verbatim).
pub const V3D_PROJ_RET_OK: i32 = 0;
pub const V3D_PROJ_RET_CLIP_NEAR: i32 = 1;
pub const V3D_PROJ_RET_CLIP_FAR: i32 = 2;
pub const V3D_PROJ_RET_CLIP_BB: i32 = 3;
pub const V3D_PROJ_RET_CLIP_ZERO: i32 = 4;
pub const V3D_PROJ_RET_INVALID: i32 = 5;
/// `V3D_PROJ_TEST_*` flags (verbatim).
pub const V3D_PROJ_TEST_CLIP_ZERO: u32 = 1 << 0;
pub const V3D_PROJ_TEST_CLIP_NEAR: u32 = 1 << 1;
pub const V3D_PROJ_TEST_CLIP_FAR: u32 = 1 << 2;
pub const V3D_PROJ_TEST_CLIP_BB: u32 = 1 << 3;
pub const V3D_PROJ_TEST_CLIP_WIN: u32 = 1 << 4;
pub const V3D_PROJ_TEST_ALL: u32 = 0x1f;
pub const V3D_PROJ_TEST_CLIP_DEFAULT: u32 =
    V3D_PROJ_TEST_CLIP_ZERO | V3D_PROJ_TEST_CLIP_NEAR | V3D_PROJ_TEST_CLIP_FAR;
const BL_ZERO_CLIP: f32 = 0.001;
const BL_NEAR_CLIP: f32 = 0.001;
/// `ed_view3d_project__internal` (verbatim
/// — project `co` through `perspmat` to
/// window coords).
pub fn ed_view3d_project_internal(
    winx: f32,
    winy: f32,
    perspmat: &[[f32; 4]; 4],
    co: [f32; 3],
    flag: u32,
) -> (i32, [f32; 2]) {
    let mut vec4 = [co[0], co[1], co[2], 1.0];
    // mul_m4_v4.
    let m = perspmat;
    let v = [
        m[0][0] * vec4[0] + m[1][0] * vec4[1] + m[2][0] * vec4[2] + m[3][0],
        m[0][1] * vec4[0] + m[1][1] * vec4[1] + m[2][1] * vec4[2] + m[3][1],
        m[0][2] * vec4[0] + m[1][2] * vec4[1] + m[2][2] * vec4[2] + m[3][2],
        m[0][3] * vec4[0] + m[1][3] * vec4[1] + m[2][3] * vec4[2] + m[3][3],
    ];
    vec4 = v;
    let w = vec4[3].abs();
    if (flag & V3D_PROJ_TEST_CLIP_ZERO) != 0 && w <= BL_ZERO_CLIP {
        return (V3D_PROJ_RET_CLIP_ZERO, [0.0; 2]);
    }
    if (flag & V3D_PROJ_TEST_CLIP_NEAR) != 0 && vec4[2] <= -w {
        return (V3D_PROJ_RET_CLIP_NEAR, [0.0; 2]);
    }
    if (flag & V3D_PROJ_TEST_CLIP_FAR) != 0 && vec4[2] >= w {
        return (V3D_PROJ_RET_CLIP_FAR, [0.0; 2]);
    }
    let scalar = if w != 0.0 { 1.0 / w } else { 0.0 };
    let fx = winx * 0.5 * (1.0 + vec4[0] * scalar);
    let fy = winy * 0.5 * (1.0 + vec4[1] * scalar);
    (V3D_PROJ_RET_OK, [fx, fy])
}
/// `ED_view3d_win_to_vector` (verbatim —
/// window → persp direction).
pub fn ed_view3d_win_to_vector(
    winx: f32,
    winy: f32,
    perspmat_inv: &[[f32; 4]; 4],
    mval: [f32; 2],
) -> [f32; 3] {
    let mut vec4 = [
        2.0 * mval[0] / winx - 1.0,
        2.0 * mval[1] / winy - 1.0,
        -1.0,
        1.0,
    ];
    let m = perspmat_inv;
    let v = [
        m[0][0] * vec4[0] + m[1][0] * vec4[1] + m[2][0] * vec4[2] + m[3][0],
        m[0][1] * vec4[0] + m[1][1] * vec4[1] + m[2][1] * vec4[2] + m[3][1],
        m[0][2] * vec4[0] + m[1][2] * vec4[1] + m[2][2] * vec4[2] + m[3][2],
        m[0][3] * vec4[0] + m[1][3] * vec4[1] + m[2][3] * vec4[2] + m[3][3],
    ];
    vec4 = v;
    if vec4[3] != 0.0 {
        let s = 1.0 / vec4[3];
        [vec4[0] * s, vec4[1] * s, vec4[2] * s]
    } else {
        [vec4[0], vec4[1], vec4[2]]
    }
}
/// `ED_view3d_win_to_ray` (verbatim —
/// ray origin+direction, persp/ortho).
pub fn ed_view3d_win_to_ray(
    is_persp: bool,
    cam_pos: [f32; 3],
    winx: f32,
    winy: f32,
    perspmat_inv: &[[f32; 4]; 4],
    mval: [f32; 2],
) -> ([f32; 3], [f32; 3]) {
    let dir = ed_view3d_win_to_vector(winx, winy, perspmat_inv, mval);
    if is_persp {
        let origin = cam_pos;
        let d = [
            dir[0] - origin[0],
            dir[1] - origin[1],
            dir[2] - origin[2],
        ];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let n = if l > 0.0 { [d[0] / l, d[1] / l, d[2] / l] } else { d };
        (origin, n)
    } else {
        // Ortho: origin is on the near plane; dir is camera -Z.
        (dir, [0.0, 0.0, -1.0])
    }
}
/// `ED_view3d_calc_zfac` (verbatim —
/// projected z factor for overlay).
pub fn ed_view3d_calc_zfac(
    perspmat: &[[f32; 4]; 4],
    co: [f32; 3],
) -> f32 {
    let w = perspmat[0][3] * co[0]
        + perspmat[1][3] * co[1]
        + perspmat[2][3] * co[2]
        + perspmat[3][3];
    if w == 0.0 {
        BL_NEAR_CLIP
    } else {
        -w
    }
}
