//! Port of `uuid.cc` — RFC4122 v4 `bUUID`/`UUID`.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// `bUUID`.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BUuid {
    pub time_low: u32,
    pub time_mid: u16,
    pub time_hi_and_version: u16,
    pub clock_seq_hi_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

/// `UUID_STRING_SIZE` (37 incl NUL).
pub const UUID_STRING_SIZE: usize = 37;

/// Simple xorshift64 PRNG seeded from system clock (mirrors mt19937_64 seeded from timespec).
fn rng_next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

static RNG_STATE: AtomicU64 = AtomicU64::new(0);

fn random_u64() -> u64 {
    loop {
        let old = RNG_STATE.load(Ordering::Relaxed);
        let mut s = if old == 0 {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let mut s = now.as_nanos() as u64 ^ now.as_secs();
            if s == 0 {
                s = 0x9E3779B97F4A7C15;
            }
            s
        } else {
            old
        };
        let new = rng_next(&mut s);
        if RNG_STATE
            .compare_exchange_weak(old, new, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            return new;
        }
    }
}

/// `BLI_uuid()` — random v4 UUID.
pub fn uuid() -> BUuid {
    let mut u = BUuid::default();
    let a = random_u64();
    let b = random_u64();
    // Fill struct fields with the random bits (like reinterpret_cast to int64).
    let bytes: [u8; 16] = unsafe { std::mem::transmute([a, b]) };
    u.time_low = u32::from_ne_bytes(bytes[0..4].try_into().unwrap());
    u.time_mid = u16::from_ne_bytes(bytes[4..6].try_into().unwrap());
    u.time_hi_and_version = u16::from_ne_bytes(bytes[6..8].try_into().unwrap());
    u.clock_seq_hi_and_reserved = bytes[8];
    u.clock_seq_low = bytes[9];
    u.node = bytes[10..16].try_into().unwrap();

    u.time_hi_and_version &= !0xF000;
    u.time_hi_and_version |= 0x4000;
    u.clock_seq_hi_and_reserved &= !0x40;
    u.clock_seq_hi_and_reserved |= 0x80;
    u
}

/// `BLI_uuid_nil`.
pub fn uuid_nil() -> BUuid {
    BUuid::default()
}

/// `BLI_uuid_is_nil`.
pub fn uuid_is_nil(u: BUuid) -> bool {
    u == uuid_nil()
}

/// `BLI_uuid_equal`.
pub fn uuid_equal(a: BUuid, b: BUuid) -> bool {
    a == b
}

/// `BLI_uuid_format` — `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`.
pub fn uuid_format(u: &BUuid) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        u.time_low,
        u.time_mid,
        u.time_hi_and_version,
        u.clock_seq_hi_and_reserved,
        u.clock_seq_low,
        u.node[0],
        u.node[1],
        u.node[2],
        u.node[3],
        u.node[4],
        u.node[5]
    )
}

/// `BLI_uuid_parse_string` — parses all 11 hex fields.
pub fn uuid_parse_string(s: &str) -> Option<BUuid> {
    let mut u = BUuid::default();
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    let (a, b, c, d, e) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
    if a.len() != 8 || b.len() != 4 || c.len() != 4 || d.len() != 4 || e.len() != 12 {
        return None;
    }
    u.time_low = u32::from_str_radix(a, 16).ok()?;
    u.time_mid = u16::from_str_radix(b, 16).ok()?;
    u.time_hi_and_version = u16::from_str_radix(c, 16).ok()?;
    u.clock_seq_hi_and_reserved = u8::from_str_radix(&d[0..2], 16).ok()?;
    u.clock_seq_low = u8::from_str_radix(&d[2..4], 16).ok()?;
    for i in 0..6 {
        u.node[i] = u8::from_str_radix(&e[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(u)
}

impl fmt::Display for BUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&uuid_format(self))
    }
}

/// `blender::UUID` C++ wrapper — alias in Rust.
pub type Uuid = BUuid;
