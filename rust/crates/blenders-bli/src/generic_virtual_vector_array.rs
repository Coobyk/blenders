//! Port of `generic_virtual_vector_array.cc` —
//! `GVArray_For_GVVectorArrayIndex`, `GVVectorArray_For_SingleGVArray`,
//! `GVVectorArray_For_SingleGSpan`: virtual-array adapters.
//!
//! `GVArray`/`GVVectorArray` are trait objects: `get(i)` writes the
//! element into a `Vec<u8>` via `copy_assign` semantics.

/// `GVArray` — virtual array over type-erased values (byte buffers).
pub trait GVArray {
    /// `varray.size()`.
    fn size(&self) -> usize;
    /// `varray.get(index, r_value)` — copy element bytes into `r_value`.
    fn get(&self, index: usize, r_value: &mut [u8]);
    /// `varray.get_to_uninitialized` — default-init then copy.
    fn get_to_uninitialized(&self, index: usize, r_value: &mut [u8]) {
        self.get(index, r_value);
    }
}

/// `GVVectorArray` — virtual array of vectors.
pub trait GVVectorArray {
    /// `varray.size()` (outer count).
    fn size(&self) -> usize;
    /// `get_vector_size(index)`.
    fn get_vector_size(&self, index: usize) -> usize;
    /// `get_vector_element(index, index_in_vector, r_value)`.
    fn get_vector_element(&self, index: usize, index_in_vector: usize, r_value: &mut [u8]);
    /// `is_single_vector`.
    fn is_single_vector(&self) -> bool {
        false
    }
}

/// `GVArray_For_GVVectorArrayIndex` — one vector of a vector-array
/// viewed as a flat array.
pub struct GVArrayForGVVectorArrayIndex<'a> {
    pub vector_array: &'a dyn GVVectorArray,
    pub index: usize,
}

impl<'a> GVArray for GVArrayForGVVectorArrayIndex<'a> {
    fn size(&self) -> usize {
        self.vector_array.get_vector_size(self.index)
    }
    fn get(&self, index_in_vector: usize, r_value: &mut [u8]) {
        self.vector_array
            .get_vector_element(self.index, index_in_vector, r_value);
    }
}

/// `GVVectorArray_For_SingleGVArray` — a 1-outer vector array wrapping
/// a flat GVArray.
pub struct GVVectorArrayForSingleGVArray<'a> {
    pub varray: &'a dyn GVArray,
}

impl<'a> GVVectorArray for GVVectorArrayForSingleGVArray<'a> {
    fn size(&self) -> usize {
        1
    }
    fn get_vector_size(&self, _index: usize) -> usize {
        self.varray.size()
    }
    fn get_vector_element(&self, _index: usize, index_in_vector: usize, r_value: &mut [u8]) {
        self.varray.get(index_in_vector, r_value);
    }
    fn is_single_vector(&self) -> bool {
        true
    }
}

/// `GVVectorArray_For_SingleGSpan` — a 1-outer vector array over a
/// contiguous byte span of fixed-size elements.
pub struct GVVectorArrayForSingleGSpan<'a> {
    pub span: &'a [u8],
    pub elem_size: usize,
}

impl<'a> GVVectorArray for GVVectorArrayForSingleGSpan<'a> {
    fn size(&self) -> usize {
        1
    }
    fn get_vector_size(&self, _index: usize) -> usize {
        self.span.len() / self.elem_size
    }
    fn get_vector_element(&self, _index: usize, index_in_vector: usize, r_value: &mut [u8]) {
        let o = index_in_vector * self.elem_size;
        r_value[..self.elem_size].copy_from_slice(&self.span[o..o + self.elem_size]);
    }
    fn is_single_vector(&self) -> bool {
        true
    }
}
