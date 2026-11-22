/// Port of `vulkan/vk_debug.cc` — VK backend
/// `debug`.
/// `VKDebug` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDebug {
    /// `vk_` object handle.
    pub handle: u64,
}
