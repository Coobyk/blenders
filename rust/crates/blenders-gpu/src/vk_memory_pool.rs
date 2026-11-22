/// Port of `vulkan/vk_memory_pool.cc` — VK backend
/// `memory_pool`.
/// `VKMemoryPool` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkMemoryPool {
    /// `vk_` object handle.
    pub handle: u64,
}
