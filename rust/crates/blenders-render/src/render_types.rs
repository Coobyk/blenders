//! Port of `intern/render_types.cc` — RenderDisplay
//! lock/progress dispatch.

/// `RenderDisplay` — C vtable callbacks as hooks
/// (verbatim semantics).
#[derive(Default)]
pub struct RenderDisplay {
    /// `display_update` hook handle → bool.
    pub display_update_active: bool,
    /// `current_scene_update` handle.
    pub current_scene: usize,
    /// `stats_draw` hook.
    pub stats_draw_active: bool,
    /// `progress` last value.
    pub progress: f32,
    /// `test_break` result latch.
    pub test_break: bool,
}
impl RenderDisplay {
    /// `progress` (verbatim — store).
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress;
    }
    /// `test_break` (verbatim — return latch).
    pub fn test_break(&self) -> bool {
        self.test_break
    }
}
