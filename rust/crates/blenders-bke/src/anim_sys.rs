//! Port of `intern/anim_sys.cc` — the NLA/channel evaluation
//! engine's portable core: `AnimationEvalContext`, `NlaEvalStrip`,
//! channel/snapshot bookkeeping (`nlaevalchan_*`, `nlaeval_
//! snapshot_*`, `nlavalidmask_*`), `nlastrip_get_influence`,
//! `nlastrips_ctime_get_strip`, `nlastrip_evaluate_controls`, and
//! the full blend/mix math (`nla_blend_value`, `nla_combine_
//! value`, `nla_blend_get_inverted_{lower,strip}_value`,
//! `nla_combine_get_inverted_{lower,strip}_value`,
//! `nla_combine_quaternion_get_inverted_lower_values`).
//!
//! `PORT_PENDING`: `animsys_evaluate_{fcurves,action,drivers}` +
//! `BKE_animsys_*_rna_path` (RNA `PointerRNA`/`PropertyRNA`),
//! `BKE_animsys_eval_context_construct` (Depsgraph), keyingset
//! list mgmt (writes to `Main`), blend read/write.
//! Those take injected callbacks/keys.

use blenders_dna::anim_enums::*;
use blenders_dna::anim_types::{AnimData, FCurve, NlaStrip};
use blenders_math::Quaternion;

/// `NES_TIME_*` — which side of the strip `ctime` fell on.
pub const NES_TIME_WITHIN: i16 = 0;
/// `NES_TIME_BEFORE`.
pub const NES_TIME_BEFORE: i16 = 1;
/// `NES_TIME_AFTER`.
pub const NES_TIME_AFTER: i16 = 2;

/// `NEC_MIX_*` — channel value mix modes.
pub const NEC_MIX_ADD: i8 = 0;
/// `NEC_MIX_QUATERNION`.
pub const NEC_MIX_QUATERNION: i8 = 1;
/// `NEC_MIX_MULTIPLY`.
pub const NEC_MIX_MULTIPLY: i8 = 2;
/// `NEC_MIX_AXIS_ANGLE`.
pub const NEC_MIX_AXIS_ANGLE: i8 = 3;

fn is_eqf(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

/// `AnimationEvalContext` — eval time + caller-supplied
/// depsgraph/action context handles.
#[derive(Clone, Copy)]
pub struct AnimationEvalContext {
    /// `depsgraph` (usize handle — `PORT_PENDING` Depsgraph).
    pub depsgraph: usize,
    /// `eval_time` — frame in `cframe` units.
    pub eval_time: f32,
}

/// `BKE_animsys_eval_context_construct_at` — clone with a new
/// eval time.
pub fn animsys_eval_context_construct_at(
    ctx: &AnimationEvalContext,
    eval_time: f32,
) -> AnimationEvalContext {
    AnimationEvalContext {
        depsgraph: ctx.depsgraph,
        eval_time,
    }
}

/// `NlaEvalStrip` — an eval-ready strip (index into the
/// strips list, evaluated `ctime`, strip's `side`).
#[derive(Clone)]
pub struct NlaEvalStrip {
    /// `strip_index` — index into the parent `Vec<NlaStrip>`.
    pub strip_index: usize,
    /// `side` — `NES_TIME_*`.
    pub side: i16,
    /// `list` — child eval strips of meta strips.
    pub children: Vec<NlaEvalStrip>,
}

/// `NlaEvalChannelKey` — property identity (path + index).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NlaEvalChannelKey {
    /// `rna_path` (verbatim from the driving `FCurve`).
    pub rna_path: String,
    /// `array_index`.
    pub array_index: i32,
    /// `prop_subtype` — `PROP_QUATERNION`/axis-angle detection
    /// result cached (verbatim `detect_mix_mode`).
    pub mix_mode: i8,
}

/// `NlaEvalChannel` — value accumulation channel.
#[derive(Clone)]
pub struct NlaEvalChannel {
    /// `key`.
    pub key: NlaEvalChannelKey,
    /// `values` — blended accumulation.
    pub values: Vec<f32>,
    /// `is_snapshot` — values came from a snapshot.
    pub is_snapshot: bool,
}

/// `NlaEvalChannelSnapshot` — saved channel values for the
/// remapping pass.
#[derive(Clone)]
pub struct NlaEvalChannelSnapshot {
    /// `key`.
    pub key: NlaEvalChannelKey,
    /// `base_values` — action defaults.
    pub base_values: Vec<f32>,
    /// `values` — accumulated.
    pub values: Vec<f32>,
}

/// `nlaevalchan_snapshot_new`/`copy`.
pub fn nlaevalchan_snapshot_new(nec: &NlaEvalChannel, base_values: &[f32]) -> NlaEvalChannelSnapshot {
    NlaEvalChannelSnapshot {
        key: nec.key.clone(),
        base_values: base_values.to_vec(),
        values: nec.values.clone(),
    }
}
/// `nlaevalchan_snapshot_copy`.
pub fn nlaevalchan_snapshot_copy(
    dst: &mut NlaEvalChannelSnapshot,
    src: &NlaEvalChannelSnapshot,
) {
    dst.base_values = src.base_values.clone();
    dst.values = src.values.clone();
}

/// `NlaEvalSnapshot` — per-eval frame snapshot store.
#[derive(Default)]
pub struct NlaEvalSnapshot {
    /// `channel_snapshots` — resolved by `key`.
    pub channel_snapshots: Vec<NlaEvalChannelSnapshot>,
}

impl NlaEvalSnapshot {
    /// `nlaeval_snapshot_get`.
    pub fn get(&self, index: usize) -> Option<&NlaEvalChannelSnapshot> {
        self.channel_snapshots.get(index)
    }
    /// `nlaeval_snapshot_find_channel`.
    pub fn find_channel(&self, key: &NlaEvalChannelKey) -> Option<usize> {
        self.channel_snapshots.iter().position(|s| &s.key == key)
    }
    /// `nlaeval_snapshot_ensure_channel`.
    pub fn ensure_channel(&mut self, key: &NlaEvalChannelKey) -> usize {
        if let Some(i) = self.find_channel(key) {
            return i;
        }
        self.channel_snapshots.push(NlaEvalChannelSnapshot {
            key: key.clone(),
            base_values: Vec::new(),
            values: Vec::new(),
        });
        self.channel_snapshots.len() - 1
    }
}

/// `NlaValidMask` — bitset over channel values (`nlavalidmask_*`).
#[derive(Clone)]
pub struct NlaValidMask {
    /// `buffer` — `length` rounded to u32 words.
    pub buffer: Vec<u32>,
    /// `length` — number of channel slots.
    pub length: usize,
}

/// `nlavalidmask_init` — all bits clear or set per `all_valid`.
pub fn nlavalidmask_init(length: usize, all_valid: bool) -> NlaValidMask {
    let fill = if all_valid { u32::MAX } else { 0 };
    NlaValidMask {
        buffer: vec![fill; length.div_ceil(32)],
        length,
    }
}
/// `nlavalidmask` bit get/set (verbatim bit ops).
pub fn nlavalidmask_get(mask: &NlaValidMask, index: usize) -> bool {
    (mask.buffer[index / 32] & (1 << (index % 32))) != 0
}
/// `nlavalidmask_set`.
pub fn nlavalidmask_set(mask: &mut NlaValidMask, index: usize, valid: bool) {
    if valid {
        mask.buffer[index / 32] |= 1 << (index % 32);
    } else {
        mask.buffer[index / 32] &= !(1 << (index % 32));
    }
}

/// `NlaEvalData` — the per-object eval state.
#[derive(Default)]
pub struct NlaEvalData {
    /// `eval_channels` — flat channel list.
    pub eval_channels: Vec<NlaEvalChannel>,
    /// `valid_mask`.
    pub valid_mask: Option<NlaValidMask>,
    /// `base_snapshot`.
    pub base_snapshot: NlaEvalSnapshot,
    /// `eval_snapshot`.
    pub eval_snapshot: NlaEvalSnapshot,
}

/// `nlaevalchan_verify` — find-or-create the channel for `key`.
pub fn nlaevalchan_verify(
    nlaeval: &mut NlaEvalData,
    key: &NlaEvalChannelKey,
    length: usize,
) -> usize {
    if let Some(i) = nlaeval.eval_channels.iter().position(|c| &c.key == key) {
        return i;
    }
    nlaeval.eval_channels.push(NlaEvalChannel {
        key: key.clone(),
        values: vec![0.0; length],
        is_snapshot: false,
    });
    nlaeval.eval_channels.len() - 1
}

/// `nlaevalchan_validate_index` — index within `[0, length)`.
pub fn nlaevalchan_validate_index(nec: &NlaEvalChannel, index: i32) -> i32 {
    if index < 0 {
        0
    } else if index as usize >= nec.values.len() {
        nec.values.len() as i32 - 1
    } else {
        index
    }
}

/// `nlaevalchan_detect_mix_mode` — `PROP_QUATERNION`→QUATERNION,
/// `PROP_AXISANGLE`→AXIS_ANGLE, else MULTIPLY for scale-named
/// paths, else ADD (verbatim rules where subtype is given).
pub fn nlaevalchan_detect_mix_mode(prop_subtype: i32, length: usize, rna_path: &str) -> i8 {
    const PROP_QUATERNION: i32 = 1;
    const PROP_AXISANGLE: i32 = 2;
    if prop_subtype == PROP_QUATERNION && length == 4 {
        return NEC_MIX_QUATERNION;
    }
    if prop_subtype == PROP_AXISANGLE && length == 4 {
        return NEC_MIX_AXIS_ANGLE;
    }
    if rna_path.ends_with("scale") {
        return NEC_MIX_MULTIPLY;
    }
    NEC_MIX_ADD
}

/// `nlaevalchan_get_default_values` — RNA-default driven; the
/// injected `defaults` supplies array/scalar defaults (verbatim
/// unit/zero rules + multiply-zero fix).
pub fn nlaevalchan_get_default_values(
    mix_mode: i8,
    defaults: &mut dyn FnMut() -> Vec<f32>,
    r_values: &mut [f32],
) {
    if mix_mode == NEC_MIX_QUATERNION {
        for v in r_values.iter_mut() {
            *v = 0.0;
        }
        r_values[0] = 1.0;
        return;
    }
    if mix_mode == NEC_MIX_AXIS_ANGLE {
        for v in r_values.iter_mut() {
            *v = 0.0;
        }
        return;
    }
    let d = defaults();
    for (i, v) in r_values.iter_mut().enumerate() {
        *v = d.get(i).copied().unwrap_or(0.0);
    }
    if mix_mode == NEC_MIX_MULTIPLY {
        for v in r_values.iter_mut() {
            if *v == 0.0 {
                *v = 1.0;
            }
        }
    }
}

/// `nlastrip_get_influence` — verbatim blendin/out falloff.
pub fn nlastrip_get_influence(strip: &mut NlaStrip, cframe: f32) -> f32 {
    strip.blendin = strip.blendin.abs();
    strip.blendout = strip.blendout.abs();
    if !is_eqf(strip.blendin, 0.0) && cframe <= (strip.start + strip.blendin) {
        return (cframe - strip.start).abs() / strip.blendin;
    }
    if !is_eqf(strip.blendout, 0.0) && cframe >= (strip.end - strip.blendout) {
        return (strip.end - cframe).abs() / strip.blendout;
    }
    1.0
}

/// `nlastrip_evaluate_controls` — strip's own FCurves
/// (injected) + analytic influence/time.
pub fn nlastrip_evaluate_controls(
    strip: &mut NlaStrip,
    anim_eval_context: &AnimationEvalContext,
    eval_fcurves: &mut dyn FnMut(&mut NlaStrip, &AnimationEvalContext),
) {
    eval_fcurves(strip, anim_eval_context);

    if (strip.flag & NLASTRIP_FLAG_USR_INFLUENCE) == 0 {
        strip.influence = nlastrip_get_influence(strip, anim_eval_context.eval_time);
    }
    if (strip.flag & NLASTRIP_FLAG_NO_TIME_MAP) != 0 {
        strip.strip_time = anim_eval_context.eval_time;
        return;
    }
    if (strip.flag & NLASTRIP_FLAG_USR_TIME) == 0 {
        strip.strip_time = crate::nla::nlastrip_get_frame(
            strip,
            anim_eval_context.eval_time,
            crate::nla::NLATIME_CONVERT_EVAL,
        );
    }
    if (strip.flag & NLASTRIP_FLAG_USR_TIME) != 0
        && (strip.flag & NLASTRIP_FLAG_USR_TIME_CYCLIC) != 0
    {
        strip.strip_time =
            (strip.strip_time - strip.actstart).rem_euclid(strip.actend - strip.actstart);
    }
}

/// `nlastrips_ctime_get_strip` — find the eval strip for
/// `ctime` over `strips` (returns `(index, side)`);
/// `in_range` respects `NLASTRIP_FLAG_NO_TIME_MAP` +
/// `extendmode` verbatim.
pub fn nlastrips_ctime_get_strip(
    strips: &[NlaStrip],
    ctime: f32,
) -> Option<(usize, i16)> {
    let mut estrip: Option<usize> = None;
    let mut side = 0i16;

    for (i, strip) in strips.iter().enumerate() {
        let mut in_range = ctime >= strip.start && ctime <= strip.end;
        if (strip.flag & NLASTRIP_FLAG_NO_TIME_MAP) != 0 {
            match strip.extendmode {
                NLASTRIP_EXTEND_HOLD => in_range = true,
                NLASTRIP_EXTEND_HOLD_FORWARD => in_range = ctime >= strip.start,
                _ => {}
            }
        }

        if in_range {
            estrip = Some(i);
            side = NES_TIME_WITHIN;
            break;
        }

        if ctime < strip.start {
            if i == 0 {
                if strip.extendmode == NLASTRIP_EXTEND_HOLD {
                    estrip = Some(i);
                }
                side = NES_TIME_BEFORE;
            } else {
                if strips[i - 1].extendmode != NLASTRIP_EXTEND_NOTHING {
                    estrip = Some(i - 1);
                }
                side = NES_TIME_AFTER;
            }
            break;
        }

        if ctime > strip.end && i == strips.len() - 1 {
            if strip.extendmode != NLASTRIP_EXTEND_NOTHING {
                estrip = Some(i);
            }
            side = NES_TIME_AFTER;
            break;
        }
    }

    let i = estrip?;
    if (strips[i].flag & NLASTRIP_FLAG_MUTED) != 0 {
        return None;
    }
    /* Transitions need both neighbors. */
    if strips[i].type_ == NLASTRIP_TYPE_TRANSITION && (i == 0 || i + 1 >= strips.len()) {
        return None;
    }
    Some((i, side))
}

/* -------------------------------------------------------------------- */
/** Blend/mix math — verbatim from `anim_sys.cc`. */

/// `nla_blend_value` — non-COMBINE modes.
pub fn nla_blend_value(
    blendmode: i16,
    lower_value: f32,
    strip_value: f32,
    influence: f32,
) -> f32 {
    if is_eqf(influence, 0.0) {
        return lower_value;
    }
    match blendmode {
        NLASTRIP_MODE_ADD => lower_value + (strip_value * influence),
        NLASTRIP_MODE_SUBTRACT => lower_value - (strip_value * influence),
        NLASTRIP_MODE_MULTIPLY => {
            influence * (lower_value * strip_value) + (1.0 - influence) * lower_value
        }
        _ => lower_value * (1.0 - influence) + (strip_value * influence),
    }
}

/// `nla_combine_value` — NEC_MIX modes.
pub fn nla_combine_value(
    mix_mode: i8,
    mut base_value: f32,
    lower_value: f32,
    strip_value: f32,
    influence: f32,
) -> f32 {
    if is_eqf(influence, 0.0) {
        return lower_value;
    }
    match mix_mode {
        NEC_MIX_ADD | NEC_MIX_AXIS_ANGLE => {
            lower_value + (strip_value - base_value) * influence
        }
        NEC_MIX_MULTIPLY => {
            if is_eqf(base_value, 0.0) {
                base_value = 1.0;
            }
            lower_value * (strip_value / base_value).powf(influence)
        }
        _ => lower_value,
    }
}

/// `nla_blend_get_inverted_lower_value` — solve for the lower
/// stack value that produces `blended_value` (verbatim cases).
pub fn nla_blend_get_inverted_lower_value(
    blendmode: i16,
    strip_value: f32,
    blended_value: f32,
    influence: f32,
) -> Option<f32> {
    if is_eqf(influence, 0.0) {
        return Some(blended_value);
    }
    match blendmode {
        NLASTRIP_MODE_ADD => Some(blended_value - (strip_value * influence)),
        NLASTRIP_MODE_SUBTRACT => Some(blended_value + (strip_value * influence)),
        NLASTRIP_MODE_MULTIPLY => {
            let denominator = influence * strip_value + (1.0 - influence);
            if is_eqf(denominator, 0.0) {
                if is_eqf(blended_value, 0.0) {
                    return Some(1.0);
                }
                return None;
            }
            Some(blended_value / denominator)
        }
        NLASTRIP_MODE_REPLACE => {
            if is_eqf(influence, 1.0) {
                return None;
            }
            Some((blended_value - (strip_value * influence)) / (1.0 - influence))
        }
        _ => None,
    }
}

/// `nla_combine_get_inverted_lower_value`.
pub fn nla_combine_get_inverted_lower_value(
    mix_mode: i8,
    mut base_value: f32,
    strip_value: f32,
    blended_value: f32,
    influence: f32,
) -> Option<f32> {
    if is_eqf(influence, 0.0) {
        return Some(blended_value);
    }
    match mix_mode {
        NEC_MIX_ADD | NEC_MIX_AXIS_ANGLE => {
            Some(blended_value - (strip_value - base_value) * influence)
        }
        NEC_MIX_MULTIPLY => {
            if is_eqf(strip_value, 0.0) {
                if is_eqf(blended_value, 0.0) {
                    return Some(1.0);
                }
                return None;
            }
            if is_eqf(base_value, 0.0) {
                base_value = 1.0;
            }
            Some(blended_value / (strip_value / base_value).powf(influence))
        }
        _ => None,
    }
}

fn normalize_qt(q: [f32; 4]) -> Quaternion {
    Quaternion::new(q[0], q[1], q[2], q[3]).normalized()
}

/// `pow_qt_fl_normalized` — `q^t` on a unit quaternion.
fn pow_qt_fl_normalized(q: Quaternion, t: f32) -> Quaternion {
    /* `log` then `exp` — standard qt power: slerp(identity, q, t). */
    let half = (q.w.clamp(-1.0, 1.0)).acos();
    let s = (half * t).sin();
    let vlen = (q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
    let (vx, vy, vz) = if vlen > 1e-8 {
        (q.x / vlen * s, q.y / vlen * s, q.z / vlen * s)
    } else {
        (0.0, 0.0, 0.0)
    };
    Quaternion::new((half * t).cos(), vx, vy, vz)
}

/// `nla_combine_quaternion_get_inverted_lower_values` — solve
/// `lower = blended @ inv(strip^inf)` (verbatim).
pub fn nla_combine_quaternion_get_inverted_lower_values(
    strip_values: [f32; 4],
    blended_values: [f32; 4],
    influence: f32,
) -> [f32; 4] {
    if is_eqf(influence, 0.0) {
        let q = normalize_qt(blended_values);
        return [q.w, q.x, q.y, q.z];
    }
    let mut tmp_strip = normalize_qt(strip_values);
    let tmp_blended = normalize_qt(blended_values);
    tmp_strip = pow_qt_fl_normalized(tmp_strip, influence).inverted();
    let q = tmp_blended * tmp_strip;
    let qn = q.normalized();
    [qn.w, qn.x, qn.y, qn.z]
}

/// `nla_blend_get_inverted_strip_value`.
pub fn nla_blend_get_inverted_strip_value(
    blendmode: i16,
    lower_value: f32,
    blended_value: f32,
    influence: f32,
) -> Option<f32> {
    if is_eqf(influence, 0.0) {
        return None;
    }
    match blendmode {
        NLASTRIP_MODE_ADD => Some((blended_value - lower_value) / influence),
        NLASTRIP_MODE_SUBTRACT => Some((lower_value - blended_value) / influence),
        NLASTRIP_MODE_MULTIPLY => {
            if is_eqf(lower_value, 0.0) {
                if is_eqf(blended_value, 0.0) {
                    return Some(1.0);
                }
                return None;
            }
            Some(((blended_value - lower_value) / (influence * lower_value)) + 1.0)
        }
        _ => Some((blended_value - lower_value * (1.0 - influence)) / influence),
    }
}

/// `nla_combine_get_inverted_strip_value`.
pub fn nla_combine_get_inverted_strip_value(
    mix_mode: i8,
    mut base_value: f32,
    lower_value: f32,
    blended_value: f32,
    influence: f32,
) -> Option<f32> {
    if is_eqf(influence, 0.0) {
        return None;
    }
    match mix_mode {
        NEC_MIX_ADD | NEC_MIX_AXIS_ANGLE => {
            Some((blended_value - lower_value) / influence + base_value)
        }
        NEC_MIX_MULTIPLY => {
            if is_eqf(lower_value, 0.0) {
                if is_eqf(blended_value, 0.0) {
                    return Some(base_value);
                }
                return None;
            }
            if is_eqf(base_value, 0.0) {
                base_value = 1.0;
            }
            Some(base_value * (blended_value / lower_value).powf(1.0 / influence))
        }
        _ => None,
    }
}

/// `is_fcurve_evaluatable` — nonzero totvert + evaluatable flag
/// (verbatim subset; driver/mute injected).
pub fn is_fcurve_evaluatable(fcu: &FCurve) -> bool {
    fcu.totvert > 0 || !fcu.bezt.is_empty() || !fcu.fpt.is_empty()
}

/// `BKE_animsys_evaluate_animdata` driver-list eval marker
/// (verbatim flag semantic: `ADT_NLA_EVAL_*` gating).
pub fn animsys_adt_needs_eval(adt: &AnimData) -> bool {
    (adt.flag & ADT_NLA_EVAL_OFF) == 0
}
