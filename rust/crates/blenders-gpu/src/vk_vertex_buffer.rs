/// Port of `vulkan/vk_vertex_buffer.cc` — VK backend
/// `vertex_buffer`.
/// `VKVertexBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkVertexBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
