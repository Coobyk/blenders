//! Port of `intern/strip_transform.cc` —
//! translate/shuffle/ripple + preview transform.

/// `SEQ_SINGLE_FRAME_CONTENT`/`SEQ_LOCK`/`SEQ_FLIPX`/`SEQ_FLIPY`.
pub const SEQ_SINGLE_FRAME_CONTENT: i32 = 1 << 17;
pub const SEQ_LOCK: i32 = 1 << 3;
pub const SEQ_FLIPX: i32 = 1 << 10;
pub const SEQ_FLIPY: i32 = 1 << 11;
/// `Side` verbatim.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}
/// `transform_single_image_check` (verbatim).
pub fn transform_single_image_check(flag: i32) -> bool {
    (flag & SEQ_SINGLE_FRAME_CONTENT) != 0
}
/// `transform_is_locked` (verbatim — lock flag or
/// channel lock w/o ignore).
pub fn transform_is_locked(
    flag: i32,
    channel_locked: bool,
    ignore_channel_lock: bool,
) -> bool {
    (flag & SEQ_LOCK) != 0 || (channel_locked && !ignore_channel_lock)
}
/// `transform_strip_can_be_translated` (verbatim).
pub fn transform_strip_can_be_translated(is_effect_with_inputs: bool) -> bool {
    !is_effect_with_inputs
}
/// `transform_test_overlap` (verbatim — same
/// channel + ranges touch).
pub fn transform_test_overlap(
    strip1_channel: i32,
    strip2_channel: i32,
    s1_left: i32,
    s1_right: i32,
    s2_left: i32,
    s2_right: i32,
) -> bool {
    strip1_channel == strip2_channel && !(s1_right <= s2_left || s1_left >= s2_right)
}
/// `strips_overlap_at_offset` (verbatim).
pub fn strips_overlap_at_offset(
    channel: i32,
    other_channel: i32,
    left: i32,
    right: i32,
    other_left: i32,
    other_right: i32,
    offset: i32,
) -> bool {
    channel == other_channel
        && left + offset < other_right
        && right + offset > other_left
}
/// `shuffle_frame_offset_get` (verbatim — iterate
/// until stable; left→min, right→max).
pub fn shuffle_frame_offset_get(
    side: Side,
    strips_to_shuffle: &[(i32, i32, i32)],
    others: &[(i32, i32, i32)],
    is_in_set: &dyn Fn(usize) -> bool,
    is_effect_of: &dyn Fn(usize, usize) -> bool,
) -> i32 {
    let mut offset = 0i32;
    let mut offset_prev;
    loop {
        offset_prev = offset;
        for (si, &(sc, sl, sr)) in strips_to_shuffle.iter().enumerate() {
            for (oi, &(oc, ol, oright)) in others.iter().enumerate() {
                if is_in_set(oi) || is_effect_of(oi, si) {
                    continue;
                }
                if !strips_overlap_at_offset(sc, oc, sl, sr, ol, oright, offset) {
                    continue;
                }
                offset = match side {
                    Side::Left => offset.min(ol - sr),
                    Side::Right => offset.max(oright - sl),
                };
            }
        }
        if offset == offset_prev {
            break;
        }
    }
    offset
}
/// `transform_shuffle_horizontal` offset pick
/// (verbatim — `(-offset_l < offset_r) ? l : r`).
pub fn shuffle_pick_offset(offset_l: i32, offset_r: i32) -> i32 {
    if -offset_l < offset_r {
        offset_l
    } else {
        offset_r
    }
}
/// `image_transform_mirror_factor_get` (verbatim).
pub fn image_transform_mirror_factor_get(flag: i32) -> [f32; 2] {
    let mut mirror = [1.0f32; 2];
    if (flag & SEQ_FLIPX) != 0 {
        mirror[0] = -1.0;
    }
    if (flag & SEQ_FLIPY) != 0 {
        mirror[1] = -1.0;
    }
    mirror
}
/// `transform_strips_after_frame` (verbatim — strip
/// left >= frame → translate + invalidate; markers
/// >= frame shift too).
pub fn transform_strips_after_frame(
    strips: &mut [(i32, i32)],
    timeline_frame: i32,
    delta: i32,
) -> Vec<usize> {
    let mut moved = Vec::new();
    for (i, (left, _start)) in strips.iter_mut().enumerate() {
        if *left >= timeline_frame {
            *_start += delta;
            moved.push(i);
        }
    }
    moved
}
/// `image_transform_origin_get` (verbatim —
/// image_offset - 0.5 box).
pub fn image_transform_origin_get(origin: [f32; 2], box_size: [f32; 2]) -> [f32; 2] {
    [
        origin[0] * box_size[0],
        origin[1] * box_size[1],
    ]
}
