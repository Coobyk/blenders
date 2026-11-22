/// Port of `vulkan/vk_scheduler.cc` — VK backend
/// `scheduler`.
/// `VKScheduler` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkScheduler {
    /// `vk_` object handle.
    pub handle: u64,
}
