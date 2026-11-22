/// Port of `vulkan/vk_context.cc` — VK backend
/// `context`.
/// `VKContext` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkContext {
    /// `vk_` object handle.
    pub handle: u64,
}
