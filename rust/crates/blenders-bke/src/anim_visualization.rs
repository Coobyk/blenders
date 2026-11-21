//! Port of `intern/anim_visualization.cc` — `animviz`/`motionpath` utils.
//!
//! GPU buffers (`points_vbo`/`batch_*`) are opaque handles; discard just
//! nils them (`GPU_*_DISCARD_SAFE` → set NIL). `Runtime` is a real struct.

use blenders_dna::action_types::{
    BAnimVizSettings, BMotionPath, MOTIONPATH_BAKE_CAMERA_SPACE, MOTIONPATH_BAKE_HEADS,
    MOTIONPATH_FLAG_BAKE_CAMERA, MOTIONPATH_FLAG_BHEAD, MOTIONPATH_VIEW_KFRAS,
    MOTIONPATH_VIEW_KFNOS,
};
use blenders_math::vector::Vec3;

/// `bke::motionpath::Runtime` (not in DNA).
#[derive(Clone, Default)]
pub struct MotionPathRuntime {
    pub vbo_preview: usize,
    pub batch_preview: usize,
    pub preview_location: Vec3,
    pub show_preview: bool,
}

/// `animviz::settings_init` (`BKE_animviz_settings_init`).
pub fn animviz_settings_init(avs: &mut BAnimVizSettings) {
    avs.path_bc = 10;
    avs.path_ac = 10;
    avs.path_sf = 1;   /* XXX: take from scene instead? */
    avs.path_ef = 250;
    avs.path_viewflag = MOTIONPATH_VIEW_KFRAS | MOTIONPATH_VIEW_KFNOS;
    avs.path_step = 1;
    avs.path_bakeflag |= MOTIONPATH_BAKE_HEADS;
}

/// `motionpath::set_preview_location` (`BKE_motionpath_set_preview_location`).
pub fn set_preview_location(mpath_runtime: &mut MotionPathRuntime, location: Vec3) {
    mpath_runtime.preview_location = location;
    mpath_runtime.show_preview = true;
    /* GPU_VERTBUF_DISCARD_SAFE → release handles. */
    mpath_runtime.vbo_preview = 0;
    mpath_runtime.batch_preview = 0;
}

/// `motionpath::clear_preview_location`.
pub fn clear_preview_location(mpath_runtime: &mut MotionPathRuntime) {
    mpath_runtime.show_preview = false;
    mpath_runtime.vbo_preview = 0;
    mpath_runtime.batch_preview = 0;
}

/// `motionpath::free_cache` — drops points + GPU handles.
pub fn motionpath_free_cache(mpath: &mut BMotionPath, runtime: &mut MotionPathRuntime) {
    mpath.points.clear();
    mpath.points_vbo = 0;
    mpath.batch_line = 0;
    mpath.batch_points = 0;
    runtime.vbo_preview = 0;
    runtime.batch_preview = 0;
    mpath.length = 0;
}

/// `motionpath::copy` — `points` dup'd, GPU handles cleared, runtime new.
pub fn motionpath_copy(mpath_src: &BMotionPath, runtime: &MotionPathRuntime) -> (BMotionPath, MotionPathRuntime) {
    let mut dst = mpath_src.clone();
    dst.points_vbo = 0;
    dst.batch_line = 0;
    dst.batch_points = 0;
    let rt = MotionPathRuntime {
        show_preview: runtime.show_preview,
        preview_location: runtime.preview_location,
        ..Default::default()
    };
    (dst, rt)
}

/// `motionpath::ensure` — validated motion-path (re)allocation.
/// `dst` = pchan->mpath or ob->mpath storage, `name` for the error report.
/// Returns Err(report-string) on the `path_sf >= path_ef` failure.
pub fn motionpath_ensure(
    avs: &BAnimVizSettings,
    dst: &mut Option<(BMotionPath, MotionPathRuntime)>,
    name: &str,
) -> Result<(), String> {
    if avs.path_sf >= avs.path_ef {
        return Err(format!(
            "Motion path frame extents invalid for {} ({} to {}){}",
            name,
            avs.path_sf,
            avs.path_ef,
            if avs.path_sf == avs.path_ef { ", cannot have single-frame paths" } else { "" }
        ));
    }
    let expected_length = (avs.path_ef - avs.path_sf) + 1;
    debug_assert!(expected_length > 1);
    if let Some((mpath, rt)) = dst.as_mut() {
        if (avs.path_bakeflag & MOTIONPATH_BAKE_CAMERA_SPACE) != 0 {
            mpath.flag |= MOTIONPATH_FLAG_BAKE_CAMERA;
        } else {
            mpath.flag &= !MOTIONPATH_FLAG_BAKE_CAMERA;
        }
        if mpath.start_frame != mpath.end_frame && mpath.length == expected_length {
            mpath.start_frame = avs.path_sf;
            mpath.end_frame = avs.path_ef + 1;
            return Ok(());
        }
        motionpath_free_cache(mpath, rt);
    } else {
        *dst = Some((BMotionPath::default(), MotionPathRuntime::default()));
    }
    let (mpath, _) = dst.as_mut().unwrap();
    mpath.start_frame = avs.path_sf;
    mpath.end_frame = avs.path_ef + 1;
    mpath.length = expected_length;
    if (avs.path_bakeflag & MOTIONPATH_BAKE_HEADS) != 0 {
        mpath.flag |= MOTIONPATH_FLAG_BHEAD;
    } else {
        mpath.flag &= !MOTIONPATH_FLAG_BHEAD;
    }
    if (avs.path_bakeflag & MOTIONPATH_BAKE_CAMERA_SPACE) != 0 {
        mpath.flag |= MOTIONPATH_FLAG_BAKE_CAMERA;
    } else {
        mpath.flag &= !MOTIONPATH_FLAG_BAKE_CAMERA;
    }
    mpath.color[0] = 1.0;
    mpath.color[1] = 0.0;
    mpath.color[2] = 0.0;
    mpath.line_thickness = 1;
    /* `color_post`/`runtime` defaults — new() ctor. */
    Ok(())
}

/// `motionpath::verify` — refresh flag from bake-flag (called on load).
pub fn motionpath_verify(avs: &BAnimVizSettings, mpath: &mut BMotionPath) {
    if (avs.path_bakeflag & MOTIONPATH_BAKE_CAMERA_SPACE) != 0 {
        mpath.flag |= MOTIONPATH_FLAG_BAKE_CAMERA;
    } else {
        mpath.flag &= !MOTIONPATH_FLAG_BAKE_CAMERA;
    }
}
