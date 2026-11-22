/// Port of `vulkan/vk_resource_tracker.cc` — VK backend
/// `resource_tracker`.
/// `VKResourceTracker` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkResourceTracker {
    /// `vk_` object handle.
    pub handle: u64,
}
