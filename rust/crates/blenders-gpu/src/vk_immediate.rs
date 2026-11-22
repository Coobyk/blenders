/// Port of `vulkan/vk_immediate.cc` — VK backend
/// `immediate`.
/// `VKImmediate` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkImmediate {
    /// `vk_` object handle.
    pub handle: u64,
}
