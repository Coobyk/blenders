//! Port of `intern/strip_retiming.cc` — retiming
//! keys, segments, transitions, evaluate.

/// `SEQ_SHOW_RETIMING` verbatim.
pub const SEQ_SHOW_RETIMING: i32 = 1 << 16;
/// `SeqRetimingKeyFlag` verbatim.
pub const SEQ_FREEZE_FRAME_IN: i32 = 1 << 0;
pub const SEQ_FREEZE_FRAME_OUT: i32 = 1 << 1;
pub const SEQ_SPEED_TRANSITION_IN: i32 = 1 << 2;
pub const SEQ_SPEED_TRANSITION_OUT: i32 = 1 << 3;
pub const SEQ_KEY_SELECTED: i32 = 1 << 4;
/// `MINAFRAMEF`/`MAXFRAMEF` verbatim.
pub const MINAFRAMEF: f32 = -1048574.0;
pub const MAXFRAMEF: f32 = 1048574.0;
/// `SeqRetimingKey` verbatim fields.
#[derive(Clone, Copy, Default)]
pub struct SeqRetimingKey {
    /// `strip_frame_index`.
    pub strip_frame_index: f64,
    /// `retiming_factor`.
    pub retiming_factor: f64,
    /// `original_strip_frame_index`.
    pub original_strip_frame_index: f64,
    /// `flag`.
    pub flag: i32,
    /// `sound_length`.
    pub sound_length: f32,
}
/// `retiming_is_last_key` (verbatim).
pub fn retiming_is_last_key(keys: &[SeqRetimingKey], key_index: usize) -> bool {
    key_index == keys.len() - 1
}
/// `retiming_last_key_get` (verbatim).
pub fn retiming_last_key_get(keys: &[SeqRetimingKey]) -> Option<&SeqRetimingKey> {
    keys.last()
}
/// `retiming_key_index_get` (verbatim).
pub fn retiming_key_index_get(keys: &[SeqRetimingKey], key: usize) -> usize {
    key.min(keys.len().saturating_sub(1))
}
/// `left_fake_key_frame_get`/`right_fake_key_frame_get`
/// (verbatim — content_start/end + sound offset
/// clamped to handles).
pub fn left_fake_key_frame_get(content_start: i32, sound_offset: i32, left_handle: i32) -> i32 {
    (content_start + sound_offset).max(left_handle)
}
pub fn right_fake_key_frame_get(content_end: i32, sound_offset: i32, right_handle: i32) -> i32 {
    (content_end + sound_offset).min(right_handle)
}
/// `content_frame_index_get` (verbatim).
pub fn content_frame_index_get(
    frame: i32,
    content_start: f64,
    sound_offset: i32,
    rate_factor: f32,
) -> f32 {
    ((frame as f64 - content_start - sound_offset as f64) * rate_factor as f64) as f32
}
/// `retiming_find_segment_start_key` (verbatim —
/// last key with index <= frame_index).
pub fn retiming_find_segment_start_key(
    keys: &[SeqRetimingKey],
    frame_index: f32,
) -> Option<usize> {
    let mut start_key = None;
    for (i, key) in keys.iter().enumerate() {
        if retiming_is_last_key(keys, i) {
            break;
        }
        if key.strip_frame_index > frame_index as f64 {
            break;
        }
        start_key = Some(i);
    }
    start_key
}
/// `retiming_keys_count`/`retiming_show_keys`/
/// `retiming_has_keys` (verbatim).
pub fn retiming_keys_count(keys: &[SeqRetimingKey]) -> usize {
    keys.len()
}
pub fn retiming_show_keys(flag: i32) -> bool {
    (flag & SEQ_SHOW_RETIMING) != 0
}
pub fn retiming_has_keys(keys: &[SeqRetimingKey]) -> bool {
    !keys.is_empty()
}
/// `retiming_is_allowed` (verbatim — content>1 +
/// media strip type).
pub fn retiming_is_allowed(strip_type: i32, content_length: i32) -> bool {
    if content_length <= 1 {
        return false;
    }
    matches!(
        strip_type,
        crate::utils::STRIP_TYPE_SOUND
            | crate::utils::STRIP_TYPE_IMAGE
            | crate::utils::STRIP_TYPE_META
            | crate::utils::STRIP_TYPE_SCENE
            | crate::utils::STRIP_TYPE_MOVIE
            | crate::utils::STRIP_TYPE_MOVIECLIP
            | crate::utils::STRIP_TYPE_MASK
    )
}
/// `retiming_key_frame_get` (verbatim —
/// content_start + index/rate).
pub fn retiming_key_frame_get(
    content_start: f64,
    sound_offset: i32,
    key: &SeqRetimingKey,
    rate_factor: f32,
) -> i32 {
    (content_start + sound_offset as f64 + key.strip_frame_index / rate_factor as f64).round() as i32
}
/// `retiming_key_speed_get` (verbatim — segment
/// step of key segment).
pub fn retiming_key_speed_get(keys: &[SeqRetimingKey], key_index: usize) -> f64 {
    strip_retiming_segment_step_get(&keys[key_index], keys.get(key_index + 1))
}
/// `retiming_key_is_freeze_frame` (verbatim).
pub fn retiming_key_is_freeze_frame(key: &SeqRetimingKey) -> bool {
    (key.flag & (SEQ_FREEZE_FRAME_IN | SEQ_FREEZE_FRAME_OUT)) != 0
}
/// `retiming_key_is_transition_type`/`_start`
/// (verbatim).
pub fn retiming_key_is_transition_type(key: &SeqRetimingKey) -> bool {
    (key.flag & (SEQ_SPEED_TRANSITION_IN | SEQ_SPEED_TRANSITION_OUT)) != 0
}
pub fn retiming_key_is_transition_start(key: &SeqRetimingKey) -> bool {
    (key.flag & SEQ_SPEED_TRANSITION_IN) != 0
}
/// `retiming_transition_start_get` (verbatim —
/// out→key-1, in→key, else none).
pub fn retiming_transition_start_get(keys: &[SeqRetimingKey], key_index: usize) -> Option<usize> {
    let key = &keys[key_index];
    if (key.flag & SEQ_SPEED_TRANSITION_OUT) != 0 {
        return key_index.checked_sub(1);
    }
    if (key.flag & SEQ_SPEED_TRANSITION_IN) != 0 {
        return Some(key_index);
    }
    None
}
/// `strip_retiming_transition_is_linear` (verbatim —
/// |prev_speed - next_speed| < 0.01).
pub fn strip_retiming_transition_is_linear(keys: &[SeqRetimingKey], key_index: usize) -> bool {
    let prev_speed = retiming_key_speed_get(keys, key_index);
    let next_speed = retiming_key_speed_get(keys, key_index + 2);
    (prev_speed - next_speed).abs() < 0.01
}
/// `strip_retiming_segment_length_get` (verbatim).
pub fn strip_retiming_segment_length_get(start_key: &SeqRetimingKey, end_key: Option<&SeqRetimingKey>) -> f64 {
    end_key.map(|e| e.strip_frame_index - start_key.strip_frame_index).unwrap_or(0.0)
}
/// `strip_retiming_segment_step_get` (verbatim —
/// factor diff / length).
pub fn strip_retiming_segment_step_get(start_key: &SeqRetimingKey, end_key: Option<&SeqRetimingKey>) -> f64 {
    let len = strip_retiming_segment_length_get(start_key, end_key);
    if len == 0.0 {
        return 0.0;
    }
    let fac_diff = end_key.map(|e| e.retiming_factor - start_key.retiming_factor).unwrap_or(0.0);
    fac_diff / len
}
/// `strip_retiming_segment_as_line_segment`.
fn segment_as_line_segment(start_key: &SeqRetimingKey, end_key: Option<&SeqRetimingKey>) -> ([f64; 2], [f64; 2]) {
    let v1 = [start_key.strip_frame_index, start_key.retiming_factor];
    let v2 = end_key.map(|e| [e.strip_frame_index, e.retiming_factor]).unwrap_or(v1);
    (v1, v2)
}
/// `strip_retiming_line_segments_tangent_circle`
/// (verbatim — rotate seg dirs 90°, intersect,
/// radius = |s1_2 - center|).
pub fn strip_retiming_line_segments_tangent_circle(
    keys: &[SeqRetimingKey],
    key_index: usize,
) -> ([f64; 2], f64) {
    let (s1_1_orig, s1_2_orig) = segment_as_line_segment(&keys[key_index - 1], keys.get(key_index));
    let (s2_1_orig, s2_2_orig) = segment_as_line_segment(&keys[key_index + 1], keys.get(key_index + 2));
    let p1_2 = s1_2_orig;
    let mut v1 = [s1_1_orig[0] - s1_2_orig[0], s1_1_orig[1] - s1_2_orig[1]];
    let mut v2 = [s2_1_orig[0] - s2_2_orig[0], s2_1_orig[1] - s2_2_orig[1]];
    std::mem::swap(&mut v1[0], &mut v1[1]);
    std::mem::swap(&mut v2[0], &mut v2[1]);
    v1[0] = -v1[0];
    v2[0] = -v2[0];
    let s1_1 = s1_2_orig;
    let s1_2 = [s1_2_orig[0] + v1[0], s1_2_orig[1] + v1[1]];
    let s2_1 = s2_2_orig;
    let s2_2 = [s2_1_orig[0] + v2[0], s2_1_orig[1] + v2[1]];
    let (lambda, _mu) = isect_seg_seg_v2_lambda_mu(s1_1, s1_2, s2_1, s2_2);
    let center = [
        s1_1[0] + lambda * (s1_2[0] - s1_1[0]),
        s1_1[1] + lambda * (s1_2[1] - s1_1[1]),
    ];
    let radius = ((p1_2[0] - center[0]).powi(2) + (p1_2[1] - center[1]).powi(2)).sqrt();
    (center, radius)
}
fn isect_seg_seg_v2_lambda_mu(a1: [f64; 2], a2: [f64; 2], b1: [f64; 2], b2: [f64; 2]) -> (f64, f64) {
    let da = [a2[0] - a1[0], a2[1] - a1[1]];
    let db = [b2[0] - b1[0], b2[1] - b1[1]];
    let dc = [b1[0] - a1[0], b1[1] - a1[1]];
    let denom = da[0] * db[1] - da[1] * db[0];
    if denom.abs() < 1e-20 {
        return (0.0, 0.0);
    }
    ((dc[0] * db[1] - dc[1] * db[0]) / denom, (dc[0] * da[1] - dc[1] * da[0]) / denom)
}
/// `strip_retiming_evaluate_arc_segment` (verbatim —
/// tangent-circle arc height).
pub fn strip_retiming_evaluate_arc_segment(
    keys: &[SeqRetimingKey],
    key_index: usize,
    frame_index: f32,
) -> f32 {
    let (c, r) = strip_retiming_line_segments_tangent_circle(keys, key_index);
    let key = &keys[key_index];
    let side = if c[1] > key.retiming_factor { -1.0 } else { 1.0 };
    let dx = frame_index as f64 - c[0];
    (c[1] + side * (r * r - dx * dx).sqrt()) as f32
}
/// `strip_retiming_evaluate` (verbatim — linear
/// segment / linear-transition / arc).
pub fn strip_retiming_evaluate(keys: &[SeqRetimingKey], frame_index: f32) -> f32 {
    let Some(start_key_index) = retiming_find_segment_start_key(keys, frame_index) else {
        return 0.0;
    };
    let start_key = &keys[start_key_index];
    let segment_frame_index = frame_index as f64 - start_key.strip_frame_index;
    if !retiming_key_is_transition_start(start_key) {
        let step = strip_retiming_segment_step_get(start_key, keys.get(start_key_index + 1));
        return (start_key.retiming_factor + step * segment_frame_index).min(1.0) as f32;
    }
    if strip_retiming_transition_is_linear(keys, start_key_index) {
        let step = strip_retiming_segment_step_get(&keys[start_key_index - 1], keys.get(start_key_index));
        return (start_key.retiming_factor + step * segment_frame_index).min(1.0) as f32;
    }
    strip_retiming_evaluate_arc_segment(keys, start_key_index, frame_index).min(1.0)
}
/// `strip_retiming_add_key` (verbatim — allowed,
/// clamp ends, find segment, insert key w/ eval).
pub fn strip_retiming_add_key(
    keys: &mut Vec<SeqRetimingKey>,
    strip_type: i32,
    content_length: i32,
    frame_index: f32,
) -> Option<usize> {
    if !retiming_is_allowed(strip_type, content_length) {
        return None;
    }
    if frame_index <= 0.0 {
        return Some(0);
    }
    if let Some(last) = keys.last() {
        if frame_index >= last.strip_frame_index as f32 {
            return Some(keys.len() - 1);
        }
    }
    let start_key_index = retiming_find_segment_start_key(keys, frame_index)?;
    if keys[start_key_index].strip_frame_index == frame_index as f64 {
        return Some(start_key_index);
    }
    if (keys[start_key_index].flag & SEQ_SPEED_TRANSITION_IN) != 0
        || (keys[start_key_index].flag & SEQ_FREEZE_FRAME_IN) != 0
    {
        return None;
    }
    let value = strip_retiming_evaluate(keys, frame_index);
    let new_index = start_key_index + 1;
    keys.insert(
        new_index,
        SeqRetimingKey {
            strip_frame_index: frame_index as f64,
            retiming_factor: value as f64,
            ..Default::default()
        },
    );
    Some(new_index)
}
