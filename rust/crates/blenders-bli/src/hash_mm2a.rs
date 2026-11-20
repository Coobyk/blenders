//! Port of `hash_mm2a.cc` — MurmurHash2A (streaming + one-shot).

const MM2A_M: u32 = 0x5bd1e995;

#[inline]
fn mm2a_mix(h: &mut u32, mut k: u32) {
    k = k.wrapping_mul(MM2A_M);
    k ^= k >> 24;
    k = k.wrapping_mul(MM2A_M);
    *h = h.wrapping_mul(MM2A_M) ^ k;
}

#[inline]
fn mm2a_mix_finalize(h: &mut u32) {
    *h ^= *h >> 13;
    *h = h.wrapping_mul(MM2A_M);
    *h ^= *h >> 15;
}

/// `BLI_HashMurmur2A` — streaming state.
#[derive(Default, Clone, Copy)]
pub struct HashMurmur2A {
    pub hash: u32,
    pub tail: u32,
    pub count: u32,
    pub size: u32,
}

impl HashMurmur2A {
    /// `BLI_hash_mm2a_init`.
    pub fn new(seed: u32) -> Self {
        HashMurmur2A {
            hash: seed,
            tail: 0,
            count: 0,
            size: 0,
        }
    }

    fn mix_tail(&mut self, data: &mut &[u8]) {
        while !data.is_empty() && (data.len() < 4 || self.count != 0) {
            self.tail |= (data[0] as u32) << (self.count * 8);
            self.count += 1;
            *data = &data[1..];
            if self.count == 4 {
                let t = self.tail;
                mm2a_mix(&mut self.hash, t);
                self.tail = 0;
                self.count = 0;
            }
        }
    }

    /// `BLI_hash_mm2a_add`.
    pub fn add(&mut self, data: &[u8]) {
        self.size += data.len() as u32;
        let mut d: &[u8] = data;
        self.mix_tail(&mut d);
        while d.len() >= 4 {
            let k = u32::from_le_bytes([d[0], d[1], d[2], d[3]]);
            mm2a_mix(&mut self.hash, k);
            d = &d[4..];
        }
        self.mix_tail(&mut d);
    }

    /// `BLI_hash_mm2a_add_int`.
    pub fn add_int(&mut self, data: i32) {
        self.add(&data.to_le_bytes());
    }

    /// `BLI_hash_mm2a_end`.
    pub fn end(mut self) -> u32 {
        let tail = self.tail;
        let size = self.size;
        mm2a_mix(&mut self.hash, tail);
        mm2a_mix(&mut self.hash, size);
        mm2a_mix_finalize(&mut self.hash);
        self.hash
    }
}

/// `BLI_hash_mm2` — one-shot Murmur2.
pub fn hash_mm2(data: &[u8], seed: u32) -> u32 {
    let mut h = seed ^ data.len() as u32;
    let mut d = data;
    while d.len() >= 4 {
        let k = u32::from_le_bytes([d[0], d[1], d[2], d[3]]);
        mm2a_mix(&mut h, k);
        d = &d[4..];
    }
    match d.len() {
        3 => {
            h ^= (d[2] as u32) << 16;
            h ^= (d[1] as u32) << 8;
            h ^= d[0] as u32;
            h = h.wrapping_mul(MM2A_M);
        }
        2 => {
            h ^= (d[1] as u32) << 8;
            h ^= d[0] as u32;
            h = h.wrapping_mul(MM2A_M);
        }
        1 => {
            h ^= d[0] as u32;
            h = h.wrapping_mul(MM2A_M);
        }
        _ => {}
    }
    mm2a_mix_finalize(&mut h);
    h
}
