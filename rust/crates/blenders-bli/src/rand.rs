//! Port of `rand.cc` — `RandomNumberGenerator` (48-bit LCG) + `RNG` C API
//! + halton/hammersley low-discrepancy sequences.
//! `BLI_noise_hash_uchar_512` table lives in noise_c.cc; duplicated here
//! until noise.rs lands.

const HASH_UCHAR_512: [u8; 512] = [
    0xa2, 0xa0, 0x19, 0x3b, 0xf8, 0xeb, 0xaa, 0xee, 0xf3, 0x1c, 0x67, 0x28, 0x1d, 0xed, 0x00, 0xde,
    0x95, 0x2e, 0xdc, 0x3f, 0x3a, 0x82, 0x35, 0x4d, 0x6c, 0xba, 0x36, 0xd0, 0xf6, 0x0c, 0x79, 0x32,
    0xd1, 0x59, 0xf4, 0x08, 0x8b, 0x63, 0x89, 0x2f, 0xb8, 0xb4, 0x97, 0x83, 0xf2, 0x8f, 0x18, 0xc7,
    0x51, 0x14, 0x65, 0x87, 0x48, 0x20, 0x42, 0xa8, 0x80, 0xb5, 0x40, 0x13, 0xb2, 0x22, 0x7e, 0x57,
    0xbc, 0x7f, 0x6b, 0x9d, 0x86, 0x4c, 0xc8, 0xdb, 0x7c, 0xd5, 0x25, 0x4e, 0x5a, 0x55, 0x74, 0x50,
    0xcd, 0xb3, 0x7a, 0xbb, 0xc3, 0xcb, 0xb6, 0xe2, 0xe4, 0xec, 0xfd, 0x98, 0x0b, 0x96, 0xd3, 0x9e,
    0x5c, 0xa1, 0x64, 0xf1, 0x81, 0x61, 0xe1, 0xc4, 0x24, 0x72, 0x49, 0x8c, 0x90, 0x4b, 0x84, 0x34,
    0x38, 0xab, 0x78, 0xca, 0x1f, 0x01, 0xd7, 0x93, 0x11, 0xc1, 0x58, 0xa9, 0x31, 0xf9, 0x44, 0x6d,
    0xbf, 0x33, 0x9c, 0x5f, 0x09, 0x94, 0xa3, 0x85, 0x06, 0xc6, 0x9a, 0x1e, 0x7b, 0x46, 0x15, 0x30,
    0x27, 0x2b, 0x1b, 0x71, 0x3c, 0x5b, 0xd6, 0x6f, 0x62, 0xac, 0x4f, 0xc2, 0xc0, 0x0e, 0xb1, 0x23,
    0xa7, 0xdf, 0x47, 0xb0, 0x77, 0x69, 0x05, 0xe9, 0xe6, 0xe7, 0x76, 0x73, 0x0f, 0xfe, 0x6e, 0x9b,
    0x56, 0xef, 0x12, 0xa5, 0x37, 0xfc, 0xae, 0xd9, 0x03, 0x8e, 0xdd, 0x10, 0xb9, 0xce, 0xc9, 0x8d,
    0xda, 0x2a, 0xbd, 0x68, 0x17, 0x9f, 0xbe, 0xd4, 0x0a, 0xcc, 0xd2, 0xe8, 0x43, 0x3d, 0x70, 0xb7,
    0x02, 0x7d, 0x99, 0xd8, 0x0d, 0x60, 0x8a, 0x04, 0x2c, 0x3e, 0x92, 0xe5, 0xaf, 0x53, 0x07, 0xe0,
    0x29, 0xa6, 0xc5, 0xe3, 0xf5, 0xf7, 0x4a, 0x41, 0x26, 0x6a, 0x16, 0x5e, 0x52, 0x2d, 0x21, 0xad,
    0xf0, 0x91, 0xff, 0xea, 0x54, 0xfa, 0x66, 0x1a, 0x45, 0x39, 0xcf, 0x75, 0xa4, 0x88, 0xfb, 0x5d,
    0xa2, 0xa0, 0x19, 0x3b, 0xf8, 0xeb, 0xaa, 0xee, 0xf3, 0x1c, 0x67, 0x28, 0x1d, 0xed, 0x00, 0xde,
    0x95, 0x2e, 0xdc, 0x3f, 0x3a, 0x82, 0x35, 0x4d, 0x6c, 0xba, 0x36, 0xd0, 0xf6, 0x0c, 0x79, 0x32,
    0xd1, 0x59, 0xf4, 0x08, 0x8b, 0x63, 0x89, 0x2f, 0xb8, 0xb4, 0x97, 0x83, 0xf2, 0x8f, 0x18, 0xc7,
    0x51, 0x14, 0x65, 0x87, 0x48, 0x20, 0x42, 0xa8, 0x80, 0xb5, 0x40, 0x13, 0xb2, 0x22, 0x7e, 0x57,
    0xbc, 0x7f, 0x6b, 0x9d, 0x86, 0x4c, 0xc8, 0xdb, 0x7c, 0xd5, 0x25, 0x4e, 0x5a, 0x55, 0x74, 0x50,
    0xcd, 0xb3, 0x7a, 0xbb, 0xc3, 0xcb, 0xb6, 0xe2, 0xe4, 0xec, 0xfd, 0x98, 0x0b, 0x96, 0xd3, 0x9e,
    0x5c, 0xa1, 0x64, 0xf1, 0x81, 0x61, 0xe1, 0xc4, 0x24, 0x72, 0x49, 0x8c, 0x90, 0x4b, 0x84, 0x34,
    0x38, 0xab, 0x78, 0xca, 0x1f, 0x01, 0xd7, 0x93, 0x11, 0xc1, 0x58, 0xa9, 0x31, 0xf9, 0x44, 0x6d,
    0xbf, 0x33, 0x9c, 0x5f, 0x09, 0x94, 0xa3, 0x85, 0x06, 0xc6, 0x9a, 0x1e, 0x7b, 0x46, 0x15, 0x30,
    0x27, 0x2b, 0x1b, 0x71, 0x3c, 0x5b, 0xd6, 0x6f, 0x62, 0xac, 0x4f, 0xc2, 0xc0, 0x0e, 0xb1, 0x23,
    0xa7, 0xdf, 0x47, 0xb0, 0x77, 0x69, 0x05, 0xe9, 0xe6, 0xe7, 0x76, 0x73, 0x0f, 0xfe, 0x6e, 0x9b,
    0x56, 0xef, 0x12, 0xa5, 0x37, 0xfc, 0xae, 0xd9, 0x03, 0x8e, 0xdd, 0x10, 0xb9, 0xce, 0xc9, 0x8d,
    0xda, 0x2a, 0xbd, 0x68, 0x17, 0x9f, 0xbe, 0xd4, 0x0a, 0xcc, 0xd2, 0xe8, 0x43, 0x3d, 0x70, 0xb7,
    0x02, 0x7d, 0x99, 0xd8, 0x0d, 0x60, 0x8a, 0x04, 0x2c, 0x3e, 0x92, 0xe5, 0xaf, 0x53, 0x07, 0xe0,
    0x29, 0xa6, 0xc5, 0xe3, 0xf5, 0xf7, 0x4a, 0x41, 0x26, 0x6a, 0x16, 0x5e, 0x52, 0x2d, 0x21, 0xad,
    0xf0, 0x91, 0xff, 0xea, 0x54, 0xfa, 0x66, 0x1a, 0x45, 0x39, 0xcf, 0x75, 0xa4, 0x88, 0xfb, 0x5d
];


/// `RandomNumberGenerator` — the 48-bit LCG (glibc/Java params).
pub struct RandomNumberGenerator {
    x: u64,
}

const MULTIPLIER: u64 = 0x5DEECE66D;
const ADDEND: u64 = 0xB;
const MASK: u64 = 0x0000FFFFFFFFFFFF;

impl RandomNumberGenerator {
    /// `RandomNumberGenerator(seed)`.
    pub fn new(seed: u32) -> Self {
        let mut r = RandomNumberGenerator { x: 0 };
        r.seed(seed);
        r
    }
    /// `RandomNumberGenerator::from_random_seed` — OS entropy.
    pub fn from_random_seed() -> Self {
        let mut b = [0u8; 4];
        // std has no random_device; use time+pid hash as entropy source.
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        b.copy_from_slice(&(t as u32 ^ (std::process::id() << 8)).to_le_bytes());
        Self::new(u32::from_le_bytes(b))
    }
    /// `seed`.
    pub fn seed(&mut self, seed: u32) {
        const LOWSEED: u64 = 0x330E;
        self.x = ((seed as u64) << 16) | LOWSEED;
    }
    /// `seed_random` — hash-stirred seeding.
    pub fn seed_random(&mut self, mut seed: u32) {
        self.seed(seed + HASH_UCHAR_512[(seed & 255) as usize] as u32);
        seed = self.get_uint32();
        self.seed(seed + HASH_UCHAR_512[(seed & 255) as usize] as u32);
        seed = self.get_uint32();
        self.seed(seed + HASH_UCHAR_512[(seed & 255) as usize] as u32);
    }
    /// `step`.
    fn step(&mut self) {
        self.x = (MULTIPLIER.wrapping_mul(self.x) + ADDEND) & MASK;
    }
    /// `get_uint32`.
    pub fn get_uint32(&mut self) -> u32 {
        self.step();
        (self.x >> 17) as u32
    }
    /// `get_int32`.
    pub fn get_int32(&mut self) -> i32 {
        self.step();
        (self.x >> 17) as i32
    }
    /// `get_uint64`.
    pub fn get_uint64(&mut self) -> u64 {
        ((self.get_uint32() as u64) << 32) | self.get_uint32() as u64
    }
    /// `get_int32(max_exclusive)`.
    pub fn get_int32_max(&mut self, max_exclusive: i32) -> i32 {
        debug_assert!(max_exclusive > 0);
        self.get_int32() % max_exclusive
    }
    /// `get_double`.
    pub fn get_double(&mut self) -> f64 {
        self.get_int32() as f64 / 2147483648.0
    }
    /// `get_float`.
    pub fn get_float(&mut self) -> f32 {
        self.get_int32() as f32 / 2147483648.0
    }
    /// `shuffle`.
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        let n = values.len();
        for i in (2..n).rev() {
            let j = self.get_int32_max(i as i32) as usize;
            if i != j {
                values.swap(i, j);
            }
        }
    }
    /// `get_barycentric_coordinates`.
    pub fn get_barycentric_coordinates(&mut self) -> [f32; 3] {
        let mut r1 = self.get_float();
        let mut r2 = self.get_float();
        if r1 + r2 > 1.0 {
            r1 = 1.0 - r1;
            r2 = 1.0 - r2;
        }
        [r1, r2, 1.0 - r1 - r2]
    }
    /// `round_probabilistic`.
    pub fn round_probabilistic(&mut self, x: f32) -> i32 {
        debug_assert!(x >= 0.0);
        let up_prob = x.fract();
        let up = up_prob > self.get_float();
        x as i32 + up as i32
    }
    /// `get_unit_float2`.
    pub fn get_unit_float2(&mut self) -> [f32; 2] {
        let a = std::f32::consts::PI * 2.0 * self.get_float();
        [a.cos(), a.sin()]
    }
    /// `get_unit_float3`.
    pub fn get_unit_float3(&mut self) -> [f32; 3] {
        let z = 2.0 * self.get_float() - 1.0;
        let r = 1.0 - z * z;
        if r > 0.0 {
            let a = std::f32::consts::PI * 2.0 * self.get_float();
            let r = r.sqrt();
            [r * a.cos(), r * a.sin(), z]
        } else {
            [0.0, 0.0, 1.0]
        }
    }
    /// `get_triangle_sample` (2d).
    pub fn get_triangle_sample(&mut self, v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> [f32; 2] {
        let mut u = self.get_float();
        let mut v = self.get_float();
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        [
            v1[0] + (v2[0] - v1[0]) * u + (v3[0] - v1[0]) * v,
            v1[1] + (v2[1] - v1[1]) * u + (v3[1] - v1[1]) * v,
        ]
    }
    /// `get_triangle_sample_3d`.
    pub fn get_triangle_sample_3d(
        &mut self, v1: [f32; 3], v2: [f32; 3], v3: [f32; 3],
    ) -> [f32; 3] {
        let mut u = self.get_float();
        let mut v = self.get_float();
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        [
            v1[0] + (v2[0] - v1[0]) * u + (v3[0] - v1[0]) * v,
            v1[1] + (v2[1] - v1[1]) * u + (v3[1] - v1[1]) * v,
            v1[2] + (v2[2] - v1[2]) * u + (v3[2] - v1[2]) * v,
        ]
    }
    /// `get_bytes` — state bytes copied out, stepped every stride.
    pub fn get_bytes(&mut self, r_bytes: &mut [u8]) {
        const MASK_BYTES: usize = 2;
        let rand_stride = 8 - MASK_BYTES;
        let mut last_len = r_bytes.len();
        let mut trim_len = last_len;
        if trim_len > rand_stride {
            last_len = trim_len % rand_stride;
            trim_len -= last_len;
        } else {
            trim_len = 0;
        }
        let mut i = 0;
        while i < trim_len {
            let src = self.x.to_le_bytes();
            for j in 0..rand_stride {
                r_bytes[i] = src[j];
                i += 1;
            }
            self.step();
        }
        if last_len > 0 {
            let src = self.x.to_le_bytes();
            for j in 0..last_len {
                r_bytes[i] = src[j];
                i += 1;
            }
        }
    }
    /// `skip`.
    pub fn skip(&mut self, n: i64) {
        for _ in 0..n {
            self.step();
        }
    }
}

/// `RNG` C-API wrapper.
pub struct Rng(pub RandomNumberGenerator);
impl Rng {
    /// `BLI_rng_new` / `BLI_rng_new_srandom`.
    pub fn new(seed: u32) -> Self {
        Rng(RandomNumberGenerator::new(seed))
    }
    pub fn new_srandom(seed: u32) -> Self {
        let mut r = Rng(RandomNumberGenerator::new(0));
        r.0.seed_random(seed);
        r
    }
    pub fn seed(&mut self, s: u32) { self.0.seed(s); }
    pub fn srandom(&mut self, s: u32) { self.0.seed_random(s); }
    pub fn get_char_n(&mut self, b: &mut [u8]) { self.0.get_bytes(b); }
    pub fn get_int(&mut self) -> i32 { self.0.get_int32() }
    pub fn get_uint(&mut self) -> u32 { self.0.get_uint32() }
    pub fn get_double(&mut self) -> f64 { self.0.get_double() }
    pub fn get_float(&mut self) -> f32 { self.0.get_float() }
    pub fn get_tri_sample_float_v2(&mut self, v1: [f32;2], v2: [f32;2], v3: [f32;2]) -> [f32;2] {
        self.0.get_triangle_sample(v1, v2, v3)
    }
    /// `BLI_rng_shuffle_array` — generic over T.
    pub fn shuffle_array<T>(&mut self, data: &mut [T]) {
        let n = data.len();
        if n <= 1 {
            return;
        }
        for i in (1..n).rev() {
            let j = (self.get_uint() % n as u32) as usize;
            if i != j {
                data.swap(i, j);
            }
        }
    }
    /// `BLI_rng_shuffle_bitmap`.
    pub fn shuffle_bitmap(&mut self, bitmap: &mut [u8], bits_num: usize) {
        if bits_num <= 1 {
            return;
        }
        let test = |b: &[u8], i: usize| (b[i / 8] >> (i % 8)) & 1 != 0;
        for i in (1..bits_num).rev() {
            let j = (self.get_uint() % bits_num as u32) as usize;
            if i != j {
                let ib = test(bitmap, i);
                let jb = test(bitmap, j);
                let set = |b: &mut [u8], i: usize, v: bool| {
                    if v { b[i / 8] |= 1 << (i % 8); } else { b[i / 8] &= !(1 << (i % 8)); }
                };
                set(bitmap, i, jb);
                set(bitmap, j, ib);
            }
        }
    }
    /// `BLI_rng_skip`.
    pub fn skip(&mut self, n: i32) {
        self.0.skip(n as i64);
    }
}

/// `BLI_hash_frand`.
pub fn hash_frand(seed: u32) -> f32 {
    Rng::new_srandom(seed).get_float()
}

/// `BLI_array_randomize`.
pub fn array_randomize<T>(data: &mut [T], seed: u32) {
    Rng::new(seed).shuffle_array(data);
}

/// `halton_ex`.
fn halton_ex(invprimes: f64, offset: &mut f64) {
    let e = ((1.0 - *offset) - 1e-10).abs();
    if invprimes >= e {
        let mut h = invprimes;
        let mut lasth;
        loop {
            lasth = h;
            h *= invprimes;
            if h < e { break; }
        }
        *offset = if invprimes > e {
            h - e
        } else {
            lasth
        };
    } else {
        *offset += invprimes;
    }
}

/// `BLI_halton_1d`.
pub fn halton_1d(prime: u32, offset: &mut f64, n: usize, r: &mut [f64]) {
    let invprime = 1.0 / prime as f64;
    for s in 0..n {
        halton_ex(invprime, offset);
        r[s] = *offset;
    }
}
/// `BLI_halton_2d`.
pub fn halton_2d(prime: [u32; 2], offset: &mut [f64; 2], n: usize, r: &mut [f64; 2]) {
    let inv = [1.0 / prime[0] as f64, 1.0 / prime[1] as f64];
    for _ in 0..n {
        for i in 0..2 {
            halton_ex(inv[i], &mut offset[i]);
            r[i] = offset[i];
        }
    }
}
/// `BLI_halton_3d`.
pub fn halton_3d(prime: [u32; 3], offset: &mut [f64; 3], n: usize, r: &mut [f64; 3]) {
    let inv = [1.0 / prime[0] as f64, 1.0 / prime[1] as f64, 1.0 / prime[2] as f64];
    for _ in 0..n {
        for i in 0..3 {
            halton_ex(inv[i], &mut offset[i]);
            r[i] = offset[i];
        }
    }
}

/// `radical_inverse`.
pub fn radical_inverse(mut n: u32) -> f64 {
    let mut u = 0.0;
    let mut p = 0.5;
    while n != 0 {
        if n & 1 != 0 {
            u += p;
        }
        p *= 0.5;
        n >>= 1;
    }
    u
}

/// `BLI_hammersley_1d`.
pub fn hammersley_1d(n: u32, r: &mut f64) {
    *r = radical_inverse(n);
}
