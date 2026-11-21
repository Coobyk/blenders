//! Port of `intern/fcurve_driver.cc` — driver variable management
//! (`driver_variable_name_validate`, `driver_variable_unique_name`,
//! `driver_add_new_variable`, `driver_change_variable_type`,
//! `driver_variables_copy`, `driver_free_variable`), target→rotation
//! conversion (`BKE_driver_target_matrix_to_rot_channels`,
//! `quaternion_to_angles`), simple-expression compile/eval
//! (`driver_compile_simple_expr` via `ExprPyLike`), and the
//! `evaluate_driver` dispatch (SUM/AVERAGE/MIN/MAX; PYTHON through an
//! injected executor). Property/pose lookups are injected.
//! `PORT_PENDING`: `dvar_eval_rotDiff`/`locDiff`/`transChan` (need
//! `AnimationEvalContext` + pose/channel lookups), context props.

use blenders_dna::anim_enums::*;
use blenders_dna::anim_types::{ChannelDriver, DriverTarget, DriverVar};
use blenders_math::base::{safe_acos, safe_asin};
use blenders_math::euler::{mat3_to_euler, EulerOrder};
use blenders_math::matrix::Mat4;
use blenders_math::quaternion::{quat_split_swing_and_twist, Quaternion};

/// `MAX_DRIVER_TARGETS`.
pub const MAX_DRIVER_TARGETS: usize = 8;

/// `quaternion_to_angles` — convert quat components to pseudo-angles
/// in place (`channel` < 0 → all four).
pub fn quaternion_to_angles(quat: &mut [f32; 4], channel: i32) {
    if channel < 0 {
        quat[0] = 2.0 * safe_acos(quat[0]);
        for i in 1..4 {
            quat[i] = 2.0 * safe_asin(quat[i]);
        }
    } else if channel == 0 {
        quat[0] = 2.0 * safe_acos(quat[0]);
    } else {
        quat[channel as usize] = 2.0 * safe_asin(quat[channel as usize]);
    }
}

/// `BKE_driver_target_matrix_to_rot_channels` — decompose `mat` into
/// `r_buf` (euler in `r_buf[1..4]` or quat in `r_buf[0..4]`) per
/// `rotation_mode`; `angles` requests pseudo-angles for quats.
pub fn bke_driver_target_matrix_to_rot_channels(
    mat: &Mat4,
    auto_order: i8,
    rotation_mode: DriverTargetRotationMode,
    channel: i32,
    angles: bool,
    r_buf: &mut [f32; 4],
) {
    *r_buf = [0.0; 4];
    let mat3 = blenders_math::matrix::Mat3::from_cols(
        blenders_math::vector::Vec3::new(mat[0].x, mat[0].y, mat[0].z),
        blenders_math::vector::Vec3::new(mat[1].x, mat[1].y, mat[1].z),
        blenders_math::vector::Vec3::new(mat[2].x, mat[2].y, mat[2].z),
    );
    let order = |mode: i8| -> EulerOrder {
        match mode {
            1 => EulerOrder::Xyz,
            2 => EulerOrder::Xzy,
            3 => EulerOrder::Yxz,
            4 => EulerOrder::Yzx,
            5 => EulerOrder::Zxy,
            _ => EulerOrder::Zyx,
        }
    };
    if rotation_mode == DTAR_ROTMODE_AUTO {
        let e = mat3_to_euler(&mat3, order(auto_order));
        r_buf[1] = e.xyz.x;
        r_buf[2] = e.xyz.y;
        r_buf[3] = e.xyz.z;
    } else if (DTAR_ROTMODE_EULER_MIN..=DTAR_ROTMODE_EULER_MAX).contains(&rotation_mode) {
        let e = mat3_to_euler(&mat3, order(rotation_mode));
        r_buf[1] = e.xyz.x;
        r_buf[2] = e.xyz.y;
        r_buf[3] = e.xyz.z;
    } else if rotation_mode == DTAR_ROTMODE_QUATERNION {
        let q = Quaternion::from_mat3(mat3);
        *r_buf = [q.w, q.x, q.y, q.z];
        if angles {
            quaternion_to_angles(r_buf, channel);
        }
    } else if (DTAR_ROTMODE_SWING_TWIST_X..=DTAR_ROTMODE_SWING_TWIST_Z)
        .contains(&rotation_mode)
    {
        let axis = (rotation_mode - DTAR_ROTMODE_SWING_TWIST_X) as usize;
        let raw_quat = Quaternion::from_mat3(mat3);
        let mut quat = [0.0; 4];
        let twist;
        if channel == axis as i32 + 1 {
            /* Twist only — skip the swing. */
            twist = quat_split_swing_and_twist(raw_quat, axis, false, false).0;
        } else {
            let (t, swing, _) = quat_split_swing_and_twist(raw_quat, axis, true, false);
            twist = t;
            let s = swing.unwrap_or(Quaternion { w: 1.0, x: 0.0, y: 0.0, z: 0.0 });
            quat = [s.w, s.x, s.y, s.z];
            quaternion_to_angles(&mut quat, channel);
        }
        quat[axis + 1] = twist;
        *r_buf = quat;
    }
}

/// `DriverVarTypeInfo` — per-type eval callback signature:
/// evaluates the variable to a float; injected per type.
#[derive(Clone, Copy)]
pub struct DriverVarTypeInfo {
    /// `num_targets`.
    pub num_targets: i8,
    /// `target_flags` (per slot).
    pub target_flags: [DriverTargetFlag; MAX_DRIVER_TARGETS],
}

/// `dvar_types` — `num_targets`/`target_flags` table (eval fns are
/// dispatched in `driver_get_variable_value`).
pub fn get_dvar_typeinfo(type_: DriverVarTypes) -> Option<DriverVarTypeInfo> {
    const S: DriverTargetFlag = DTAR_FLAG_STRUCT_REF;
    const O: DriverTargetFlag = DTAR_FLAG_ID_OB_ONLY;
    let (n, fl): (i8, [i16; 8]) = match type_ {
        t if t == DVAR_TYPE_SINGLE_PROP => (1, [S, 0, 0, 0, 0, 0, 0, 0]),
        t if t == DVAR_TYPE_ROT_DIFF => (2, [O, O, 0, 0, 0, 0, 0, 0]),
        t if t == DVAR_TYPE_LOC_DIFF => (2, [O, O, 0, 0, 0, 0, 0, 0]),
        t if t == DVAR_TYPE_TRANSFORM_CHAN => (1, [O, 0, 0, 0, 0, 0, 0, 0]),
        t if t == DVAR_TYPE_CONTEXT_PROP => (1, [0, 0, 0, 0, 0, 0, 0, 0]),
        _ => return None,
    };
    Some(DriverVarTypeInfo {
        num_targets: n,
        target_flags: fl,
    })
}

/// `driver_change_variable_type`.
pub fn driver_change_variable_type(dvar: &mut DriverVar, type_: DriverVarTypes) {
    let dvti = match get_dvar_typeinfo(type_) {
        Some(d) => d,
        None => return,
    };
    dvar.type_ = type_;
    dvar.num_targets = dvti.num_targets;
    for tar_index in 0..(dvar.num_targets as usize).min(MAX_DRIVER_TARGETS) {
        let flags = dvti.target_flags[tar_index];
        let dtar = &mut dvar.targets[tar_index];
        dtar.flag = flags;
        if (flags & DTAR_FLAG_ID_OB_ONLY) != 0 || dtar.idtype == 0 {
            dtar.idtype = blenders_dna::id_enums::ID_OB as i32;
        }
    }
}

/// `driver_variable_name_validate` — blacklist check + flag set.
pub fn driver_variable_name_validate(dvar: &mut DriverVar) {
    const SPECIAL_CHAR_BLACKLIST: &[u8] = b"~`!@#$%^&*+=-/\\?:;<>{}[]| .\t\n\r";
    dvar.flag &= !DVAR_ALL_INVALID_FLAGS;
    let name = dvar.name;
    let n0 = name[0];
    if n0 == 0 {
        dvar.flag |= DVAR_FLAG_INVALID_EMPTY;
    }
    if (b'0'..=b'9').contains(&n0) {
        dvar.flag |= DVAR_FLAG_INVALID_START_NUM;
    } else if n0 == b'_' {
        dvar.flag |= DVAR_FLAG_INVALID_START_CHAR;
    }
    let name_end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    let name_str = &name[..name_end];
    if name_str.contains(&b' ') {
        dvar.flag |= DVAR_FLAG_INVALID_HAS_SPACE;
    }
    if name_str.contains(&b'.') {
        dvar.flag |= DVAR_FLAG_INVALID_HAS_DOT;
    }
    for &c in SPECIAL_CHAR_BLACKLIST {
        if let Some(pos) = name_str.iter().position(|&x| x == c) {
            if pos == 0 {
                dvar.flag |= DVAR_FLAG_INVALID_START_CHAR;
            } else {
                dvar.flag |= DVAR_FLAG_INVALID_HAS_SPECIAL;
            }
        }
    }
    /* `BPY_string_is_keyword` — python keyword check PORT_PENDING. */
    if dvar.flag & DVAR_ALL_INVALID_FLAGS != 0 {
        dvar.flag |= DVAR_FLAG_INVALID_NAME;
    }
}

/// `BLI_uniquename` over driver var names — append `_NN` until unique.
pub fn driver_variable_unique_name(variables: &mut Vec<DriverVar>, index: usize) {
    let base = {
        let name = &variables[index].name;
        let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        String::from_utf8_lossy(&name[..end]).to_string()
    };
    let mut candidate = base.clone();
    let mut n = 0;
    loop {
        let clash = variables
            .iter()
            .enumerate()
            .any(|(i, d)| {
                if i == index {
                    return false;
                }
                let end = d.name.iter().position(|&c| c == 0).unwrap_or(d.name.len());
                String::from_utf8_lossy(&d.name[..end]) == candidate
            });
        if !clash {
            break;
        }
        n += 1;
        candidate = format!("{}_{:03}", base.trim_end_matches(|c: char| c.is_ascii_digit() || c == '_'), n);
    }
    let bytes = candidate.as_bytes();
    variables[index].name = [0; 64];
    variables[index].name[..bytes.len().min(63)].copy_from_slice(&bytes[..bytes.len().min(63)]);
}

/// `driver_add_new_variable` — push default "var" with unique name.
pub fn driver_add_new_variable(variables: &mut Vec<DriverVar>) -> usize {
    let mut dvar = DriverVar::default();
    dvar.name[..3].copy_from_slice(b"var");
    variables.push(dvar);
    let index = variables.len() - 1;
    driver_variable_unique_name(variables, index);
    driver_change_variable_type(&mut variables[index], DVAR_TYPE_SINGLE_PROP);
    index
}

/// `driver_variables_copy` — deep copy (`rna_path` strings dup'ed).
pub fn driver_variables_copy(src: &[DriverVar]) -> Vec<DriverVar> {
    src.to_vec()
}

/// `driver_free_variable` — remove by index.
pub fn driver_free_variable(variables: &mut Vec<DriverVar>, index: usize) {
    variables.remove(index);
}

/// `BKE_driver_invalidate_expression` — clears the simple-expr cache
/// flag bookkeeping (flags only; compiled expr is in `expr_simple`).
pub fn bke_driver_invalidate_expression(
    driver: &mut ChannelDriver,
    vars_changed: bool,
    expr_changed: bool,
) {
    let _ = (vars_changed, expr_changed);
    driver.expr_simple = usize::MAX;
}

/// `AnimationEvalContext` — minimal fields used here (`eval_time`,
/// `depsgraph`/scene injected via closures).
pub struct DriverEvalContext<'a> {
    /// `eval_time`.
    pub eval_time: f32,
    /// target property getter: `(dvar, target_index) -> Option<f32>`
    /// — handles `dtar_get_prop_val` incl. fallback.
    pub get_target_value: &'a dyn Fn(&DriverVar, usize) -> Option<f32>,
}

/// `dtar_try_use_fallback`.
pub fn dtar_try_use_fallback(dtar: &DriverTarget, value: Option<f32>) -> Option<f32> {
    if value.is_none() && (dtar.options & DTAR_OPTION_USE_FALLBACK) != 0 {
        Some(dtar.fallback_value)
    } else {
        value
    }
}

/// `driver_get_variable_value` — evaluate one variable by type.
/// `ROT_DIFF`/`LOC_DIFF`/`TRANSFORM_CHAN` come through
/// `ctx.get_target_value` (they need pose channels — injected).
pub fn driver_get_variable_value(
    ctx: &DriverEvalContext,
    _driver: &ChannelDriver,
    dvar: &DriverVar,
) -> f32 {
    if dvar.flag & DVAR_FLAG_ERROR != 0 {
        return 0.0;
    }
    match dvar.type_ {
        t if t == DVAR_TYPE_SINGLE_PROP => {
            /* `dvar_eval_singleProp`. */
            let v = (ctx.get_target_value)(dvar, 0);
            let v = dtar_try_use_fallback(&dvar.targets[0], v);
            match v {
                Some(x) => x,
                None => 0.0,
            }
        }
        t if t == DVAR_TYPE_CONTEXT_PROP => {
            let v = (ctx.get_target_value)(dvar, 0);
            let v = dtar_try_use_fallback(&dvar.targets[0], v);
            v.unwrap_or(0.0)
        }
        _ => {
            /* ROT_DIFF/LOC_DIFF/TRANSFORM_CHAN — evaluated against pose. */
            let mut sum = 0.0f32;
            for i in 0..(dvar.num_targets as usize).min(MAX_DRIVER_TARGETS) {
                if let Some(v) = (ctx.get_target_value)(dvar, i) {
                    sum += v;
                }
            }
            sum
        }
    }
}

/// `evaluate_driver_sum` (and AVERAGE).
pub fn evaluate_driver_sum(ctx: &DriverEvalContext, driver: &mut ChannelDriver, variables: &[DriverVar]) {
    if variables.len() == 1 {
        driver.curval = driver_get_variable_value(ctx, driver, &variables[0]);
        return;
    }
    let mut value = 0.0f32;
    let mut tot = 0i32;
    for dvar in variables {
        value += driver_get_variable_value(ctx, driver, dvar);
        tot += 1;
    }
    driver.curval = if driver.type_ == DRIVER_TYPE_AVERAGE {
        if tot > 0 { value / tot as f32 } else { 0.0 }
    } else {
        value
    };
}

/// `evaluate_driver_min_max`.
pub fn evaluate_driver_min_max(
    ctx: &DriverEvalContext,
    driver: &mut ChannelDriver,
    variables: &[DriverVar],
) {
    let mut value = 0.0f32;
    for (i, dvar) in variables.iter().enumerate() {
        let tmp_val = driver_get_variable_value(ctx, driver, dvar);
        if i > 0 {
            value = if driver.type_ == DRIVER_TYPE_MAX {
                tmp_val.max(value)
            } else {
                tmp_val.min(value)
            };
        } else {
            value = tmp_val;
        }
    }
    driver.curval = value;
}

/// `driver_compile_simple_expr` — try `ExprPyLike` compile of the
/// driver expression against variable names. Returns the parsed expr.
pub fn driver_compile_simple_expr(
    driver: &ChannelDriver,
    variables: &[DriverVar],
) -> Option<blenders_bli::expr_pylike_eval::ExprPyLikeParsed> {
    let end = driver
        .expression
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(driver.expression.len());
    let expr = String::from_utf8_lossy(&driver.expression[..end]).to_string();
    let names: Vec<String> = variables
        .iter()
        .map(|d| {
            let e = d.name.iter().position(|&c| c == 0).unwrap_or(d.name.len());
            String::from_utf8_lossy(&d.name[..e]).to_string()
        })
        .collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    blenders_bli::expr_pylike_eval::expr_pylike_parse(&expr, &name_refs)
}

/// `driver_evaluate_simple_expr` — evaluate a compiled expr with the
/// variable values in order (`eval_time` appended as last param when
/// the expression uses `frame`/`time` — kept as separate param name).
pub fn driver_evaluate_simple_expr(
    parsed: &blenders_bli::expr_pylike_eval::ExprPyLikeParsed,
    values: &[f64],
) -> Option<f32> {
    parsed.eval(values).ok().map(|v| v as f32)
}

/// `BKE_driver_has_simple_expression` — true when `expression`
/// parses as a simple ExprPyLike expression.
pub fn bke_driver_has_simple_expression(
    driver: &ChannelDriver,
    variables: &[DriverVar],
) -> bool {
    driver_compile_simple_expr(driver, variables).is_some()
}

/// `evaluate_driver` — dispatch by `driver->type`; python path via
/// `python_exec` injection (returns curval).
pub fn evaluate_driver(
    ctx: &DriverEvalContext,
    driver: &mut ChannelDriver,
    driver_orig: &ChannelDriver,
    variables: &[DriverVar],
    python_exec: &dyn Fn(&ChannelDriver, &ChannelDriver) -> f32,
) -> f32 {
    if driver_orig.flag & DRIVER_FLAG_INVALID != 0 {
        return 0.0;
    }
    match driver.type_ {
        t if t == DRIVER_TYPE_AVERAGE || t == DRIVER_TYPE_SUM => {
            evaluate_driver_sum(ctx, driver, variables);
        }
        t if t == DRIVER_TYPE_MIN || t == DRIVER_TYPE_MAX => {
            evaluate_driver_min_max(ctx, driver, variables);
        }
        t if t == DRIVER_TYPE_PYTHON => {
            /* `evaluate_driver_python`. */
            let expr_empty = driver_orig.expression[0] == 0;
            if expr_empty || (driver_orig.flag & DRIVER_FLAG_INVALID) != 0 {
                driver.curval = 0.0;
            } else {
                driver.curval = python_exec(driver, driver_orig);
            }
        }
        _ => {}
    }
    driver.curval
}
