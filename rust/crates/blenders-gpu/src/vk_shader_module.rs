/// Port of `vulkan/vk_shader_module.cc` — VK backend
/// `shader_module`.
/// `VKShaderModule` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkShaderModule {
    /// `vk_` object handle.
    pub handle: u64,
}
