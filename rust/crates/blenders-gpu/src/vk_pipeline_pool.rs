/// Port of `vulkan/vk_pipeline_pool.cc` — VK backend
/// `pipeline_pool`.
/// `VKPipelinePool` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkPipelinePool {
    /// `vk_` object handle.
    pub handle: u64,
}
