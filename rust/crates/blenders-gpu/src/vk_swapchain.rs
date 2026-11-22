/// Port of `vulkan/vk_swapchain.cc` — VK backend
/// `swapchain`.
/// `VKSwapchain` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkSwapchain {
    /// `vk_` object handle.
    pub handle: u64,
}
