//! Port of `array_store_utils.cc` — `BArrayStore_AtSize` per-stride
//! store table (`BLI_array_store_at_size_ensure/get/clear/
//! calc_memory_usage`).

/// `BArrayStore_AtSize` — `stride_table[i]` is the store for stride `i+1`.
pub struct BArrayStoreAtSize<T> {
    pub stride_table: Vec<Option<T>>,
}

impl<T> BArrayStoreAtSize<T> {
    pub fn new() -> Self {
        BArrayStoreAtSize {
            stride_table: Vec::new(),
        }
    }
    /// `BLI_array_store_at_size_ensure` — get-or-create store for `stride`.
    /// `make` builds the store when absent (C sizes it to a power of two).
    pub fn ensure(&mut self, stride: usize, make: impl FnOnce(usize) -> T) -> &mut T {
        if self.stride_table.len() < stride {
            self.stride_table.resize_with(stride, || None);
        }
        let slot = &mut self.stride_table[stride - 1];
        if slot.is_none() {
            *slot = Some(make(stride));
        }
        slot.as_mut().unwrap()
    }
    /// `BLI_array_store_at_size_get`.
    pub fn get(&self, stride: usize) -> Option<&T> {
        assert!(stride > 0 && stride <= self.stride_table.len());
        self.stride_table[stride - 1].as_ref()
    }
    /// `BLI_array_store_at_size_clear`.
    pub fn clear(&mut self) {
        self.stride_table.clear();
    }
    /// `BLI_array_store_at_size_calc_memory_usage` — caller supplies the
    /// per-store `(expanded, compacted)` byte counts.
    pub fn calc_memory_usage<F: Fn(&T) -> (usize, usize)>(
        &self,
        f: F,
    ) -> (usize, usize) {
        let mut expanded = 0;
        let mut compacted = 0;
        for s in self.stride_table.iter().flatten() {
            let (e, c) = f(s);
            expanded += e;
            compacted += c;
        }
        (expanded, compacted)
    }
}

impl<T> Default for BArrayStoreAtSize<T> {
    fn default() -> Self {
        Self::new()
    }
}
