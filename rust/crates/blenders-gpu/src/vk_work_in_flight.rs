/// Port of `vulkan/vk_work_in_flight.cc` — VK backend
/// `work_in_flight`.
/// `VKWorkInFlight` (verbatim — Vk
/// object handle + state).
#[derive(Default)]
pub struct VkWorkInFlight {
    /// `vk_` object handle.
    pub handle: u64,
}
