/// Port of `vulkan/vk_samplers.cc` — VK backend
/// `samplers`.
/// `VKSamplers` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkSamplers {
    /// `vk_` object handle.
    pub handle: u64,
}
