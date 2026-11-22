/// Port of `vulkan/vk_common.cc` — VK backend
/// `common`.
/// `VKCommon` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkCommon {
    /// `vk_` object handle.
    pub handle: u64,
}
