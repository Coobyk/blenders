//! Port of `intern/engine.cc` — engine type registry
//! + RenderEngine state/dispatch.

/// `RE_*` engine/type flags verbatim.
pub const RE_INTERNAL: i32 = 1;
pub const RE_USE_GPU_CONTEXT: i32 = 1 << 1;
pub const RE_ENGINE_CAN_DRAW: i32 = 1;
pub const RE_ENGINE_HIGHLIGHT_TILES: i32 = 1 << 1;
/// `R_PERSISTENT_DATA` verbatim.
pub const R_PERSISTENT_DATA: i32 = 1 << 17;
/// Fallback engine idname verbatim.
pub const ENGINE_FALLBACK_IDNAME: &str = "BLENDER_EEVEE";

/// `RenderEngineType` verbatim fields.
#[derive(Default)]
pub struct RenderEngineType {
    /// `idname`.
    pub idname: String,
    /// `flag`.
    pub flag: i32,
    /// `render` callback present (external engine).
    pub has_render_cb: bool,
    /// `draw` callback present.
    pub has_draw_cb: bool,
}

/// `RenderEngine` verbatim fields.
#[derive(Default)]
pub struct RenderEngine {
    /// `type` index.
    pub type_index: Option<usize>,
    /// `flag` (CAN_DRAW/HIGHLIGHT_TILES).
    pub flag: i32,
    /// `view_pause`/`view_auto_pause`.
    pub view_pause: bool,
    pub view_auto_pause: bool,
}

/// `RE_engines_find` (verbatim — idname else fallback).
pub fn engines_find(types: &[RenderEngineType], idname: &str) -> Option<usize> {
    if let Some(i) = types.iter().position(|t| t.idname == idname) {
        return Some(i);
    }
    types.iter().position(|t| t.idname == ENGINE_FALLBACK_IDNAME)
}
/// `RE_engines_is_registered`.
pub fn engines_is_registered(types: &[RenderEngineType], idname: &str) -> bool {
    types.iter().any(|t| t.idname == idname)
}
/// `RE_engine_is_external` (verbatim — engine+type+render cb).
pub fn engine_is_external(engine: Option<&RenderEngine>, types: &[RenderEngineType]) -> bool {
    engine
        .and_then(|e| e.type_index)
        .map(|ti| types[ti].has_render_cb)
        .unwrap_or(false)
}

/// `RE_engine_draw_acquire` decision (verbatim —
/// engine+draw cb+CAN_DRAW flag; mutex held by caller).
pub fn engine_draw_can_acquire(engine: Option<&RenderEngine>, types: &[RenderEngineType]) -> bool {
    let Some(e) = engine else {
        return false;
    };
    let Some(ti) = e.type_index else {
        return false;
    };
    if !types[ti].has_draw_cb {
        return false;
    }
    (e.flag & RE_ENGINE_CAN_DRAW) != 0
}

/// `RE_engine_tile_highlight_set` gate (verbatim —
/// re present + HIGHLIGHT_TILES flag).
pub fn engine_tile_highlight_allowed(engine: &RenderEngine, re_present: bool) -> bool {
    if !re_present {
        return false;
    }
    (engine.flag & RE_ENGINE_HIGHLIGHT_TILES) != 0
}

/// `RE_engine_use_persistent_data` (verbatim —
/// R_PERSISTENT_DATA && !RE_USE_GPU_CONTEXT).
pub fn engine_use_persistent_data(r_mode: i32, type_flag: i32) -> bool {
    (r_mode & R_PERSISTENT_DATA) != 0 && (type_flag & RE_USE_GPU_CONTEXT) == 0
}

/// `RE_engine_view_pause_set`/`_get` (verbatim).
pub fn engine_view_pause_set(engine: &mut RenderEngine, pause: bool) {
    engine.view_pause = pause;
}
/// `RE_engine_view_pause_get` (verbatim — pause || auto).
pub fn engine_view_pause_get(engine: &RenderEngine) -> bool {
    engine.view_pause || engine.view_auto_pause
}
