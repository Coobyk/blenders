/// Port of `intern/draw_context.cc` —
/// `DRWContext` (the global draw context:
/// data, view, engines, manager, state).
/// `DRWContext` (verbatim — field layout).
#[derive(Default)]
pub struct DrwContext {
    /// `data` (DRWData per-viewport).
    pub data: Vec<usize>,
    /// `view` (current DRWView).
    pub view: Option<usize>,
    /// `engines` (enabled DrawEngine list).
    pub engines: Vec<usize>,
    /// `manager` (DRWManager — call buffer).
    pub manager: usize,
    /// `state` (DRWState — current pipeline).
    pub state: u32,
    /// `viewport` (scene handle).
    pub viewport: usize,
    /// `scene`/`view_layer`/`v3d`/`rv3d`.
    pub scene: usize,
    pub view_layer: usize,
    pub v3d: usize,
    pub rv3d: usize,
}
/// `DRW_context_state_init` (verbatim —
/// reset to defaults).
pub fn drw_context_state_init(ctx: &mut DrwContext) {
    *ctx = DrwContext::default();
}
/// `DRW_global_context` (verbatim — the
/// thread-local singleton handle).
static mut G_CTX: usize = 0;
pub fn drw_global_context() -> usize {
    unsafe { G_CTX }
}
