//! Port of `BLI_mempool.cc` — fixed-element pool allocator (`BLI_mempool`).
//! Rust version stores `Option<T>` slots with an explicit free-list;
//! iteration is in insertion-index order (like chunk order in C).

/// `BLI_MEMPOOL_NOP` / `BLI_MEMPOOL_ALLOW_ITER`.
pub const MEMPOOL_NOP: u32 = 0;
pub const MEMPOOL_ALLOW_ITER: u32 = 1 << 0;

/// `BLI_mempool`.
pub struct Mempool<T> {
    /// Slot storage: `None` = free.
    slots: Vec<Option<T>>,
    /// Free-list head chain: free slot indices.
    free: Vec<usize>,
    /// `BLI_mempool` elem count.
    used: usize,
    flag: u32,
}

impl<T> Mempool<T> {
    /// `BLI_mempool_create`.
    pub fn create(_esize: usize, elem_num: usize, _pchunk: usize, flag: u32) -> Self {
        Mempool {
            slots: Vec::with_capacity(elem_num.max(1)),
            free: Vec::new(),
            used: 0,
            flag,
        }
    }

    /// `BLI_mempool_alloc` — allocate slot, return index (stable handle).
    pub fn alloc(&mut self, value: T) -> usize {
        self.used += 1;
        if let Some(i) = self.free.pop() {
            self.slots[i] = Some(value);
            i
        } else {
            self.slots.push(Some(value));
            self.slots.len() - 1
        }
    }

    /// `BLI_mempool_calloc` — same for default values.
    pub fn calloc(&mut self) -> usize
    where
        T: Default,
    {
        self.alloc(T::default())
    }

    /// `BLI_mempool_free` — free slot `index`.
    pub fn free(&mut self, index: usize) {
        if self.slots.get(index).and_then(|s| s.as_ref()).is_some() {
            self.slots[index] = None;
            self.free.push(index);
            self.used -= 1;
        }
    }

    /// `BLI_mempool_len`.
    pub fn len(&self) -> usize {
        self.used
    }

    /// `BLI_mempool_findelem` — element at (occupied) index `index`.
    pub fn findelem(&self, index: usize) -> Option<&T> {
        self.slots.get(index)?.as_ref()
    }
    pub fn findelem_mut(&mut self, index: usize) -> Option<&mut T> {
        self.slots.get_mut(index)?.as_mut()
    }

    /// `BLI_mempool_as_array` — collect all live elements in slot order.
    pub fn as_array(&self) -> Vec<&T> {
        self.slots.iter().filter_map(|s| s.as_ref()).collect()
    }

    /// `BLI_mempool_iternew` + `BLI_mempool_iterstep` — index iterator.
    pub fn iter(&self) -> MempoolIter<'_, T> {
        MempoolIter { pool: self, index: 0 }
    }

    /// `BLI_mempool_clear`.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.free.clear();
        self.used = 0;
    }

    /// `BLI_mempool_destroy` — Drop handles this.
    pub fn flag(&self) -> u32 {
        self.flag
    }
}

/// `BLI_mempool_iter`.
pub struct MempoolIter<'a, T> {
    pool: &'a Mempool<T>,
    index: usize,
}

impl<'a, T> MempoolIter<'a, T> {
    /// `bli_mempool_iternext` — next occupied element.
    pub fn next_elem(&mut self) -> Option<&'a T> {
        while self.index < self.pool.slots.len() {
            let i = self.index;
            self.index += 1;
            if let Some(v) = self.pool.slots[i].as_ref() {
                return Some(v);
            }
        }
        None
    }
}
