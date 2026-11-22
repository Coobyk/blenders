/// Port of `vulkan/vk_staging_buffer.cc` — VK backend
/// `staging_buffer`.
/// `VKStagingBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkStagingBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
