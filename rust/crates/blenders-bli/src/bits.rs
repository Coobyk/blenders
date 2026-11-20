//! Port of `bit_bool_conversion.cc` + `bit_ref.cc`.
//! Scalar implementation (SSE2 path is an optimization, same result).
//! Minimal `BitRef`/`BitSpan`/`MutableBitSpan` from `BLI_bit_span.hh` —
//! enough to host the ported functions; grows as more bit code lands.

pub const BITS_PER_INT: usize = 64;

fn int_containing_bit(data: &mut [u64], bit_index: usize) -> &mut u64 {
    &mut data[bit_index / BITS_PER_INT]
}

/// `BitRef` — read-only reference to a single bit.
#[derive(Clone, Copy)]
pub struct BitRef {
    data: u64,
    index: usize,
}

impl BitRef {
    pub fn test(self) -> bool {
        (self.data >> self.index) & 1 != 0
    }
}

impl std::fmt::Display for BitRef {
    /// `std::ostream &operator<<(std::ostream &, const BitRef &)`.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(if self.test() { "1" } else { "0" })
    }
}

/// `MutableBitRef` — read/write reference to a single bit.
pub struct MutableBitRef<'a> {
    int: &'a mut u64,
    index: usize,
}

impl MutableBitRef<'_> {
    /// `MutableBitRef::set`.
    pub fn set(&mut self) {
        *self.int |= 1 << self.index;
    }
    /// `MutableBitRef::reset`.
    pub fn reset(&mut self) {
        *self.int &= !(1 << self.index);
    }
    /// `MutableBitRef::operator bool`.
    pub fn test(&self) -> bool {
        (*self.int >> self.index) & 1 != 0
    }
}

/// `MutableBitSpan` — contiguous bit range inside an int slice.
pub struct MutableBitSpan<'a> {
    pub data: &'a mut [u64],
    /// `bit_range().start()` — first bit's index within `data`.
    pub start: usize,
    pub size: usize,
}

impl MutableBitSpan<'_> {
    /// `MutableBitSpan::operator[]` -> `MutableBitRef`.
    pub fn bit(&mut self, i: usize) -> MutableBitRef<'_> {
        let idx = self.start + i;
        let int = int_containing_bit(self.data, idx);
        MutableBitRef {
            int,
            index: idx % BITS_PER_INT,
        }
    }
}

/// `or_bytes_into_bits` — set bit i when `byte_to_bit(b[i])`; returns any_true.
pub fn or_bytes_into_bits<F: Fn(u8) -> bool>(
    bytes: &[u8],
    r_bits: &mut MutableBitSpan,
    byte_to_bit: F,
) -> bool {
    debug_assert!(r_bits.size >= bytes.len());
    let mut any_true = false;
    for (i, &b) in bytes.iter().enumerate() {
        if byte_to_bit(b) {
            r_bits.bit(i).set();
            any_true = true;
        }
    }
    any_true
}

/// `or_bools_into_bits` (`BoolToBit` policy: `single(c) = c != 0`).
pub fn or_bools_into_bits(bools: &[bool], r_bits: &mut MutableBitSpan) -> bool {
    or_bytes_into_bits(
        // bools are 0/1 bytes; transmute via byte equality
        unsafe { std::slice::from_raw_parts(bools.as_ptr() as *const u8, bools.len()) },
        r_bits,
        |b| b != 0,
    )
}
