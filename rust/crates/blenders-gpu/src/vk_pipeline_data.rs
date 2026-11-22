/// Port of `vulkan/vk_pipeline_data.cc` — VK backend
/// `pipeline_data`.
/// `VKPipelineData` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkPipelineData {
    /// `vk_` object handle.
    pub handle: u64,
}
