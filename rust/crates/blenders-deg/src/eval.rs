//! Port of `intern/depsgraph_eval.cc` —
/// `DEG_evaluate_on_refresh`/`on_framechange` +
/// `deg_flush_updates_and_refresh`.

use crate::writeback_sync::{DEG_EVALUATE_SYNC_WRITEBACK_NO, DEG_EVALUATE_SYNC_WRITEBACK_YES};

/// `ID_RECALC_FRAME_CHANGE` verbatim (`DNA_ID.h`).
pub const ID_RECALC_FRAME_CHANGE: u32 = 1 << 3;

/// Depsgraph fields touched by the refresh path (subset).
pub struct EvalCtx {
    /// `deg_graph->frame`.
    pub frame: f32,
    /// `deg_graph->ctime`.
    pub ctime: f32,
    /// `deg_graph->sync_writeback`.
    pub sync_writeback: i32,
    /// `deg_graph->is_active`.
    pub is_active: bool,
    /// `deg_graph->sync_writeback_callbacks`.
    pub sync_writeback_callbacks: Vec<Box<dyn FnMut() + Send>>,
    /// Scene `id.recalc`.
    pub scene_id_recalc: u32,
}

/// `deg_flush_updates_and_refresh` (verbatim —
/// `scene_frame_set`/`tag_visible`/`flush_updates`/`evaluate`
/// injected).
pub fn flush_updates_and_refresh(
    ctx: &mut EvalCtx,
    sync_writeback: i32,
    scene_frame_set: &mut dyn FnMut(f32),
    frame: f32,
    tag_ids_for_visible_update: &mut dyn FnMut(),
    deg_graph_flush_updates: &mut dyn FnMut(),
    evaluate_on_refresh: &mut dyn FnMut(),
) {
    /* Update the time on the cow scene. */
    scene_frame_set(frame);
    tag_ids_for_visible_update();
    deg_graph_flush_updates();
    ctx.sync_writeback = sync_writeback;
    evaluate_on_refresh();
    if ctx.sync_writeback == DEG_EVALUATE_SYNC_WRITEBACK_YES && ctx.is_active {
        for f in ctx.sync_writeback_callbacks.iter_mut() {
            f();
        }
    }
    ctx.sync_writeback = DEG_EVALUATE_SYNC_WRITEBACK_NO;
    ctx.sync_writeback_callbacks.clear();
}

/// `DEG_evaluate_on_refresh` (verbatim — `frame`/`ctime`
/// from the input scene, `tag_time_source` injected).
pub fn evaluate_on_refresh(
    ctx: &mut EvalCtx,
    frame: f32,
    ctime: f32,
    sync_writeback: i32,
    tag_time_source: &mut dyn FnMut(),
    scene_frame_set: &mut dyn FnMut(f32),
    tag_ids_for_visible_update: &mut dyn FnMut(),
    deg_graph_flush_updates: &mut dyn FnMut(),
    evaluate: &mut dyn FnMut(),
) {
    if ctx.frame != frame || ctime != ctx.ctime {
        tag_time_source();
        ctx.frame = frame;
        ctx.ctime = ctime;
    } else if ctx.scene_id_recalc & ID_RECALC_FRAME_CHANGE != 0 {
        /* Undo-state frame-change detection (#66913). */
        tag_time_source();
    }
    flush_updates_and_refresh(
        ctx,
        sync_writeback,
        scene_frame_set,
        frame,
        tag_ids_for_visible_update,
        deg_graph_flush_updates,
        evaluate,
    );
}

/// `DEG_evaluate_on_framechange` (verbatim).
pub fn evaluate_on_framechange(
    ctx: &mut EvalCtx,
    frame: f32,
    frame_to_ctime: f32,
    sync_writeback: i32,
    tag_time_source: &mut dyn FnMut(),
    scene_frame_set: &mut dyn FnMut(f32),
    tag_ids_for_visible_update: &mut dyn FnMut(),
    deg_graph_flush_updates: &mut dyn FnMut(),
    evaluate: &mut dyn FnMut(),
) {
    tag_time_source();
    ctx.frame = frame;
    ctx.ctime = frame_to_ctime;
    flush_updates_and_refresh(
        ctx,
        sync_writeback,
        scene_frame_set,
        frame,
        tag_ids_for_visible_update,
        deg_graph_flush_updates,
        evaluate,
    );
}
