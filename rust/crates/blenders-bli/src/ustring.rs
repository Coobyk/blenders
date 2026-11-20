//! Port of `ustring.cc` + `BLI_ustring.hh` — interned strings (`UString`).
//! Global string table interning by value, hash cached.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

fn hash_string(s: &str) -> u64 {
    crate::hash_mm3::hash_mm3(s.as_bytes(), 0) as u64
}

/// `UStringEntry` — interned payload.
#[derive(Debug)]
pub struct UStringEntry {
    pub value: Box<str>,
    pub hash: u64,
}

fn global_string_table() -> &'static Mutex<HashMap<u64, Box<UStringEntry>>> {
    static TABLE: OnceLock<Mutex<HashMap<u64, Box<UStringEntry>>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut m = HashMap::new();
        m.reserve(16384);
        Mutex::new(m)
    })
}

/// `ustring_ensure_entry` — intern `str`, returning a stable pointer.
pub fn ustring_ensure_entry(s: &str) -> &'static UStringEntry {
    let h = hash_string(s);
    let mut table = global_string_table().lock().unwrap();
    // The table is append-only; entries are boxed so pointer stays stable.
    let entry = table.entry(h).or_insert_with(|| {
        Box::new(UStringEntry {
            value: s.into(),
            hash: h,
        })
    });
    let ptr: *const UStringEntry = &**entry;
    unsafe { &*ptr }
}

/// `UString` — cheap-to-copy interned string.
#[derive(Clone, Copy)]
pub struct UString {
    ptr: &'static UStringEntry,
}

impl UString {
    pub fn new(s: &str) -> Self {
        UString {
            ptr: ustring_ensure_entry(s),
        }
    }
    /// `ustr.string()`.
    pub fn string(&self) -> &'static str {
        &self.ptr.value
    }
    /// `ustr.c_str()`.
    pub fn c_str(&self) -> &'static str {
        &self.ptr.value
    }
    /// `ustr.hash()`.
    pub fn hash(&self) -> u64 {
        self.ptr.hash
    }
    /// `ustr.size()`.
    pub fn size(&self) -> usize {
        self.ptr.value.len()
    }
    /// `ustr.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.ptr.value.is_empty()
    }
}

impl Default for UString {
    fn default() -> Self {
        UString::new("")
    }
}
impl PartialEq for UString {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr, other.ptr)
    }
}
impl Eq for UString {}
impl std::hash::Hash for UString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash.hash(state);
    }
}
impl fmt::Display for UString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.string())
    }
}
