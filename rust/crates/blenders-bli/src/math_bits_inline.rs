//! Port of `math_bits_inline.cc` — bit-scan, highest-order-bit, bit-count
//! and float/int reinterpret helpers (`blender::`).

/// `bitscan_forward_uint`.
#[inline]
pub fn bitscan_forward_uint(a: u32) -> u32 {
    debug_assert!(a != 0);
    a.trailing_zeros()
}
/// `bitscan_forward_uint64`.
#[inline]
pub fn bitscan_forward_uint64(a: u64) -> u32 {
    debug_assert!(a != 0);
    a.trailing_zeros()
}
/// `bitscan_forward_i`.
#[inline]
pub fn bitscan_forward_i(a: i32) -> i32 {
    bitscan_forward_uint(a as u32) as i32
}
/// `bitscan_forward_clear_uint`.
#[inline]
pub fn bitscan_forward_clear_uint(a: &mut u32) -> u32 {
    let i = bitscan_forward_uint(*a);
    *a &= *a - 1;
    i
}
/// `bitscan_forward_clear_uint64`.
#[inline]
pub fn bitscan_forward_clear_uint64(a: &mut u64) -> u32 {
    let i = bitscan_forward_uint64(*a);
    *a &= *a - 1;
    i
}
/// `bitscan_forward_clear_i`.
#[inline]
pub fn bitscan_forward_clear_i(a: &mut i32) -> i32 {
    let mut u = *a as u32;
    let i = bitscan_forward_clear_uint(&mut u);
    *a = u as i32;
    i as i32
}
/// `bitscan_reverse_uint` — index of the highest set bit.
#[inline]
pub fn bitscan_reverse_uint(a: u32) -> u32 {
    debug_assert!(a != 0);
    31 - a.leading_zeros()
}
/// `bitscan_reverse_uint64`.
#[inline]
pub fn bitscan_reverse_uint64(a: u64) -> u32 {
    debug_assert!(a != 0);
    63 - a.leading_zeros()
}
/// `bitscan_reverse_i`.
#[inline]
pub fn bitscan_reverse_i(a: i32) -> i32 {
    bitscan_reverse_uint(a as u32) as i32
}
/// `bitscan_reverse_clear_uint`.
#[inline]
pub fn bitscan_reverse_clear_uint(a: &mut u32) -> u32 {
    let i = bitscan_reverse_uint(*a);
    *a &= !(1 << i);
    i
}
/// `bitscan_reverse_clear_i`.
#[inline]
pub fn bitscan_reverse_clear_i(a: &mut i32) -> i32 {
    let mut u = *a as u32;
    let i = bitscan_reverse_clear_uint(&mut u);
    *a = u as i32;
    i as i32
}
/// `highest_order_bit_uint`.
#[inline]
pub fn highest_order_bit_uint(n: u32) -> u32 {
    if n == 0 {
        0
    } else {
        1 << bitscan_reverse_uint(n)
    }
}
/// `highest_order_bit_s`.
#[inline]
pub fn highest_order_bit_s(mut n: u16) -> u16 {
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n - (n >> 1)
}
/// `count_bits_i`.
#[inline]
pub fn count_bits_i(mut i: u32) -> i32 {
    i = i - ((i >> 1) & 0x55555555);
    i = (i & 0x33333333) + ((i >> 2) & 0x33333333);
    (((i + (i >> 4)) & 0x0F0F0F0F).wrapping_mul(0x01010101) >> 24) as i32
}
/// `count_bits_uint64`.
#[inline]
pub fn count_bits_uint64(a: u64) -> i32 {
    count_bits_i(a as u32) + count_bits_i((a >> 32) as u32)
}
/// `float_as_int`.
#[inline]
pub fn float_as_int(f: f32) -> i32 {
    f.to_bits() as i32
}
/// `float_as_uint`.
#[inline]
pub fn float_as_uint(f: f32) -> u32 {
    f.to_bits()
}
/// `int_as_float`.
#[inline]
pub fn int_as_float(i: i32) -> f32 {
    f32::from_bits(i as u32)
}
/// `uint_as_float`.
#[inline]
pub fn uint_as_float(i: u32) -> f32 {
    f32::from_bits(i)
}
