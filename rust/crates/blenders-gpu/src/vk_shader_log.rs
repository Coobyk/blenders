/// Port of `vulkan/vk_shader_log.cc` — VK backend
/// `shader_log`.
/// `VKShaderLog` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkShaderLog {
    /// `vk_` object handle.
    pub handle: u64,
}
