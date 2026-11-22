/// Port of `editors/transform/transform_draw.cc` —
/// draw handlers (edge/dir/constraint line
/// emission for the viewport overlay).
use crate::transform::TransInfo;
/// `drawConstraint` — emit constraint-axis
/// line endpoints for the transform UI.
pub fn draw_constraint(
    t: &TransInfo,
    con_axes: &[[f32; 3]],
) -> Vec<([f32; 3], [f32; 3])> {
    // Per axis: a long line through center.
    let len = 1000.0;
    con_axes
        .iter()
        .map(|a| {
            (
                [
                    t.center_global[0] - a[0] * len,
                    t.center_global[1] - a[1] * len,
                    t.center_global[2] - a[2] * len,
                ],
                [
                    t.center_global[0] + a[0] * len,
                    t.center_global[1] + a[1] * len,
                    t.center_global[2] + a[2] * len,
                ],
            )
        })
        .collect()
}
/// `drawPropCircle` — proportional-editing
/// circle radius.
pub fn draw_prop_circle(t: &TransInfo, radius: f32) -> (f32, [f32; 3]) {
    (radius, t.center_global)
}
