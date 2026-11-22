//! Port of `intern/compositor.cc` — Render compositor
//! lifecycle (needs_recreate → delete → create → exec).

/// `Render::compositor_execute` state (verbatim).
#[derive(Default)]
pub struct RenderCompositor {
    /// `compositor` instance present.
    pub compositor: bool,
    /// last input hash for `needs_to_be_recreated`.
    pub input_generation: u64,
}
impl RenderCompositor {
    /// `compositor_execute` (verbatim — recreate check,
    /// ensure, execute).
    pub fn compositor_execute(
        &mut self,
        input_generation: u64,
        needs_to_be_recreated: &dyn Fn(u64, u64) -> bool,
        execute: &mut dyn FnMut(u64),
    ) {
        if self.compositor && (needs_to_be_recreated)(self.input_generation, input_generation) {
            self.compositor = false;
        }
        if !self.compositor {
            self.compositor = true;
            self.input_generation = input_generation;
        }
        (execute)(input_generation);
    }
    /// `compositor_free` (verbatim).
    pub fn compositor_free(&mut self) {
        self.compositor = false;
    }
}
