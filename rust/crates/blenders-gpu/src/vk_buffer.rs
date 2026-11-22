/// Port of `vulkan/vk_buffer.cc` — VK backend
/// `buffer`.
/// `VKBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
