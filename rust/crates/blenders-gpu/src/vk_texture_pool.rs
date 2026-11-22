/// Port of `vulkan/vk_texture_pool.cc` — VK backend
/// `texture_pool`.
/// `VKTexturePool` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkTexturePool {
    /// `vk_` object handle.
    pub handle: u64,
}
