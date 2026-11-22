/// Port of `vulkan/vk_resource.cc` — VK backend
/// `resource`.
/// `VKResource` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkResource {
    /// `vk_` object handle.
    pub handle: u64,
}
