//! Port of `BLI_ghash_utils.cc` — hash/cmp helpers for `GHash`/`GSet`.
//! `cmp` fns return "keys differ" (like memcmp semantics) to mirror the C API.

use crate::hash_mm2a::hash_mm2;

/// `BLI_ghashutil_ptrhash` — Python3.7-style pointer hash.
pub fn ghashutil_ptrhash(key: usize) -> u32 {
    let y = key;
    ((y >> 4) as u32) | ((y as u32) << (32 - 4))
}

/// `BLI_ghashutil_ptrcmp`.
pub fn ghashutil_ptrcmp(a: usize, b: usize) -> bool {
    a != b
}

/// `BLI_ghashutil_uinthash_v4` — one-at-a-time hash of `[u32; 4]`.
pub fn ghashutil_uinthash_v4(key: &[u32; 4]) -> u32 {
    let mut hash: u32 = 0;
    for &k in key {
        hash = hash.wrapping_add(k);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash
}

/// `BLI_ghashutil_uinthash_v4_murmur`.
pub fn ghashutil_uinthash_v4_murmur(key: &[u32; 4]) -> u32 {
    let mut bytes = [0u8; 16];
    for (i, k) in key.iter().enumerate() {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&k.to_le_bytes());
    }
    hash_mm2(&bytes, 0)
}

/// `BLI_ghashutil_uinthash_v4_cmp`.
pub fn ghashutil_uinthash_v4_cmp(a: &[u32; 4], b: &[u32; 4]) -> bool {
    a != b
}

/// `BLI_ghashutil_uinthash` — integer avalanche (Knuth/inthash.c).
pub fn ghashutil_uinthash(key: u32) -> u32 {
    let mut key = key;
    key += !(key << 16);
    key ^= key >> 5;
    key = key.wrapping_add(key << 3);
    key ^= key >> 13;
    key += !(key << 2);
    key ^= key >> 16;
    key
}

/// `BLI_ghashutil_inthash_p` — pointer-as-int hash.
pub fn ghashutil_inthash_p(ptr: usize) -> u32 {
    let mut key = ptr;
    key += !(key << 16);
    key ^= key >> 5;
    key = key.wrapping_add(key << 3);
    key ^= key >> 13;
    key += !(key << 2);
    key ^= key >> 16;
    key as u32
}

/// `BLI_ghashutil_inthash_p_murmur`.
pub fn ghashutil_inthash_p_murmur(ptr: usize) -> u32 {
    hash_mm2(&ptr.to_le_bytes(), 0)
}

/// `BLI_ghashutil_inthash_p_simple`.
pub fn ghashutil_inthash_p_simple(ptr: usize) -> u32 {
    (ptr & 0xffffffff) as u32
}

/// `BLI_ghashutil_intcmp`.
pub fn ghashutil_intcmp(a: usize, b: usize) -> bool {
    a != b
}

/// `BLI_ghashutil_combine_hash` — Boost hash_combine.
pub fn ghashutil_combine_hash(hash_a: usize, hash_b: usize) -> usize {
    let mut h = hash_a;
    h ^= hash_b
        .wrapping_add(0x9e3779b9)
        .wrapping_add(h << 6)
        .wrapping_add(h >> 2);
    h
}

/// `BLI_ghashutil_strhash_n` — DJB2 over n bytes (stops at NUL).
pub fn ghashutil_strhash_n(key: &[u8]) -> u32 {
    let mut h: u32 = 5381;
    for &p in key {
        if p == 0 {
            break;
        }
        h = ((h << 5).wrapping_add(h)).wrapping_add(p as u32);
    }
    h
}

/// `BLI_ghashutil_strhash_p`.
pub fn ghashutil_strhash_p(key: &str) -> u32 {
    ghashutil_strhash_n(key.as_bytes())
}

/// `BLI_ghashutil_strhash_p_murmur` — murmur over key incl. trailing NUL.
pub fn ghashutil_strhash_p_murmur(key: &str) -> u32 {
    let mut b = key.as_bytes().to_vec();
    b.push(0);
    hash_mm2(&b, 0)
}

/// `BLI_ghashutil_strcmp` — true when strings differ.
pub fn ghashutil_strcmp(a: &str, b: &str) -> bool {
    a != b
}

/// `GHashPair` — key pair, used as a hash key itself.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GHashPair {
    pub first: usize,
    pub second: usize,
}

/// `BLI_ghashutil_pairhash`.
pub fn ghashutil_pairhash(pair: &GHashPair) -> u32 {
    ghashutil_ptrhash(pair.first) ^ ghashutil_ptrhash(pair.second)
}

/// `BLI_ghashutil_paircmp`.
pub fn ghashutil_paircmp(a: &GHashPair, b: &GHashPair) -> bool {
    a.first != b.first || a.second != b.second
}
