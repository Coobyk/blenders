/// Port of `vulkan/vk_resource_pool.cc` — VK backend
/// `resource_pool`.
/// `VKResourcePool` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkResourcePool {
    /// `vk_` object handle.
    pub handle: u64,
}
