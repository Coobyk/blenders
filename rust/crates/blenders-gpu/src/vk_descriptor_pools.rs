/// Port of `vulkan/vk_descriptor_pools.cc` — VK backend
/// `descriptor_pools`.
/// `VKDescriptorPools` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDescriptorPools {
    /// `vk_` object handle.
    pub handle: u64,
}
