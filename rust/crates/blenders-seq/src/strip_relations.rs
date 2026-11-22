//! Port of `intern/strip_relations.cc` — cache
//! invalidation + recursion/loop/uid checks.

/// `relation_is_effect_of_strip` (verbatim).
pub fn relation_is_effect_of_strip(
    effect_input1: Option<usize>,
    effect_input2: Option<usize>,
    input: usize,
) -> bool {
    effect_input1 == Some(input) || effect_input2 == Some(input)
}
/// `CacheCleanup` verbatim bits.
pub const CACHE_CLEANUP_THUMBNAILS: i32 = 1 << 0;
pub const CACHE_CLEANUP_FINAL: i32 = 1 << 1;
pub const CACHE_CLEANUP_RAW: i32 = 1 << 2;
/// `SEQ_SCENE_STRIPS` verbatim flag.
pub const SEQ_SCENE_STRIPS: i32 = 1 << 6;
/// `sequencer_check_scene_recursion` (verbatim —
/// self-scene strip, scene-strip recurse, meta
/// recurse).
pub fn sequencer_check_scene_recursion(
    scene: usize,
    strips: &[(usize, i32, Option<usize>, i32, Vec<usize>)],
    get_child_strips: &dyn Fn(usize) -> Vec<(usize, i32, Option<usize>, i32, Vec<usize>)>,
    scene_is_scene_strips: &dyn Fn(i32) -> bool,
) -> Option<usize> {
    for &(strip, strip_type, strip_scene, flag, ref children) in strips {
        if strip_type == crate::utils::STRIP_TYPE_SCENE && strip_scene == Some(scene) {
            return Some(strip);
        }
        if strip_type == crate::utils::STRIP_TYPE_SCENE && scene_is_scene_strips(flag) {
            if let Some(_sc) = strip_scene {
                let inner = get_child_strips(strip);
                if sequencer_check_scene_recursion(scene, &inner, get_child_strips, scene_is_scene_strips).is_some() {
                    return Some(strip);
                }
            }
        }
        if strip_type == crate::utils::STRIP_TYPE_META && !children.is_empty() {
            let inner: Vec<(usize, i32, Option<usize>, i32, Vec<usize>)> = children
                .iter()
                .map(|&s| (s, 0, None, 0, Vec::new()))
                .collect();
            if sequencer_check_scene_recursion(scene, &inner, get_child_strips, scene_is_scene_strips).is_some() {
                return Some(strip);
            }
        }
    }
    None
}
/// `relations_render_loop_check` (verbatim — main
/// strip reaches `strip` via inputs or masks).
pub fn relations_render_loop_check(
    strip_main: Option<usize>,
    strip: usize,
    input1: &dyn Fn(usize) -> Option<usize>,
    input2: &dyn Fn(usize) -> Option<usize>,
    mask_strips: &dyn Fn(usize) -> Vec<usize>,
) -> bool {
    let Some(main) = strip_main else {
        return false;
    };
    if main == strip {
        return true;
    }
    if let Some(i1) = input1(main) {
        if relations_render_loop_check(Some(i1), strip, input1, input2, mask_strips) {
            return true;
        }
    }
    if let Some(i2) = input2(main) {
        if relations_render_loop_check(Some(i2), strip, input1, input2, mask_strips) {
            return true;
        }
    }
    for mask in mask_strips(main) {
        if relations_render_loop_check(Some(mask), strip, input1, input2, mask_strips) {
            return true;
        }
    }
    false
}
/// `relations_check_uids_unique_and_report`
/// (verbatim — generated + unique).
pub fn check_uids_unique(uids: &[u64]) -> Vec<usize> {
    let mut seen = std::collections::HashSet::new();
    let mut bad = Vec::new();
    for (i, &uid) in uids.iter().enumerate() {
        if uid == 0 || !seen.insert(uid) {
            bad.push(i);
        }
    }
    bad
}
/// `exists_in_seqbase` (verbatim — recursive into
/// metas).
pub fn exists_in_seqbase(
    strip: usize,
    seqbase: &[(usize, i32)],
    meta_children: &dyn Fn(usize) -> Vec<(usize, i32)>,
) -> bool {
    for &(id, typ) in seqbase {
        if id == strip {
            return true;
        }
        if typ == crate::utils::STRIP_TYPE_META
            && exists_in_seqbase(strip, &meta_children(id), meta_children)
        {
            return true;
        }
    }
    false
}
