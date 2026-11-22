/// Port of `editors/transform/transform_convert_object.cc` —
/// object-mode `createTransObject`:
/// per-selected-object TransData with
/// loc/rot/scale + obmat basis.
use crate::transform::{TransData, TransInfo, TD_SELECTED};
/// `transdata_elem_translate` for one
/// object — record `ob->loc`, basis =
/// `ob->obmat`.
pub fn transdata_object(
    td: &mut TransData,
    loc: [f32; 3],
    obmat: [[f32; 4]; 4],
) {
    td.loc = loc;
    td.iloc = loc;
    td.flag = TD_SELECTED;
    td.center = [
        obmat[0][3],
        obmat[1][3],
        obmat[2][3],
    ];
    td.mtx = [
        [obmat[0][0], obmat[0][1], obmat[0][2]],
        [obmat[1][0], obmat[1][1], obmat[1][2]],
        [obmat[2][0], obmat[2][1], obmat[2][2]],
    ];
    // smtx = transpose (inverse of orthonormal basis).
    td.smtx = [
        [td.mtx[0][0], td.mtx[1][0], td.mtx[2][0]],
        [td.mtx[0][1], td.mtx[1][1], td.mtx[2][1]],
        [td.mtx[0][2], td.mtx[1][2], td.mtx[2][2]],
    ];
}
/// `createTransObject` — one TransData per
/// selected object.
pub fn create_trans_objects(
    t: &mut TransInfo,
    objects: &[([f32; 3], [[f32; 4]; 4])],
) {
    for tc in t.data.iter_mut() {
        for (loc, mat) in objects {
            let mut td = TransData::default();
            td.identity();
            transdata_object(&mut td, *loc, *mat);
            tc.data.push(td);
        }
    }
}
