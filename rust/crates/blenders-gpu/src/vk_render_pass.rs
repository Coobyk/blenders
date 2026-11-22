/// Port of `vulkan/vk_render_pass.cc` — VK backend
/// `render_pass`.
/// `VKRenderPass` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkRenderPass {
    /// `vk_` object handle.
    pub handle: u64,
}
