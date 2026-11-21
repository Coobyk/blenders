//! Port of `intern/scene.cc` pure parts — frame/ctime math
//! (`BKE_scene_frame_*`, `frame_snap_by_seconds`,
//! `get_playback_range`, `frame_clamp_for_playback`,
//! `validate_setscene`), orientation-slot helpers, engine
//! id tests, `render_num_threads`/`render_resolution`/
//! `preview_pixel_size`, `check_rigidbody_active`,
//! `object_base_flag_sync_from_base`.
//!
//! `PORT_PENDING`: `BKE_scene_copy_data_eevee`/`groups_
//! relink`/`object_find`/`set_background`/`base_iter_next`
//! (Main/ViewLayer iteration), `BKE_scene_graph_update_*`
//! (depsgraph), `BKE_scene_update_sound`/`_tag_audio_volume`
//! (AUD), `BKE_toolsettings_free`/`copy` (pointers-heavy),
//! multiview view handling (`BKE_scene_multiview_*` —
//! `SceneRenderView` list walk), color management.

use blenders_dna::scene_types::{
    RenderData, Scene, SCE_ORIENT_DEFAULT, SCE_ORIENT_ROTATE, SCE_ORIENT_SCALE,
    SCE_ORIENT_TRANSLATE, SCER_ALLOW_PREROLL, SCER_PRV_RANGE, R_BORDER, R_CROP, R_FIXED_THREADS,
};
use blenders_math::base::{clamp_i, max_ii};

/// `ScenePlaybackRange` verbatim.
#[derive(Clone, Copy)]
pub struct ScenePlaybackRange {
    /// `start_frame`.
    pub start_frame: i32,
    /// `end_frame`.
    pub end_frame: i32,
}

/// `BKE_scene_frame_snap_by_seconds` (verbatim).
pub fn scene_frame_snap_by_seconds(
    frames_per_second: f64,
    interval_in_seconds: f64,
    frame: f64,
) -> f32 {
    debug_assert!(interval_in_seconds > 0.0);
    debug_assert!(frames_per_second > 0.0);
    let interval_in_frames = frames_per_second * interval_in_seconds;
    let second_prev = interval_in_frames * (frame / interval_in_frames).floor();
    let second_next = second_prev + interval_in_frames.ceil();
    let delta_prev = frame - second_prev;
    let delta_next = second_next - frame;
    if delta_prev < delta_next {
        second_prev as f32
    } else {
        second_next as f32
    }
}

/// `BKE_scene_frame_to_ctime` (verbatim).
pub fn scene_frame_to_ctime(r: &RenderData, frame: i32) -> f32 {
    let mut ctime = frame as f32;
    ctime += r.subframe;
    ctime * r.framelen
}
/// `BKE_scene_ctime_get` (verbatim).
pub fn scene_ctime_get(r: &RenderData) -> f32 {
    scene_frame_to_ctime(r, r.cfra)
}
/// `BKE_scene_frame_get` (verbatim).
pub fn scene_frame_get(r: &RenderData) -> f32 {
    r.cfra as f32 + r.subframe
}
/// `BKE_scene_frame_set` (verbatim).
pub fn scene_frame_set(r: &mut RenderData, frame: f32) {
    let intpart = frame.floor();
    r.subframe = frame - intpart as f32;
    r.cfra = intpart as i32;
}

/// `BKE_scene_get_playback_range` (verbatim).
pub fn scene_get_playback_range(r: &RenderData) -> ScenePlaybackRange {
    if r.flag & SCER_PRV_RANGE != 0 {
        ScenePlaybackRange {
            start_frame: r.psfra,
            end_frame: r.pefra,
        }
    } else {
        ScenePlaybackRange {
            start_frame: r.sfra,
            end_frame: r.efra,
        }
    }
}

/// `BKE_scene_frame_clamp_for_playback` (verbatim).
pub fn scene_frame_clamp_for_playback(r: &mut RenderData, is_playing_forward: bool) {
    let range = scene_get_playback_range(r);
    if is_playing_forward {
        if r.cfra > range.end_frame {
            r.cfra = range.start_frame;
        }
    } else if r.cfra < range.start_frame {
        r.cfra = range.end_frame;
    }
    if r.flag & SCER_ALLOW_PREROLL == 0 {
        r.cfra = clamp_i(r.cfra, range.start_frame, range.end_frame);
    }
}

/// `BKE_scene_validate_setscene` (verbatim — walks
/// `scene.set` chain, cycle iff longer than scene count).
/// `next` resolves the `set` pointer of a scene index.
pub fn scene_validate_setscene(
    sce: usize,
    totscene: usize,
    next: &dyn Fn(usize) -> Option<usize>,
) -> bool {
    if next(sce).is_none() {
        return true;
    }
    let mut iter = sce;
    let mut a = 0usize;
    while let Some(s) = next(iter) {
        iter = s;
        a += 1;
        if a > totscene {
            return false;
        }
    }
    true
}

/// `V3D_GIZMO_SHOW_OBJECT_*` verbatim.
pub const V3D_GIZMO_SHOW_OBJECT_TRANSLATE: i32 = 1 << 0;
/// `V3D_GIZMO_SHOW_OBJECT_ROTATE`.
pub const V3D_GIZMO_SHOW_OBJECT_ROTATE: i32 = 1 << 1;
/// `V3D_GIZMO_SHOW_OBJECT_SCALE`.
pub const V3D_GIZMO_SHOW_OBJECT_SCALE: i32 = 1 << 2;
/// `V3D_ORIENT_CUSTOM`.
pub const V3D_ORIENT_CUSTOM: i32 = 100;

/// `BKE_scene_orientation_slot_get` — returns slot index
/// (falls back to `SCE_ORIENT_DEFAULT` when unselected).
pub fn scene_orientation_slot_get(flags: &[i32; 4], slot_index: usize) -> usize {
    let mut slot_index = slot_index;
    if flags[slot_index] & 1 == 0 {
        slot_index = SCE_ORIENT_DEFAULT as usize;
    }
    slot_index
}
/// `BKE_scene_orientation_slot_get_from_flag` (verbatim).
pub fn scene_orientation_slot_get_from_flag(flags: &[i32; 4], flag: i32) -> usize {
    debug_assert!(
        flag != 0
            && (flag
                & !(V3D_GIZMO_SHOW_OBJECT_TRANSLATE
                    | V3D_GIZMO_SHOW_OBJECT_ROTATE
                    | V3D_GIZMO_SHOW_OBJECT_SCALE))
                == 0
    );
    let mut slot_index = SCE_ORIENT_DEFAULT;
    if flag & V3D_GIZMO_SHOW_OBJECT_TRANSLATE != 0 {
        slot_index = SCE_ORIENT_TRANSLATE;
    } else if flag & V3D_GIZMO_SHOW_OBJECT_ROTATE != 0 {
        slot_index = SCE_ORIENT_ROTATE;
    } else if flag & V3D_GIZMO_SHOW_OBJECT_SCALE != 0 {
        slot_index = SCE_ORIENT_SCALE;
    }
    scene_orientation_slot_get(flags, slot_index as usize)
}

/// `RE_engine_id_*` verbatim.
pub const RE_ENGINE_ID_BLENDER_EEVEE: &[u8] = b"BLENDER_EEVEE";
/// `RE_engine_id_BLENDER_WORKBENCH`.
pub const RE_ENGINE_ID_BLENDER_WORKBENCH: &[u8] = b"BLENDER_WORKBENCH";
/// `RE_engine_id_CYCLES`.
pub const RE_ENGINE_ID_CYCLES: &[u8] = b"CYCLES";

fn engine_name(engine: &[u8; 32]) -> &[u8] {
    let end = engine.iter().position(|&c| c == 0).unwrap_or(32);
    &engine[..end]
}

/// `BKE_scene_uses_blender_eevee` (verbatim).
pub fn scene_uses_blender_eevee(scene: &Scene) -> bool {
    engine_name(&scene.r.engine) == RE_ENGINE_ID_BLENDER_EEVEE
}
/// `BKE_scene_uses_blender_workbench` (verbatim).
pub fn scene_uses_blender_workbench(scene: &Scene) -> bool {
    engine_name(&scene.r.engine) == RE_ENGINE_ID_BLENDER_WORKBENCH
}
/// `BKE_scene_uses_cycles` (verbatim).
pub fn scene_uses_cycles(scene: &Scene) -> bool {
    engine_name(&scene.r.engine) == RE_ENGINE_ID_CYCLES
}
/// `BKE_scene_uses_shader_previews` (verbatim).
pub fn scene_uses_shader_previews(scene: &Scene) -> bool {
    scene_uses_blender_eevee(scene) || scene_uses_cycles(scene)
}

/// `BKE_scene_check_rigidbody_active` (verbatim).
pub fn scene_check_rigidbody_active(
    has_rigidbody_world: bool,
    rbw_has_group: bool,
    rbw_flag_muted: bool,
) -> bool {
    has_rigidbody_world && rbw_has_group && !rbw_flag_muted
}

/// `BKE_render_num_threads` (verbatim — injected system
/// thread counts).
pub fn render_num_threads(
    rd: &RenderData,
    system_override_threads: i32,
    system_thread_count: i32,
) -> i32 {
    if system_override_threads > 0 {
        return system_override_threads;
    }
    let threads = if rd.mode & R_FIXED_THREADS != 0 {
        rd.threads as i32
    } else {
        system_thread_count
    };
    max_ii(threads, 1)
}
/// `BKE_scene_num_threads` (verbatim).
pub fn scene_num_threads(
    scene: &Scene,
    system_override_threads: i32,
    system_thread_count: i32,
) -> i32 {
    render_num_threads(&scene.r, system_override_threads, system_thread_count)
}

/// `BKE_render_resolution` (verbatim).
pub fn render_resolution(rd: &RenderData, use_crop: bool) -> (i32, i32) {
    let mut width = (rd.xsch * rd.size as i32) / 100;
    let mut height = (rd.ysch * rd.size as i32) / 100;
    if use_crop && (rd.mode & R_BORDER) != 0 && (rd.mode & R_CROP) != 0 {
        width = (rd.border.xmax * width as f32) as i32 - (rd.border.xmin * width as f32) as i32;
        height = (rd.border.ymax * height as f32) as i32 - (rd.border.ymin * height as f32) as i32;
    }
    (width, height)
}

/// `BKE_render_preview_pixel_size` (verbatim — `pixelsize`
/// is `U.pixelsize` from user prefs).
pub fn render_preview_pixel_size(rd: &RenderData, userdef_pixelsize: f32) -> i32 {
    if rd.preview_pixel_size == 0 {
        if userdef_pixelsize > 1.5 {
            2
        } else {
            1
        }
    } else {
        rd.preview_pixel_size as i32
    }
}
