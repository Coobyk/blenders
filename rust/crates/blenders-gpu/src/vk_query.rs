/// Port of `vulkan/vk_query.cc` — VK backend
/// `query`.
/// `VKQuery` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkQuery {
    /// `vk_` object handle.
    pub handle: u64,
}
