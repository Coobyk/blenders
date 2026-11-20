//! Port of `hash_md5.cc` — MD5 (RFC 1321), from GNU glibc layout.
//! API: `hash_md5_buffer`, `hash_md5_stream` (Reader), plus streaming ctx.

use std::io::Read;

const T: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

/// `md5_ctx` equivalent.
pub struct Md5Ctx {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    total: u64,
    buf: [u8; 64],
    buflen: usize,
}

impl Md5Ctx {
    /// `md5_init_ctx`.
    pub fn new() -> Self {
        Md5Ctx {
            a: 0x67452301,
            b: 0xefcdab89,
            c: 0x98badcfe,
            d: 0x10325476,
            total: 0,
            buf: [0; 64],
            buflen: 0,
        }
    }

    /// `md5_process_bytes`.
    pub fn update(&mut self, data: &[u8]) {
        self.total += data.len() as u64;
        let mut d = data;
        if self.buflen > 0 {
            let need = 64 - self.buflen;
            let take = need.min(d.len());
            self.buf[self.buflen..self.buflen + take].copy_from_slice(&d[..take]);
            self.buflen += take;
            d = &d[take..];
            if self.buflen == 64 {
                let block = self.buf;
                self.process_block(&block);
                self.buflen = 0;
            }
        }
        while d.len() >= 64 {
            self.process_block(&d[..64]);
            d = &d[64..];
        }
        if !d.is_empty() {
            self.buf[..d.len()].copy_from_slice(d);
            self.buflen = d.len();
        }
    }

    /// `md5_finish_ctx` — append padding + length, produce digest.
    pub fn finish(mut self) -> [u8; 16] {
        let bitlen = self.total * 8;
        self.update(&[0x80]);
        while self.buflen != 56 {
            self.update(&[0]);
        }
        let mut lenbuf = [0u8; 8];
        lenbuf.copy_from_slice(&bitlen.to_le_bytes());
        self.update(&lenbuf);
        debug_assert_eq!(self.buflen, 0);
        let mut out = [0u8; 16];
        for (i, w) in [self.a, self.b, self.c, self.d].iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        out
    }

    /// `md5_process_block` — 64-byte block rounds.
    fn process_block(&mut self, block: &[u8]) {
        let mut x = [0u32; 16];
        for i in 0..16 {
            x[i] = u32::from_le_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        let (mut a, mut b, mut c, mut d) = (self.a, self.b, self.c, self.d);
        macro_rules! op {
            ($f:expr, $a:ident, $b:ident, $c:ident, $d:ident, $k:expr, $s:expr, $i:expr) => {{
                $a = $a
                    .wrapping_add($f($b, $c, $d))
                    .wrapping_add(x[$k])
                    .wrapping_add(T[$i]);
                $a = $a.rotate_left($s).wrapping_add($b);
            }};
        }
        let ff = |b: u32, c: u32, d: u32| d ^ (b & (c ^ d));
        let gg = |b: u32, c: u32, d: u32| c ^ (d & (b ^ c));
        let hh = |b: u32, c: u32, d: u32| b ^ c ^ d;
        let ii = |b: u32, c: u32, d: u32| c ^ (b | !d);

        op!(ff, a, b, c, d, 0, 7, 0);
        op!(ff, d, a, b, c, 1, 12, 1);
        op!(ff, c, d, a, b, 2, 17, 2);
        op!(ff, b, c, d, a, 3, 22, 3);
        op!(ff, a, b, c, d, 4, 7, 4);
        op!(ff, d, a, b, c, 5, 12, 5);
        op!(ff, c, d, a, b, 6, 17, 6);
        op!(ff, b, c, d, a, 7, 22, 7);
        op!(ff, a, b, c, d, 8, 7, 8);
        op!(ff, d, a, b, c, 9, 12, 9);
        op!(ff, c, d, a, b, 10, 17, 10);
        op!(ff, b, c, d, a, 11, 22, 11);
        op!(ff, a, b, c, d, 12, 7, 12);
        op!(ff, d, a, b, c, 13, 12, 13);
        op!(ff, c, d, a, b, 14, 17, 14);
        op!(ff, b, c, d, a, 15, 22, 15);

        op!(gg, a, b, c, d, 1, 5, 16);
        op!(gg, d, a, b, c, 6, 9, 17);
        op!(gg, c, d, a, b, 11, 14, 18);
        op!(gg, b, c, d, a, 0, 20, 19);
        op!(gg, a, b, c, d, 5, 5, 20);
        op!(gg, d, a, b, c, 10, 9, 21);
        op!(gg, c, d, a, b, 15, 14, 22);
        op!(gg, b, c, d, a, 4, 20, 23);
        op!(gg, a, b, c, d, 9, 5, 24);
        op!(gg, d, a, b, c, 14, 9, 25);
        op!(gg, c, d, a, b, 3, 14, 26);
        op!(gg, b, c, d, a, 8, 20, 27);
        op!(gg, a, b, c, d, 13, 5, 28);
        op!(gg, d, a, b, c, 2, 9, 29);
        op!(gg, c, d, a, b, 7, 14, 30);
        op!(gg, b, c, d, a, 12, 20, 31);

        op!(hh, a, b, c, d, 5, 4, 32);
        op!(hh, d, a, b, c, 8, 11, 33);
        op!(hh, c, d, a, b, 11, 16, 34);
        op!(hh, b, c, d, a, 14, 23, 35);
        op!(hh, a, b, c, d, 1, 4, 36);
        op!(hh, d, a, b, c, 4, 11, 37);
        op!(hh, c, d, a, b, 7, 16, 38);
        op!(hh, b, c, d, a, 10, 23, 39);
        op!(hh, a, b, c, d, 13, 4, 40);
        op!(hh, d, a, b, c, 0, 11, 41);
        op!(hh, c, d, a, b, 3, 16, 42);
        op!(hh, b, c, d, a, 6, 23, 43);
        op!(hh, a, b, c, d, 9, 4, 44);
        op!(hh, d, a, b, c, 12, 11, 45);
        op!(hh, c, d, a, b, 15, 16, 46);
        op!(hh, b, c, d, a, 2, 23, 47);

        op!(ii, a, b, c, d, 0, 6, 48);
        op!(ii, d, a, b, c, 7, 10, 49);
        op!(ii, c, d, a, b, 14, 15, 50);
        op!(ii, b, c, d, a, 5, 21, 51);
        op!(ii, a, b, c, d, 12, 6, 52);
        op!(ii, d, a, b, c, 3, 10, 53);
        op!(ii, c, d, a, b, 10, 15, 54);
        op!(ii, b, c, d, a, 1, 21, 55);
        op!(ii, a, b, c, d, 8, 6, 56);
        op!(ii, d, a, b, c, 15, 10, 57);
        op!(ii, c, d, a, b, 6, 15, 58);
        op!(ii, b, c, d, a, 13, 21, 59);
        op!(ii, a, b, c, d, 4, 6, 60);
        op!(ii, d, a, b, c, 11, 10, 61);
        op!(ii, c, d, a, b, 2, 15, 62);
        op!(ii, b, c, d, a, 9, 21, 63);

        self.a = self.a.wrapping_add(a);
        self.b = self.b.wrapping_add(b);
        self.c = self.c.wrapping_add(c);
        self.d = self.d.wrapping_add(d);
    }
}

/// `BLI_hash_md5_buffer` — one-shot.
pub fn hash_md5_buffer(data: &[u8]) -> [u8; 16] {
    let mut ctx = Md5Ctx::new();
    ctx.update(data);
    ctx.finish()
}

/// `BLI_hash_md5_stream` — hash all of `reader`; Err on read failure.
pub fn hash_md5_stream<R: Read>(reader: &mut R) -> std::io::Result<[u8; 16]> {
    const BLOCKSIZE: usize = 4096;
    let mut ctx = Md5Ctx::new();
    let mut buf = [0u8; BLOCKSIZE];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
    }
    Ok(ctx.finish())
}

/// `BLI_hash_md5_to_hexdigest` — 32-char lowercase hex.
pub fn hash_md5_to_hexdigest(digest: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in digest {
        s.push_str(&format!("{:02x}", b));
    }
    s
}
