/// Port of `vulkan/vk_pixel_buffer.cc` — VK backend
/// `pixel_buffer`.
/// `VKPixelBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkPixelBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
