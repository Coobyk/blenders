/// Port of `vulkan/vk_descriptor_set.cc` — VK backend
/// `descriptor_set`.
/// `VKDescriptorSet` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDescriptorSet {
    /// `vk_` object handle.
    pub handle: u64,
}
