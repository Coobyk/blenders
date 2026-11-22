//! Port of `intern/keyframing_auto.cc` — autokey
//! flags/mode checks + keyframe dispatch.

/// `AUTOKEY_*`/`INSERTKEY_*`/`KEYING_FLAG_*` verbatim.
pub const AUTOKEY_ON: i16 = 1;
pub const AUTOKEY_MODE_NORMAL: i16 = 0;
pub const AUTOKEY_MODE_EDITKEYS: i16 = 1;
pub const INSERTKEY_NOFLAGS: i32 = 0;
pub const INSERTKEY_MATRIX: i32 = 1 << 0;
pub const INSERTKEY_NEEDED: i32 = 1 << 3;
pub const INSERTKEY_AVAILABLE: i32 = 1 << 4;
pub const INSERTKEY_REPLACE: i32 = 1 << 5;
pub const INSERTKEY_CYCLE_AWARE: i32 = 1 << 7;
pub const AUTOKEY_FLAG_INSERTNEEDED: i32 = 1 << 1;
pub const AUTOKEY_FLAG_INSERTAVAILABLE: i32 = 1 << 2;
pub const AUTOKEY_FLAG_ONLYKEYINGSET: i32 = 1 << 6;
pub const KEYING_FLAG_VISUALKEY: i32 = 1 << 4;
pub const KEYING_FLAG_CYCLEAWARE: i32 = 1 << 5;

/// `is_autokey_on`/`is_autokey_mode` (verbatim — scene
/// fallback to userpref).
pub fn is_autokey_on(scene_mode: Option<i16>, userpref_mode: i16) -> bool {
    let mode = scene_mode.unwrap_or(userpref_mode);
    (mode & AUTOKEY_ON) != 0
}
pub fn is_autokey_mode(scene_mode: Option<i16>, userpref_mode: i16, mode: i16) -> bool {
    let m = scene_mode.unwrap_or(userpref_mode);
    m == mode
}

/// `get_autokey_flags` (verbatim — bit assembly from
/// keying flags/mode).
pub fn get_autokey_flags(
    is_visualkey: bool,
    is_insertneeded: bool,
    is_insertavailable: bool,
    is_editkeys: bool,
    is_cycleaware: bool,
) -> i32 {
    let mut flag = INSERTKEY_NOFLAGS;
    if is_visualkey {
        flag |= INSERTKEY_MATRIX;
    }
    if is_insertneeded {
        flag |= INSERTKEY_NEEDED;
    }
    if is_insertavailable {
        flag |= INSERTKEY_AVAILABLE;
    }
    if is_editkeys {
        flag |= INSERTKEY_REPLACE;
    }
    if is_cycleaware {
        flag |= INSERTKEY_CYCLE_AWARE;
    }
    flag
}

/// `autokeyframe_cfra_can_key` (verbatim — on + editkeys
/// → has-keyframe-at-cfra).
pub fn autokeyframe_cfra_can_key(
    autokey_on: bool,
    is_editkeys: bool,
    id_frame_has_keyframe: bool,
) -> bool {
    if !autokey_on {
        return false;
    }
    if is_editkeys {
        return id_frame_has_keyframe;
    }
    true
}
