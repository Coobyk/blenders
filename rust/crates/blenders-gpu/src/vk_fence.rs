/// Port of `vulkan/vk_fence.cc` — VK backend
/// `fence`.
/// `VKFence` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkFence {
    /// `vk_` object handle.
    pub handle: u64,
}
