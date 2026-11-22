/// Port of `vulkan/vk_shader.cc` — VK backend
/// `shader`.
/// `VKShader` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkShader {
    /// `vk_` object handle.
    pub handle: u64,
}
