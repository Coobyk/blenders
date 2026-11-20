//! Port of `generic_vector_array.cc` — `blender::GVectorArray`: an
//! array-of-vectors over type-erased (byte) elements.

use crate::generic_virtual_vector_array::{GVArray, GVVectorArray};

/// `GVectorArray` — each index owns a growable list of fixed-size elements.
pub struct GVectorArray {
    pub element_size: usize,
    items: Vec<Vec<u8>>,
}

impl GVectorArray {
    /// `GVectorArray(type, array_size)`.
    pub fn new(element_size: usize, array_size: usize) -> Self {
        GVectorArray {
            element_size,
            items: vec![Vec::new(); array_size],
        }
    }
    /// `varray.append(index, value)` — push `element_size` bytes.
    pub fn append(&mut self, index: usize, value: &[u8]) {
        self.items[index].extend_from_slice(&value[..self.element_size]);
    }
    /// `varray.extend(index, values)` — append all of a GVArray.
    pub fn extend(&mut self, index: usize, values: &dyn GVArray) {
        let mut buf = vec![0u8; self.element_size];
        for i in 0..values.size() {
            values.get(i, &mut buf);
            self.items[index].extend_from_slice(&buf);
        }
    }
    /// `varray.extend(mask, values)` — `for i in mask: extend(i, values[i])`.
    pub fn extend_masked(&mut self, mask: &[usize], values: &dyn GVVectorArray) {
        let mut buf = vec![0u8; self.element_size];
        for &i in mask {
            for j in 0..values.get_vector_size(i) {
                values.get_vector_element(i, j, &mut buf);
                self.items[i].extend_from_slice(&buf);
            }
        }
    }
    /// `varray.clear(mask)` — clear items at masked indices.
    pub fn clear_masked(&mut self, mask: &[usize]) {
        for &i in mask {
            self.items[i].clear();
        }
    }
    /// `varray.clear()` — clear all.
    pub fn clear(&mut self) {
        for it in self.items.iter_mut() {
            it.clear();
        }
    }
    /// `varray.items[index]` — raw byte view of one item's elements.
    pub fn item(&self, index: usize) -> &[u8] {
        &self.items[index]
    }
    /// `varray.items[index]` mutable.
    pub fn item_mut(&mut self, index: usize) -> &mut Vec<u8> {
        &mut self.items[index]
    }
    /// `realloc_to_at_least` — reserve capacity for one item.
    pub fn realloc_to_at_least(&mut self, index: usize, min_capacity: usize) {
        let cap = self.items[index].capacity();
        let need = min_capacity * self.element_size;
        if cap < need {
            self.items[index].reserve(need - cap);
        }
    }
}

impl GVVectorArray for GVectorArray {
    fn size(&self) -> usize {
        self.items.len()
    }
    fn get_vector_size(&self, index: usize) -> usize {
        self.items[index].len() / self.element_size
    }
    fn get_vector_element(&self, index: usize, index_in_vector: usize, r_value: &mut [u8]) {
        let o = index_in_vector * self.element_size;
        r_value[..self.element_size]
            .copy_from_slice(&self.items[index][o..o + self.element_size]);
    }
}
