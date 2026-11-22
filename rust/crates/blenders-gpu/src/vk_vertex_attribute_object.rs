/// Port of `vulkan/vk_vertex_attribute_object.cc` — VK backend
/// `vertex_attribute_object`.
/// `VKVertexAttributeObject` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkVertexAttributeObject {
    /// `vk_` object handle.
    pub handle: u64,
}
