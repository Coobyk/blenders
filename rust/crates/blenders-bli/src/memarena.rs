//! Port of `BLI_memarena.cc` — bump arena allocator.

/// `MemArena` — bump allocator over Vec-backed memory blocks.
///
/// Rust owns the allocated values via `Vec` rather than returning raw
/// pointers; semantics match: allocations are cheap and released en-masse.
pub struct MemArena<T> {
    items: Vec<T>,
}

impl<T: Default> MemArena<T> {
    /// `BLI_memarena_new`.
    pub fn new() -> Self {
        MemArena { items: Vec::new() }
    }
    /// `BLI_memarena_alloc` — allocate a default `T`, return index.
    pub fn alloc(&mut self) -> usize {
        self.items.push(T::default());
        self.items.len() - 1
    }
    /// `BLI_memarena_calloc` — same as alloc (Rust zero/default).
    pub fn calloc(&mut self) -> usize {
        self.alloc()
    }
    /// `BLI_memarena_merge` — move src's items into self, emptying src.
    pub fn merge(&mut self, src: &mut MemArena<T>) {
        self.items.append(&mut src.items);
    }
    /// `BLI_memarena_clear` — free all, keep backing capacity.
    pub fn clear(&mut self) {
        self.items.clear();
    }
    /// `BLI_memarena_free` — drop all.
    pub fn free(&mut self) {
        self.items = Vec::new();
    }
    /// Borrow the arena contents.
    pub fn items(&self) -> &[T] {
        &self.items
    }
    pub fn items_mut(&mut self) -> &mut [T] {
        &mut self.items
    }
}
