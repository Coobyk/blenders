/// Port of `editors/transform/transform_convert_mball.cc` —
/// metaball editmode: TransData per
/// selected MetaElem.
use crate::transform::{TransData, TransInfo, TD_SELECTED};
/// `createTransMBallVerts` — TransData per
/// selected metaelement (center + rad).
pub fn create_trans_mball_verts(
    t: &mut TransInfo,
    elems: &[([f32; 3], f32, bool)], // (co, rad, selected)
) {
    for (co, _rad, sel) in elems {
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
