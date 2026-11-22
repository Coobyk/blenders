/// Port of `vulkan/vk_vertex_input_state.cc` — VK backend
/// `vertex_input_state`.
/// `VKVertexInputState` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkVertexInputState {
    /// `vk_` object handle.
    pub handle: u64,
}
