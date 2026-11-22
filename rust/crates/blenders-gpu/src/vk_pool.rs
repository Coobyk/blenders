/// Port of `vulkan/vk_pool.cc` — VK backend
/// `pool`.
/// `VKPool` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkPool {
    /// `vk_` object handle.
    pub handle: u64,
}
