//! Port of `string_ref.cc` + `BLI_string_ref.hh` — non-owning string views.
//! `StringRef` maps to `&[u8]` (may contain NULs); `StringRefNull` is the
//! NUL-terminated variant. Both wrap `&'a str` in Rust.

/// `StringRef` — borrowed string slice (Rust `&str` semantics).
#[derive(Clone, Copy)]
pub struct StringRef<'a>(pub &'a str);

impl<'a> StringRef<'a> {
    pub fn new(s: &'a str) -> Self {
        StringRef(s)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn size(&self) -> i64 {
        self.0.len() as i64
    }
    /// `substr` — substring from `start` of `len` bytes (clamped).
    pub fn substr(&self, start: usize, len: usize) -> StringRef<'a> {
        StringRef(&self.0[start..(start + len).min(self.0.len())])
    }
    /// `drop_prefix`.
    pub fn drop_prefix(&self, n: usize) -> StringRef<'a> {
        StringRef(&self.0[n.min(self.0.len())..])
    }
    /// `drop_suffix`.
    pub fn drop_suffix(&self, n: usize) -> StringRef<'a> {
        let n = n.min(self.0.len());
        StringRef(&self.0[..self.0.len() - n])
    }
    /// `find` — byte offset or `usize::MAX` (NOT_FOUND).
    pub fn find(&self, needle: &str) -> usize {
        self.0.find(needle).unwrap_or(usize::MAX)
    }
    /// `rfind`.
    pub fn rfind(&self, needle: &str) -> usize {
        self.0.rfind(needle).unwrap_or(usize::MAX)
    }
    /// `startswith` / `endswith`.
    pub fn startswith(&self, prefix: &str) -> bool {
        self.0.starts_with(prefix)
    }
    pub fn endswith(&self, suffix: &str) -> bool {
        self.0.ends_with(suffix)
    }
}

impl<'a> From<&'a str> for StringRef<'a> {
    fn from(s: &'a str) -> Self {
        StringRef(s)
    }
}
impl<'a> AsRef<str> for StringRef<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}
impl<'a> std::fmt::Display for StringRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// `StringRefNull` — NUL-terminated string ref (same slice, guarantees
/// the underlying data ends with `\0`; exposed as `&str` here).
pub type StringRefNull<'a> = StringRef<'a>;

/// `STRING_REF_NOT_FOUND`.
pub const NOT_FOUND: usize = usize::MAX;
