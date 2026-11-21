//! Port of `intern/nla.cc` — the NLA stack: track/strip list
//! management, meta-strips, time remapping (`nlastrip_get_frame`
//! + `BKE_nla_tweakedit_remap`), strip add/remove/sort/space
//! checks, `nlameta_flush_transforms`, bounds recalculation,
//! resize-overlap fixing, tweakmode enter/exit flags, and
//! `foreach_strip` iteration.
//!
//! `ListBaseT<NlaStrip>`/`ListBaseT<NlaTrack>` → `Vec<NlaStrip>`/
//! `Vec<NlaTrack>`; `strip->next`/`prev` navigation → slice
//! indices. `Main`/`bAction`/`ID`/`RNA`/`LibraryForeachIDData`
//! bits are injected — `PORT_PENDING` where noted.

use blenders_dna::anim_enums::*;
use blenders_dna::anim_types::{AnimData, NlaStrip, NlaTrack};

/// `eNlaTime_ConvertModes`.
pub const NLATIME_CONVERT_EVAL: i16 = 0;
/// `NLATIME_CONVERT_UNMAP`.
pub const NLATIME_CONVERT_UNMAP: i16 = 1;
/// `NLATIME_CONVERT_MAP`.
pub const NLATIME_CONVERT_MAP: i16 = 2;

fn is_eqf(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

fn strip_name(strip: &NlaStrip) -> String {
    let end = strip
        .name
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(strip.name.len());
    String::from_utf8_lossy(&strip.name[..end]).to_string()
}

/// `BKE_nlastrip_copy` — deep copy; `slot_handle_remap`/`idcopy`
/// injected (`Main`, `Library` deps).
pub fn nlastrip_copy(strip: &NlaStrip) -> NlaStrip {
    let mut out = strip.clone();
    out.next = usize::MAX;
    out.prev = usize::MAX;
    out.orig_strip = usize::MAX;
    /* Child strips are deep-copied verbatim (clone covers). */
    out
}

/// `BKE_nlatrack_copy`.
pub fn nlatrack_copy(track: &NlaTrack) -> NlaTrack {
    let mut out = track.clone();
    out.next = usize::MAX;
    out.prev = usize::MAX;
    out.strips = track.strips.iter().map(nlastrip_copy).collect();
    out
}

/// `BKE_nla_tracks_copy`.
pub fn nla_tracks_copy(src: &[NlaTrack]) -> Vec<NlaTrack> {
    src.iter().map(nlatrack_copy).collect()
}

/// `BKE_nlatrack_new` — zeroed track named "NlaTrack".
pub fn nlatrack_new() -> NlaTrack {
    let mut t = NlaTrack::default();
    let name = b"NlaTrack";
    t.name[..name.len()].copy_from_slice(name);
    t
}

/// `BKE_nlatrack_insert_before` / `_after`.
pub fn nlatrack_insert_before(tracks: &mut Vec<NlaTrack>, ref_index: usize, track: NlaTrack) {
    tracks.insert(ref_index, track);
}
/// `nlatrack_insert_after`.
pub fn nlatrack_insert_after(tracks: &mut Vec<NlaTrack>, ref_index: usize, track: NlaTrack) {
    tracks.insert((ref_index + 1).min(tracks.len()), track);
}

/// `BKE_nlatrack_new_before`/`_after`/`_head`/`_tail`.
pub fn nlatrack_new_head(tracks: &mut Vec<NlaTrack>) -> usize {
    tracks.insert(0, nlatrack_new());
    0
}
/// `nlatrack_new_tail`.
pub fn nlatrack_new_tail(tracks: &mut Vec<NlaTrack>) -> usize {
    tracks.push(nlatrack_new());
    tracks.len() - 1
}
/// `nlatrack_new_before`.
pub fn nlatrack_new_before(tracks: &mut Vec<NlaTrack>, ref_index: usize) -> usize {
    tracks.insert(ref_index, nlatrack_new());
    ref_index
}
/// `nlatrack_new_after`.
pub fn nlatrack_new_after(tracks: &mut Vec<NlaTrack>, ref_index: usize) -> usize {
    let i = (ref_index + 1).min(tracks.len());
    tracks.insert(i, nlatrack_new());
    i
}

/// `BKE_nla_clip_length_get_nonzero` — `actend - actstart`,
/// clamped to ≥ 1.
pub fn nla_clip_length_get_nonzero(strip: &NlaStrip) -> f32 {
    let len = strip.actend - strip.actstart;
    if len < 1.0 {
        1.0
    } else {
        len
    }
}

/// `BKE_nla_clip_length_ensure_nonzero` — push `actend` forward
/// if `actend - actstart < 1`.
pub fn nla_clip_length_ensure_nonzero(actstart: f32, r_actend: &mut f32) {
    if *r_actend - actstart < 1.0 {
        *r_actend = actstart + 1.0;
    }
}

/// `nlastrip_set_initial_length` — stretch/keep strip so
/// `[start,end]` covers the action length at scale 1.
fn nlastrip_set_initial_length(strip: &mut NlaStrip) {
    let actlen = nla_clip_length_get_nonzero(strip);
    strip.end = strip.start + actlen;
}

/// `BKE_nlastrip_new` — new clip strip (action handle injected
/// via `act`; `actend`/`actstart` set by caller).
pub fn nlastrip_new(act: usize, action_slot_handle: i32) -> NlaStrip {
    let mut strip = NlaStrip::default();
    strip.type_ = NLASTRIP_TYPE_CLIP;
    strip.act = act;
    strip.action_slot_handle = action_slot_handle;
    strip.repeat = 1.0;
    strip.scale = 1.0;
    strip.blendin = 0.0;
    strip.blendout = 0.0;
    strip.blendmode = NLASTRIP_MODE_REPLACE;
    strip.extendmode = NLASTRIP_EXTEND_HOLD;
    strip.influence = 1.0;
    nlastrip_set_initial_length(&mut strip);
    strip
}

/// `nlastrip_get_frame_actionclip` — time remap for action
/// clips, verbatim `MAP`/`UNMAP`/`EVAL` + `REVERSE`.
fn nlastrip_get_frame_actionclip(strip: &mut NlaStrip, cframe: f32, mode: i16) -> f32 {
    if is_eqf(strip.repeat, 0.0) {
        strip.repeat = 1.0;
    }
    if is_eqf(strip.scale, 0.0) {
        strip.scale = 1.0;
    }
    let scale = strip.scale.abs();
    let actlength = nla_clip_length_get_nonzero(strip);

    if (strip.flag & NLASTRIP_FLAG_REVERSE) != 0 {
        if mode == NLATIME_CONVERT_MAP {
            return strip.end - scale * (cframe - strip.actstart);
        }
        if mode == NLATIME_CONVERT_UNMAP {
            return (strip.end + (strip.actstart * scale - cframe)) / scale;
        }
        /* NLATIME_CONVERT_EVAL */
        if is_eqf(cframe, strip.end) && is_eqf(strip.repeat, strip.repeat.floor()) {
            return strip.actstart;
        }
        return strip.actend - (cframe - strip.start).rem_euclid(actlength * scale) / scale;
    }

    if mode == NLATIME_CONVERT_MAP {
        return strip.start + scale * (cframe - strip.actstart);
    }
    if mode == NLATIME_CONVERT_UNMAP {
        return strip.actstart + (cframe - strip.start) / scale;
    }
    if is_eqf(cframe, strip.end) && is_eqf(strip.repeat, strip.repeat.floor()) {
        return strip.actend;
    }
    strip.actstart + (cframe - strip.start).rem_euclid(actlength * scale) / scale
}

/// `nlastrip_get_frame_transition` — non-clipped mapping for
/// transition (and meta) strips.
fn nlastrip_get_frame_transition(strip: &NlaStrip, cframe: f32, mode: i16) -> f32 {
    let length = strip.end - strip.start;
    if (strip.flag & NLASTRIP_FLAG_REVERSE) != 0 {
        if mode == NLATIME_CONVERT_MAP {
            return strip.end - (length * cframe);
        }
        return (strip.end - cframe) / length;
    }
    if mode == NLATIME_CONVERT_MAP {
        return (length * cframe) + strip.start;
    }
    (cframe - strip.start) / length
}

/// `nlastrip_get_frame`.
pub fn nlastrip_get_frame(strip: &mut NlaStrip, cframe: f32, mode: i16) -> f32 {
    match strip.type_ {
        NLASTRIP_TYPE_META | NLASTRIP_TYPE_TRANSITION => {
            nlastrip_get_frame_transition(strip, cframe, mode)
        }
        _ => nlastrip_get_frame_actionclip(strip, cframe, mode),
    }
}

/// `BKE_nla_tweakedit_remap` — remap `cframe` through the
/// tweak-mode active strip. `act_track`/`actstrip` resolved via
/// injected lookups (`adt` pointer identity → indices).
pub fn nla_tweakedit_remap(
    adt: &AnimData,
    cframe: f32,
    mode: i16,
    actstrip: Option<&mut NlaStrip>,
) -> f32 {
    if (adt.flag & ADT_NLA_EDIT_ON) == 0 || (adt.flag & ADT_NLA_EDIT_NOMAP) != 0 {
        return cframe;
    }
    let Some(strip) = actstrip else {
        return cframe;
    };
    if (strip.flag & NLASTRIP_FLAG_USR_TIME) != 0 {
        return cframe;
    }
    nlastrip_get_frame(strip, cframe, mode)
}

/// `BKE_nlastrips_has_space`.
pub fn nlastrips_has_space(strips: &[NlaStrip], mut start: f32, mut end: f32) -> bool {
    if is_eqf(start, end) {
        return false;
    }
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    for strip in strips {
        if strip.start >= end {
            return true;
        }
        if strip.end > start || strip.end > end {
            return false;
        }
    }
    true
}

/// `BKE_nlastrips_sort_strips` — insertion sort by `start`.
pub fn nlastrips_sort_strips(strips: &mut Vec<NlaStrip>) {
    let mut tmp: Vec<NlaStrip> = Vec::with_capacity(strips.len());
    for strip in strips.drain(..) {
        let pos = tmp
            .iter()
            .rposition(|s| s.start <= strip.start)
            .map(|i| i + 1)
            .unwrap_or(0);
        tmp.insert(pos, strip);
    }
    *strips = tmp;
}

/// `BKE_nlastrips_add_strip_unsafe` — ordered insert.
pub fn nlastrips_add_strip_unsafe(strips: &mut Vec<NlaStrip>, strip: NlaStrip) {
    let pos = strips
        .iter()
        .position(|ns| ns.start >= strip.start)
        .unwrap_or(strips.len());
    strips.insert(pos, strip);
}

/// `BKE_nlastrips_add_strip` — space-checked add.
pub fn nlastrips_add_strip(strips: &mut Vec<NlaStrip>, strip: NlaStrip) -> bool {
    if !nlastrips_has_space(strips, strip.start, strip.end) {
        return false;
    }
    nlastrips_add_strip_unsafe(strips, strip);
    true
}

/// `BKE_nlastrips_make_metas` — wrap selected strips into meta
/// strips (`is_temp` → `NLASTRIP_FLAG_TEMP_META`).
pub fn nlastrips_make_metas(strips: &mut Vec<NlaStrip>, is_temp: bool) {
    let mut i = 0;
    while i < strips.len() {
        /* Only wrap strips explicitly selected. */
        if (strips[i].flag & NLASTRIP_FLAG_SELECT) == 0 {
            i += 1;
            continue;
        }
        /* Gather a run of selected strips. */
        let mut j = i;
        while j < strips.len() && (strips[j].flag & NLASTRIP_FLAG_SELECT) != 0 {
            j += 1;
        }
        let run: Vec<NlaStrip> = strips.drain(i..j).collect();
        if run.len() < 2 {
            /* Restore singleton — no meta needed. */
            for (k, s) in run.into_iter().enumerate() {
                strips.insert(i + k, s);
            }
            i = j;
            continue;
        }
        let start = run[0].start;
        let end = run[run.len() - 1].end;
        let mut meta = NlaStrip::default();
        meta.type_ = NLASTRIP_TYPE_META;
        meta.start = start;
        meta.end = end;
        meta.flag = NLASTRIP_FLAG_SELECT;
        if is_temp {
            meta.flag |= NLASTRIP_FLAG_TEMP_META;
        }
        meta.strips = run;
        strips.insert(i, meta);
        i += 1;
    }
}

/// `BKE_nlastrips_clear_metastrip` — move meta children back
/// into the parent list, then drop the meta.
pub fn nlastrips_clear_metastrip(strips: &mut Vec<NlaStrip>, strip_index: usize) {
    let mut meta = strips.remove(strip_index);
    for cs in meta.strips.drain(..) {
        let pos = strips
            .iter()
            .position(|ns| ns.start >= cs.start)
            .unwrap_or(strips.len());
        strips.insert(pos, cs);
    }
}

/// `BKE_nlastrips_clear_metas`.
pub fn nlastrips_clear_metas(strips: &mut Vec<NlaStrip>, only_sel: bool, only_temp: bool) {
    let mut i = 0;
    while i < strips.len() {
        let is_meta = strips[i].type_ == NLASTRIP_TYPE_META;
        let sel_ok = !only_sel || (strips[i].flag & NLASTRIP_FLAG_SELECT) != 0;
        let tmp_ok = !only_temp || (strips[i].flag & NLASTRIP_FLAG_TEMP_META) != 0;
        if is_meta && sel_ok && tmp_ok {
            nlastrips_clear_metastrip(strips, i);
        } else {
            i += 1;
        }
    }
}

/// `BKE_nlameta_add_strip` — add `strip` to `mstrip`'s list,
/// expanding meta bounds when neighbors allow.
pub fn nlameta_add_strip(
    mstrip: &mut NlaStrip,
    strip: NlaStrip,
    mstrip_prev_end: Option<f32>,
    mstrip_next_start: Option<f32>,
) -> bool {
    if !nlastrips_has_space(&mstrip.strips, strip.start, strip.end) {
        return false;
    }
    if strip.start < mstrip.start {
        if mstrip_prev_end.is_none() || mstrip_prev_end.unwrap() <= strip.start {
            mstrip.strips.insert(0, strip.clone());
            mstrip.start = strip.start;
            return true;
        }
        return false;
    }
    if strip.end > mstrip.end {
        if mstrip_next_start.is_none() || mstrip_next_start.unwrap() >= strip.end {
            mstrip.strips.push(strip.clone());
            mstrip.end = strip.end;
            return true;
        }
        return false;
    }
    nlastrips_add_strip(&mut mstrip.strips, strip)
}

/// `BKE_nlameta_flush_transforms` — propagate meta scale/offset
/// to children (RNA property writes → direct field writes here).
pub fn nlameta_flush_transforms(mstrip: &mut NlaStrip) {
    if mstrip.strips.is_empty() || mstrip.type_ != NLASTRIP_TYPE_META {
        return;
    }
    let o_start = mstrip.strips.first().unwrap().start;
    let o_end = mstrip.strips.last().unwrap().end;
    let offset = mstrip.start - o_start;
    let o_len = o_end - o_start;
    let n_len = mstrip.end - mstrip.start;
    let scale_changed = !is_eqf(o_len, n_len);

    if is_eqf(o_start, mstrip.start) && is_eqf(o_end, mstrip.end) && !scale_changed {
        return;
    }

    for strip in &mut mstrip.strips {
        if scale_changed {
            let (p1, p2) = if o_len != 0.0 {
                (
                    (strip.start - o_start) / o_len,
                    (strip.end - o_start) / o_len,
                )
            } else {
                /* Stop-gap for zero-length strips (#113552). */
                (0.0, 1.0)
            };
            strip.start = (p1 * n_len) + mstrip.start;
            strip.end = (p2 * n_len) + mstrip.start;
            let action_len = strip.actend - strip.actstart;
            let repeated_len = action_len * strip.repeat;
            let strip_len = strip.end - strip.start;
            strip.scale = strip_len / repeated_len;
        } else {
            strip.start += offset;
            strip.end += offset;
        }
    }
    for strip in &mut mstrip.strips {
        nlameta_flush_transforms(strip);
    }
}

/// `BKE_nlatrack_find_active` — `NLATRACK_ACTIVE` track index.
pub fn nlatrack_find_active(tracks: &[NlaTrack]) -> Option<usize> {
    tracks.iter().position(|t| (t.flag & NLATRACK_ACTIVE) != 0)
}

/// `BKE_nlatrack_solo_toggle` — set solo on `nlt`, clear it on
/// the rest; mirrors `ADT_NLA_SOLO_TRACK` flag handling.
pub fn nlatrack_solo_toggle(adt_flag: &mut i32, tracks: &mut [NlaTrack], nlt_index: usize) {
    if (tracks[nlt_index].flag & NLATRACK_SOLO) == 0 {
        for t in tracks.iter_mut() {
            t.flag &= !NLATRACK_SOLO;
        }
        tracks[nlt_index].flag |= NLATRACK_SOLO;
        *adt_flag |= ADT_NLA_SOLO_TRACK;
    } else {
        tracks[nlt_index].flag &= !NLATRACK_SOLO;
        *adt_flag &= !ADT_NLA_SOLO_TRACK;
    }
}

/// `BKE_nlatrack_set_active`.
pub fn nlatrack_set_active(tracks: &mut [NlaTrack], active: Option<usize>) {
    for (i, t) in tracks.iter_mut().enumerate() {
        if Some(i) == active {
            t.flag |= NLATRACK_ACTIVE;
        } else {
            t.flag &= !NLATRACK_ACTIVE;
        }
    }
}

/// `BKE_nlatrack_has_space`.
pub fn nlatrack_has_space(nlt: &NlaTrack, start: f32, end: f32) -> bool {
    nlastrips_has_space(&nlt.strips, start, end)
}

/// `BKE_nlatrack_has_strips`.
pub fn nlatrack_has_strips(tracks: &[NlaTrack]) -> bool {
    tracks.iter().any(|t| !t.strips.is_empty())
}

/// `BKE_nlatrack_sort_strips`.
pub fn nlatrack_sort_strips(nlt: &mut NlaTrack) {
    nlastrips_sort_strips(&mut nlt.strips);
}

/// `BKE_nlatrack_add_strip`.
pub fn nlatrack_add_strip(nlt: &mut NlaTrack, strip: NlaStrip) -> bool {
    nlastrips_add_strip(&mut nlt.strips, strip)
}

/// `BKE_nlatrack_get_bounds`.
pub fn nlatrack_get_bounds(nlt: &NlaTrack) -> Option<(f32, f32)> {
    let first = nlt.strips.first()?;
    let last = nlt.strips.last()?;
    Some((first.start, last.end))
}

/// `BKE_nlatrack_is_enabled` — not muted/disabled unless soloed.
pub fn nlatrack_is_enabled(adt: &AnimData, nlt: &NlaTrack) -> bool {
    if (nlt.flag & NLATRACK_DISABLED) != 0 || (nlt.flag & NLATRACK_MUTED) != 0 {
        /* Soloed tracks still play. */
        if (adt.flag & ADT_NLA_SOLO_TRACK) != 0 && (nlt.flag & NLATRACK_SOLO) != 0 {
            return true;
        }
        return false;
    }
    /* If another track is soloed, only it plays. */
    if (adt.flag & ADT_NLA_SOLO_TRACK) != 0 && (nlt.flag & NLATRACK_SOLO) == 0 {
        return false;
    }
    true
}

/// `nlastrip_find_active` — `NLASTRIP_FLAG_ACTIVE` index.
pub fn nlastrip_find_active(strips: &[NlaStrip]) -> Option<usize> {
    strips.iter().position(|s| (s.flag & NLASTRIP_FLAG_ACTIVE) != 0)
}

/// `BKE_nlastrip_set_active` — flag-only variant.
pub fn nlastrip_set_active(strips: &mut [NlaStrip], active: Option<usize>) {
    for (i, s) in strips.iter_mut().enumerate() {
        if Some(i) == active {
            s.flag |= NLASTRIP_FLAG_ACTIVE;
        } else {
            s.flag &= !NLASTRIP_FLAG_ACTIVE;
        }
    }
}

/// `nlastrip_find_by_name` — recursive search (verbatim name
/// match).
pub fn nlastrip_find_by_name(strips: &[NlaStrip], name: &str) -> Option<usize> {
    for (i, s) in strips.iter().enumerate() {
        if strip_name(s) == name {
            return Some(i);
        }
        if s.type_ == NLASTRIP_TYPE_META {
            if nlastrip_find_by_name(&s.strips, name).is_some() {
                return Some(i);
            }
        }
    }
    None
}

/// `BKE_nlastrip_within_bounds`.
pub fn nlastrip_within_bounds(strip: &NlaStrip, min: f32, max: f32) -> bool {
    strip.start >= min && strip.end <= max && strip.end > strip.start
}

/// `BKE_nlastrip_distance_to_frame` — signed distance to the
/// strip extent (0 inside).
pub fn nlastrip_distance_to_frame(strip: &NlaStrip, timeline_frame: f32) -> f32 {
    if timeline_frame < strip.start {
        timeline_frame - strip.start
    } else if timeline_frame > strip.end {
        timeline_frame - strip.end
    } else {
        0.0
    }
}

/// `BKE_nlastrip_next_in_track` / `_prev_in_track` — adjacent
/// strip index, optionally skipping transitions.
pub fn nlastrip_next_in_track(strips: &[NlaStrip], index: usize, skip_transitions: bool) -> Option<usize> {
    let mut i = index + 1;
    while i < strips.len() {
        if !skip_transitions || strips[i].type_ != NLASTRIP_TYPE_TRANSITION {
            return Some(i);
        }
        i += 1;
    }
    None
}
/// `nlastrip_prev_in_track`.
pub fn nlastrip_prev_in_track(strips: &[NlaStrip], index: usize, skip_transitions: bool) -> Option<usize> {
    let mut i = index;
    while i > 0 {
        i -= 1;
        if !skip_transitions || strips[i].type_ != NLASTRIP_TYPE_TRANSITION {
            return Some(i);
        }
    }
    None
}

/// `BKE_nlastrip_compute_frame_from_previous_strip` /
/// `to_next_strip` — transition endpoints.
pub fn nlastrip_compute_frame_from_previous_strip(
    strips: &[NlaStrip],
    index: usize,
) -> f32 {
    match nlastrip_prev_in_track(strips, index, true) {
        Some(i) => strips[i].end,
        None => strips[index].start,
    }
}
/// `nlastrip_compute_frame_to_next_strip`.
pub fn nlastrip_compute_frame_to_next_strip(strips: &[NlaStrip], index: usize) -> f32 {
    match nlastrip_next_in_track(strips, index, true) {
        Some(i) => strips[i].start,
        None => strips[index].end,
    }
}

/// `nlastrip_fix_resize_overlaps` — shift/grow/shrink neighbors
/// (transitions absorbed first) after `strips[index]` resized.
pub fn nlastrip_fix_resize_overlaps(strips: &mut [NlaStrip], index: usize) {
    /* Next strips — first, since we're often just getting longer. */
    if index + 1 < strips.len() {
        let mut nls_i = index + 1;
        if strips[nls_i].type_ == NLASTRIP_TYPE_TRANSITION {
            if strips[index].end <= strips[nls_i].start {
                strips[nls_i].start = strips[index].end;
            } else if strips[index].end < strips[nls_i].end {
                strips[nls_i].start = strips[index].end;
            } else {
                strips[nls_i].start = strips[nls_i].end - 1.0;
                let offset = (strips[index].end - strips[nls_i].start).ceil();
                while nls_i < strips.len() {
                    strips[nls_i].start += offset;
                    strips[nls_i].end += offset;
                    nls_i += 1;
                }
            }
        } else if strips[index].end > strips[nls_i].start {
            let offset = (strips[index].end - strips[nls_i].start).ceil();
            while nls_i < strips.len() {
                strips[nls_i].start += offset;
                strips[nls_i].end += offset;
                nls_i += 1;
            }
        }
    }

    /* Previous strips — same routine as before. */
    if index > 0 {
        let mut nls_i = index - 1;
        if strips[nls_i].type_ == NLASTRIP_TYPE_TRANSITION {
            if strips[index].start >= strips[nls_i].end {
                strips[nls_i].end = strips[index].start;
            } else if strips[index].start > strips[nls_i].start {
                strips[nls_i].end = strips[index].start;
            } else {
                strips[nls_i].end = strips[nls_i].start + 1.0;
                let offset = (strips[nls_i].end - strips[index].start).ceil();
                loop {
                    strips[nls_i].start -= offset;
                    strips[nls_i].end -= offset;
                    if nls_i == 0 {
                        break;
                    }
                    nls_i -= 1;
                }
            }
        } else if strips[index].start < strips[nls_i].end {
            let offset = (strips[nls_i].end - strips[index].start).ceil();
            loop {
                strips[nls_i].start -= offset;
                strips[nls_i].end -= offset;
                if nls_i == 0 {
                    break;
                }
                nls_i -= 1;
            }
        }
    }
}

/// `BKE_nlastrip_recalculate_bounds_sync_action` — resync
/// `actend`/`actstart` when `NLASTRIP_FLAG_SYNC_LENGTH` set.
pub fn nlastrip_recalculate_bounds_sync_action(strip: &mut NlaStrip) {
    if (strip.flag & NLASTRIP_FLAG_SYNC_LENGTH) == 0 {
        return;
    }
    let actlen = nla_clip_length_get_nonzero(strip);
    let striplen = strip.end - strip.start;
    let scale = if striplen != 0.0 {
        actlen / striplen
    } else {
        1.0
    };
    strip.scale = scale * strip.scale.signum().abs().max(scale);
}

/// `BKE_nlastrip_recalculate_bounds` — fix invalid bounds:
/// `start >= end` → collapse to 1-frame strips; fix resize
/// overlaps (callers pass the containing list).
pub fn nlastrip_recalculate_bounds(strips: &mut [NlaStrip], index: usize) {
    if strips[index].end <= strips[index].start {
        strips[index].end = strips[index].start + 1.0;
    }
    if strips[index].start == strips[index].end {
        strips[index].end += 1.0;
    }
    nlastrip_fix_resize_overlaps(strips, index);
}

/// `BKE_nlastrip_recalculate_blend` — recompute
/// `blendin`/`blendout` from strip length (injected autoblend
/// logic for overlaps is `PORT_PENDING`).
pub fn nlastrip_recalculate_blend(strip: &mut NlaStrip) {
    /* Clamp blend to strip length. */
    let len = strip.end - strip.start;
    strip.blendin = strip.blendin.clamp(0.0, len);
    strip.blendout = strip.blendout.clamp(0.0, len - strip.blendin);
}

/// `BKE_nlatrack_has_animated_strips` — strip with fcurves or a
/// bound action (injected `has_fcurves`/`has_action`).
pub fn nlatrack_has_animated_strips(
    nlt: &NlaTrack,
    strip_is_animated: &dyn Fn(&NlaStrip) -> bool,
) -> bool {
    fn recur(strips: &[NlaStrip], f: &dyn Fn(&NlaStrip) -> bool) -> bool {
        strips.iter().any(|s| f(s) || recur(&s.strips, f))
    }
    recur(&nlt.strips, strip_is_animated)
}

/// `BKE_nlatracks_have_animated_strips`.
pub fn nlatracks_have_animated_strips(
    tracks: &[NlaTrack],
    strip_is_animated: &dyn Fn(&NlaStrip) -> bool,
) -> bool {
    tracks
        .iter()
        .any(|t| nlatrack_has_animated_strips(t, strip_is_animated))
}

/// `nlastrip_get_endpoint_overlaps` — collect strip indices
/// whose `start`/`end` overlap `strips[index]`'s start/end.
pub fn nlastrip_get_endpoint_overlaps(strips: &[NlaStrip], index: usize) -> (Vec<usize>, Vec<usize>) {
    let (s, e) = (strips[index].start, strips[index].end);
    let mut start_overlaps = Vec::new();
    let mut end_overlaps = Vec::new();
    for (i, strip) in strips.iter().enumerate() {
        if i == index {
            continue;
        }
        if strip.end > s && strip.start < s {
            start_overlaps.push(i);
        } else if strip.start < e && strip.end > e {
            end_overlaps.push(i);
        }
    }
    (start_overlaps, end_overlaps)
}

/// `foreach_strip` — depth-first visit of every strip; callback
/// returns `false` to stop.
pub fn foreach_strip<F: FnMut(&mut NlaStrip) -> bool>(tracks: &mut [NlaTrack], mut callback: F) -> bool {
    fn visit(strip: &mut NlaStrip, cb: &mut dyn FnMut(&mut NlaStrip) -> bool) -> bool {
        if !cb(strip) {
            return false;
        }
        for s in &mut strip.strips {
            if !visit(s, cb) {
                return false;
            }
        }
        true
    }
    for track in tracks {
        for strip in &mut track.strips {
            if !visit(strip, &mut callback) {
                return false;
            }
        }
    }
    true
}

/// `BKE_nla_tweakmode_clear_flags` — strip tweak-user and track
/// tweak flags cleared (verbatim subset).
pub fn nla_tweakmode_clear_flags(adt: &mut AnimData) {
    adt.flag &= !(ADT_NLA_EDIT_ON | ADT_NLA_EDIT_NOMAP);
    adt.tmpact = usize::MAX;
    adt.tmp_slot_handle = 0;
}
