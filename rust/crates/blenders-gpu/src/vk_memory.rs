/// Port of `vulkan/vk_memory.cc` — VK backend
/// `memory`.
/// `VKMemory` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkMemory {
    /// `vk_` object handle.
    pub handle: u64,
}
