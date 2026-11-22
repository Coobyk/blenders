/// Port of `vulkan/vk_push_constants.cc` — VK backend
/// `push_constants`.
/// `VKPushConstants` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkPushConstants {
    /// `vk_` object handle.
    pub handle: u64,
}
