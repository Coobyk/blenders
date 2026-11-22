/// Port of `intern/gpu_context.cc` — GPUContext
/// (thread-local active ctx, state, matrix,
/// debug groups).
/// `GPUContext` (verbatim — global state).
pub struct GpuContext {
    /// `state`.
    pub state: super::gpu_state::GpuState,
    /// `state_mutable`.
    pub state_mutable: super::gpu_state::GpuStateMutable,
    /// `matrix` stack.
    pub matrix: super::gpu_matrix::GpuMatrixStack,
    /// `active` flag.
    pub active: bool,
}
