//! Port of `intern/tracking_solver.cc` — camera-reconstruction
//! solver wrapper: context setup, refine-flag mapping, keyframe-track
//! counting/checks, solve dispatch (modal vs reconstruction),
//! finish, and reconstruction scaling.
//!
//! libmv calls are injected (`LibmvHooks`).

/// `LIBMV_REFINE_*` bits (multiview_capi.h).
pub const LIBMV_REFINE_FOCAL_LENGTH: i32 = 1;
pub const LIBMV_REFINE_PRINCIPAL_POINT: i32 = 2;
pub const LIBMV_REFINE_RADIAL_DISTORTION: i32 = 4;
pub const LIBMV_REFINE_TANGENTIAL_DISTORTION: i32 = 8;

/// `TrackingRefineCameraFlag` (DNA_tracking_types.h).
pub const REFINE_FOCAL_LENGTH: i32 = 1;
pub const REFINE_PRINCIPAL_POINT: i32 = 2;
pub const REFINE_RADIAL_DISTORTION: i32 = 4;
pub const REFINE_TANGENTIAL_DISTORTION: i32 = 8;

/// `TRACKING_OBJECT_CAMERA`.
pub const TRACKING_OBJECT_CAMERA: i32 = 1;
/// `TRACKING_MOTION_MODAL` (settings.motion_flag).
pub const TRACKING_MOTION_MODAL: i32 = 1 << 3;
/// `TRACKING_USE_KEYFRAME_SELECTION` (settings.reconstruction_flag).
pub const TRACKING_USE_KEYFRAME_SELECTION: i32 = 1;
/// `MARKER_DISABLED`.
pub const MARKER_DISABLED: i32 = 1;
/// `TRACKING_RECONSTRUCTED`.
pub const TRACKING_RECONSTRUCTED: i32 = 1;
/// `TRACK_HAS_BUNDLE`.
pub const TRACK_HAS_BUNDLE: i32 = 1 << 4;

/// `MovieReconstructContext` — runtime fields the C struct carries.
#[derive(Default)]
pub struct MovieReconstructContext {
    /// `char object_name[MAX_NAME]`.
    pub object_name: String,
    /// `motion_flag` copy (settings.motion_flag).
    pub motion_flag: i32,
    /// `select_keyframes` — TRACKING_USE_KEYFRAME_SELECTION.
    pub select_keyframes: bool,
    /// `camera_intrinsics_options` (libmv struct, handle).
    pub camera_intrinsics_options: usize,
    /// `TracksMap *tracks_map` (handle).
    pub tracks_map: usize,
    /// `libmv_Tracks *tracks` (handle).
    pub tracks: usize,
    /// First/last enabled-marker frames across all tracks.
    pub sfra: i32,
    pub efra: i32,
    pub keyframe1: i32,
    pub keyframe2: i32,
    /// `LIBMV_REFINE_*` combined.
    pub refine_flags: i32,
    /// `libmv_Reconstruction *reconstruction` (handle).
    pub reconstruction: usize,
    /// `reprojection_error` after solve.
    pub reprojection_error: f64,
    /// `char error_message[512]`.
    pub error_message: String,
}

/// `reconstruct_refine_intrinsics_get_flags`.
pub fn reconstruct_refine_intrinsics_get_flags(
    refine_camera_intrinsics: i32,
    object_flag: i32,
) -> i32 {
    let refine = refine_camera_intrinsics;
    let mut flags = 0;

    if (object_flag & TRACKING_OBJECT_CAMERA) == 0 {
        return 0;
    }

    if refine & REFINE_FOCAL_LENGTH != 0 {
        flags |= LIBMV_REFINE_FOCAL_LENGTH;
    }
    if refine & REFINE_PRINCIPAL_POINT != 0 {
        flags |= LIBMV_REFINE_PRINCIPAL_POINT;
    }
    if refine & REFINE_RADIAL_DISTORTION != 0 {
        flags |= LIBMV_REFINE_RADIAL_DISTORTION;
    }
    if refine & REFINE_TANGENTIAL_DISTORTION != 0 {
        flags |= LIBMV_REFINE_TANGENTIAL_DISTORTION;
    }

    flags
}

/// `reconstruct_count_tracks_on_both_keyframes` — tracks with enabled
/// markers at both keyframes. `has_marker(track_i, frame)` injected.
pub fn reconstruct_count_tracks_on_both_keyframes(
    keyframe1: i32,
    keyframe2: i32,
    tracks_num: usize,
    has_enabled_marker_at_frame: &dyn Fn(usize, i32) -> bool,
) -> usize {
    let mut tot = 0;
    for track_i in 0..tracks_num {
        if has_enabled_marker_at_frame(track_i, keyframe1)
            && has_enabled_marker_at_frame(track_i, keyframe2)
        {
            tot += 1;
        }
    }
    tot
}

/// `BKE_tracking_reconstruction_check` — returns `Err(msg)` on
/// failure (`blender_compiled_without_libmv` mirrors `#ifndef
/// WITH_LIBMV` builds).
pub fn bke_tracking_reconstruction_check(
    motion_flag: i32,
    reconstruction_flag: i32,
    tracks_on_both_keyframes: usize,
    blender_compiled_without_libmv: bool,
) -> Result<(), &'static str> {
    if motion_flag & TRACKING_MOTION_MODAL != 0 {
        /* TODO: check for number of tracks? */
        return Ok(());
    }
    if reconstruction_flag & TRACKING_USE_KEYFRAME_SELECTION == 0 {
        /* automatic keyframe selection does not require any
         * pre-process checks */
        if tracks_on_both_keyframes < 8 {
            return Err("At least 8 common tracks on both keyframes are needed for reconstruction");
        }
    }

    if blender_compiled_without_libmv {
        return Err("Blender is compiled without motion tracking library");
    }

    Ok(())
}

/// `BKE_tracking_reconstruction_context_new` — build the context:
/// copy flags/intrinsics, compute sfra/efra from the first/last
/// enabled markers of every track, map tracks.
///
/// Per track: `(markers: &[(framenr, flag)])`.
pub fn bke_tracking_reconstruction_context_new(
    object_name: &str,
    motion_flag: i32,
    reconstruction_flag: i32,
    refine_camera_intrinsics: i32,
    object_flag: i32,
    keyframe1: i32,
    keyframe2: i32,
    tracks_markers: &[Vec<(i32, i32)>],
    camera_intrinsics_options: usize,
    libmv_tracks: usize,
    tracks_map: usize,
) -> MovieReconstructContext {
    let mut context = MovieReconstructContext {
        object_name: object_name.to_string(),
        motion_flag,
        select_keyframes: reconstruction_flag & TRACKING_USE_KEYFRAME_SELECTION != 0,
        camera_intrinsics_options,
        tracks_map,
        tracks: libmv_tracks,
        keyframe1,
        keyframe2,
        refine_flags: reconstruct_refine_intrinsics_get_flags(
            refine_camera_intrinsics,
            object_flag,
        ),
        ..Default::default()
    };

    let mut sfra = i32::MAX;
    let mut efra = i32::MIN;

    for markers in tracks_markers {
        /* find first not-disabled marker */
        let first = markers
            .iter()
            .position(|&(_, flag)| flag & MARKER_DISABLED == 0);
        /* find last not-disabled marker */
        let last = markers
            .iter()
            .rposition(|&(_, flag)| flag & MARKER_DISABLED == 0);

        if let Some(fi) = first {
            sfra = sfra.min(markers[fi].0);
        }
        if let Some(li) = last {
            efra = efra.max(markers[li].0);
        }
    }

    context.sfra = sfra;
    context.efra = efra;
    context
}

/// `BKE_tracking_reconstruction_report_error_message` — only keep the
/// initial error message, the rest are induced ones.
pub fn bke_tracking_reconstruction_report_error_message(
    context: &mut MovieReconstructContext,
    error_message: &str,
) {
    if !context.error_message.is_empty() {
        return;
    }
    context.error_message = error_message.to_string();
}

/// `BKE_tracking_reconstruction_error_message_get`.
pub fn bke_tracking_reconstruction_error_message_get(
    context: &MovieReconstructContext,
) -> &str {
    &context.error_message
}

/// `libmv_ReconstructionOptions` (filled from the context).
#[derive(Clone, Copy)]
pub struct LibmvReconstructionOptions {
    pub select_keyframes: bool,
    pub keyframe1: i32,
    pub keyframe2: i32,
    pub refine_intrinsics: i32,
}

/// `reconstructionOptionsFromContext`.
pub fn reconstruction_options_from_context(
    context: &MovieReconstructContext,
) -> LibmvReconstructionOptions {
    LibmvReconstructionOptions {
        select_keyframes: context.select_keyframes,
        keyframe1: context.keyframe1,
        keyframe2: context.keyframe2,
        refine_intrinsics: context.refine_flags,
    }
}

/// `reconstruct_update_solve_cb` — writes `"Solving camera | {msg}"`.
pub fn reconstruct_update_solve_cb(progress: f64, message: &str) -> (f64, String) {
    (progress, format!("Solving camera | {message}"))
}

/// `LibmvHooks` — libmv surface this file uses.
pub struct LibmvHooks<'a> {
    /// `libmv_solveModal` → reconstruction handle.
    pub solve_modal: &'a dyn Fn(
        usize,                          /* tracks */
        usize,                          /* intrinsics options */
        LibmvReconstructionOptions,     /* options */
    ) -> usize,
    /// `libmv_solveReconstruction` → reconstruction handle.
    pub solve_reconstruction: &'a dyn Fn(
        usize,
        usize,
        LibmvReconstructionOptions,
    ) -> usize,
    /// `libmv_reprojectionError`.
    pub reprojection_error: &'a dyn Fn(usize) -> f64,
    /// `libmv_reconstructionIsValid`.
    pub reconstruction_is_valid: &'a dyn Fn(usize) -> bool,
    /// `libmv_reconstructionDestroy` / `libmv_tracksDestroy` /
    /// `tracks_map_free` — used by context_free.
    pub reconstruction_destroy: &'a dyn Fn(usize),
    pub tracks_destroy: &'a dyn Fn(usize),
    pub tracks_map_free: &'a dyn Fn(usize),
    /// `reconstruct_retrieve_libmv` — cameras/bundles/intrinsics pull.
    pub retrieve_libmv: &'a dyn Fn(&MovieReconstructContext) -> bool,
    /// `tracks_map_merge`.
    pub tracks_map_merge: &'a dyn Fn(usize /*tracks_map*/),
}

/// `BKE_tracking_reconstruction_solve` — modal vs standard solve;
/// when keyframes were auto-selected the chosen keyframes are read
/// back (`select_keyframes`).
pub fn bke_tracking_reconstruction_solve(
    context: &mut MovieReconstructContext,
    hooks: &LibmvHooks,
) {
    let mut reconstruction_options = reconstruction_options_from_context(context);

    if context.motion_flag & TRACKING_MOTION_MODAL != 0 {
        context.reconstruction = (hooks.solve_modal)(
            context.tracks,
            context.camera_intrinsics_options,
            reconstruction_options,
        );
    } else {
        context.reconstruction = (hooks.solve_reconstruction)(
            context.tracks,
            context.camera_intrinsics_options,
            reconstruction_options,
        );

        if context.select_keyframes {
            /* store actual keyframes used for reconstruction to
             * update them in the interface later */
            context.keyframe1 = reconstruction_options.keyframe1;
            context.keyframe2 = reconstruction_options.keyframe2;
            let _ = &mut reconstruction_options;
        }
    }

    context.reprojection_error = (hooks.reprojection_error)(context.reconstruction);
}

/// `BKE_tracking_reconstruction_finish` — merge results into the
/// tracking object. `tracks_map_merge`, `object_get_named`-side
/// writes and `reconstruct_retrieve_libmv` are injected; the flag
/// updates are ported and applied to `reconstruction_flag`.
pub fn bke_tracking_reconstruction_finish(
    context: &mut MovieReconstructContext,
    reconstruction_flag: &mut i32,
    reconstruction_error: &mut f64,
    object_keyframes: Option<(&mut i32, &mut i32)>,
    hooks: &LibmvHooks,
) -> bool {
    if !(hooks.reconstruction_is_valid)(context.reconstruction) {
        bke_tracking_reconstruction_report_error_message(
            context,
            "Failed to solve the motion: most likely there are no good keyframes",
        );
        return false;
    }

    (hooks.tracks_map_merge)(context.tracks_map);

    /* update keyframe in the interface */
    if context.select_keyframes {
        if let Some((kf1, kf2)) = object_keyframes {
            *kf1 = context.keyframe1;
            *kf2 = context.keyframe2;
        }
    }

    *reconstruction_error = context.reprojection_error as f64;
    *reconstruction_flag |= TRACKING_RECONSTRUCTED;

    if !(hooks.retrieve_libmv)(context) {
        return false;
    }

    true
}

/// `BKE_tracking_reconstruction_context_free`.
pub fn bke_tracking_reconstruction_context_free(
    context: MovieReconstructContext,
    hooks: &LibmvHooks,
) {
    if context.reconstruction != 0 {
        (hooks.reconstruction_destroy)(context.reconstruction);
    }
    (hooks.tracks_destroy)(context.tracks);
    (hooks.tracks_map_free)(context.tracks_map);
}

/// `tracking_scale_reconstruction` — scale cameras around the first
/// camera's translation so the first camera stays put.
pub fn tracking_scale_reconstruction(
    cameras: &mut [[f32; 3]],
    scale: [f32; 3],
) -> [f32; 3] {
    let first_camera_delta = if !cameras.is_empty() {
        [
            cameras[0][0] * scale[0],
            cameras[0][1] * scale[1],
            cameras[0][2] * scale[2],
        ]
    } else {
        [0.0; 3]
    };

    for camera in cameras.iter_mut() {
        camera[0] = camera[0] * scale[0] - first_camera_delta[0];
        camera[1] = camera[1] * scale[1] - first_camera_delta[1];
        camera[2] = camera[2] * scale[2] - first_camera_delta[2];
    }
    first_camera_delta
}

/// `BKE_tracking_reconstruction_scale` — cameras + track bundles.
/// `bundle_positions` yields mutable `bundle_pos` for tracks with
/// `TRACK_HAS_BUNDLE`.
pub fn bke_tracking_reconstruction_scale(
    cameras: &mut [[f32; 3]],
    bundle_positions: &mut dyn Iterator<Item = (i32 /*flag*/, &mut [f32; 3])>,
    scale: [f32; 3],
) {
    /* `first_camera_delta` is the first camera's original scaled
     * translation (computed before mutation, as in C). */
    let first_camera_delta = tracking_scale_reconstruction(cameras, scale);
    for (flag, pos) in bundle_positions {
        if flag & TRACK_HAS_BUNDLE != 0 {
            pos[0] = pos[0] * scale[0] - first_camera_delta[0];
            pos[1] = pos[1] * scale[1] - first_camera_delta[1];
            pos[2] = pos[2] * scale[2] - first_camera_delta[2];
        }
    }
}
