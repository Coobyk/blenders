/// Port of `spz/intern/spz_read.cc` — SPZ
/// decompression to f32 attribute arrays.

/// `spz_unpack` (verbatim — fixed-point→float
/// per channel).
pub fn spz_unpack(raw: &[i32], fractional_bits: u8, scale: f32) -> Vec<f32> {
    let div = (1 << fractional_bits) as f32;
    raw.iter().map(|&v| v as f32 / div * scale).collect()
}
