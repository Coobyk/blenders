//! Port of `endian_switch.cc` — byte-swap helpers for arrays.
//! Scalar `BLI_endian_switch_*` single-value fns live in `BLI_endian_switch.hh`;
//! they map to `swap_bytes`.

/// `BLI_endian_switch_int16`.
pub fn endian_switch_int16(v: i16) -> i16 {
    v.swap_bytes()
}
/// `BLI_endian_switch_uint16`.
pub fn endian_switch_uint16(v: u16) -> u16 {
    v.swap_bytes()
}
/// `BLI_endian_switch_int32`.
pub fn endian_switch_int32(v: i32) -> i32 {
    v.swap_bytes()
}
/// `BLI_endian_switch_uint32`.
pub fn endian_switch_uint32(v: u32) -> u32 {
    v.swap_bytes()
}
/// `BLI_endian_switch_float` — swaps bytes of the f32 bit pattern.
pub fn endian_switch_float(v: f32) -> f32 {
    f32::from_bits(v.to_bits().swap_bytes())
}
/// `BLI_endian_switch_int64`.
pub fn endian_switch_int64(v: i64) -> i64 {
    v.swap_bytes()
}
/// `BLI_endian_switch_uint64`.
pub fn endian_switch_uint64(v: u64) -> u64 {
    v.swap_bytes()
}
/// `BLI_endian_switch_double`.
pub fn endian_switch_double(v: f64) -> f64 {
    f64::from_bits(v.to_bits().swap_bytes())
}

/// `BLI_endian_switch_int16_array`.
pub fn endian_switch_int16_array(v: &mut [i16]) {
    v.iter_mut().for_each(|x| *x = endian_switch_int16(*x));
}
/// `BLI_endian_switch_uint16_array`.
pub fn endian_switch_uint16_array(v: &mut [u16]) {
    v.iter_mut().for_each(|x| *x = endian_switch_uint16(*x));
}
/// `BLI_endian_switch_int32_array`.
pub fn endian_switch_int32_array(v: &mut [i32]) {
    v.iter_mut().for_each(|x| *x = endian_switch_int32(*x));
}
/// `BLI_endian_switch_uint32_array`.
pub fn endian_switch_uint32_array(v: &mut [u32]) {
    v.iter_mut().for_each(|x| *x = endian_switch_uint32(*x));
}
/// `BLI_endian_switch_float_array`.
pub fn endian_switch_float_array(v: &mut [f32]) {
    v.iter_mut().for_each(|x| *x = endian_switch_float(*x));
}
/// `BLI_endian_switch_int64_array`.
pub fn endian_switch_int64_array(v: &mut [i64]) {
    v.iter_mut().for_each(|x| *x = endian_switch_int64(*x));
}
/// `BLI_endian_switch_uint64_array`.
pub fn endian_switch_uint64_array(v: &mut [u64]) {
    v.iter_mut().for_each(|x| *x = endian_switch_uint64(*x));
}
/// `BLI_endian_switch_double_array`.
pub fn endian_switch_double_array(v: &mut [f64]) {
    v.iter_mut().for_each(|x| *x = endian_switch_double(*x));
}
