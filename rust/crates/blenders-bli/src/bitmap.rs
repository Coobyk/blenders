//! Port of `bitmap.cc` + `BLI_bitmap.hh` — `BLI_bitmap` (u32 blocks).

/// `BLI_bitmap` element type.
pub type BliBitmap = u32;

/// `_BITMAP_POWER` (log2 of bits per block).
pub const BITMAP_POWER: usize = 5;
/// `_BITMAP_MASK`.
pub const BITMAP_MASK: usize = (1 << BITMAP_POWER) - 1;
/// `_BITMAP_NUM_BLOCKS(num)`.
pub const fn bitmap_num_blocks(num: usize) -> usize {
    (num + BITMAP_MASK) >> BITMAP_POWER
}
/// `BLI_BITMAP_SIZE(num)` — bytes.
pub const fn bitmap_size(num: usize) -> usize {
    bitmap_num_blocks(num) * std::mem::size_of::<BliBitmap>()
}
/// `BLI_BITMAP_NEW` — zeroed bitmap with `bits` capacity.
pub fn bitmap_new(bits: usize) -> Vec<BliBitmap> {
    vec![0; bitmap_num_blocks(bits)]
}
/// `BLI_BITMAP_GET(bitmap, index)`.
#[inline]
pub fn bitmap_get(bitmap: &[BliBitmap], index: usize) -> bool {
    bitmap[index >> BITMAP_POWER] & (1 << (index & BITMAP_MASK)) != 0
}
/// `BLI_BITMAP_SET(bitmap, index)` (set to 1).
#[inline]
pub fn bitmap_set(bitmap: &mut [BliBitmap], index: usize) {
    bitmap[index >> BITMAP_POWER] |= 1 << (index & BITMAP_MASK);
}
/// `BLI_BITMAP_CLEAR(bitmap, index)`.
#[inline]
pub fn bitmap_clear(bitmap: &mut [BliBitmap], index: usize) {
    bitmap[index >> BITMAP_POWER] &= !(1 << (index & BITMAP_MASK));
}
/// `BLI_BITMAP_ENABLE` / `BLI_BITMAP_DISABLE` aliases used in C.
pub use bitmap_set as bitmap_enable;
pub use bitmap_clear as bitmap_disable;
/// `BLI_BITMAP_FLIP`.
#[inline]
pub fn bitmap_flip(bitmap: &mut [BliBitmap], index: usize) {
    bitmap[index >> BITMAP_POWER] ^= 1 << (index & BITMAP_MASK);
}
/// `BLI_BITMAP_SET_BOOL(bitmap, index, set)`.
#[inline]
pub fn bitmap_set_bool(bitmap: &mut [BliBitmap], index: usize, set: bool) {
    if set {
        bitmap_set(bitmap, index);
    } else {
        bitmap_clear(bitmap, index);
    }
}
/// `BLI_BITMAP_TEST_AND_SET` — returns previous value then sets.
#[inline]
pub fn bitmap_test_and_set(bitmap: &mut [BliBitmap], index: usize) -> bool {
    let prev = bitmap_get(bitmap, index);
    bitmap_set(bitmap, index);
    prev
}
/// `BLI_bitmap_set_all`.
pub fn bitmap_set_all(bitmap: &mut [BliBitmap], set: bool, _bits: usize) {
    bitmap.fill(if set { u32::MAX } else { 0 });
}
/// `BLI_bitmap_flip_all`.
pub fn bitmap_flip_all(bitmap: &mut [BliBitmap], bits: usize) {
    let n = bitmap_num_blocks(bits).min(bitmap.len());
    for b in &mut bitmap[..n] {
        *b = !*b;
    }
}
/// `BLI_bitmap_copy_all`.
pub fn bitmap_copy_all(dst: &mut [BliBitmap], src: &[BliBitmap], bits: usize) {
    let n = bitmap_num_blocks(bits).min(dst.len()).min(src.len());
    dst[..n].copy_from_slice(&src[..n]);
}
/// `BLI_bitmap_and_all`.
pub fn bitmap_and_all(dst: &mut [BliBitmap], src: &[BliBitmap], bits: usize) {
    let n = bitmap_num_blocks(bits).min(dst.len()).min(src.len());
    for i in 0..n {
        dst[i] &= src[i];
    }
}
/// `BLI_bitmap_or_all`.
pub fn bitmap_or_all(dst: &mut [BliBitmap], src: &[BliBitmap], bits: usize) {
    let n = bitmap_num_blocks(bits).min(dst.len()).min(src.len());
    for i in 0..n {
        dst[i] |= src[i];
    }
}
/// `BLI_bitmap_find_first_unset` — index of first 0 bit, -1 when all set.
pub fn bitmap_find_first_unset(bitmap: &[BliBitmap], bits: usize) -> i32 {
    let blocks = bitmap_num_blocks(bits);
    let mut index = 0;
    while index < blocks && index < bitmap.len() && bitmap[index] == !0 {
        index += 1;
    }
    if index < blocks && index < bitmap.len() {
        let m = !bitmap[index];
        debug_assert!(m != 0);
        let bit_index = m.trailing_zeros() as i32;
        bit_index + ((index as i32) << BITMAP_POWER)
    } else {
        -1
    }
}
