//! Port of `intern/fcurve.cc` — keyframe insert/delete,
//! cyclic remap, sampling.

/// `INSERTKEY_*` extra flags verbatim.
pub const INSERTKEY_FAST: i32 = 1 << 1;
pub const INSERTKEY_OVERWRITE_FULL: i32 = 1 << 2;
/// `eFCU_Cycle_Type` verbatim.
pub const FCU_CYCLE_NONE: i32 = 0;
pub const FCU_CYCLE_PERFECT: i32 = 1;
pub const FCU_CYCLE_OFFSET: i32 = 2;
/// `FCM_EXTRAPOLATE_CYCLIC_OFFSET` verbatim.
pub const FCM_EXTRAPOLATE_CYCLIC_OFFSET: i32 = 2;

/// `replace_bezt_keyframe_ypos` (verbatim — dy shift on
/// all handles + f flags).
pub fn replace_bezt_keyframe_ypos(dst_vec: &mut [[f32; 2]; 3], dst_f: &mut [u8; 3], src_vec: [[f32; 2]; 3], src_f: [u8; 3]) {
    let dy = src_vec[1][1] - dst_vec[1][1];
    dst_vec[0][1] += dy;
    dst_vec[1][1] += dy;
    dst_vec[2][1] += dy;
    *dst_f = src_f;
}

/// `new_key_needed` (verbatim — replace→value-diff,
/// else relative compare to eval).
pub fn new_key_needed(
    totvert: usize,
    replace_found: bool,
    existing_value: f32,
    fcu_eval_at_frame: f32,
    value: f32,
    flt_epsilon: f32,
) -> bool {
    if totvert == 0 {
        return true;
    }
    if replace_found {
        return existing_value != value;
    }
    let diff_ulp = 32.0 * flt_epsilon;
    if (fcu_eval_at_frame - value).abs() <= diff_ulp.max(flt_epsilon) {
        return false;
    }
    true
}

/// `remap_cyclic_keyframe_location` (verbatim — wrap
/// into [start,end], offset mode shifts y).
pub fn remap_cyclic_keyframe_location(
    totvert: usize,
    cycle_type: i32,
    start_x: f32,
    end_x: f32,
    first_y: f32,
    last_y: f32,
    step_ge0_mode: i32,
    step_lt0_mode: i32,
    mut position: [f32; 2],
) -> [f32; 2] {
    if totvert < 2 || cycle_type == FCU_CYCLE_NONE {
        return position;
    }
    let start = start_x;
    let end = end_x;
    if start >= end {
        return position;
    }
    if position[0] < start || position[0] > end {
        let period = end - start;
        let step = ((position[0] - start) / period).floor();
        position[0] -= step * period;
        if cycle_type == FCU_CYCLE_OFFSET {
            let mode = if step >= 0.0 { step_ge0_mode } else { step_lt0_mode };
            if mode == FCM_EXTRAPOLATE_CYCLIC_OFFSET {
                position[1] -= step * (last_y - first_y);
            }
        }
    }
    position
}

/// `sample_fcurve_segment` (verbatim — evaluate at
/// start + i/rate).
pub fn sample_fcurve_segment(
    start_frame: f32,
    sample_rate: f32,
    sample_count: usize,
    evaluate: &mut dyn FnMut(f32) -> f32,
) -> Vec<f32> {
    (0..sample_count)
        .map(|i| (evaluate)(start_frame + (i as f32) / sample_rate))
        .collect()
}

/// `fcurve_frame_has_keyframe` (verbatim — not muted &&
/// binarysearch replace).
pub fn fcurve_frame_has_keyframe(
    bezt_present: bool,
    fcu_flag: i32,
    binary_search_replace: bool,
    index_in_bounds: bool,
) -> bool {
    if !bezt_present {
        return false;
    }
    if (fcu_flag & crate::pose::FCURVE_MUTED) == 0 {
        if binary_search_replace && index_in_bounds {
            return true;
        }
    }
    false
}

/// `insert_bezt_fcurve` decision (verbatim —
/// binarysearch; replace: overwrite/dy-shift + perfect-
/// cycle endpoint twin; insert when !REPLACE).
pub enum InsertBezt {
    /// Overwrite existing at index.
    ReplaceAt(usize, bool),
    /// Insert new at index.
    InsertAt(usize),
    /// Return -1 (cannot add).
    Fail,
}
pub fn insert_bezt_fcurve_decision(
    has_bezt: bool,
    replace_found: bool,
    binary_index: i32,
    totvert: usize,
    flag: i32,
    cycle_is_perfect: bool,
) -> InsertBezt {
    if has_bezt {
        if replace_found {
            if binary_index >= 0 && (binary_index as usize) < totvert {
                let twin = (flag & crate::keyframing_auto::INSERTKEY_CYCLE_AWARE) != 0
                    && (binary_index == 0 || binary_index as usize == totvert - 1)
                    && cycle_is_perfect;
                return InsertBezt::ReplaceAt(binary_index as usize, twin);
            }
            return InsertBezt::ReplaceAt(usize::MAX, false);
        }
        if (flag & crate::keyframing_auto::INSERTKEY_REPLACE) == 0 {
            return InsertBezt::InsertAt(binary_index.max(0) as usize);
        }
        return InsertBezt::Fail;
    }
    if (flag & crate::keyframing_auto::INSERTKEY_REPLACE) == 0 {
        InsertBezt::InsertAt(0)
    } else {
        InsertBezt::Fail
    }
}
