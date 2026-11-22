/// Port of `vulkan/vk_texture.cc` — VK backend
/// `texture`.
/// `VKTexture` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkTexture {
    /// `vk_` object handle.
    pub handle: u64,
}
