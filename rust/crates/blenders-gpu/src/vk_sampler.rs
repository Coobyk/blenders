/// Port of `vulkan/vk_sampler.cc` — VK backend
/// `sampler`.
/// `VKSampler` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkSampler {
    /// `vk_` object handle.
    pub handle: u64,
}
