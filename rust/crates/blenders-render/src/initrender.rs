//! Port of `intern/initrender.cc` — camera params →
//! window/viewplane extraction.

/// `R_BORDER` verbatim.
pub const R_BORDER: i32 = 1 << 9;

/// `RE_GetWindowMatrixWithOverscan` viewplane dilation
/// (verbatim — overscan × max(size_x,size_y) all sides).
pub fn get_window_matrix_with_overscan(
    viewplane: &mut [f32; 4],
    overscan: f32,
) {
    let size_x = viewplane[1] - viewplane[0];
    let size_y = viewplane[3] - viewplane[2];
    let os = overscan * size_x.max(size_y);
    viewplane[0] -= os;
    viewplane[1] += os;
    viewplane[2] -= os;
    viewplane[3] += os;
}

/// `RE_GetViewPlane` disprect rule (verbatim —
/// border → copy, else zero).
pub fn get_view_plane(re_viewplane: [f32; 4], r_mode: i32, re_disprect: [i32; 4]) -> ([f32; 4], [i32; 4]) {
    let disprect = if (r_mode & R_BORDER) != 0 {
        re_disprect
    } else {
        [0, 0, 0, 0]
    };
    (re_viewplane, disprect)
}
