//! Port of `DNA_defs.h` — DNA plumbing macros as Rust helpers.

/// `MAX_NAME` — max length of an ID name buffer (with NUL in C).
pub const MAX_NAME: usize = 64;

/// `_DNA_internal_memcpy` — raw shallow copy used by `DNA_DEFINE_CXX_METHODS`.
/// In Rust the shallow-copy constructors become `Clone`.
pub fn dna_internal_memcpy<T: Clone>(dst: &mut T, src: &T) {
    *dst = src.clone();
}

/// `_DNA_internal_memzero` — used by `ShallowZeroInitializeTag`.
/// Rust uses `Default`/zeroed construction instead.
pub fn dna_internal_memzero<T: Default>(dst: &mut T) {
    *dst = T::default();
}

/// `_DNA_internal_swap`.
pub fn dna_internal_swap<T: Clone>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}

/// `dna::shallow_copy` — shallow (non-managed) copy semantics.
/// Rust `Clone` performs the equivalent shallow field copy for our POD ports.
pub fn shallow_copy<T: Clone>(v: &T) -> T {
    v.clone()
}

/// `dna::shallow_zero_initialize`.
pub fn shallow_zero_initialize<T: Default>() -> T {
    T::default()
}

/// `dna::shallow_swap`.
pub fn shallow_swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}
