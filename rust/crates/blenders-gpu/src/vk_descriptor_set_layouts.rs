/// Port of `vulkan/vk_descriptor_set_layouts.cc` — VK backend
/// `descriptor_set_layouts`.
/// `VKDescriptorSetLayouts` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDescriptorSetLayouts {
    /// `vk_` object handle.
    pub handle: u64,
}
