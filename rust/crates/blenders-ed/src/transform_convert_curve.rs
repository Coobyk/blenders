/// Port of `editors/transform/transform_convert_curve.cc` —
/// curve editmode: TransData per selected
/// bezier/nurb control point.
use crate::transform::{TransData, TransInfo, TD_SELECTED};
/// `createTransCurveVerts` — TransData per
/// selected control point.
pub fn create_trans_curve_verts(
    t: &mut TransInfo,
    points: &[([f32; 3], bool)], // (co, selected)
) {
    for (co, sel) in points {
        if !*sel {
            continue;
        }
        let mut td = TransData::default();
        td.identity();
        td.loc = *co;
        td.iloc = *co;
        td.center = *co;
        td.flag = TD_SELECTED;
        t.data.push(td);
    }
}
