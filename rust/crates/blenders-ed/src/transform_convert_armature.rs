/// Port of `editors/transform/transform_convert_armature.cc` —
/// armature editmode: TransData per
/// selected bone head/tail.
use crate::transform::{TransData, TransInfo, TD_SELECTED, TD_NO_CENTER, TD_NOT_CONNECTED};
/// `createTransArmatureVerts` — head+tail
/// TransData per selected editbone.
/// (head, tail, connected) per bone.
pub fn create_trans_armature_verts(
    t: &mut TransInfo,
    bones: &[([f32; 3], [f32; 3], bool, bool)], // head, tail, head_sel, tail_sel
) {
    for (head, tail, hsel, tsel) in bones {
        if *hsel {
            let mut td = TransData::default();
            td.identity();
            td.loc = *head;
            td.iloc = *head;
            td.center = *head;
            td.flag = TD_SELECTED | if *tsel { 0 } else { TD_NOT_CONNECTED };
            t.data.push(td);
        }
        if *tsel {
            let mut td = TransData::default();
            td.identity();
            td.loc = *tail;
            td.iloc = *tail;
            td.center = *tail;
            td.flag = TD_SELECTED | if *hsel { TD_NO_CENTER } else { 0 };
            t.data.push(td);
        }
    }
}
