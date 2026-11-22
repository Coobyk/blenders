/// Port of `vulkan/vk_vertex_attribute_object_cache.cc` — VK backend
/// `vertex_attribute_object_cache`.
/// `VKVertexAttributeObjectCache` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkVertexAttributeObjectCache {
    /// `vk_` object handle.
    pub handle: u64,
}
