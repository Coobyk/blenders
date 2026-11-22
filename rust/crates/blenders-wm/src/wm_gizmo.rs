//! Port of `gizmo/intern/wm_gizmo.cc` — `wmGizmo`
//! create/free/matrix setters, select/highlight, scale
//! calc, matrix_final composition, visibility.

/// `WM_GIZMO_*` flags verbatim.
pub const WM_GIZMO_HIDDEN: i32 = 1 << 0;
pub const WM_GIZMO_DRAW_MODAL: i32 = 1 << 4;
pub const WM_GIZMO_DRAW_VALUE: i32 = 1 << 5;
pub const WM_GIZMO_DRAW_HOVER: i32 = 1 << 6;
pub const WM_GIZMO_DRAW_NO_SCALE: i32 = 1 << 7;
pub const WM_GIZMO_DRAW_OFFSET_SCALE: i32 = 1 << 8;
/// `WM_GIZMO_STATE_*` verbatim.
pub const WM_GIZMO_STATE_MODAL: i32 = 1 << 0;
pub const WM_GIZMO_STATE_HIGHLIGHT: i32 = 1 << 1;
pub const WM_GIZMO_STATE_SELECT: i32 = 1 << 2;
/// `WM_GIZMO_IS_VISIBLE_*` verbatim.
pub const WM_GIZMO_IS_VISIBLE_UPDATE: i32 = 1;
pub const WM_GIZMO_IS_VISIBLE_DRAW: i32 = 2;
/// `WM_GIZMOGROUPTYPE_SCALE` verbatim.
pub const WM_GIZMOGROUPTYPE_SCALE: i32 = 1 << 0;
/// `WM_GIZMOMAP_DRAWSTEP_*` verbatim.
pub const WM_GIZMOMAP_DRAWSTEP_2D: i32 = 0;
pub const WM_GIZMOMAP_DRAWSTEP_3D: i32 = 1;

/// `wm_gizmo_set_matrix_rotation_from_z_axis__internal`
/// (verbatim — normalize z into col 2, ortho basis x/y).
pub fn gizmo_matrix_rotation_from_z_axis(matrix: &mut [[f32; 4]; 4], z_axis: [f32; 3]) {
    let len = (z_axis[0] * z_axis[0] + z_axis[1] * z_axis[1] + z_axis[2] * z_axis[2]).sqrt();
    let z = if len != 0.0 {
        [z_axis[0] / len, z_axis[1] / len, z_axis[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    };
    matrix[2] = [z[0], z[1], z[2], matrix[2][3]];
    /* `ortho_basis_v3v3_v3(x, y, z)`. */
    let axis = if z[0].abs() < z[1].abs() { 0 } else { 1 };
    let mut x = [0.0f32; 3];
    x[axis] = 1.0;
    let xcross = [
        x[1] * z[2] - x[2] * z[1],
        x[2] * z[0] - x[0] * z[2],
        x[0] * z[1] - x[1] * z[0],
    ];
    let xlen = (xcross[0] * xcross[0] + xcross[1] * xcross[1] + xcross[2] * xcross[2]).sqrt();
    let nx = if xlen != 0.0 {
        [xcross[0] / xlen, xcross[1] / xlen, xcross[2] / xlen]
    } else {
        [1.0, 0.0, 0.0]
    };
    let y = [
        z[1] * nx[2] - z[2] * nx[1],
        z[2] * nx[0] - z[0] * nx[2],
        z[0] * nx[1] - z[1] * nx[0],
    ];
    matrix[0] = [nx[0], nx[1], nx[2], matrix[0][3]];
    matrix[1] = [y[0], y[1], y[2], matrix[1][3]];
}

/// `wm_gizmo_set_matrix_rotation_from_yz_axis__internal`
/// (verbatim — normalize y/z, x = y×z).
pub fn gizmo_matrix_rotation_from_yz_axis(matrix: &mut [[f32; 4]; 4], y_axis: [f32; 3], z_axis: [f32; 3]) {
    let norm = |v: [f32; 3]| {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l != 0.0 { [v[0] / l, v[1] / l, v[2] / l] } else { v }
    };
    let y = norm(y_axis);
    let z = norm(z_axis);
    let mut x = [
        y[1] * z[2] - y[2] * z[1],
        y[2] * z[0] - y[0] * z[2],
        y[0] * z[1] - y[1] * z[0],
    ];
    x = norm(x);
    matrix[0] = [x[0], x[1], x[2], matrix[0][3]];
    matrix[1] = [y[0], y[1], y[2], matrix[1][3]];
    matrix[2] = [z[0], z[1], z[2], matrix[2][3]];
}

/// `WM_gizmo_set_flag` (verbatim).
pub fn wm_gizmo_set_flag(flag: &mut i32, flag_bit: i32, enable: bool) {
    if enable {
        *flag |= flag_bit;
    } else {
        *flag &= !flag_bit;
    }
}

/// `wm_gizmo_select_set_ex` (verbatim — state toggle +
/// optional array + refresh cb).
pub fn wm_gizmo_select_set_ex(
    gz_state: &mut i32,
    select: bool,
    use_array: bool,
    use_callback: bool,
    array_push: &mut dyn FnMut(),
    array_remove: &mut dyn FnMut(),
    select_refresh: &mut dyn FnMut(),
) -> bool {
    let mut changed = false;
    if select {
        if (*gz_state & WM_GIZMO_STATE_SELECT) == 0 {
            if use_array {
                (array_push)();
            }
            *gz_state |= WM_GIZMO_STATE_SELECT;
            changed = true;
        }
    } else if (*gz_state & WM_GIZMO_STATE_SELECT) != 0 {
        if use_array {
            (array_remove)();
        }
        *gz_state &= !WM_GIZMO_STATE_SELECT;
        changed = true;
    }
    if use_callback && changed {
        (select_refresh)();
    }
    changed
}

/// `wm_gizmo_calculate_scale` (verbatim — UI_SCALE ×
/// gizmo_size × pixel_size when not group-scaled).
pub fn wm_gizmo_calculate_scale(
    scale_basis: f32,
    group_flag_scaled: bool,
    ui_scale_fac: f32,
    gizmo_size: f32,
    pixel_size_no_ui_scale: Option<f32>,
) -> f32 {
    let mut scale = ui_scale_fac;
    if !group_flag_scaled {
        scale *= gizmo_size;
        if let Some(ps) = pixel_size_no_ui_scale {
            scale *= ps;
        }
    }
    scale_basis * scale
}

/// `wm_gizmo_is_visible` (verbatim — hidden / modal /
/// hover).
pub fn wm_gizmo_is_visible(flag: i32, state: i32) -> i32 {
    if (flag & WM_GIZMO_HIDDEN) != 0 {
        return 0;
    }
    if (state & WM_GIZMO_STATE_MODAL) != 0
        && (flag & (WM_GIZMO_DRAW_MODAL | WM_GIZMO_DRAW_VALUE)) == 0
    {
        return 0;
    }
    if (flag & WM_GIZMO_DRAW_HOVER) != 0
        && (state & WM_GIZMO_STATE_HIGHLIGHT) == 0
        && (state & WM_GIZMO_STATE_SELECT) == 0
    {
        return WM_GIZMO_IS_VISIBLE_UPDATE;
    }
    WM_GIZMO_IS_VISIBLE_UPDATE | WM_GIZMO_IS_VISIBLE_DRAW
}

/// `wmGizmoMatrixParams` verbatim.
#[derive(Clone, Copy, Default)]
pub struct WmGizmoMatrixParams {
    /// `matrix_space`/`matrix_basis`/`matrix_offset`/`scale_final`.
    pub matrix_space: Option<usize>,
    pub matrix_basis: Option<usize>,
    pub matrix_offset: Option<usize>,
    pub scale_final: Option<f32>,
}

/// `WM_gizmo_calc_matrix_final_params` order (verbatim —
/// basis → offset-mul ± scale → space premul).
pub fn wm_gizmo_calc_matrix_final(
    matrix_space: [[f32; 4]; 4],
    matrix_basis: [[f32; 4]; 4],
    matrix_offset: [[f32; 4]; 4],
    scale_final: f32,
    no_scale: bool,
    offset_scale: bool,
) -> [[f32; 4]; 4] {
    let mul = |a: [[f32; 4]; 4], b: [[f32; 4]; 4]| {
        let mut o = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                o[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum();
            }
        }
        o
    };
    let mut final_matrix = matrix_basis;
    let mul_mat3_fl = |m: &mut [[f32; 4]; 4], s: f32| {
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] *= s;
            }
        }
    };
    if no_scale {
        final_matrix = mul(final_matrix, matrix_offset);
    } else if offset_scale {
        mul_mat3_fl(&mut final_matrix, scale_final);
        final_matrix = mul(final_matrix, matrix_offset);
    } else {
        final_matrix = mul(final_matrix, matrix_offset);
        mul_mat3_fl(&mut final_matrix, scale_final);
    }
    mul(matrix_space, final_matrix)
}
