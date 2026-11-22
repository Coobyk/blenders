/// Port of `vulkan/vk_device_submission.cc` — VK backend
/// `device_submission`.
/// `VKDeviceSubmission` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDeviceSubmission {
    /// `vk_` object handle.
    pub handle: u64,
}
