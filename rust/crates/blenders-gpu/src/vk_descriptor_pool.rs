/// Port of `vulkan/vk_descriptor_pool.cc` — VK backend
/// `descriptor_pool`.
/// `VKDescriptorPool` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDescriptorPool {
    /// `vk_` object handle.
    pub handle: u64,
}
