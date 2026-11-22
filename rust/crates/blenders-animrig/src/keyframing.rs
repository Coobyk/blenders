//! Port of `intern/keyframing.cc` —
//! `CombinedKeyingResult`, keying flags,
//! `id_frame_has_keyframe`, `delete_keyframe`.

/// `eSingleKeyingResult` verbatim ordinals.
pub const SINGLE_KEYING_RESULT_SUCCESS: usize = 0;
pub const SINGLE_KEYING_RESULT_COUNT: usize = 9;

/// `CombinedKeyingResult` verbatim.
#[derive(Default)]
pub struct CombinedKeyingResult {
    /// `result_counter`.
    pub result_counter: [i32; SINGLE_KEYING_RESULT_COUNT],
}
impl CombinedKeyingResult {
    /// `add` (verbatim).
    pub fn add(&mut self, result: usize, count: i32) {
        self.result_counter[result] += count;
    }
    /// `merge` (verbatim).
    pub fn merge(&mut self, other: &CombinedKeyingResult) {
        for i in 0..SINGLE_KEYING_RESULT_COUNT {
            self.result_counter[i] += other.result_counter[i];
        }
    }
    /// `get_count`.
    pub fn get_count(&self, result: usize) -> i32 {
        self.result_counter[result]
    }
    /// `has_errors` (verbatim — any non-SUCCESS > 0).
    pub fn has_errors(&self) -> bool {
        self.result_counter[1..].iter().any(|&c| c > 0)
    }
}

/// `is_keying_flag` (verbatim — scene || userpref).
pub fn is_keying_flag(scene_keying_flag: Option<i32>, userpref_keying_flag: i32, flag: i32) -> bool {
    match scene_keying_flag {
        Some(s) => (s & flag) != 0 || (userpref_keying_flag & flag) != 0,
        None => (userpref_keying_flag & flag) != 0,
    }
}

/// `key_insertion_may_create_fcurve` (verbatim —
/// neither REPLACE nor AVAILABLE).
pub fn key_insertion_may_create_fcurve(insert_key_flags: i32) -> bool {
    (insert_key_flags
        & (crate::keyframing_auto::INSERTKEY_REPLACE | crate::keyframing_auto::INSERTKEY_AVAILABLE))
        == 0
}

/// `assigned_action_has_keyframe_at` (verbatim — action
/// present, not ACT_MUTED, any fcurve has keyframe).
pub const ACT_MUTED: i32 = 1 << 0;
pub fn assigned_action_has_keyframe_at(
    action_present: bool,
    action_flag: i32,
    fcurve_has_keyframe: &mut dyn FnMut() -> bool,
) -> bool {
    if !action_present {
        return false;
    }
    if (action_flag & ACT_MUTED) != 0 {
        return false;
    }
    (fcurve_has_keyframe)()
}
