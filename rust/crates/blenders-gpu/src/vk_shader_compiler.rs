/// Port of `vulkan/vk_shader_compiler.cc` — VK backend
/// `shader_compiler`.
/// `VKShaderCompiler` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkShaderCompiler {
    /// `vk_` object handle.
    pub handle: u64,
}
