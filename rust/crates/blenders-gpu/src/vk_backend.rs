/// Port of `vulkan/vk_backend.cc` — VK backend
/// `backend`.
/// `VKBackend` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkBackend {
    /// `vk_` object handle.
    pub handle: u64,
}
