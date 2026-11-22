/// Port of `vulkan/vk_storage_buffer.cc` — VK backend
/// `storage_buffer`.
/// `VKStorageBuffer` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkStorageBuffer {
    /// `vk_` object handle.
    pub handle: u64,
}
