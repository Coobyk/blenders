//! Port of `DNA_uuid_types.h` — `bUUID`.

/// `bUUID` — 128-bit UUID in RFC4122 field order.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct BUuid {
    pub time_low: u32,
    pub time_mid: u16,
    pub time_hi_and_version: u16,
    pub clock_seq_hi_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

/// `UUID_STRING_SIZE` — 36 chars + NUL.
pub const UUID_STRING_SIZE: usize = 37;
