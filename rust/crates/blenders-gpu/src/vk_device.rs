/// Port of `vulkan/vk_device.cc` — VK backend
/// `device`.
/// `VKDevice` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDevice {
    /// `vk_` object handle.
    pub handle: u64,
}
