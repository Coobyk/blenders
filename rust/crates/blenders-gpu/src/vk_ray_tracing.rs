/// Port of `vulkan/vk_ray_tracing.cc` — VK backend
/// `ray_tracing`.
/// `VKRayTracing` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkRayTracing {
    /// `vk_` object handle.
    pub handle: u64,
}
