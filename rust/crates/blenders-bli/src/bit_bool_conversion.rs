//! Port of `bit_bool_conversion.cc` — `blender::bits::or_bools_into_bits`.
//! Implemented in `bits.rs` with the SSE2 path flattened to scalar;
//! re-exported here for file parity.

pub use crate::bits::{or_bools_into_bits, or_bytes_into_bits};
