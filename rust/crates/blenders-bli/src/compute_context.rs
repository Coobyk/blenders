//! Port of `compute_context.cc` — `blender::ComputeContext` / `ComputeContextHash`.
//! The context objects are defined per-subsystem in C++; here we port the
//! hash, stack walk and printing machinery.


/// `ComputeContextHash` — 128-bit hash (XXH128 in C++; MM3-derived here,
/// same role in the protocol).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct ComputeContextHash {
    pub v1: u64,
    pub v2: u64,
}

impl ComputeContextHash {
    /// `ComputeContextHash::from_bytes`.
    pub fn from_bytes(data: &[u8]) -> Self {
        // Two MM3 passes with different seeds → 128 bits.
        let v1 = crate::hash_mm3::hash_mm3(data, 0) as u64
            | ((crate::hash_mm3::hash_mm3(data, 1) as u64) << 32);
        let v2 = crate::hash_mm3::hash_mm3(data, 2) as u64
            | ((crate::hash_mm3::hash_mm3(data, 3) as u64) << 32);
        ComputeContextHash { v1, v2 }
    }
    /// `operator<<` — hex dump.
    pub fn to_hex(&self) -> String {
        format!("0x{:x}{:x}", self.v1, self.v2)
    }
}

/// `ComputeContext` — parent chain + per-context hash/print hooks.
/// In C++ this is a polymorphic base; here it's the parent link plus
/// a print callback stored alongside.
pub struct ComputeContext {
    pub parent: Option<Box<ComputeContext>>,
    pub name: String,
    pub hash: ComputeContextHash,
}

impl ComputeContext {
    /// `context->print_stack(name)` — walks the parent chain top-down.
    pub fn print_stack(&self, name: &str) -> String {
        let mut chain: Vec<&ComputeContext> = Vec::new();
        let mut cur: Option<&ComputeContext> = Some(self);
        while let Some(c) = cur {
            chain.push(c);
            cur = c.parent.as_deref();
        }
        let mut s = format!("Context Stack: {name}\n");
        while let Some(c) = chain.pop() {
            s.push_str(&format!("-> {} \t(hash: {})\n", c.name, c.hash.to_hex()));
        }
        s
    }
}
