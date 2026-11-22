/// Port of `vulkan/vk_image_view.cc` — VK backend
/// `image_view`.
/// `VKImageView` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkImageView {
    /// `vk_` object handle.
    pub handle: u64,
}
