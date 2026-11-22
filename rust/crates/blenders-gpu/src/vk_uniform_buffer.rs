/// Port of `vulkan/vk_uniform_buffer.cc` — VK backend
/// `uniform_buffer`.
/// `VKUniformBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkUniformBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
