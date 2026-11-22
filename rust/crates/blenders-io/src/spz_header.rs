/// Port of `spz/intern/spz_header.cc` —
/// gaussian-splat SPZ header.

/// `spz` magic + version (verbatim).
pub const SPZ_MAGIC: u32 = 0x5053474e;
pub const SPZ_VERSION: u32 = 2;
/// `SpzHeader` (verbatim fields).
pub struct SpzHeader {
    /// `num_points`.
    pub num_points: u32,
    /// `sh_degree`.
    pub sh_degree: u8,
    /// `fractional_bits`.
    pub fractional_bits: u8,
    /// `flags` (antialiased etc.).
    pub flags: u8,
}
