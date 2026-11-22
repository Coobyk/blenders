/// Port of `vulkan/vk_streaming_buffer.cc` — VK backend
/// `streaming_buffer`.
/// `VKStreamingBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkStreamingBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
