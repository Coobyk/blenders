/// Port of `editors/transform/transform_ops.cc` —
/// per-mode apply dispatch + numeric input
/// mapping for TFM_TRANSLATION/RESIZE/ROTATION.
use crate::transform::{
    apply_resize, apply_rotation, apply_translation, TransInfo, TFM_RESIZE,
    TFM_ROTATION, TFM_TRANSLATION,
};
/// `applyTranslation`/`applyResize`/`applyRotation`
/// dispatch over all TransData.
pub fn apply_transform(t: &mut TransInfo) {
    match t.mode {
        TFM_TRANSLATION => apply_translation(t),
        TFM_RESIZE => {
            // Default scale components to 1 (C's `values` semantics).
            for i in 0..3 {
                if t.values_final[i] == 0.0 {
                    t.values_final[i] = 1.0;
                }
            }
            apply_resize(t);
        }
        TFM_ROTATION => apply_rotation(t, [0.0, 0.0, 1.0]),
        _ => {}
    }
}
/// `recalcData` — per-mode write-back of
/// TransData to the scene (injected per
/// data type).
pub fn recalc_data(
    t: &TransInfo,
    write: &mut dyn FnMut(usize, [f32; 3]),
) {
    for (i, td) in t.data.iter().enumerate() {
        write(i, td.loc);
    }
}
