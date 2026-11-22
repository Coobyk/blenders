/// Port of `vulkan/vk_to_string.cc` — VK backend
/// `to_string`.
/// `VKToString` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkToString {
    /// `vk_` object handle.
    pub handle: u64,
}
