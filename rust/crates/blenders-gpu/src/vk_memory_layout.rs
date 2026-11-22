/// Port of `vulkan/vk_memory_layout.cc` — VK backend
/// `memory_layout`.
/// `VKMemoryLayout` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkMemoryLayout {
    /// `vk_` object handle.
    pub handle: u64,
}
