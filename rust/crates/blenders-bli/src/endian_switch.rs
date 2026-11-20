//! Port of `endian_switch.cc` — `BLI_endian_switch_*_array` in-place
//! byte-order reversal of whole arrays.

/// `BLI_endian_switch_int16_array`.
pub fn endian_switch_i16_array(val: &mut [i16]) {
    for v in val.iter_mut() {
        *v = v.swap_bytes();
    }
}
/// `BLI_endian_switch_uint16_array`.
pub fn endian_switch_u16_array(val: &mut [u16]) {
    for v in val.iter_mut() {
        *v = v.swap_bytes();
    }
}
/// `BLI_endian_switch_int32_array`.
pub fn endian_switch_i32_array(val: &mut [i32]) {
    for v in val.iter_mut() {
        *v = v.swap_bytes();
    }
}
/// `BLI_endian_switch_uint32_array`.
pub fn endian_switch_u32_array(val: &mut [u32]) {
    for v in val.iter_mut() {
        *v = v.swap_bytes();
    }
}
/// `BLI_endian_switch_float_array`.
pub fn endian_switch_f32_array(val: &mut [f32]) {
    for v in val.iter_mut() {
        *v = f32::from_bits(v.to_bits().swap_bytes());
    }
}
/// `BLI_endian_switch_int64_array`.
pub fn endian_switch_i64_array(val: &mut [i64]) {
    for v in val.iter_mut() {
        *v = v.swap_bytes();
    }
}
/// `BLI_endian_switch_uint64_array`.
pub fn endian_switch_u64_array(val: &mut [u64]) {
    for v in val.iter_mut() {
        *v = v.swap_bytes();
    }
}
/// `BLI_endian_switch_double_array`.
pub fn endian_switch_f64_array(val: &mut [f64]) {
    for v in val.iter_mut() {
        *v = f64::from_bits(v.to_bits().swap_bytes());
    }
}
