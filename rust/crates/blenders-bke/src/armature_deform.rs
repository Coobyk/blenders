//! Port of `intern/armature_deform.cc` — `BKE_armature_deform_coords_*`
//! (envelope + vertex-group bone deformation).
//!
//! `Object`/`Mesh`/`bmesh` iteration → caller-supplied vertex slices;
//! `BKE_defvert_find_weight` → `MDeformVert` helpers in dna.

use blenders_dna::action_types::{
    BPoseChannel, BPoseChannelBboneSegmentBoundary,
};
use blenders_dna::armature_types::Bone;
use blenders_math::matrix::{Mat3, Mat4};
use blenders_math::quaternion::{
    add_weighted_dq_dq_pivot, mul_v3m3_dq, normalize_dq, DualQuat,
};
use blenders_math::vector::Vec3;

/// `MAX_BBONE_SUBDIV`.
pub const MAX_BBONE_SUBDIV: usize = 8;

/// `bone_envelope_falloff`.
pub fn bone_envelope_falloff(distance_squared: f32, closest_radius: f32, falloff_distance: f32) -> f32 {
    if distance_squared < closest_radius * closest_radius {
        return 1.0;
    }
    if falloff_distance == 0.0
        || distance_squared >= (closest_radius + falloff_distance) * (closest_radius + falloff_distance)
    {
        return 0.0;
    }
    let dist_envelope = distance_squared.sqrt() - closest_radius;
    1.0 - (dist_envelope * dist_envelope) / (falloff_distance * falloff_distance)
}

/// `distfactor_to_bone`.
pub fn distfactor_to_bone(
    position: Vec3,
    head: Vec3,
    tail: Vec3,
    radius_head: f32,
    radius_tail: f32,
    falloff_distance: f32,
) -> f32 {
    let axis_v = tail - head;
    let bone_length = axis_v.length();
    let bone_axis = if bone_length != 0.0 { axis_v / bone_length } else { Vec3::ZERO };
    let height = (position - head).dot(bone_axis);
    if height < 0.0 {
        return bone_envelope_falloff((position - head).length_squared(), radius_head, falloff_distance);
    }
    if height > bone_length {
        return bone_envelope_falloff((tail - position).length_squared(), radius_tail, falloff_distance);
    }
    let distance_squared = (position - head).length_squared() - height * height;
    let closest_radius = if bone_length != 0.0 {
        radius_head + (radius_tail - radius_head) * (height / bone_length)
    } else {
        radius_head
    };
    bone_envelope_falloff(distance_squared, closest_radius, falloff_distance)
}

/// `BKE_pchan_bbone_deform_clamp_segment_index` → `(index, blend_next)`.
pub fn bbone_deform_clamp_segment_index(segments: i32, head_tail: f32) -> (i32, f32) {
    let head_tail = head_tail.clamp(0.0, 1.0);
    let pre_blend = head_tail * segments as f32;
    let index = (pre_blend.floor() as i32).clamp(0, segments - 1);
    let blend = (pre_blend - index as f32).clamp(0.0, 1.0);
    (index, blend)
}

/// `bbone_segment_bsp_signed_distance`.
fn bbone_segment_bsp_signed_distance(b: &BPoseChannelBboneSegmentBoundary, co: Vec3) -> f32 {
    Vec3::from(b.plane_normal).dot(co) - b.plane_offset
}

/// `find_bbone_segment_index_straight` — `co` projected on bone Y via
/// `runtime.bbone_deform_mats[0]` (inverse world→bone-space matrix).
fn find_bbone_segment_index_straight(
    bone: &Bone,
    mat0: &Mat4,
    co: Vec3,
) -> (i32, f32) {
    let m = mat0;
    let y = m[0][1] * co.x + m[1][1] * co.y + m[2][1] * co.z + m[3][1];
    bbone_deform_clamp_segment_index(bone.segments as i32, y / bone.length)
}

/// `find_bbone_segment_index_curved` — binary-search BSP boundaries with
/// the smooth-rollback stack (verbatim).
fn find_bbone_segment_index_curved(
    bone: &Bone,
    boundaries: &[BPoseChannelBboneSegmentBoundary],
    segments: i32,
    arc_length_reciprocal: f32,
    co: Vec3,
) -> (i32, f32) {
    let mut boundary_dist = [0.0f32; MAX_BBONE_SUBDIV + 1];
    let mut boundary_idx_stack = [0usize; MAX_BBONE_SUBDIV];
    let mut stack_top: i32 = -1;
    let mut start: i32 = -1;
    let mut end: i32 = segments + 1;
    let mut bias = 0;
    while end - start > 1 {
        let mid = (start + end + bias) / 2;
        let dist = bbone_segment_bsp_signed_distance(&boundaries[mid as usize], co);
        stack_top += 1;
        boundary_idx_stack[stack_top as usize] = mid as usize;
        boundary_dist[mid as usize] = dist;
        if dist < 0.0 {
            end = mid;
            bias = 1;
        } else {
            start = mid;
            bias = 0;
        }
    }
    let segment_size = 1.0 / segments as f32;
    let mut head_tail;
    if end <= 0 {
        head_tail = 0.0;
    } else if start >= segments {
        head_tail = 1.0;
    } else {
        let d1 = boundary_dist[start as usize].abs();
        let d2 = boundary_dist[end as usize].abs();
        let t = d1 / (d1 + d2);
        head_tail = segment_size * (start as f32 + t);
    }
    let segment_scale = arc_length_reciprocal;
    for i in (0..=stack_top).rev() {
        let boundary_idx = boundary_idx_stack[i as usize];
        let boundary_pos = boundary_idx as f32 * segment_size;
        let point_dist = boundary_dist[boundary_idx] * segment_scale;
        let point_dist_abs = point_dist.abs();
        let mapped_dist = (head_tail - boundary_pos).abs();
        let slope_gap = mapped_dist - point_dist_abs;
        if slope_gap <= 0.0 {
            continue;
        }
        let dist_coeff = 1.0 - point_dist_abs * boundaries[i as usize].depth_scale;
        if dist_coeff <= 0.0 {
            continue;
        }
        const SLOPE_TUNE_COEFF: f32 = 3.0;
        let scaled_gap = slope_gap * SLOPE_TUNE_COEFF;
        let slope_coeff = scaled_gap / (scaled_gap + point_dist_abs);
        const DIST_TUNE_COEFF: f32 = 7.0;
        let dist_coeff_smooth = dist_coeff * dist_coeff * (DIST_TUNE_COEFF + 1.0)
            / (DIST_TUNE_COEFF * dist_coeff + 1.0);
        let target_pos = boundary_pos + point_dist;
        head_tail += (target_pos - head_tail) * (slope_coeff * dist_coeff_smooth);
    }
    bbone_deform_clamp_segment_index(segments, head_tail)
}

/// `BKE_pchan_bbone_deform_segment_index` — straight when no
/// `bbone_segment_boundaries`, curved otherwise.
pub fn bke_pchan_bbone_deform_segment_index(
    bone: &Bone,
    pchan: &BPoseChannel,
    co: Vec3,
) -> (i32, f32) {
    let rt = &pchan.runtime;
    if !rt.bbone_segment_boundaries.is_empty() {
        find_bbone_segment_index_curved(
            bone,
            &rt.bbone_segment_boundaries,
            rt.bbone_segments,
            rt.bbone_arc_length_reciprocal,
            co,
        )
    } else {
        find_bbone_segment_index_straight(bone, &rt.bbone_deform_mats[0], co)
    }
}

/// `BoneDeformLinearMixer` — weighted position-delta + deform matrix.
pub struct BoneDeformLinearMixer {
    pub position_delta: Vec3,
    pub deform: Mat3,
}

impl BoneDeformLinearMixer {
    pub fn new() -> Self {
        Self { position_delta: Vec3::ZERO, deform: Mat3::ZERO }
    }
    /// `accumulate` — `transform_point(pose_mat, co) - co` weighted.
    pub fn accumulate(&mut self, pose_mat: &Mat4, co: Vec3, weight: f32, full_deform: bool) {
        self.position_delta += (*pose_mat * co.extend(1.0)).xyz() * weight
            - co * weight;
        if full_deform {
            self.deform += Mat3::from(*pose_mat) * weight;
        }
    }
    /// `accumulate_bbone` — uses `bbone_deform_mats[index + 1]`.
    pub fn accumulate_bbone(
        &mut self,
        pchan: &BPoseChannel,
        co: Vec3,
        weight: f32,
        index: usize,
        full_deform: bool,
    ) {
        let mats = &pchan.runtime.bbone_deform_mats;
        let pose_mat = &mats[index + 1];
        self.accumulate(pose_mat, co, weight, full_deform);
    }
    /// `finalize` → `(delta_co, deform_mat)`.
    pub fn finalize(&self, total: f32, armature_weight: f32) -> (Vec3, Mat3) {
        let scale_factor = armature_weight / total;
        (self.position_delta * scale_factor, self.deform * scale_factor)
    }
}

/// `BoneDeformDualQuaternionMixer`.
pub struct BoneDeformDualQuaternionMixer {
    pub dq: DualQuat,
}

impl BoneDeformDualQuaternionMixer {
    pub fn new() -> Self {
        Self { dq: DualQuat::default() }
    }
    pub fn accumulate(&mut self, deform_quat: &DualQuat, co: Vec3, weight: f32, full_deform: bool) {
        add_weighted_dq_dq_pivot(&mut self.dq, deform_quat, co, weight, full_deform);
    }
    /// `accumulate_bbone` — `bbone_dual_quats[index]`.
    pub fn accumulate_bbone(
        &mut self,
        pchan: &BPoseChannel,
        co: Vec3,
        weight: f32,
        index: usize,
        full_deform: bool,
    ) {
        let quats = &pchan.runtime.bbone_dual_quats;
        self.accumulate(&quats[index], co, weight, full_deform);
    }
    /// `finalize` → `(delta_co, deform_mat)` from normalized dq.
    pub fn finalize(mut self, co: Vec3, total: f32, armature_weight: f32) -> (Vec3, Mat3) {
        normalize_dq(&mut self.dq, total);
        let (dco, dmat) = mul_v3m3_dq(co, &self.dq);
        ((dco - co) * armature_weight, dmat)
    }
}

/// `dist_bone_deform` — envelope factor for `(co)` under `bone`;
/// returns `(weight, bbone_index_or_-1)`. The actual mixer accumulation
/// is done by the caller via `accumulate_bbone`/`accumulate`
/// (pchan bone deform decides which pose-mat/dq feeds it).
pub fn dist_bone_deform(bone: &Bone, pchan: &BPoseChannel, co: Vec3) -> f32 {
    if bone.weight == 0.0 {
        return 0.0;
    }
    let fac = distfactor_to_bone(
        co,
        Vec3::from(bone.arm_head),
        Vec3::from(bone.arm_tail),
        bone.rad_head,
        bone.rad_tail,
        bone.dist,
    );
    if fac == 0.0 {
        return 0.0;
    }
    fac * bone.weight
}

/// `pchan_bone_deform` — single pchan contribution weight (bone may be
/// None for defgrp-only channels; then it's the caller's defvert weight).
/// Mirrors the C which folds envelope*weight or the defvert weight.
pub fn pchan_bone_deform_weight(bone: Option<&Bone>, pchan: &BPoseChannel, co: Vec3, defvert_weight: f32) -> f32 {
    match bone {
        Some(b) => dist_bone_deform(b, pchan, co),
        None => defvert_weight,
    }
}
