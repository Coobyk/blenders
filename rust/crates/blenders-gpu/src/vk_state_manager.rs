/// Port of `vulkan/vk_state_manager.cc` — VK backend
/// `state_manager`.
/// `VKStateManager` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkStateManager {
    /// `vk_` object handle.
    pub handle: u64,
}
