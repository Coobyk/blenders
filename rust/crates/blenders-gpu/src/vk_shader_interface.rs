/// Port of `vulkan/vk_shader_interface.cc` — VK backend
/// `shader_interface`.
/// `VKShaderInterface` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkShaderInterface {
    /// `vk_` object handle.
    pub handle: u64,
}
