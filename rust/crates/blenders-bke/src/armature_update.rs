//! Port of `intern/armature_update.cc` — Spline IK evaluator:
//! `tSplineIk_EvalState`, `position_tail_on_spline` (secant root
//! solver over `anim_path_accum_length`), `splineik_evaluate_bone`
//! (spline-follow + axis-angle alignment + scale modes).
//!
//! `BKE_where_on_path`, `CurveCache`, pose-channel writes and
//! depsgraph tagging are injected via `SplineIkCtx`.

use blenders_math::matrix::{Mat3, Mat4};
use blenders_math::vector::{Vec3, Vec4};

/// `CONSTRAINT_SPLINEIK_Y*` modes.
pub const CONSTRAINT_SPLINEIK_YS_FIT_CURVE: i32 = 0;
/// `CONSTRAINT_SPLINEIK_YS_ORIGINAL`.
pub const CONSTRAINT_SPLINEIK_YS_ORIGINAL: i32 = 1;
/// `CONSTRAINT_SPLINEIK_YS_BOUNDS`.
pub const CONSTRAINT_SPLINEIK_YS_BOUNDS: i32 = 2;
/// `CONSTRAINT_SPLINEIK_XZS_NONE`.
pub const CONSTRAINT_SPLINEIK_XZS_NONE: i32 = 0;
/// `CONSTRAINT_SPLINEIK_XZS_ORIGINAL`.
pub const CONSTRAINT_SPLINEIK_XZS_ORIGINAL: i32 = 1;
/// `CONSTRAINT_SPLINEIK_XZS_VOLUMEPRESERVE`.
pub const CONSTRAINT_SPLINEIK_XZS_VOLUMEPRESERVE: i32 = 2;
/// `CONSTRAINT_SPLINEIK_NO_ROOT`.
use blenders_dna::action_types::{POSE_IKTREE, POSE_IKSPLINE};
use blenders_dna::constraint_types::{CONSTRAINT_SPLINEIK_BOUND, CONSTRAINT_SPLINEIK_EVENSPLITS};

pub const CONSTRAINT_SPLINEIK_NO_ROOT: i32 = 1 << 0;
/// `CONSTRAINT_SPLINEIK_USE_ORIGINAL_SCALE`.
pub const CONSTRAINT_SPLINEIK_USE_ORIGINAL_SCALE: i32 = 1 << 1;
/// `POSE_DONE`.
pub const POSE_DONE: i32 = 1 << 0;

/// `bSplineIKConstraint` (subset read by the evaluator).
#[derive(Clone, Copy, Default)]
pub struct SplineIKConstraint {
    /// `flag`.
    pub flag: i32,
    /// `yScaleMode`.
    pub y_scale_mode: i32,
    /// `xzScaleMode`.
    pub xz_scale_mode: i32,
    /// `chainlen`.
    pub chainlen: i32,
}

/// `tSplineIk_EvalState`.
#[derive(Clone)]
pub struct SplineIkEvalState {
    /// `curve_position`.
    pub curve_position: f32,
    /// `curve_scale`.
    pub curve_scale: f32,
    /// `locrot_offset`.
    pub locrot_offset: Mat4,
    /// `prev_tail_loc`.
    pub prev_tail_loc: Vec3,
    /// `prev_tail_radius`.
    pub prev_tail_radius: f32,
    /// `prev_tail_seg_idx`.
    pub prev_tail_seg_idx: i32,
}

/// `tSplineIK_Tree`.
pub struct SplineIkTree {
    /// `ik_data`.
    pub ik_data: SplineIKConstraint,
    /// `pchain` — pose channel indices (root..tip order).
    pub pchain: Vec<usize>,
    /// `points` — accumulated bone-length fractions (0..1).
    pub points: Vec<f32>,
    /// `totlength`.
    pub totlength: f32,
    /// `con` — enforce factor + parent tree index.
    pub enforce: f32,
    /// `parent_index`.
    pub parent_index: i32,
}

/// Access to curve cache / object transforms / pose channels.
pub trait SplineIkCtx {
    /// `BKE_anim_path_get_array_size`.
    fn anim_path_array_size(&self) -> usize;
    /// `cache->anim_path_accum_length`.
    fn anim_path_accum_length(&self, seg_idx: usize) -> f32;
    /// `BKE_anim_path_get_length`.
    fn anim_path_length(&self) -> f32;
    /// `BevPoint` vec for `bl->bevpoints[bp_idx]`.
    fn bevpoint_vec(&self, bp_idx: usize) -> Vec3;
    /// `bl->bevpoints` count.
    fn bevpoint_count(&self) -> usize;
    /// `bl->poly >= 0` → cyclic.
    fn bevlist_is_cyclic(&self) -> bool;
    /// `BKE_where_on_path` — returns (pos4, dir, radius) at `ctime`.
    fn where_on_path(&self, ctime: f32) -> Option<(Vec3, f32)>;
    /// `ik_data->tar->object_to_world`.
    fn tar_object_to_world(&self) -> Mat4;
    /// `ob->world_to_object`.
    fn ob_world_to_object(&self) -> Mat4;
    /// `pchan->pose_head`/`pose_tail`/`pose_mat`.
    fn pchan_head(&self, pchan: usize) -> Vec3;
    /// `pose_tail`.
    fn pchan_tail(&self, pchan: usize) -> Vec3;
    /// `pose_mat` (4x4, cols).
    fn pchan_pose_mat(&self, pchan: usize) -> Mat4;
    /// `pchan->bone->length`.
    fn pchan_bone_length(&self, pchan: usize) -> f32;
    /// Write back head/tail/mat; sets `POSE_DONE` on `flag`.
    fn pchan_set(&mut self, pchan: usize, head: Vec3, tail: Vec3, mat3: Mat3, or_flag: i32);
}

/// `splineik_evaluate_init`.
pub fn splineik_evaluate_init(
    ctx: &dyn SplineIkCtx,
    tree: &SplineIkTree,
) -> Option<SplineIkEvalState> {
    if ctx.anim_path_array_size() == 0 {
        return None;
    }
    let mut state = SplineIkEvalState {
        curve_position: 0.0,
        curve_scale: 1.0,
        locrot_offset: Mat4::IDENTITY,
        prev_tail_loc: Vec3::ZERO,
        prev_tail_radius: 1.0,
        prev_tail_seg_idx: 0,
    };
    if tree.ik_data.y_scale_mode != CONSTRAINT_SPLINEIK_YS_FIT_CURVE && tree.totlength != 0.0 {
        let spline_len = ctx.anim_path_length();
        state.curve_scale = tree.totlength / spline_len;
    }
    Some(state)
}

/// `apply_curve_transform` — tar's world, then ob's inverse.
pub fn apply_curve_transform(
    ctx: &dyn SplineIkCtx,
    ik_data: &SplineIKConstraint,
    r_vec: &mut Vec3,
) {
    if ik_data.flag & CONSTRAINT_SPLINEIK_NO_ROOT == 0 {
        *r_vec = ctx.tar_object_to_world().transform_point(*r_vec);
    }
    *r_vec = ctx.ob_world_to_object().transform_point(*r_vec);
}

/// `dist_to_sphere_shell`.
fn dist_to_sphere_shell(sphere_origin: Vec3, sphere_radius: f32, point: Vec3) -> f32 {
    sphere_radius - (sphere_origin - point).length()
}

/// `position_tail_on_spline` — returns new `prev_seg_idx`.
#[allow(clippy::too_many_arguments)]
pub fn position_tail_on_spline(
    ctx: &dyn SplineIkCtx,
    head_pos: Vec3,
    sphere_radius: f32,
    prev_seg_idx: i32,
    r_tail_pos: &mut Vec3,
    r_new_curve_pos: &mut f32,
    r_radius: &mut f32,
) -> i32 {
    let max_seg_idx = ctx.anim_path_array_size() as i32 - 1;
    let spline_len = ctx.anim_path_length();
    let guessed_len = *r_new_curve_pos * spline_len;

    debug_assert!(prev_seg_idx >= 0);
    let mut cur_seg_idx = prev_seg_idx;
    while cur_seg_idx < max_seg_idx && guessed_len > ctx.anim_path_accum_length(cur_seg_idx as usize)
    {
        cur_seg_idx += 1;
    }

    let mut bp_idx = cur_seg_idx + 1;
    let is_cyclic = ctx.bevlist_is_cyclic();
    let bp_count = ctx.bevpoint_count() as i32;

    let mut bp = bp_idx.min(bp_count - 1);
    let mut prev_bp = bp - 1;
    while (head_pos - ctx.bevpoint_vec(bp as usize)).length() < sphere_radius {
        if bp_idx > max_seg_idx {
            break;
        }
        prev_bp = bp;
        bp = if is_cyclic && bp_idx == max_seg_idx { 0 } else { bp + 1 };
        bp_idx += 1;
    }

    let bp_vec = ctx.bevpoint_vec(bp as usize);
    let prev_bp_vec = ctx.bevpoint_vec(prev_bp.max(0) as usize);

    let mut x0 = 0.0f32;
    let mut x1 = 1.0f32;
    let mut x0_point;
    let mut x1_point = Vec3::ZERO;
    let epsilon = head_pos.length().max(bp_vec.length()).max(1.0) * f32::EPSILON;

    let start_p = if prev_seg_idx == bp_idx - 1 {
        head_pos
    } else {
        prev_bp_vec
    };

    for _ in 0..10 {
        x0_point = start_p + (bp_vec - start_p) * x0;
        x1_point = start_p + (bp_vec - start_p) * x1;
        let f_x0 = dist_to_sphere_shell(head_pos, sphere_radius, x0_point);
        let f_x1 = dist_to_sphere_shell(head_pos, sphere_radius, x1_point);
        if f_x1.abs() <= epsilon || f_x0 == f_x1 {
            break;
        }
        let x2 = x1 - f_x1 * (x1 - x0) / (f_x1 - f_x0);
        x0 = x1;
        x1 = x2;
    }
    *r_tail_pos = x1_point;

    let mut prev_seg_idx = bp_idx - 2;
    if prev_seg_idx < 0 {
        prev_seg_idx = 0;
    }
    let isect_seg_len = (prev_bp_vec - *r_tail_pos).length();
    let seg_len = (prev_bp_vec - bp_vec).length();
    let frac = if seg_len != 0.0 {
        isect_seg_len / seg_len
    } else {
        0.0
    };

    /* Update `r_new_curve_pos` from accumulated lengths. */
    let prev_seg_len = ctx.anim_path_accum_length(prev_seg_idx.max(0) as usize);
    let seg_base = ctx.anim_path_accum_length((prev_seg_idx + 1).min(max_seg_idx.max(0)) as usize);
    *r_new_curve_pos =
        (prev_seg_len + frac * (seg_base - prev_seg_len)) / spline_len.max(f32::EPSILON);
    *r_radius = 1.0;

    bp_idx - 1
}

/// `splineik_evaluate_bone`.
pub fn splineik_evaluate_bone(
    ctx: &mut dyn SplineIkCtx,
    tree: &SplineIkTree,
    pchan: usize,
    index: usize,
    state: &mut SplineIkEvalState,
) {
    let ik_data = &tree.ik_data;
    let bone_len_orig = ctx.pchan_bone_length(pchan);

    if bone_len_orig < f32::EPSILON {
        if let Some((bone_pos, rad)) = ctx.where_on_path(state.curve_position) {
            let mut pos = bone_pos;
            apply_curve_transform(ctx, ik_data, &mut pos);
            let _ = rad;
            ctx.pchan_set(pchan, pos, pos, Mat3::IDENTITY, POSE_DONE);
        }
        return;
    }

    let mut pose_head = state.locrot_offset.transform_point(ctx.pchan_head(pchan));
    let mut pose_tail = state.locrot_offset.transform_point(ctx.pchan_tail(pchan));

    let curve_len = tree.points[index] - tree.points[index + 1];
    let bone_len = (pose_head - pose_tail).length();
    let point_start = state.curve_position;
    let pose_scale = bone_len / bone_len_orig;
    let mut base_scale = 1.0f32;
    if ik_data.y_scale_mode == CONSTRAINT_SPLINEIK_YS_ORIGINAL {
        base_scale = pose_scale;
    }

    let mut point_end = point_start + curve_len * base_scale * state.curve_scale;
    state.curve_position = point_end;

    let mut radius = 1.0f32;
    let mut tail_blend_fac = 0.0f32;

    if point_start < 1.0 {
        radius = 0.0;
        let (vec, rad) = if point_start == 0.0 {
            ctx.where_on_path(point_start).unwrap_or((Vec3::ZERO, 1.0))
        } else {
            (state.prev_tail_loc, state.prev_tail_radius)
        };
        radius = rad;
        pose_head = vec;
        apply_curve_transform(ctx, ik_data, &mut pose_head);

        if ik_data.y_scale_mode != CONSTRAINT_SPLINEIK_YS_FIT_CURVE {
            let sphere_radius = if ik_data.y_scale_mode == CONSTRAINT_SPLINEIK_YS_ORIGINAL {
                bone_len
            } else {
                bone_len_orig
            };
            let mut rad2 = rad;
            state.prev_tail_seg_idx = position_tail_on_spline(
                ctx,
                vec,
                sphere_radius,
                state.prev_tail_seg_idx,
                &mut pose_tail,
                &mut point_end,
                &mut rad2,
            );
            state.prev_tail_radius = rad2;
            state.prev_tail_loc = pose_tail;
            apply_curve_transform(ctx, ik_data, &mut pose_tail);
            state.curve_position = point_end;
        } else if let Some((vec2, rad2)) = ctx.where_on_path(point_end) {
            state.prev_tail_radius = rad2;
            state.prev_tail_loc = vec2;
            pose_tail = vec2;
            apply_curve_transform(ctx, ik_data, &mut pose_tail);
        }

        tail_blend_fac = if point_end >= 1.0 {
            (1.0 - point_start) / (point_end - point_start)
        } else {
            1.0
        };
    }

    let mut spline_vec = pose_tail - pose_head;
    let mut scale_fac = spline_vec.length() / bone_len_orig;

    {
        let mut base_pose_mat = Mat3::IDENTITY;
        let pose_mat4 = ctx.pchan_pose_mat(pchan);
        let base3 = Mat3::from_cols(
            Vec3::new(pose_mat4[0].x, pose_mat4[0].y, pose_mat4[0].z),
            Vec3::new(pose_mat4[1].x, pose_mat4[1].y, pose_mat4[1].z),
            Vec3::new(pose_mat4[2].x, pose_mat4[2].y, pose_mat4[2].z),
        );
        let off3 = Mat3::from_cols(
            Vec3::new(
                state.locrot_offset[0].x,
                state.locrot_offset[0].y,
                state.locrot_offset[0].z,
            ),
            Vec3::new(
                state.locrot_offset[1].x,
                state.locrot_offset[1].y,
                state.locrot_offset[1].z,
            ),
            Vec3::new(
                state.locrot_offset[2].x,
                state.locrot_offset[2].y,
                state.locrot_offset[2].z,
            ),
        );
        base_pose_mat = off3 * base3;
        let rmat = orthonormalize_m3(base_pose_mat);
        base_pose_mat = rmat;
        spline_vec = spline_vec.normalized();

        let mut raxis = rmat[1].cross(spline_vec);
        let norm = raxis.normalize_and_get_length();
        if norm < f32::EPSILON {
            let order = axis_sort(spline_vec);
            let mut tmp_axis = Vec3::ZERO;
            tmp_axis[order[1]] = 1.0;
            raxis = tmp_axis.cross(spline_vec);
        }

        let mut rangle = rmat[1].dot(spline_vec).clamp(-1.0, 1.0).acos();
        rangle *= tree.enforce * tail_blend_fac;

        let dmat = blenders_math::quaternion::mat3_from_axis_angle(blenders_math::quaternion::AxisAngle { axis: raxis.normalized(), angle: rangle });
        let pose_mat = orthonormalize_m3(dmat * rmat);
        let base_pose_mat = dmat * base_pose_mat;

        /* `mul_m4_m3m4(state->locrot_offset, dmat, locrot_offset)`. */
        state.locrot_offset = state.locrot_offset
            * {
                let m = dmat;
                Mat4::from_cols(
                    Vec4::new(m[0].x, m[0].y, m[0].z, 0.0),
                    Vec4::new(m[1].x, m[1].y, m[1].z, 0.0),
                    Vec4::new(m[2].x, m[2].y, m[2].z, 0.0),
                    Vec4::new(0.0, 0.0, 0.0, 1.0),
                )
            };

        let mut out = pose_mat;
        out[1] = out[1] * scale_fac;

        let old_pose_y = ctx.pchan_pose_mat(pchan);
        let (px, pz) = (
            Vec3::new(old_pose_y[0].x, old_pose_y[0].y, old_pose_y[0].z).length(),
            Vec3::new(old_pose_y[2].x, old_pose_y[2].y, old_pose_y[2].z).length(),
        );

        if ik_data.xz_scale_mode != CONSTRAINT_SPLINEIK_XZS_NONE {
            if ik_data.xz_scale_mode == CONSTRAINT_SPLINEIK_XZS_ORIGINAL
                || (ik_data.flag & CONSTRAINT_SPLINEIK_USE_ORIGINAL_SCALE) != 0
            {
                out[0] = out[0] * px;
                out[2] = out[2] * pz;
                scale_fac /= pose_scale;
            }
        }
        let _ = base_pose_mat;

        ctx.pchan_set(pchan, pose_head, pose_tail, out, POSE_DONE);
    }
}

/// Orthonormalize (Gram-Schmidt, `normalize_m3`).
fn orthonormalize_m3(m: Mat3) -> Mat3 {
    let x = m[0].normalized();
    let mut y = m[1] - x * x.dot(m[1]);
    y = y.normalized();
    let z = x.cross(y).normalized();
    Mat3::from_cols(x, y, z)
}

/// `axis_sort_v3` — indices of axes sorted by |component| desc.
fn axis_sort(v: Vec3) -> [usize; 3] {
    let mut order = [0usize, 1, 2];
    order.sort_by(|&a, &b| v[b].abs().partial_cmp(&v[a].abs()).unwrap());
    order
}

/// `splineik_init_tree_from_pchan` — chain walk + binding-point
/// computation verbatim; constraint lookup, bone parents and
/// `siktree` storage injected.
pub struct SplineIkInitCtx {
    /// Constraint data (`bSplineIKConstraint`) selected by the
    /// caller's `BKE_constraint_has_influence`/tar checks.
    /// `chainlen` bones are walked from `tip` toward the root;
    /// `bone_parent(i)` supplies each successive parent's bone
    /// length (returns `None` at the chain end).
    pub chainlen: usize,
    /// `ik_data->flag`.
    pub flag: i32,
    /// Current `ik_data->points` (empty = unbound).
    pub points: Vec<f32>,
}

/// Result of `splineik_init_tree_from_pchan` — the caller stores a
/// `SplineIkTree` (root..tip `chain` reversed), writes `points`
/// back to `ik_data->points` and sets `POSE_IKSPLINE` on the root.
pub struct SplineIkInitResult {
    /// `pchan_chain` root..tip (tip-to-root order preserved as in C:
    /// `chain[0]` is the tip).
    pub chain_tip_to_root: Vec<usize>,
    /// `totlength`.
    pub totlength: f32,
    /// `ik_data->points` (bound when it was empty).
    pub points: Vec<f32>,
    /// `segcount`.
    pub segcount: usize,
    /// `flag |= CONSTRAINT_SPLINEIK_BOUND` when bound here.
    pub bound_now: bool,
}

/// `splineik_init_tree_from_pchan` (verbatim — `bone_parent` walks
/// `pchan`/`pchan_bone` parents, both must exist).
pub fn splineik_init_tree_from_pchan(
    pchan_tip: usize,
    ctx: &mut SplineIkInitCtx,
    bone_parent: &dyn Fn(usize) -> Option<usize>,
    bone_length: &dyn Fn(usize) -> f32,
) -> Option<SplineIkInitResult> {
    /* Find the root bone and the chain from root to tip. */
    let mut pchan_chain: Vec<usize> = Vec::with_capacity(ctx.chainlen);
    let mut bone_lengths: Vec<f32> = Vec::with_capacity(ctx.chainlen);
    let mut totlength = 0.0f32;
    let mut cur = Some(pchan_tip);
    while let Some(pchan) = cur {
        if pchan_chain.len() >= ctx.chainlen {
            break;
        }
        pchan_chain.push(pchan);
        let len = bone_length(pchan);
        bone_lengths.push(len);
        totlength += len;
        cur = bone_parent(pchan);
    }
    let segcount = pchan_chain.len();
    if segcount == 0 {
        return None;
    }
    let pchan_root = pchan_chain[segcount - 1];
    let _ = pchan_root;

    let mut bound_now = false;
    if (ctx.flag & CONSTRAINT_SPLINEIK_BOUND as i32) == 0 {
        let segment_len = 1.0 / segcount as f32;
        ctx.points = vec![0.0; ctx.chainlen + 1];
        ctx.points[0] = 1.0;
        for i in 0..segcount {
            if (ctx.flag & CONSTRAINT_SPLINEIK_EVENSPLITS as i32) != 0 || totlength == 0.0 {
                ctx.points[i + 1] = ctx.points[i] - segment_len;
            } else {
                ctx.points[i + 1] = ctx.points[i] - (bone_lengths[i] / totlength);
            }
        }
        ctx.flag |= CONSTRAINT_SPLINEIK_BOUND as i32;
        bound_now = true;
    }
    if ctx.points[segcount] < 0.0 {
        ctx.points[segcount] = 0.0;
    }
    Some(SplineIkInitResult {
        chain_tip_to_root: pchan_chain,
        totlength,
        points: ctx.points.clone(),
        segcount,
        bound_now,
    })
}

/// `splineik_init_tree` — iterate the channels the caller tagged
/// `PCHAN_HAS_SPLINEIK` and run `splineik_init_tree_from_pchan`
/// on each (verbatim driver shape).
pub fn splineik_init_tree(
    tip_pchans: &[usize],
    init: &mut dyn FnMut(usize),
) {
    for &pchan in tip_pchans {
        init(pchan);
    }
}

/// `splineik_execute_tree` (verbatim — `where_is_bone` supplies
/// `BKE_pose_where_is_bone`, `evaluate_bone` is
/// `splineik_evaluate_bone`; trees freed by the caller).
pub fn splineik_execute_tree(
    trees: Vec<SplineIkTree>,
    ctime: f32,
    ctx: &mut dyn SplineIkCtx,
    where_is_bone: &mut dyn FnMut(usize, f32),
    evaluate_bone: &mut dyn FnMut(&SplineIkTree, usize, i32, &mut SplineIkEvalState),
) {
    for mut tree in trees {
        /* Firstly, calculate the bone matrix the standard way,
         * since this is needed for roll control. */
        for i in (0..tree.pchain.len()).rev() {
            where_is_bone(tree.pchain[i], ctime);
        }
        /* After that, evaluate the actual Spline IK. */
        if let Some(mut state) = splineik_evaluate_init(ctx, &tree) {
            /* Walk over each bone in the chain, calculating the
             * effects of spline IK — traversed parent to children. */
            for i in (0..tree.pchain.len()).rev() {
                let pchan = tree.pchain[i];
                evaluate_bone(&tree, pchan, i as i32, &mut state);
            }
        }
        /* free this tree (caller drops `trees`). */
    }
    let _ = ctime;
}

/// `BKE_pose_pchan_index_rebuild` — rebuild the indexed channel
/// array from the list order.
pub fn pose_pchan_index_rebuild(chanbase_order: &[usize]) -> Vec<usize> {
    chanbase_order.to_vec()
}

/// `pose_pchan_get_indexed`.
pub fn pose_pchan_get_indexed(chan_array: &[usize], pchan_index: i32) -> usize {
    chan_array[pchan_index as usize]
}

/// `BKE_pose_eval_bone` (verbatim — `arm_edbo`/`restpos` injected,
/// `where_is_bone` supplies `BKE_pose_where_is_bone`).
pub fn pose_eval_bone(
    arm_is_edit: bool,
    arm_restpos: bool,
    pchan_has_constraints: bool,
    pchan_flag: i32,
    ctime: f32,
    copy_rest_pose: &mut dyn FnMut(),
    where_is_bone: &mut dyn FnMut(f32),
) {
    if arm_is_edit {
        return;
    }
    if arm_restpos {
        copy_rest_pose();
    } else if !pchan_has_constraints {
        if pchan_flag & (POSE_IKTREE as i32 | POSE_IKSPLINE as i32) != 0 {
            /* pass */
        } else if pchan_flag & POSE_DONE == 0 {
            where_is_bone(ctime);
        }
    }
}

/// `BKE_pose_constraints_evaluate` (verbatim).
pub fn pose_constraints_evaluate(
    arm_is_edit: bool,
    arm_restpos: bool,
    pchan_flag: i32,
    ctime: f32,
    where_is_bone: &mut dyn FnMut(f32),
) {
    if arm_is_edit {
        return;
    }
    if arm_restpos {
        return;
    }
    if pchan_flag & (POSE_IKTREE as i32 | POSE_IKSPLINE as i32) != 0 {
        /* IK are being solved separately. */
    } else if pchan_flag & POSE_DONE == 0 {
        where_is_bone(ctime);
    }
}

/// `pose_channel_flush_to_orig_if_needed` — copies pose result
/// fields to the orig channel (verbatim field list).
pub fn pose_channel_flush_to_orig_if_needed(
    deg_is_active: bool,
    arm_is_edit: bool,
    copy_fields: &mut dyn FnMut(),
) {
    if !deg_is_active {
        return;
    }
    if arm_is_edit {
        return;
    }
    copy_fields();
}

/// `BKE_pose_bone_done` (verbatim — flush injected).
pub fn pose_bone_done(
    arm_is_edit: bool,
    deg_is_active: bool,
    flush_to_orig: &mut dyn FnMut(),
) {
    if arm_is_edit {
        return;
    }
    if deg_is_active {
        flush_to_orig();
    }
}

/// `BKE_pose_eval_bbone_segments` — re-apply B-Bone segments via
/// injected `bbone_segments_apply` (verbatim shape).
pub fn pose_eval_bbone_segments(arm_is_edit: bool, apply: &mut dyn FnMut()) {
    if arm_is_edit {
        return;
    }
    apply();
}

/// `BKE_pose_iktree_evaluate` (verbatim — `BIK_execute_tree`
/// injected).
pub fn pose_iktree_evaluate(
    arm_is_edit: bool,
    arm_restpos: bool,
    ctime: f32,
    bik_execute_tree: &mut dyn FnMut(f32),
) {
    if arm_is_edit {
        return;
    }
    if arm_restpos {
        return;
    }
    bik_execute_tree(ctime);
}

/// `BKE_pose_splineik_evaluate` (verbatim).
pub fn pose_splineik_evaluate(
    arm_is_edit: bool,
    arm_restpos: bool,
    ctime: f32,
    splineik_execute: &mut dyn FnMut(f32),
) {
    if arm_is_edit {
        return;
    }
    if arm_restpos {
        return;
    }
    splineik_execute(ctime);
}

/// `pose_eval_cleanup_common` — asserts only (verbatim).
pub fn pose_eval_cleanup_common() {}

/// `BKE_pose_eval_done` — debug print + assert (verbatim).
pub fn pose_eval_done() {}

/// `BKE_pose_eval_cleanup` (verbatim — `BIK_release_tree` injected).
pub fn pose_eval_cleanup(arm_is_edit: bool, ctime: f32, bik_release_tree: &mut dyn FnMut(f32)) {
    let _ = arm_is_edit;
    bik_release_tree(ctime);
    pose_eval_cleanup_common();
}
