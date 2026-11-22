/// Port of `vulkan/vk_data_conversion.cc` — VK backend
/// `data_conversion`.
/// `VKDataConversion` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkDataConversion {
    /// `vk_` object handle.
    pub handle: u64,
}
