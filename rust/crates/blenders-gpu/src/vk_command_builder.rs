/// Port of `vulkan/vk_command_builder.cc` — VK backend
/// `command_builder`.
/// `VKCommandBuilder` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkCommandBuilder {
    /// `vk_` object handle.
    pub handle: u64,
}
