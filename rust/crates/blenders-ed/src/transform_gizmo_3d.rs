/// Port of `editors/transform/transform_gizmo_3d.cc` —
/// 3D gizmo: axis/plane handle positions,
/// pick → constraint, draw data.
use crate::transform::{TransInfo, CON_AXIS0, CON_AXIS1, CON_AXIS2};
/// `GizmoElem` — one gizmo handle.
#[derive(Clone)]
pub struct GizmoElem {
    /// `gizmo_type` — arrow/plane/dial.
    pub gizmo_type: i32,
    /// `axis` — constrained axis index.
    pub axis: i32,
    /// `matrix` — world transform.
    pub matrix: [[f32; 4]; 4],
}
/// `manipulator_axis_getmap` — axes for
/// the current orientation + constraint.
pub fn manipulator_axis_getmap(t: &TransInfo) -> Vec<[f32; 3]> {
    // World axes through spacemtx.
    let m = t.spacemtx;
    let mut out = vec![
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ];
    for a in out.iter_mut() {
        let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-9);
        a[0] /= l;
        a[1] /= l;
        a[2] /= l;
    }
    out
}
/// `gizmo_elem_pick` → con_mode bitmask.
pub fn gizmo_elem_pick(axis: i32, plane: bool) -> u32 {
    if plane {
        match axis {
            0 => CON_AXIS1 | CON_AXIS2,
            1 => CON_AXIS0 | CON_AXIS2,
            _ => CON_AXIS0 | CON_AXIS1,
        }
    } else {
        match axis {
            0 => CON_AXIS0,
            1 => CON_AXIS1,
            _ => CON_AXIS2,
        }
    }
}
/// `calc_gizmo_stats` — gizmo scale from
/// view distance (uniform scale factor).
pub fn calc_gizmo_stats(t: &TransInfo, dist: f32) -> f32 {
    // C: gizmo scales with view distance.
    let _ = t;
    dist * 0.15
}
