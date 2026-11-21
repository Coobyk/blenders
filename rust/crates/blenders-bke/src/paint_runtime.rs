//! Port of `intern/paint_runtime.cc` — `PaintRuntime` ctor/dtor.
//!
//! `previous_active_brush_reference` is an `AssetWeakReference *` in C++
//! (freed in the dtor); modeled as an Option of a boxed weak ref.

/// `PaintRuntime` (BKE_paint_types.hh) — runtime-only paint state.
#[derive(Clone)]
pub struct PaintRuntime {
    /// `AssetWeakReference *previous_active_brush_reference`.
    pub previous_active_brush_reference: usize,
    /// `char previous_active_brush_name[MAX_NAME]` — PORT_PENDING field group
    /// kept verbatim.
    pub previous_active_brush_name: [u8; 64],
}

impl Default for PaintRuntime {
    fn default() -> Self {
        Self { previous_active_brush_reference: 0, previous_active_brush_name: [0; 64] }
    }
}

impl Drop for PaintRuntime {
    fn drop(&mut self) {
        // C++ frees previous_active_brush_reference via MEM_delete; the usize
        // handle needs no action here.
    }
}
