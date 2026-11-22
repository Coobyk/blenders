/// Port of `editors/transform/transform_mode_seq_slide.cc` —
/// seq-slide modal: shift strip frame
/// handles by value.
use crate::transform::{TransInfo, TD_SKIP};
/// `applySeqSlide` (verbatim — per strip:
/// frame += value, clamped to neighbors).
pub fn apply_seq_slide(
    t: &mut TransInfo,
    frames: &mut [f32],
    neighbors: &[(f32, f32)],
) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        if t.data[i].flag & TD_SKIP != 0 {
            continue;
        }
        if let Some(f) = frames.get_mut(i) {
            let (lo, hi) = neighbors.get(i).copied().unwrap_or((f32::MIN, f32::MAX));
            *f = (*f + s).clamp(lo, hi);
        }
    }
}
/// `initSeqSlide`.
pub fn init_seq_slide(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
