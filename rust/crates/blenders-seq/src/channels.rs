//! Port of `intern/channels.cc` — SeqTimelineChannel
//! list management.

/// `MAX_CHANNELS` verbatim.
pub const MAX_CHANNELS: usize = 128;
/// `SeqTimelineChannel` verbatim fields.
#[derive(Clone)]
pub struct SeqTimelineChannel {
    /// `name`.
    pub name: String,
    /// `index`.
    pub index: i32,
    /// `flag`.
    pub flag: i32,
}
/// `channels_ensure` (verbatim — MAX_CHANNELS+1,
/// channel 0 unused).
pub fn channels_ensure() -> Vec<SeqTimelineChannel> {
    let mut channels = Vec::with_capacity(MAX_CHANNELS + 1);
    for i in 0..=MAX_CHANNELS {
        channels.push(SeqTimelineChannel {
            name: format!("Channel {}", i),
            index: i as i32,
            flag: 0,
        });
    }
    channels
}
/// `channels_duplicate` (verbatim — clone each).
pub fn channels_duplicate(src: &[SeqTimelineChannel]) -> Vec<SeqTimelineChannel> {
    src.to_vec()
}
/// `channel_get_by_index` (verbatim — findlink).
pub fn channel_get_by_index(channels: &[SeqTimelineChannel], channel_index: usize) -> Option<&SeqTimelineChannel> {
    channels.get(channel_index)
}
