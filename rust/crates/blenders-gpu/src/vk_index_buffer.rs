/// Port of `vulkan/vk_index_buffer.cc` — VK backend
/// `index_buffer`.
/// `VKIndexBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkIndexBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
