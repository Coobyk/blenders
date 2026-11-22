/// Port of `vulkan/vk_pipeline.cc` — VK backend
/// `pipeline`.
/// `VKPipeline` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkPipeline {
    /// `vk_` object handle.
    pub handle: u64,
}
