//! Port of `intern/tracking_stabilize.cc` — 2D tracking
//! stabilization: per-track baseline
//! (`TrackStabilizationBase`/`init_track_for_stabilization`/
//! `init_all_tracks`), contribution averaging
//! (`translation_contribution`, `rotation_contribution`,
//! `average_track_contributions`, `average_marker_positions`,
//! `interpolate_averaged_track_contributions`), frame search
//! (`find_next_working_frames` & friends), `compensate_rotation_center`,
//! `stabilization_determine_offset_for_frame`,
//! `stabilization_calculate_data`, `stabilization_data_to_mat4`,
//! `calculate_autoscale_factor`. Marker/fcurve access is injected.
//! `PORT_PENDING`: `BKE_tracking_stabilize_frame` (ImBuf warp).

use std::collections::HashMap;

use blenders_dna::tracking_types::{
    MovieTrackingMarker, MovieTrackingStabilization, MovieTrackingTrack, MARKER_DISABLED,
    TRACKING_2D_STABILIZATION, TRACKING_STABILIZE_ROTATION, TRACKING_STABILIZE_SCALE,
    TRACK_USE_2D_STAB, TRACK_USE_2D_STAB_ROT,
};
use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec2;

const SCALE_ERROR_LIMIT_BIAS: f32 = 0.01;
const EPSILON_WEIGHT: f32 = 0.005;
const MINAFRAME: i32 = i32::MIN + 1;
const MAXFRAME: i32 = i32::MAX - 1;
const MINFRAME: i32 = i32::MIN + 2;

/// `TrackStabilizationBase`.
#[derive(Clone, Default)]
pub struct TrackStabilizationBase {
    /// `stabilization_offset_base`.
    pub stabilization_offset_base: [f32; 2],
    /// `stabilization_rotation_base` (2×2, measured rel. to pivot).
    pub stabilization_rotation_base: [[f32; 2]; 2],
    /// `stabilization_scale_base`.
    pub stabilization_scale_base: f32,
    /// `is_init_for_stabilization`.
    pub is_init_for_stabilization: bool,
}

/// `TrackInitOrder`.
pub struct TrackInitOrder {
    /// `sort_value`.
    pub sort_value: i32,
    /// `reference_frame`.
    pub reference_frame: i32,
    /// `data` — track index.
    pub data: usize,
}

/// `StabContext` — per-frame working context; animated settings are
/// injected through closures (`locinf`, `rotinf`, `scaleinf`,
/// `target_pos`, `target_rot`, `target_scale`, `weight`).
pub struct StabContext<'a> {
    /// `tracking->stabilization` settings snapshot.
    pub stab: MovieTrackingStabilization,
    /// Tracks of the camera object.
    pub tracks: Vec<MovieTrackingTrack>,
    /// `MovieTracking -> TrackStabilizationBase` per track index.
    pub stab_map: HashMap<usize, TrackStabilizationBase>,
    /// `camera.pixel_aspect`.
    pub pixel_aspect: f32,
    /// `get_animated_locinf`.
    pub get_locinf: &'a dyn Fn(i32) -> f32,
    /// `get_animated_rotinf`.
    pub get_rotinf: &'a dyn Fn(i32) -> f32,
    /// `get_animated_scaleinf`.
    pub get_scaleinf: &'a dyn Fn(i32) -> f32,
    /// `get_animated_target_pos`.
    pub get_target_pos: &'a dyn Fn(i32) -> [f32; 2],
    /// `get_animated_target_rot`.
    pub get_target_rot: &'a dyn Fn(i32) -> f32,
    /// `get_animated_target_scale`.
    pub get_target_scale: &'a dyn Fn(i32) -> f32,
    /// `get_animated_weight` for a track at a frame.
    pub get_weight: &'a dyn Fn(usize, i32) -> f32,
}

impl<'a> StabContext<'a> {
    /// `access_stabilization_baseline_data`.
    pub fn baseline(&self, track: usize) -> Option<&TrackStabilizationBase> {
        self.stab_map.get(&track)
    }
    /// mutable variant.
    pub fn baseline_mut(&mut self, track: usize) -> &mut TrackStabilizationBase {
        self.stab_map.entry(track).or_default()
    }

    /// `is_init_for_stabilization`.
    pub fn is_init(&self, track: usize) -> bool {
        self.baseline(track)
            .map(|b| b.is_init_for_stabilization)
            .unwrap_or(false)
    }

    /// `is_usable_for_stabilization`.
    pub fn is_usable(&self, track: usize) -> bool {
        (self.tracks[track].flag & TRACK_USE_2D_STAB) != 0 && self.is_init(track)
    }

    /// `is_effectively_disabled`.
    fn is_effectively_disabled(&self, track: usize, marker: &MovieTrackingMarker) -> bool {
        (marker.flag & MARKER_DISABLED) != 0
            || EPSILON_WEIGHT > (self.get_weight)(track, marker.framenr)
    }

    /// `BKE_tracking_marker_get_exact` — exact frame match.
    fn marker_exact(tracks: &[MovieTrackingTrack], track: usize, framenr: i32) -> Option<usize> {
        tracks[track]
            .markers
            .iter()
            .position(|m| m.framenr == framenr)
    }

    /// `BKE_tracking_marker_get` — nearest marker by frame distance.
    fn marker_get(tracks: &[MovieTrackingTrack], track: usize, framenr: i32) -> usize {
        let markers = &tracks[track].markers;
        let mut best = 0;
        let mut best_d = i64::MAX;
        for (i, m) in markers.iter().enumerate() {
            let d = (m.framenr as i64 - framenr as i64).abs();
            if d < best_d {
                best_d = d;
                best = i;
            }
        }
        best
    }

    /// `search_closest_marker_index`.
    fn search_closest_marker_index(tracks: &[MovieTrackingTrack], track: usize, ref_frame: i32) -> usize {
        Self::marker_get(tracks, track, ref_frame)
    }

    /// `retrieve_next_higher_usable_frame`.
    fn retrieve_next_higher_usable_frame(
        &self,
        track: usize,
        mut i: usize,
        ref_frame: i32,
        next_higher: &mut i32,
    ) {
        let markers = &self.tracks[track].markers;
        let end = markers.len();
        while i < end
            && (markers[i].framenr < ref_frame
                || self.is_effectively_disabled(track, &markers[i]))
        {
            i += 1;
        }
        if i < end && markers[i].framenr < *next_higher {
            *next_higher = markers[i].framenr;
        }
    }

    /// `retrieve_next_lower_usable_frame`.
    fn retrieve_next_lower_usable_frame(
        &self,
        track: usize,
        i0: usize,
        ref_frame: i32,
        next_lower: &mut i32,
    ) {
        let markers = &self.tracks[track].markers;
        let mut i = i0 as i32;
        while i >= 0
            && (markers[i as usize].framenr > ref_frame
                || self.is_effectively_disabled(track, &markers[i as usize]))
        {
            i -= 1;
        }
        if i >= 0 && markers[i as usize].framenr > *next_lower {
            *next_lower = markers[i as usize].framenr;
        }
    }

    /// `find_next_working_frames`.
    pub fn find_next_working_frames(
        &self,
        framenr: i32,
        next_lower: &mut i32,
        next_higher: &mut i32,
    ) {
        for track in 0..self.tracks.len() {
            if self.is_usable(track) {
                let startpoint =
                    Self::search_closest_marker_index(&self.tracks, track, framenr);
                self.retrieve_next_higher_usable_frame(
                    track, startpoint, framenr, next_higher,
                );
                self.retrieve_next_lower_usable_frame(
                    track, startpoint, framenr, next_lower,
                );
            }
        }
    }

    /// `get_closest_marker` — nearest usable marker.
    fn get_closest_marker(&self, track: usize, ref_frame: i32) -> Option<usize> {
        let mut next_lower = MINAFRAME;
        let mut next_higher = MAXFRAME;
        let i = Self::search_closest_marker_index(&self.tracks, track, ref_frame);
        self.retrieve_next_higher_usable_frame(track, i, ref_frame, &mut next_higher);
        self.retrieve_next_lower_usable_frame(track, i, ref_frame, &mut next_lower);
        if (next_higher - ref_frame) < (ref_frame - next_lower) {
            Self::marker_exact(&self.tracks, track, next_higher)
        } else {
            Self::marker_exact(&self.tracks, track, next_lower)
        }
    }

    /// `get_tracking_data_point` — marker + weight or none.
    fn get_tracking_data_point(
        &self,
        track: usize,
        framenr: i32,
        r_weight: &mut f32,
    ) -> Option<usize> {
        if let Some(mi) = Self::marker_exact(&self.tracks, track, framenr) {
            let marker = &self.tracks[track].markers[mi];
            if (marker.flag & MARKER_DISABLED) == 0 {
                *r_weight = (self.get_weight)(track, framenr);
                return Some(mi);
            }
        }
        *r_weight = 0.0;
        None
    }
}

/// `setup_pivot`.
pub fn setup_pivot(ref_pos: [f32; 2], r_pivot: &mut [f32; 2]) {
    *r_pivot = ref_pos;
}

/// `translation_contribution`.
fn translation_contribution(
    base: &TrackStabilizationBase,
    marker: &MovieTrackingMarker,
) -> [f32; 2] {
    [
        base.stabilization_offset_base[0] + marker.pos[0],
        base.stabilization_offset_base[1] + marker.pos[1],
    ]
}

fn mul_m2_v2(m: &[[f32; 2]; 2], v: &mut [f32; 2]) {
    let x = m[0][0] * v[0] + m[0][1] * v[1];
    let y = m[1][0] * v[0] + m[1][1] * v[1];
    v[0] = x;
    v[1] = y;
}

fn angle_to_mat2(angle: f32) -> [[f32; 2]; 2] {
    let (s, c) = angle.sin_cos();
    [[c, -s], [s, c]]
}

/// `rotation_contribution` — returns (quality, angle, scale).
fn rotation_contribution(
    base: &TrackStabilizationBase,
    marker: &MovieTrackingMarker,
    aspect: f32,
    pivot: &[f32; 2],
) -> (f32, f32, f32) {
    let mut pos = [marker.pos[0] - pivot[0], marker.pos[1] - pivot[1]];
    pos[0] *= aspect;
    mul_m2_v2(&base.stabilization_rotation_base, &mut pos);
    let result_angle = pos[1].atan2(pos[0]);
    let mut len = (pos[0] * pos[0] + pos[1] * pos[1]).sqrt();
    /* prevent points very close to the pivot from poisoning the result */
    let quality =
        1.0 - (-len * len / SCALE_ERROR_LIMIT_BIAS * SCALE_ERROR_LIMIT_BIAS).exp();
    len += SCALE_ERROR_LIMIT_BIAS;
    let result_scale = len * base.stabilization_scale_base;
    (quality, result_angle, result_scale)
}

/// `compensate_rotation_center`.
pub fn compensate_rotation_center(
    size: i32,
    aspect: f32,
    angle: f32,
    scale: f32,
    pivot: &[f32; 2],
    result_translation: &mut [f32; 2],
) {
    let origin = [0.5 * aspect * size as f32, 0.5 * size as f32];
    let intended_pivot = *pivot;
    let mut rotated_pivot = *pivot;
    let rotation_mat = angle_to_mat2(angle);
    rotated_pivot[0] -= origin[0];
    rotated_pivot[1] -= origin[1];
    mul_m2_v2(&rotation_mat, &mut rotated_pivot);
    rotated_pivot[0] *= scale;
    rotated_pivot[1] *= scale;
    rotated_pivot[0] += origin[0];
    rotated_pivot[1] += origin[1];
    result_translation[0] += intended_pivot[0];
    result_translation[1] += intended_pivot[1];
    result_translation[0] -= rotated_pivot[0];
    result_translation[1] -= rotated_pivot[1];
}

/// `average_track_contributions` — weighted translation + pivot,
/// then rotation/scale contributions.
#[allow(clippy::too_many_arguments)]
pub fn average_track_contributions(
    ctx: &StabContext,
    framenr: i32,
    aspect: f32,
    r_translation: &mut [f32; 2],
    r_pivot: &mut [f32; 2],
    r_angle: &mut f32,
    r_scale_step: &mut f32,
) -> bool {
    debug_assert!(ctx.stab.flag & TRACKING_2D_STABILIZATION != 0);
    *r_translation = [0.0; 2];
    let mut ref_pos = [0.0f32; 2];
    let mut ok = false;
    let mut weight_sum = 0.0f32;
    for track in 0..ctx.tracks.len() {
        if !ctx.is_init(track) {
            continue;
        }
        if ctx.tracks[track].flag & TRACK_USE_2D_STAB != 0 {
            let mut weight = 0.0;
            if let Some(mi) = ctx.get_tracking_data_point(track, framenr, &mut weight) {
                let base = ctx.baseline(track).unwrap();
                let offset = translation_contribution(base, &ctx.tracks[track].markers[mi]);
                weight_sum += weight;
                r_translation[0] += weight * offset[0];
                r_translation[1] += weight * offset[1];
                ref_pos[0] += weight * ctx.tracks[track].markers[mi].pos[0];
                ref_pos[1] += weight * ctx.tracks[track].markers[mi].pos[1];
                ok |= weight_sum > EPSILON_WEIGHT;
            }
        }
    }
    if !ok {
        return false;
    }
    ref_pos[0] /= weight_sum;
    ref_pos[1] /= weight_sum;
    r_translation[0] /= weight_sum;
    r_translation[1] /= weight_sum;
    setup_pivot(ref_pos, r_pivot);
    if ctx.stab.flag & TRACKING_STABILIZE_ROTATION == 0 {
        return true;
    }
    let mut ok = false;
    let mut weight_sum = 0.0f32;
    *r_angle = 0.0;
    *r_scale_step = 0.0;
    for track in 0..ctx.tracks.len() {
        if !ctx.is_init(track) {
            continue;
        }
        if ctx.tracks[track].flag & TRACK_USE_2D_STAB_ROT != 0 {
            let mut weight = 0.0;
            if let Some(mi) = ctx.get_tracking_data_point(track, framenr, &mut weight) {
                let base = ctx.baseline(track).unwrap();
                let (quality, rotation, scale) = rotation_contribution(
                    base,
                    &ctx.tracks[track].markers[mi],
                    aspect,
                    r_pivot,
                );
                let quality_weight = weight * quality;
                weight_sum += quality_weight;
                *r_angle += rotation * quality_weight;
                if ctx.stab.flag & TRACKING_STABILIZE_SCALE != 0 {
                    *r_scale_step += scale.ln() * quality_weight;
                } else {
                    *r_scale_step = 0.0;
                }
                ok |= weight > EPSILON_WEIGHT;
            }
        }
    }
    if ok {
        *r_scale_step /= weight_sum;
        *r_angle /= weight_sum;
    } else {
        *r_scale_step = 0.0;
        *r_angle = 0.0;
    }
    true
}

/// `average_marker_positions` — weighted average of usable marker
/// positions; falls back to the nearest working frame when the
/// current frame has no usable data.
pub fn average_marker_positions(ctx: &StabContext, framenr: i32, r_ref_pos: &mut [f32; 2]) {
    *r_ref_pos = [0.0; 2];
    let mut weight_sum = 0.0f32;
    let mut ok = false;
    for track in 0..ctx.tracks.len() {
        if ctx.tracks[track].flag & TRACK_USE_2D_STAB != 0 {
            let mut weight = 0.0;
            if let Some(mi) = ctx.get_tracking_data_point(track, framenr, &mut weight) {
                weight_sum += weight;
                r_ref_pos[0] += weight * ctx.tracks[track].markers[mi].pos[0];
                r_ref_pos[1] += weight * ctx.tracks[track].markers[mi].pos[1];
                ok |= weight_sum > EPSILON_WEIGHT;
            }
        }
    }
    if ok {
        r_ref_pos[0] /= weight_sum;
        r_ref_pos[1] /= weight_sum;
    } else {
        let mut next_lower = MINAFRAME;
        let mut next_higher = MAXFRAME;
        for track in 0..ctx.tracks.len() {
            if ctx.tracks[track].flag & TRACK_USE_2D_STAB != 0 {
                let startpoint =
                    StabContext::search_closest_marker_index(&ctx.tracks, track, framenr);
                ctx.retrieve_next_higher_usable_frame(
                    track, startpoint, framenr, &mut next_higher,
                );
                ctx.retrieve_next_lower_usable_frame(
                    track, startpoint, framenr, &mut next_lower,
                );
            }
        }
        if next_lower >= MINFRAME {
            average_marker_positions(ctx, next_lower, r_ref_pos);
        } else if next_higher < MAXFRAME {
            average_marker_positions(ctx, next_higher, r_ref_pos);
        }
    }
}

/// `interpolate_averaged_track_contributions` — lerp between two
/// bracketing frames.
#[allow(clippy::too_many_arguments)]
pub fn interpolate_averaged_track_contributions(
    ctx: &StabContext,
    framenr: i32,
    frame_a: i32,
    frame_b: i32,
    aspect: f32,
    r_translation: &mut [f32; 2],
    r_pivot: &mut [f32; 2],
    r_angle: &mut f32,
    r_scale_step: &mut f32,
) -> bool {
    debug_assert!(frame_a <= frame_b && frame_a <= framenr && framenr <= frame_b);
    let t = (framenr - frame_a) as f32 / (frame_b - frame_a) as f32;
    let s = 1.0 - t;
    let mut trans_a = [0.0; 2];
    let mut trans_b = [0.0; 2];
    let mut pivot_a = [0.0; 2];
    let mut pivot_b = [0.0; 2];
    let (mut angle_a, mut angle_b) = (0.0f32, 0.0f32);
    let (mut scale_a, mut scale_b) = (0.0f32, 0.0f32);
    if !average_track_contributions(
        ctx, frame_a, aspect, &mut trans_a, &mut pivot_a, &mut angle_a, &mut scale_a,
    ) {
        return false;
    }
    if !average_track_contributions(
        ctx, frame_b, aspect, &mut trans_b, &mut pivot_b, &mut angle_b, &mut scale_b,
    ) {
        return false;
    }
    r_translation[0] = trans_a[0] + t * (trans_b[0] - trans_a[0]);
    r_translation[1] = trans_a[1] + t * (trans_b[1] - trans_a[1]);
    r_pivot[0] = pivot_a[0] + t * (pivot_b[0] - pivot_a[0]);
    r_pivot[1] = pivot_a[1] + t * (pivot_b[1] - pivot_a[1]);
    *r_scale_step = s * scale_a + t * scale_b;
    *r_angle = s * angle_a + t * angle_b;
    true
}

/// `establish_track_initialization_order` — order tracks by marker
/// nearest to `anchor_frame`; returns usable count.
pub fn establish_track_initialization_order(
    ctx: &StabContext,
    order: &mut Vec<TrackInitOrder>,
) -> usize {
    let anchor = ctx.stab.anchor_frame;
    for (ti, track) in ctx.tracks.iter().enumerate() {
        if track.markers.is_empty() {
            continue;
        }
        let mut best = i32::MAX;
        let mut ref_frame = track.markers[0].framenr;
        for m in &track.markers {
            let d = (m.framenr - anchor).abs();
            if d < best {
                best = d;
                ref_frame = m.framenr;
            }
        }
        order.push(TrackInitOrder {
            sort_value: best,
            reference_frame: ref_frame,
            data: ti,
        });
    }
    order.sort_by_key(|o| o.sort_value);
    order.len()
}

/// `init_track_for_stabilization`.
#[allow(clippy::too_many_arguments)]
pub fn init_track_for_stabilization(
    ctx: &mut StabContext,
    track: usize,
    reference_frame: i32,
    aspect: f32,
    average_translation: &[f32; 2],
    pivot: &[f32; 2],
    average_angle: f32,
    average_scale_step: f32,
) {
    let marker = ctx.tracks[track]
        .markers
        .iter()
        .find(|m| m.framenr == reference_frame)
        .cloned();
    let marker = match marker {
        Some(m) => m,
        None => return,
    };
    let local_data = ctx.baseline_mut(track);
    local_data.stabilization_offset_base = [
        average_translation[0] - marker.pos[0],
        average_translation[1] - marker.pos[1],
    ];
    let pos = [
        (marker.pos[0] - pivot[0]) * aspect,
        marker.pos[1] - pivot[1],
    ];
    let angle = average_angle - pos[1].atan2(pos[0]);
    local_data.stabilization_rotation_base = angle_to_mat2(angle);
    let len = (pos[0] * pos[0] + pos[1] * pos[1]).sqrt() + SCALE_ERROR_LIMIT_BIAS;
    local_data.stabilization_scale_base = average_scale_step.exp() / len;
    local_data.is_init_for_stabilization = true;
}

/// `init_all_tracks`.
pub fn init_all_tracks(ctx: &mut StabContext, aspect: f32) {
    if ctx.tracks.is_empty() {
        return;
    }
    for track in 0..ctx.tracks.len() {
        ctx.baseline_mut(track).is_init_for_stabilization = false;
    }
    let mut order = Vec::new();
    let track_len = establish_track_initialization_order(ctx, &mut order);
    if track_len == 0 {
        return;
    }
    let mut reference_frame = ctx.stab.anchor_frame;
    let mut average_pos = [0.0; 2];
    let mut pivot = [0.0; 2];
    average_marker_positions(ctx, reference_frame, &mut average_pos);
    setup_pivot(average_pos, &mut pivot);
    let (mut average_angle, mut average_scale_step) = (0.0f32, 0.0f32);
    let mut average_translation = [0.0; 2];
    for i in 0..track_len {
        let track = order[i].data;
        if reference_frame != order[i].reference_frame {
            reference_frame = order[i].reference_frame;
            average_track_contributions(
                ctx,
                reference_frame,
                aspect,
                &mut average_translation,
                &mut pivot,
                &mut average_angle,
                &mut average_scale_step,
            );
        }
        init_track_for_stabilization(
            ctx,
            track,
            reference_frame,
            aspect,
            &average_translation,
            &pivot,
            average_angle,
            average_scale_step,
        );
    }
}

/// `stabilization_determine_offset_for_frame`.
#[allow(clippy::too_many_arguments)]
pub fn stabilization_determine_offset_for_frame(
    ctx: &StabContext,
    framenr: i32,
    aspect: f32,
    r_translation: &mut [f32; 2],
    r_pivot: &mut [f32; 2],
    r_angle: &mut f32,
    r_scale_step: &mut f32,
) -> bool {
    if ctx.stab.flag & TRACKING_2D_STABILIZATION == 0 {
        *r_translation = [0.0; 2];
        return false;
    }
    let mut success = average_track_contributions(
        ctx, framenr, aspect, r_translation, r_pivot, r_angle, r_scale_step,
    );
    if !success {
        let mut next_lower = MINAFRAME;
        let mut next_higher = MAXFRAME;
        ctx.find_next_working_frames(framenr, &mut next_lower, &mut next_higher);
        if next_lower >= MINFRAME && next_higher < MAXFRAME {
            success = interpolate_averaged_track_contributions(
                ctx,
                framenr,
                next_lower,
                next_higher,
                aspect,
                r_translation,
                r_pivot,
                r_angle,
                r_scale_step,
            );
        } else if next_higher < MAXFRAME {
            success = average_track_contributions(
                ctx, next_higher, aspect, r_translation, r_pivot, r_angle, r_scale_step,
            );
        } else if next_lower >= MINFRAME {
            success = average_track_contributions(
                ctx, next_lower, aspect, r_translation, r_pivot, r_angle, r_scale_step,
            );
        }
    }
    success
}

/// `stabilization_calculate_data` — apply animated influences,
/// convert to absolute pixel coords, optionally invert.
#[allow(clippy::too_many_arguments)]
pub fn stabilization_calculate_data(
    ctx: &StabContext,
    framenr: i32,
    size: i32,
    aspect: f32,
    do_compensate: bool,
    scale_step: f32,
    r_translation: &mut [f32; 2],
    r_pivot: &mut [f32; 2],
    r_scale: &mut f32,
    r_angle: &mut f32,
) {
    let scaleinf = (ctx.get_scaleinf)(framenr);
    if ctx.stab.flag & TRACKING_STABILIZE_SCALE != 0 {
        /* Averaged in log scale. */
        *r_scale = (scale_step * scaleinf).exp();
    } else {
        *r_scale = 1.0;
    }
    let locinf = (ctx.get_locinf)(framenr);
    r_translation[0] *= locinf;
    r_translation[1] *= locinf;
    *r_angle *= (ctx.get_rotinf)(framenr);
    let target_pos = (ctx.get_target_pos)(framenr);
    r_translation[0] -= target_pos[0];
    r_translation[1] -= target_pos[1];
    *r_angle -= (ctx.get_target_rot)(framenr);
    let target_scale = (ctx.get_target_scale)(framenr);
    if target_scale != 0.0 {
        *r_scale /= target_scale;
    }
    /* relative → absolute, square pixels */
    r_translation[0] *= size as f32 * aspect;
    r_translation[1] *= size as f32;
    r_pivot[0] *= size as f32 * aspect;
    r_pivot[1] *= size as f32;
    if do_compensate {
        r_translation[0] *= -1.0;
        r_translation[1] *= -1.0;
        *r_angle *= -1.0;
        if *r_scale != 0.0 {
            *r_scale = 1.0 / *r_scale;
        }
    }
}

/// `stabilization_data_to_mat4` — aspect·translate·pivot·scale·
/// rotate·inv_pivot·inv_aspect.
pub fn stabilization_data_to_mat4(
    pixel_aspect: f32,
    pivot: &[f32; 2],
    translation: &[f32; 2],
    scale: f32,
    angle: f32,
) -> Mat4 {
    let mut aspect_mat = Mat4::IDENTITY;
    aspect_mat[0].x /= pixel_aspect;
    let inv_aspect_mat = aspect_mat.inverted().unwrap_or(Mat4::IDENTITY);
    let mut pivot_mat = Mat4::IDENTITY;
    pivot_mat[3].x += pivot[0];
    pivot_mat[3].y += pivot[1];
    let mut inv_pivot_mat = Mat4::IDENTITY;
    inv_pivot_mat[3].x -= pivot[0];
    inv_pivot_mat[3].y -= pivot[1];
    let mut scale_mat = Mat4::IDENTITY;
    scale_mat[0].x *= scale;
    scale_mat[1].y *= scale;
    let mut translation_mat = Mat4::IDENTITY;
    translation_mat[3].x += translation[0];
    translation_mat[3].y += translation[1];
    let mut rotation_mat = Mat4::IDENTITY;
    blenders_math::matrix_c::rotate_m4(&mut rotation_mat, b'Z', angle);
    aspect_mat
        * translation_mat
        * pivot_mat
        * scale_mat
        * rotation_mat
        * inv_pivot_mat
        * inv_aspect_mat
}

/// `calculate_autoscale_factor` — max required zoom over all frames
/// so stabilized footage stays inside the canvas.
pub fn calculate_autoscale_factor(
    ctx: &StabContext,
    size: i32,
    aspect: f32,
) -> f32 {
    let pixel_aspect = ctx.pixel_aspect;
    let height = size as f32;
    let width = aspect * size as f32;
    let mut sfra = i32::MAX;
    let mut efra = i32::MIN;
    let mut scale = 1.0f32;
    let mut scale_step = 0.0f32;
    for track in &ctx.tracks {
        if (track.flag & TRACK_USE_2D_STAB) != 0
            || ((ctx.stab.flag & TRACKING_STABILIZE_ROTATION) != 0
                && (track.flag & TRACK_USE_2D_STAB_ROT) != 0)
        {
            if track.markers.is_empty() {
                continue;
            }
            let first_frame = track.markers[0].framenr;
            let last_frame = track.markers[track.markers.len() - 1].framenr;
            sfra = sfra.min(first_frame);
            efra = efra.max(last_frame);
        }
    }
    for cfra in sfra..=efra {
        let mut translation = [0.0; 2];
        let mut pivot = [0.0; 2];
        let mut angle = 0.0;
        let mut tmp_scale = 0.0;
        stabilization_determine_offset_for_frame(
            ctx, cfra, aspect, &mut translation, &mut pivot, &mut angle, &mut scale_step,
        );
        stabilization_calculate_data(
            ctx,
            cfra,
            size,
            aspect,
            true,
            scale_step,
            &mut translation,
            &mut pivot,
            &mut tmp_scale,
            &mut angle,
        );
        let mat = stabilization_data_to_mat4(
            pixel_aspect, &pivot, &translation, tmp_scale, angle,
        );
        let points: [[f32; 2]; 4] = [
            [0.0, 0.0],
            [0.0, height],
            [width, height],
            [width, 0.0],
        ];
        for edge_index in 0..4 {
            let p1 = points[edge_index];
            let p2 = points[(edge_index + 1) % 4];
            let tp1 = mat.transform_point(Vec3::new(p1[0], p1[1], 0.0));
            let tp2 = mat.transform_point(Vec3::new(p2[0], p2[1], 0.0));
            let stable_edge_p1 = Vec2::new(tp1.x, tp1.y);
            let stable_edge_p2 = Vec2::new(tp2.x, tp2.y);
            for point in &points {
                let stable_edge_vec = stable_edge_p2 - stable_edge_p1;
                let pt = Vec2::new(point[0], point[1]);
                let point_to_edge_start = pt - stable_edge_p1;
                let cross = stable_edge_vec.x * point_to_edge_start.y
                    - stable_edge_vec.y * point_to_edge_start.x;
                if cross >= 0.0 {
                    let scale_pivot =
                        Vec2::new(pivot[0] + translation[0], pivot[1] + translation[1]);
                    let point2 = pt + stable_edge_vec;
                    let real_dist = blenders_math::geom::dist_to_line_v2(
                        scale_pivot,
                        stable_edge_p1,
                        stable_edge_p2,
                    );
                    let required_dist =
                        blenders_math::geom::dist_to_line_v2(scale_pivot, pt, point2);
                    let s = required_dist / real_dist;
                    scale = scale.max(s);
                }
            }
        }
    }
    if ctx.stab.maxscale > 0.0 {
        scale = scale.min(ctx.stab.maxscale);
    }
    scale
}

use blenders_math::vector::Vec3;
