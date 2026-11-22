/// Port of `vulkan/vk_resource_state_tracker.cc` — VK backend
/// `resource_state_tracker`.
/// `VKResourceStateTracker` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkResourceStateTracker {
    /// `vk_` object handle.
    pub handle: u64,
}
