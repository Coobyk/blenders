//! Port of `structure_of_array_conversion.cc` — AoS↔SoA byte-level
//! transposes (`aos_to_soa_3x4byte`, `soa_to_aos_4x4byte` etc.).
//! PORT_NOTE: x86 asm SSE2/AVX2 paths → scalar loops (identical results).

/// `aos_to_soa_3x4byte` — deinterleave [u32;3] elements.
pub fn aos_to_soa_3x4byte(src: &[u32], a: &mut [u32], b: &mut [u32], c: &mut [u32]) {
    for i in 0..a.len() {
        a[i] = src[i * 3];
        b[i] = src[i * 3 + 1];
        c[i] = src[i * 3 + 2];
    }
}
/// `soa_to_aos_3x4byte`.
pub fn soa_to_aos_3x4byte(a: &[u32], b: &[u32], c: &[u32], dst: &mut [u32]) {
    for i in 0..a.len() {
        dst[i * 3] = a[i];
        dst[i * 3 + 1] = b[i];
        dst[i * 3 + 2] = c[i];
    }
}
/// `aos_to_soa_4x4byte`.
pub fn aos_to_soa_4x4byte(src: &[u32], a: &mut [u32], b: &mut [u32], c: &mut [u32], d: &mut [u32]) {
    for i in 0..a.len() {
        a[i] = src[i * 4];
        b[i] = src[i * 4 + 1];
        c[i] = src[i * 4 + 2];
        d[i] = src[i * 4 + 3];
    }
}
/// `soa_to_aos_4x4byte`.
pub fn soa_to_aos_4x4byte(a: &[u32], b: &[u32], c: &[u32], d: &[u32], dst: &mut [u32]) {
    for i in 0..a.len() {
        dst[i * 4] = a[i];
        dst[i * 4 + 1] = b[i];
        dst[i * 4 + 2] = c[i];
        dst[i * 4 + 3] = d[i];
    }
}
