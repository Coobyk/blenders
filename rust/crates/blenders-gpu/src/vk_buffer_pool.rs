/// Port of `vulkan/vk_buffer_pool.cc` — VK backend
/// `buffer_pool`.
/// `VKBufferPool` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkBufferPool {
    /// `vk_` object handle.
    pub handle: u64,
}
