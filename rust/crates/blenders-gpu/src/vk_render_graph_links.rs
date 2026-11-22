/// Port of `vulkan/vk_render_graph_links.cc` — VK backend
/// `render_graph_links`.
/// `VKRenderGraphLinks` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkRenderGraphLinks {
    /// `vk_` object handle.
    pub handle: u64,
}
