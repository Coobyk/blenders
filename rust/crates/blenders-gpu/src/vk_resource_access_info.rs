/// Port of `vulkan/vk_resource_access_info.cc` — VK backend
/// `resource_access_info`.
/// `VKResourceAccessInfo` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkResourceAccessInfo {
    /// `vk_` object handle.
    pub handle: u64,
}
