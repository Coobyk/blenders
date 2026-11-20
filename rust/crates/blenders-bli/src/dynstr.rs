//! Port of `BLI_dynstr.cc` — append-only dynamic string.
//! Linked-segment storage maps to `Vec<String>`; memarena variant is noted.

#[derive(Default)]
pub struct DynStr {
    elems: Vec<String>,
    curlen: usize,
}

impl DynStr {
    /// `BLI_dynstr_new` / `BLI_dynstr_new_memarena` (arena is an allocation
    /// detail — same semantics either way).
    pub fn new() -> Self {
        DynStr::default()
    }
    /// `BLI_dynstr_new_memarena`.
    pub fn new_memarena() -> Self {
        DynStr::default()
    }
    /// `BLI_dynstr_append`.
    pub fn append(&mut self, s: &str) {
        self.curlen += s.len();
        self.elems.push(s.to_string());
    }
    /// `BLI_dynstr_nappend`.
    pub fn nappend(&mut self, s: &str, len: usize) {
        let t: String = s.chars().take(len).collect();
        self.curlen += t.len();
        self.elems.push(t);
    }
    /// `BLI_dynstr_appendf` — call sites use `format!`.
    pub fn appendf(&mut self, s: String) {
        self.append(&s);
    }
    /// `BLI_dynstr_get_len`.
    pub fn len(&self) -> usize {
        self.curlen
    }
    /// `BLI_dynstr_get_cstring`.
    pub fn get_cstring(&self) -> String {
        let mut out = String::with_capacity(self.curlen);
        for e in &self.elems {
            out.push_str(e);
        }
        out
    }
    /// `BLI_dynstr_clear`.
    pub fn clear(&mut self) {
        self.elems.clear();
        self.curlen = 0;
    }
}
