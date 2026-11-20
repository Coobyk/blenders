//! Port of `BLI_heap_simple.cc` — minimal (float, ptr) min-heap.

/// `HEAP_PARENT`.
fn heap_parent(i: usize) -> usize {
    (i - 1) >> 1
}

/// `HeapSimple`.
pub struct HeapSimple<T> {
    tree: Vec<(f32, T)>,
}

impl<T> HeapSimple<T> {
    /// `BLI_heapsimple_new`.
    pub fn new() -> Self {
        HeapSimple { tree: Vec::new() }
    }
    /// `BLI_heapsimple_free` — Drop.
    /// `BLI_heapsimple_len`.
    pub fn len(&self) -> usize {
        self.tree.len()
    }
    /// `BLI_heapsimple_is_empty`.
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
    /// `BLI_heapsimple_top` — peek min value.
    pub fn top(&self) -> Option<f32> {
        self.tree.first().map(|&(v, _)| v)
    }
    /// `BLI_heapsimple_top_ptr`.
    pub fn top_ptr(&self) -> Option<&T> {
        self.tree.first().map(|(_, p)| p)
    }

    fn down(&mut self, mut i: usize) {
        loop {
            let (l, r) = (i * 2 + 1, i * 2 + 2);
            let mut smallest = i;
            if l < self.tree.len() && self.tree[l].0 < self.tree[smallest].0 {
                smallest = l;
            }
            if r < self.tree.len() && self.tree[r].0 < self.tree[smallest].0 {
                smallest = r;
            }
            if smallest == i {
                return;
            }
            self.tree.swap(i, smallest);
            i = smallest;
        }
    }
    fn up(&mut self, mut i: usize) {
        while i > 0 {
            let p = heap_parent(i);
            if self.tree[p].0 <= self.tree[i].0 {
                return;
            }
            self.tree.swap(i, p);
            i = p;
        }
    }

    /// `BLI_heapsimple_insert`.
    pub fn insert(&mut self, value: f32, ptr: T) {
        self.tree.push((value, ptr));
        self.up(self.tree.len() - 1);
    }
    /// `BLI_heapsimple_pop_min`.
    pub fn pop_min(&mut self) -> Option<T> {
        if self.tree.is_empty() {
            return None;
        }
        let (_, ptr) = self.tree.swap_remove(0);
        if !self.tree.is_empty() {
            self.down(0);
        }
        Some(ptr)
    }
    /// `BLI_heapsimple_remove_min`.
    pub fn remove_min(&mut self) -> bool {
        self.pop_min().is_some()
    }
    /// `BLI_heapsimple_is_valid`.
    pub fn is_valid(&self) -> bool {
        for i in (1..self.tree.len()).rev() {
            if self.tree[i].0 < self.tree[heap_parent(i)].0 {
                return false;
            }
        }
        true
    }
    /// `BLI_heapsimple_build` — heapify `values`/`ptrs` arrays.
    pub fn build(values: &[f32], ptrs: Vec<T>) -> Self {
        let mut h = HeapSimple {
            tree: values.iter().copied().zip(ptrs).collect(),
        };
        for i in (0..h.tree.len() / 2).rev() {
            h.down(i);
        }
        h
    }
}
