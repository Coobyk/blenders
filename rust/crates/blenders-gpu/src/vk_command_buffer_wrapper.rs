/// Port of `vulkan/vk_command_buffer_wrapper.cc` — VK backend
/// `command_buffer_wrapper`.
/// `VKCommandBufferWrapper` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkCommandBufferWrapper {
    /// `vk_` object handle.
    pub handle: u64,
}
