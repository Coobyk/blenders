/// Port of `vulkan/vk_render_graph.cc` — VK backend
/// `render_graph`.
/// `VKRenderGraph` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkRenderGraph {
    /// `vk_` object handle.
    pub handle: u64,
}
