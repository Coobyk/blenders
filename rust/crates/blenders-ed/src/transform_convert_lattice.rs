/// Port of `editors/transform/transform_convert_lattice.cc` —
/// lattice editmode: TransData per
/// selected BPoint.
use crate::transform::{TransData, TransInfo, TD_SELECTED};
/// `createTransLatticeVerts` — TransData
/// per selected lattice point.
pub fn create_trans_lattice_verts(
    t: &mut TransInfo,
    points: &[([f32; 3], bool)],
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
