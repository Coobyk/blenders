//! Port of `intern/sound.cc` — sound strip pitch,
//! equalizer modifier, bounds updates.

/// `SOUND_EQUALIZER_DEFAULT_*` verbatim.
pub const SOUND_EQUALIZER_DEFAULT_MIN_FREQ: f32 = 30.0;
pub const SOUND_EQUALIZER_DEFAULT_MAX_FREQ: f32 = 20000.0;
pub const SOUND_EQUALIZER_DEFAULT_MAX_DB: f32 = 20.0;
/// `sound_pitch_get` (verbatim — recurse into
/// meta parent).
pub fn sound_pitch_get(
    strip_speed_factor: f32,
    meta_parent: &dyn Fn() -> Option<f32>,
) -> f32 {
    if let Some(parent_pitch) = meta_parent() {
        strip_speed_factor * parent_pitch
    } else {
        strip_speed_factor
    }
}
/// `EQCurveMappingData` verbatim.
#[derive(Clone)]
pub struct EQCurveMappingData {
    /// `curve_mapping` (min_x, min_y, max_x, max_y).
    pub clipr: [f32; 4],
}
/// `sound_equalizer_add` (verbatim — defaults +
/// median preset; max<0→20000, min<0→0).
pub fn sound_equalizer_add(
    graphics: &mut Vec<EQCurveMappingData>,
    min_x: f32,
    max_x: f32,
) -> usize {
    let max_x = if max_x < 0.0 { SOUND_EQUALIZER_DEFAULT_MAX_FREQ } else { max_x };
    let min_x = if min_x < 0.0 { 0.0 } else { min_x };
    graphics.push(EQCurveMappingData {
        clipr: [min_x, -SOUND_EQUALIZER_DEFAULT_MAX_DB, max_x, SOUND_EQUALIZER_DEFAULT_MAX_DB],
    });
    graphics.len() - 1
}
/// `sound_equalizermodifier_set_graphs` (verbatim —
/// 1→default, 2→30-2k/2k-20k, 3→30-1k/1k-5k/5k-20k).
pub fn sound_equalizermodifier_set_graphs(graphics: &mut Vec<EQCurveMappingData>, number: i32) {
    graphics.clear();
    match number {
        1 => {
            sound_equalizer_add(graphics, SOUND_EQUALIZER_DEFAULT_MIN_FREQ, SOUND_EQUALIZER_DEFAULT_MAX_FREQ);
        }
        2 => {
            sound_equalizer_add(graphics, 30.0, 2000.0);
            sound_equalizer_add(graphics, 2000.1, 20000.0);
        }
        3 => {
            sound_equalizer_add(graphics, 30.0, 1000.0);
            sound_equalizer_add(graphics, 1000.1, 5000.0);
            sound_equalizer_add(graphics, 5000.1, 20000.0);
        }
        _ => {}
    }
}
/// `sound_equalizermodifier_add_graph` (verbatim —
/// reject min<0, max<0, max<=min).
pub fn sound_equalizermodifier_add_graph(
    graphics: &mut Vec<EQCurveMappingData>,
    min_freq: f32,
    max_freq: f32,
) -> Option<usize> {
    if min_freq < 0.0 || max_freq < 0.0 || max_freq <= min_freq {
        return None;
    }
    Some(sound_equalizer_add(graphics, min_freq, max_freq))
}
/// `sound_equalizermodifier_init_data` (verbatim).
pub fn sound_equalizermodifier_init_data(graphics: &mut Vec<EQCurveMappingData>) {
    sound_equalizer_add(graphics, SOUND_EQUALIZER_DEFAULT_MIN_FREQ, SOUND_EQUALIZER_DEFAULT_MAX_FREQ);
}
/// `SoundModifierWorkerInfo` verbatim.
pub const SOUND_MODIFIER_WORKER_EQUALIZER: i32 = 0;
pub const SOUND_MODIFIER_WORKER_PITCH: i32 = 1;
