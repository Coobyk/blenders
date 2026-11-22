//! Port of `xr/intern/wm_xr.cc` — XR init/exit, error
//! handler, GPU-binding candidates, runtime data.

/// `GHOST_TXrGraphicsBinding` verbatim.
pub const GHOST_KXR_GRAPHICS_OPENGL: i32 = 1;
pub const GHOST_KXR_GRAPHICS_OPENGL_D3D11: i32 = 2;
pub const GHOST_KXR_GRAPHICS_VULKAN: i32 = 3;
pub const GHOST_KXR_GRAPHICS_VULKAN_D3D11: i32 = 4;
pub const GHOST_KXR_GRAPHICS_METAL: i32 = 5;
/// `GHOST_XrContextFlag` verbatim.
pub const GHOST_KXR_CONTEXT_DEBUG: i32 = 1 << 0;
pub const GHOST_KXR_CONTEXT_DEBUG_TIME: i32 = 1 << 1;
pub const GHOST_KXR_CONTEXT_GPU_NVIDIA: i32 = 1 << 2;
/// `G_DEBUG_XR`/`G_DEBUG_XR_TIME` verbatim.
pub const G_DEBUG_XR: i32 = 1 << 24;
pub const G_DEBUG_XR_TIME: i32 = 1 << 25;
/// `GPU_BACKEND_*` verbatim.
pub const GPU_BACKEND_NONE: i32 = 0;
pub const GPU_BACKEND_OPENGL: i32 = 1 << 0;
pub const GPU_BACKEND_METAL: i32 = 1 << 1;
pub const GPU_BACKEND_VULKAN: i32 = 1 << 3;

/// `wm_xr_init` GPU-binding candidate list (verbatim —
/// per-backend primary + WIN32 d3d11 fallback).
pub fn xr_gpu_bindings_candidates(gpu_backend: i32, is_win32: bool) -> Vec<i32> {
    let mut v = Vec::new();
    match gpu_backend {
        x if x == GPU_BACKEND_OPENGL => {
            v.push(GHOST_KXR_GRAPHICS_OPENGL);
            if is_win32 {
                v.push(GHOST_KXR_GRAPHICS_OPENGL_D3D11);
            }
        }
        x if x == GPU_BACKEND_VULKAN => {
            v.push(GHOST_KXR_GRAPHICS_VULKAN);
            if is_win32 {
                v.push(GHOST_KXR_GRAPHICS_VULKAN_D3D11);
            }
        }
        x if x == GPU_BACKEND_METAL => {
            v.push(GHOST_KXR_GRAPHICS_METAL);
        }
        _ => {}
    }
    v
}

/// `wm_xr_init` create-info flags (verbatim — debug bits
/// + WIN32 nvidia).
pub fn xr_context_flags(g_debug: i32, is_win32_nvidia: bool) -> i32 {
    let mut context_flag = 0;
    if (g_debug & G_DEBUG_XR) != 0 {
        context_flag |= GHOST_KXR_CONTEXT_DEBUG;
    }
    if (g_debug & G_DEBUG_XR_TIME) != 0 {
        context_flag |= GHOST_KXR_CONTEXT_DEBUG_TIME;
    }
    if is_win32_nvidia {
        context_flag |= GHOST_KXR_CONTEXT_GPU_NVIDIA;
    }
    context_flag
}

/// `wm_xr_events_handle` (verbatim — handle + update
/// when session ready).
pub fn wm_xr_events_handle(
    has_runtime_ghost: bool,
    session_is_ready: bool,
    ghost_events_handle: &mut dyn FnMut(),
    session_actions_update: &mut dyn FnMut(),
) -> bool {
    if has_runtime_ghost {
        (ghost_events_handle)();
        if session_is_ready {
            (session_actions_update)();
        }
        return true;
    }
    false
}

/// `wmXrRuntimeData` verbatim.
#[derive(Default)]
pub struct WmXrRuntimeData {
    /// `ghost_context`.
    pub ghost_context: usize,
    /// `b_context`.
    pub b_context: usize,
    /// `offscreen_area`.
    pub offscreen_area: usize,
    /// `session_state`.
    pub session_state: usize,
}
