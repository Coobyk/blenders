/// Port of `vulkan/vk_framebuffer.cc` — VK backend
/// `framebuffer`.
/// `VKFramebuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkFramebuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
