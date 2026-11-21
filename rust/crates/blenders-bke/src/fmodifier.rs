//! Port of `intern/fmodifier.cc` — the F-Curve modifier stack:
//! per-type data (`FMod_Generator`/`FMod_FunctionGenerator`/
//! `FMod_Envelope`/`FMod_Cycles`/`FMod_Noise`/`FMod_Limits`/
//! `FMod_Stepped`/`FMod_Smooth`), each type's `evaluate_modifier`/
//! `evaluate_modifier_time`, `eval_fmodifier_influence`,
//! `evaluate_time_fmodifiers`/`evaluate_value_fmodifiers`,
//! `BKE_fcm_envelope_find_index`, and the list management
//! (`add_fmodifier`, `copy_fmodifiers`, `remove_fmodifier`,
//! `free_fmodifiers`, `find_active_fmodifier`, `set_active_fmodifier`,
//! `list_has_suitable_fmodifier`).
//! `PORT_PENDING`: `fcurve_bake_modifiers` (needs fcurve write),
//! `evaluate_fcurve_unmodified` injected, `tFCMED_Cycles` storage
//! lives inside `FModifiersStackStorage` (here per-eval local).

use blenders_dna::anim_enums::*;
use blenders_dna::anim_types::*;

/// `sinc` — normalized sinc.
fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        (x * std::f64::consts::PI).sin() / (x * std::f64::consts::PI)
    }
}

/// `FModifierData` — typed payload for `FModifier.data`.
#[derive(Clone)]
pub enum FModifierData {
    /// `FMod_Generator`.
    Generator(FModGenerator),
    /// `FMod_FunctionGenerator`.
    FunctionGenerator(FModFunctionGenerator),
    /// `FMod_Envelope`.
    Envelope(FModEnvelope),
    /// `FMod_Cycles`.
    Cycles(FModCycles),
    /// `FMod_Noise`.
    Noise(FModNoise),
    /// `FMod_Limits`.
    Limits(FModLimits),
    /// `FMod_Stepped`.
    Stepped(FModStepped),
    /// `FMod_Smooth`.
    Smooth(FModSmooth),
    /// Unknown/other type (Python, Filter).
    Other(usize),
}

/// `tFCMED_Cycles` — per-eval cycles storage.
#[derive(Default)]
pub struct TFcmedCycles {
    /// `cycyofs`.
    pub cycyofs: f32,
}

/// `FModifiersStackStorage` — per-eval storage for a whole stack.
pub struct FModifiersStackStorage {
    /// `modifier_count`.
    pub modifier_count: usize,
    /// cycles storage per modifier.
    pub cycles: Vec<TFcmedCycles>,
}

impl FModifiersStackStorage {
    /// `BKE_fmodifiers_new_stack` — allocate storage for `count`.
    pub fn new(modifier_count: usize) -> Self {
        Self {
            modifier_count,
            cycles: (0..modifier_count).map(|_| TFcmedCycles::default()).collect(),
        }
    }
}

/// `fcm_generator_evaluate`.
pub fn fcm_generator_evaluate(
    data: &FModGenerator,
    cvalue: &mut f32,
    evaltime: f32,
) {
    match data.mode {
        FCM_GENERATOR_POLYNOMIAL => {
            let n = data.arraysize as usize;
            let mut powers = vec![0.0f32; n];
            let mut value = 0.0f32;
            let mut prev = 1.0f32;
            for (i, p) in powers.iter_mut().enumerate() {
                if i != 0 {
                    prev *= evaltime;
                }
                *p = prev;
            }
            for i in 0..n {
                value += data.coefficients[i] * powers[i];
            }
            if data.poly_order != 0 {
                if (data.flag & FCM_GENERATOR_ADDITIVE) != 0 {
                    *cvalue += value;
                } else {
                    *cvalue = value;
                }
            }
        }
        FCM_GENERATOR_POLYNOMIAL_FACTORISED => {
            let mut value = 1.0f32;
            let coeffs = &data.coefficients;
            for i in 0..data.poly_order.max(0) as usize {
                let a = coeffs.get(2 * i).copied().unwrap_or(0.0);
                let b = coeffs.get(2 * i + 1).copied().unwrap_or(0.0);
                value *= a * evaltime + b;
            }
            if data.poly_order != 0 {
                if (data.flag & FCM_GENERATOR_ADDITIVE) != 0 {
                    *cvalue += value;
                } else {
                    *cvalue = value;
                }
            }
        }
        _ => {}
    }
}

/// `fcm_fn_generator_evaluate`.
pub fn fcm_fn_generator_evaluate(
    data: &FModFunctionGenerator,
    cvalue: &mut f32,
    evaltime: f32,
) {
    let arg = data.phase_multiplier as f64 * evaltime as f64 + data.phase_offset as f64;
    let mut fnv: Option<fn(f64) -> f64> = None;
    match data.type_ {
        FCM_GENERATOR_FN_SIN => fnv = Some(f64::sin),
        FCM_GENERATOR_FN_COS => fnv = Some(f64::cos),
        FCM_GENERATOR_FN_SINC => fnv = Some(sinc),
        FCM_GENERATOR_FN_TAN => {
            let rem = (arg - std::f64::consts::FRAC_PI_2) % std::f64::consts::PI;
            if rem.abs() <= f64::EPSILON * 100.0 {
                if (data.flag & FCM_GENERATOR_ADDITIVE) == 0 {
                    *cvalue = 0.0;
                }
            } else {
                fnv = Some(f64::tan);
            }
        }
        FCM_GENERATOR_FN_LN => {
            if arg > 1.0 {
                fnv = Some(f64::ln);
            } else if (data.flag & FCM_GENERATOR_ADDITIVE) == 0 {
                *cvalue = 0.0;
            }
        }
        FCM_GENERATOR_FN_SQRT => {
            if arg > 0.0 {
                fnv = Some(f64::sqrt);
            } else if (data.flag & FCM_GENERATOR_ADDITIVE) == 0 {
                *cvalue = 0.0;
            }
        }
        _ => {}
    }
    if let Some(f) = fnv {
        let v = f(arg);
        let value = (v as f32 * data.amplitude) + data.value_offset;
        if (data.flag & FCM_GENERATOR_ADDITIVE) != 0 {
            *cvalue += value;
        } else {
            *cvalue = value;
        }
    }
}

/// `fcm_envelope_evaluate`.
pub fn fcm_envelope_evaluate(data: &FModEnvelope, cvalue: &mut f32, evaltime: f32) {
    if data.data.is_empty() {
        return;
    }
    let (mut min, mut max);
    let first = &data.data[0];
    let last = &data.data[(data.totvert.max(1) as usize - 1).min(data.data.len() - 1)];
    if first.time >= evaltime {
        min = first.min;
        max = first.max;
    } else if last.time <= evaltime {
        min = last.min;
        max = last.max;
    } else {
        min = first.min;
        max = first.max;
        for a in 0..data.totvert.max(1) as usize - 1 {
            let prevfed = &data.data[a];
            let fed = match data.data.get(a + 1) {
                Some(f) => f,
                None => break,
            };
            if prevfed.time <= evaltime && fed.time >= evaltime {
                let diff = fed.time - prevfed.time;
                let afac = (evaltime - prevfed.time) / diff;
                let bfac = (fed.time - evaltime) / diff;
                min = bfac * prevfed.min + afac * fed.min;
                max = bfac * prevfed.max + afac * fed.max;
                break;
            }
        }
    }
    let fac = (*cvalue - (data.midval + data.min)) / (data.max - data.min);
    *cvalue = min + fac * (max - min);
}

/// `BKE_fcm_envelope_find_index` — find envelope point at `frame`.
pub fn bke_fcm_envelope_find_index(
    array: &[FcmEnvelopeData],
    frame: f32,
    arraylen: i32,
) -> i32 {
    let arraylen = arraylen.max(0) as usize;
    /* Binary search for exact frame, else return insertion-ish index. */
    if arraylen == 0 {
        return -1;
    }
    let mut low = 0usize;
    let mut high = arraylen - 1;
    while low <= high {
        let mid = (low + high) / 2;
        let t = array[mid].time;
        if (t - frame).abs() <= f32::EPSILON {
            return mid as i32;
        }
        if t < frame {
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }
    -1
}

/// `fcm_cycles_time` — remap `evaltime` into the key range per the
/// before/after cycling mode.
pub fn fcm_cycles_time(
    data: &FModCycles,
    firstkey: [f32; 2],
    lastkey: [f32; 2],
    is_first: bool,
    evaltime: f32,
    storage: &mut TFcmedCycles,
) -> f32 {
    let mut evaltime = evaltime;
    let mut cycyofs = 0.0f32;
    storage.cycyofs = 0.0;

    let _ = is_first;
    let (mut side, mut mode, mut cycles, mut ofs) = (0i32, 0, 0i32, 0.0f32);
    if evaltime < firstkey[0] {
        if data.before_mode != 0 {
            side = -1;
            mode = data.before_mode;
            cycles = data.before_cycles as i32;
            ofs = firstkey[0];
        }
    } else if evaltime > lastkey[0] {
        if data.after_mode != 0 {
            side = 1;
            mode = data.after_mode;
            cycles = data.after_cycles as i32;
            ofs = lastkey[0];
        }
    }
    if side == 0 || mode == 0 {
        return evaltime;
    }

    let cycdx = lastkey[0] - firstkey[0];
    let cycdy = lastkey[1] - firstkey[1];
    if cycdx == 0.0 {
        return evaltime;
    }

    let cycle = side as f64 * (evaltime as f64 - ofs as f64) / cycdx as f64;
    let cyct = (evaltime - ofs) % cycdx;

    if cycles != 0 && cycle > cycles as f64 {
        return evaltime;
    }

    if mode == FCM_EXTRAPOLATE_CYCLIC_OFFSET {
        cycyofs = if side < 0 {
            ((evaltime - ofs) / cycdx).floor()
        } else {
            ((evaltime - ofs) / cycdx).ceil()
        };
        cycyofs *= cycdy;
    }

    if cyct == 0.0 {
        evaltime = if side == 1 { lastkey[0] } else { firstkey[0] };
        if mode == FCM_EXTRAPOLATE_MIRROR && (cycle as i32) % 2 != 0 {
            evaltime = if side == 1 { firstkey[0] } else { lastkey[0] };
        }
    } else if mode == FCM_EXTRAPOLATE_MIRROR && ((cycle as i32) + 1) % 2 != 0 {
        evaltime = if side < 0 {
            firstkey[0] - cyct
        } else {
            lastkey[0] - cyct
        };
    } else {
        evaltime = firstkey[0] + cyct;
    }
    if evaltime < firstkey[0] {
        evaltime += cycdx;
    }

    if mode == FCM_EXTRAPOLATE_CYCLIC_OFFSET {
        storage.cycyofs = cycyofs;
    }
    evaltime
}

/// `fcm_cycles_evaluate` — apply `storage->cycyofs` to the value.
pub fn fcm_cycles_evaluate(cvalue: &mut f32, storage: &TFcmedCycles) {
    *cvalue += storage.cycyofs;
}

/// `fcm_noise_evaluate` — `perlin_fbm` path (legacy `BLI_noise_turbulence`
/// pending → injected `legacy_noise_fn`).
pub fn fcm_noise_evaluate(
    data: &FModNoise,
    cvalue: &mut f32,
    evaltime: f32,
    legacy_noise_fn: Option<&dyn Fn(f32, f32, f32, f32, i16) -> f32>,
) {
    let noise = if data.legacy_noise != 0 {
        legacy_noise_fn.map(|f| {
            f(data.size, evaltime - data.offset, data.phase, 0.1, data.depth)
        }).unwrap_or(0.0)
    } else {
        let scale = if data.size == 0.0 {
            0.0
        } else {
            1.0 / data.size
        };
        let offset = 0.61803398874f32;
        blenders_bli::noise::perlin_fbm(
            blenders_math::vector::Vec2::new((evaltime - data.offset) * scale + offset, data.phase),
            data.depth as f32,
            data.roughness,
            data.lacunarity,
            true,
        )
    };
    match data.modification {
        FCM_NOISE_MODIF_ADD => *cvalue += noise * data.strength,
        FCM_NOISE_MODIF_SUBTRACT => *cvalue -= noise * data.strength,
        FCM_NOISE_MODIF_MULTIPLY => *cvalue *= noise * data.strength,
        _ => *cvalue += (noise - 0.5) * data.strength,
    }
}

/// `fcm_limits_time`.
pub fn fcm_limits_time(data: &FModLimits, evaltime: f32) -> f32 {
    if (data.flag & FCM_LIMIT_XMIN) != 0 && evaltime < data.rect.xmin {
        return data.rect.xmin;
    }
    if (data.flag & FCM_LIMIT_XMAX) != 0 && evaltime > data.rect.xmax {
        return data.rect.xmax;
    }
    evaltime
}

/// `fcm_limits_evaluate`.
pub fn fcm_limits_evaluate(data: &FModLimits, cvalue: &mut f32) {
    if (data.flag & FCM_LIMIT_YMIN) != 0 && *cvalue < data.rect.ymin {
        *cvalue = data.rect.ymin;
    }
    if (data.flag & FCM_LIMIT_YMAX) != 0 && *cvalue > data.rect.ymax {
        *cvalue = data.rect.ymax;
    }
}

/// `fcm_stepped_time`.
pub fn fcm_stepped_time(data: &FModStepped, evaltime: f32) -> f32 {
    if (data.flag & FCM_STEPPED_NO_BEFORE) != 0 && evaltime < data.start_frame {
        return evaltime;
    }
    if (data.flag & FCM_STEPPED_NO_AFTER) != 0 && evaltime > data.end_frame {
        return evaltime;
    }
    let snapblock = ((evaltime - data.offset) / data.step_size) as i32;
    snapblock as f32 * data.step_size + data.offset
}

/// `fcm_smooth_frame` — Gaussian-weighted neighborhood average.
pub fn fcm_smooth_frame(
    data: &FModSmooth,
    evaltime: i32,
    default_value: f32,
    evaluate_fcurve_unmodified: &dyn Fn(f32) -> f32,
) -> f32 {
    let sigma = data.sigma;
    let filter_width = data.filter_width as f32;
    let mut total_weighted_value = 0.0f32;
    let mut total_weight = 0.0f32;
    let start_frame = (evaltime as f32 - filter_width).floor() as i32;
    let end_frame = (evaltime as f32 + filter_width).ceil() as i32;
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut sample_time = start_frame;
    while sample_time <= end_frame {
        let sample_distance = sample_time as f32 - evaltime as f32;
        let dn = sample_distance / filter_width;
        let weight = (-(dn * dn) / two_sigma_sq).exp();
        let v = evaluate_fcurve_unmodified(sample_time as f32);
        total_weighted_value += v * weight;
        total_weight += weight;
        sample_time += 1;
    }
    if total_weight <= 0.0 {
        return default_value;
    }
    total_weighted_value / total_weight
}

/// `fcm_smooth_evaluate` — integer-frame smooth or lerped between
/// smoothed neighbors.
pub fn fcm_smooth_evaluate(
    data: &FModSmooth,
    cvalue: &mut f32,
    evaltime: f32,
    evaluate_fcurve_unmodified: &dyn Fn(f32) -> f32,
) {
    let is_integer_frame = (evaltime.round() - evaltime).abs() <= f32::EPSILON;
    if is_integer_frame {
        *cvalue = fcm_smooth_frame(data, evaltime as i32, *cvalue, evaluate_fcurve_unmodified);
        return;
    }
    let prev_frame = evaltime.floor();
    let next_frame = evaltime.ceil();
    let mut prev_value = evaluate_fcurve_unmodified(prev_frame);
    let next_value_raw = evaluate_fcurve_unmodified(next_frame);
    prev_value = fcm_smooth_frame(data, prev_frame as i32, prev_value, evaluate_fcurve_unmodified);
    let next_value =
        fcm_smooth_frame(data, next_frame as i32, next_value_raw, evaluate_fcurve_unmodified);
    *cvalue = prev_value + (next_value - prev_value) * (evaltime - prev_frame);
}

/// `eval_fmodifier_influence`.
pub fn eval_fmodifier_influence(fcm: &FModifier, evaltime: f32) -> f32 {
    let influence = if (fcm.flag & FMODIFIER_FLAG_USEINFLUENCE) != 0 {
        fcm.influence
    } else {
        1.0
    };
    if (fcm.flag & FMODIFIER_FLAG_RANGERESTRICT) != 0 {
        if evaltime < fcm.sfra || evaltime > fcm.efra {
            return 0.0;
        }
        if fcm.blendin != 0.0 && evaltime >= fcm.sfra && evaltime <= fcm.sfra + fcm.blendin {
            let a = fcm.sfra;
            let b = fcm.sfra + fcm.blendin;
            return influence * (evaltime - a) / (b - a);
        }
        if fcm.blendout != 0.0 && evaltime <= fcm.efra && evaltime >= fcm.efra - fcm.blendout {
            let a = fcm.efra;
            let b = fcm.efra - fcm.blendout;
            return influence * (evaltime - a) / (b - a);
        }
    }
    influence
}

/// `evaluate_time_fmodifiers` — walk the stack in reverse applying
/// `evaluate_modifier_time` per type.
pub fn evaluate_time_fmodifiers(
    storage: &mut FModifiersStackStorage,
    modifiers: &[FModifier],
    modifier_data: &dyn Fn(&FModifier) -> Option<FModifierData>,
    firstkey: [f32; 2],
    lastkey: [f32; 2],
    cvalue: f32,
    evaltime: f32,
) -> f32 {
    if modifiers.is_empty() {
        return evaltime;
    }
    let mut evaltime = evaltime;
    for (i, fcm) in modifiers.iter().enumerate().rev() {
        if (fcm.flag & FMODIFIER_FLAG_RANGERESTRICT) == 0
            || (fcm.sfra <= evaltime && fcm.efra >= evaltime)
        {
            if (fcm.flag & (FMODIFIER_FLAG_DISABLED | FMODIFIER_FLAG_MUTED)) == 0 {
                let data = modifier_data(fcm);
                let nval = match data {
                    Some(FModifierData::Cycles(d)) => {
                        let st = storage
                            .cycles
                            .get_mut(i)
                            .expect("cycles storage");
                        fcm_cycles_time(&d, firstkey, lastkey, i == 0, evaltime, st)
                    }
                    Some(FModifierData::Limits(d)) => fcm_limits_time(&d, evaltime),
                    Some(FModifierData::Stepped(d)) => fcm_stepped_time(&d, evaltime),
                    _ => continue,
                };
                let influence = eval_fmodifier_influence(fcm, evaltime);
                evaltime = evaltime + (nval - evaltime) * influence;
            }
        }
        let _ = cvalue;
    }
    evaltime
}

/// `evaluate_value_fmodifiers` — forward pass over `evaluate_modifier`.
pub fn evaluate_value_fmodifiers(
    storage: &mut FModifiersStackStorage,
    modifiers: &[FModifier],
    modifier_data: &dyn Fn(&FModifier) -> Option<FModifierData>,
    cvalue: &mut f32,
    evaltime: f32,
    evaluate_fcurve_unmodified: &dyn Fn(f32) -> f32,
) {
    for (i, fcm) in modifiers.iter().enumerate() {
        if (fcm.flag & FMODIFIER_FLAG_RANGERESTRICT) != 0
            && !(fcm.sfra <= evaltime && fcm.efra >= evaltime)
        {
            continue;
        }
        if (fcm.flag & (FMODIFIER_FLAG_DISABLED | FMODIFIER_FLAG_MUTED)) != 0 {
            continue;
        }
        let data = match modifier_data(fcm) {
            Some(d) => d,
            None => continue,
        };
        let mut nval = *cvalue;
        match data {
            FModifierData::Generator(d) => fcm_generator_evaluate(&d, &mut nval, evaltime),
            FModifierData::FunctionGenerator(d) => {
                fcm_fn_generator_evaluate(&d, &mut nval, evaltime)
            }
            FModifierData::Envelope(d) => fcm_envelope_evaluate(&d, &mut nval, evaltime),
            FModifierData::Cycles(_) => {
                if let Some(st) = storage.cycles.get(i) {
                    fcm_cycles_evaluate(&mut nval, st);
                }
            }
            FModifierData::Noise(d) => {
                fcm_noise_evaluate(&d, &mut nval, evaltime, None)
            }
            FModifierData::Limits(d) => fcm_limits_evaluate(&d, &mut nval),
            FModifierData::Stepped(_) => {}
            FModifierData::Smooth(d) => {
                fcm_smooth_evaluate(&d, &mut nval, evaltime, evaluate_fcurve_unmodified)
            }
            FModifierData::Other(_) => {}
        }
        let influence = eval_fmodifier_influence(fcm, evaltime);
        *cvalue += (nval - *cvalue) * influence;
    }
}

/// `add_fmodifier` — append a new modifier of `type` (returns index).
pub fn add_fmodifier(modifiers: &mut Vec<FModifier>, type_: FModifierTypes) -> usize {
    let mut fcm = FModifier::default();
    fcm.type_ = type_;
    fcm.flag = FMODIFIER_FLAG_ACTIVE;
    fcm.influence = 1.0;
    fcm.sfra = 1.0;
    fcm.efra = 100.0;
    fcm.blendin = 4.0;
    fcm.blendout = 4.0;
    modifiers.push(fcm);
    modifiers.len() - 1
}

/// `copy_fmodifiers` — deep copy of the list.
pub fn copy_fmodifiers(src: &[FModifier]) -> Vec<FModifier> {
    src.to_vec()
}

/// `remove_fmodifier` — remove `index`.
pub fn remove_fmodifier(modifiers: &mut Vec<FModifier>, index: usize) -> bool {
    if index < modifiers.len() {
        modifiers.remove(index);
        true
    } else {
        false
    }
}

/// `free_fmodifiers`.
pub fn free_fmodifiers(modifiers: &mut Vec<FModifier>) {
    modifiers.clear();
}

/// `find_active_fmodifier`.
pub fn find_active_fmodifier(modifiers: &[FModifier]) -> Option<usize> {
    modifiers
        .iter()
        .position(|fcm| (fcm.flag & FMODIFIER_FLAG_ACTIVE) != 0)
}

/// `set_active_fmodifier`.
pub fn set_active_fmodifier(modifiers: &mut [FModifier], index: usize) {
    for (i, fcm) in modifiers.iter_mut().enumerate() {
        if i == index {
            fcm.flag |= FMODIFIER_FLAG_ACTIVE;
        } else {
            fcm.flag &= !FMODIFIER_FLAG_ACTIVE;
        }
    }
}

/// `list_has_suitable_fmodifier` — a modifier of `mtype` with active
/// flag matching `acttype` (-1 = any).
pub fn list_has_suitable_fmodifier(
    modifiers: &[FModifier],
    mtype: FModifierTypes,
    acttype: i16,
) -> bool {
    modifiers.iter().any(|fcm| {
        (mtype == -1 || fcm.type_ == mtype)
            && match acttype {
                -1 => true,
                0 => (fcm.flag & FMODIFIER_FLAG_ACTIVE) == 0,
                _ => (fcm.flag & FMODIFIER_FLAG_ACTIVE) != 0,
            }
    })
}
