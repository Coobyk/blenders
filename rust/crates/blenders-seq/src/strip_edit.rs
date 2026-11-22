//! Port of `intern/strip_edit.cc` — swap, split,
//! gap removal, rename.

/// `GapInfo` verbatim.
#[derive(Default)]
pub struct GapInfo {
    /// `gap_exists`/`gap_length`/`gap_start_frame`.
    pub gap_exists: bool,
    pub gap_length: i32,
    pub gap_start_frame: i32,
}
/// `edit_strip_swap` validity (verbatim — length
/// equal + type constraints + effect input count).
pub fn edit_strip_swap_allowed(
    length_a: i32,
    length_b: i32,
    type_a: i32,
    type_b: i32,
    is_effect_a: bool,
    is_effect_b: bool,
    inputs_a: i32,
    inputs_b: i32,
) -> bool {
    if length_a != length_b {
        return false;
    }
    if type_a != type_b {
        if type_a == crate::utils::STRIP_TYPE_SOUND
            || type_b == crate::utils::STRIP_TYPE_SOUND
        {
            return false;
        }
        if is_effect_a != is_effect_b {
            return false;
        }
        if is_effect_a && is_effect_b && inputs_a != inputs_b {
            return false;
        }
    }
    true
}
/// `edit_frame_splits_strip` (verbatim — frame in
/// (start, end)).
pub fn edit_frame_splits_strip(start: f64, end: f64, timeline_frame: f64) -> bool {
    timeline_frame > start && timeline_frame < end
}
/// `edit_remove_gaps` (verbatim — collapse all
/// gaps after `initial_frame`).
pub fn edit_remove_gaps(
    initial_frame: i32,
    remove_all_gaps: bool,
    mut gap_info: impl FnMut() -> GapInfo,
    mut transform_after: impl FnMut(i32, i32),
) -> bool {
    let mut gap = gap_info();
    if !gap.gap_exists {
        return false;
    }
    if remove_all_gaps {
        while gap.gap_exists {
            transform_after(gap.gap_start_frame, -gap.gap_length);
            gap = gap_info();
        }
    } else {
        transform_after(gap.gap_start_frame, -gap.gap_length);
    }
    true
}
/// `edit_strip_name_set` (verbatim — "  " prefix
/// preserved; utf8-invalid strip).
pub fn edit_strip_name_set(new_name: &str) -> String {
    let mut name = String::with_capacity(new_name.len() + 2);
    name.push_str("  ");
    let valid: String = new_name
        .chars()
        .filter(|&c| c != '\0' && !c.is_control() || c == ' ')
        .take(62)
        .collect();
    name.push_str(&valid);
    name
}
