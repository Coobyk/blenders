/// Port of `vulkan/vk_batch.cc` — VK backend
/// `batch`.
/// `VKBatch` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkBatch {
    /// `vk_` object handle.
    pub handle: u64,
}
