/// Port of `vulkan/vk_render_target.cc` — VK backend
/// `render_target`.
/// `VKRenderTarget` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkRenderTarget {
    /// `vk_` object handle.
    pub handle: u64,
}
