//! Port of `intern/context.cc` — compositor
//! execution context (gpu flag, precision,
//! quality, tree/log refs).

/// `Context` verbatim fields.
#[derive(Default)]
pub struct Context {
    /// `use_gpu`.
    pub use_gpu: bool,
    /// `output_render_quality`.
    pub output_render_quality: i32,
    /// `precision`.
    pub precision: i32,
    /// `tree` handle / `node_tree` ref.
    pub node_tree: usize,
    /// `nodes_evaluation_log` handle.
    pub nodes_evaluation_log: Option<usize>,
    /// `static_cache_manager`.
    pub static_cache_manager: usize,
    /// `is_canvas_compositing`.
    pub is_canvas_compositing: bool,
}
/// `eCompositorExecutionMode` verbatim ordinals.
pub const EXECUTION_MODE_REALTIME: i32 = 0;
pub const EXECUTION_MODE_FINAL: i32 = 1;
/// `eCompositorPrecision` verbatim ordinals.
pub const COMPOSITOR_PRECISION_AUTO: i32 = 0;
pub const COMPOSITOR_PRECISION_FULL: i32 = 1;
