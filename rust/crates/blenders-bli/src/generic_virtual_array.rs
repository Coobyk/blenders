//! Port of `generic_virtual_array.cc` — the rest of the `GVArray`/
//! `GVMutableArray` machinery: `common_info`, `materialize*` over index
//! masks, `GSpan`/single/sliced impls, `GVArraySpan` and
//! `GMutableVArraySpan` owning wrappers.
//!
//! PORT_NOTE: the C `IndexMask` is represented as a sorted `&[i64]`
//! index list until `index_mask.cc` is ported; semantics match
//! (`mask.size()` = list length, indices ascending).

use crate::generic_virtual_vector_array::GVArray;

/// `CommonVArrayInfo::Type`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommonVArrayInfoType {
    Any,
    Span,
    Single,
}

/// `CommonVArrayInfo` — fast-path shared storage info.
pub struct CommonVArrayInfo {
    pub ty: CommonVArrayInfoType,
    pub may_have_ownership: bool,
    /// Element bytes; `Span` = base of contiguous storage,
    /// `Single` = pointer to the single value.
    pub data: *const u8,
}
unsafe impl Send for CommonVArrayInfo {}
unsafe impl Sync for CommonVArrayInfo {}

impl Default for CommonVArrayInfo {
    fn default() -> Self {
        CommonVArrayInfo {
            ty: CommonVArrayInfoType::Any,
            may_have_ownership: false,
            data: std::ptr::null(),
        }
    }
}

/// `GVArrayImpl` — extends `GVArray` with the implementation vtable.
pub trait GVArrayImpl: GVArray {
    /// `impl_->element_size()` via `type_->size`.
    fn element_size(&self) -> usize;
    /// `impl_->common_info()`.
    fn common_info(&self) -> CommonVArrayInfo {
        CommonVArrayInfo::default()
    }
    /// `impl_->materialize(mask, dst, dst_is_uninitialized)`.
    /// Default: per-element `get` like the base impl.
    fn materialize(&self, mask: &[i64], dst: &mut [u8], dst_is_uninitialized: bool) {
        let es = self.element_size();
        for (pos, &i) in mask.iter().enumerate() {
            let _ = pos;
            let d = &mut dst[(i as usize) * es..(i as usize + 1) * es];
            if dst_is_uninitialized {
                self.get_to_uninitialized(i as usize, d);
            } else {
                self.get(i as usize, d);
            }
        }
    }
    /// `impl_->materialize_compressed` — write to positions 0..mask.len().
    fn materialize_compressed(&self, mask: &[i64], dst: &mut [u8], dst_is_uninitialized: bool) {
        let es = self.element_size();
        for (pos, &i) in mask.iter().enumerate() {
            let d = &mut dst[pos * es..(pos + 1) * es];
            if dst_is_uninitialized {
                self.get_to_uninitialized(i as usize, d);
            } else {
                self.get(i as usize, d);
            }
        }
    }
    /// `impl_->try_assign_VArray`.
    fn try_assign_varray(&self) -> bool {
        false
    }
}

/// `GVMutableArrayImpl` — mutable impl vtable.
pub trait GVMutableArrayImpl: GVArrayImpl {
    /// `set_by_copy` — copy element bytes into `value`'s slot.
    fn set_by_copy(&mut self, index: usize, value: &[u8]);
    /// `set_by_move` — same as copy for trivial types.
    fn set_by_move(&mut self, index: usize, value: &[u8]) {
        self.set_by_copy(index, value);
    }
    /// `set_by_relocate`.
    fn set_by_relocate(&mut self, index: usize, value: &mut [u8]) {
        self.set_by_copy(index, value);
    }
    /// `set_all` — copy `src` (size*elem_size bytes) into every element.
    fn set_all(&mut self, src: &[u8]);
    /// `try_assign_VMutableArray`.
    fn try_assign_vmutable_array(&self) -> bool {
        false
    }
}

/// `GVArrayImpl_For_GSpan` — virtual array over an existing span.
pub struct GVArrayImplForGSpan<'a> {
    pub element_size: usize,
    pub data: &'a [u8],
    pub size: usize,
    pub may_have_ownership: bool,
}

impl<'a> GVArray for GVArrayImplForGSpan<'a> {
    fn size(&self) -> usize {
        self.size
    }
    fn get(&self, index: usize, r_value: &mut [u8]) {
        r_value.copy_from_slice(&self.data[index * self.element_size..(index + 1) * self.element_size]);
    }
    fn get_to_uninitialized(&self, index: usize, r_value: &mut [u8]) {
        self.get(index, r_value);
    }
}

impl<'a> GVArrayImpl for GVArrayImplForGSpan<'a> {
    fn element_size(&self) -> usize {
        self.element_size
    }
    fn common_info(&self) -> CommonVArrayInfo {
        CommonVArrayInfo {
            ty: CommonVArrayInfoType::Span,
            may_have_ownership: self.may_have_ownership,
            data: self.data.as_ptr(),
        }
    }
    fn materialize(&self, mask: &[i64], dst: &mut [u8], _uninit: bool) {
        let es = self.element_size;
        for &i in mask {
            dst[(i as usize) * es..(i as usize + 1) * es]
                .copy_from_slice(&self.data[(i as usize) * es..(i as usize + 1) * es]);
        }
    }
    fn materialize_compressed(&self, mask: &[i64], dst: &mut [u8], _uninit: bool) {
        let es = self.element_size;
        for (pos, &i) in mask.iter().enumerate() {
            dst[pos * es..(pos + 1) * es]
                .copy_from_slice(&self.data[(i as usize) * es..(i as usize + 1) * es]);
        }
    }
}

/// `GVMutableArrayImpl_For_GMutableSpan` — mutable span impl.
pub struct GVMutableArrayImplForGMutableSpan<'a> {
    pub element_size: usize,
    pub data: &'a mut [u8],
    pub size: usize,
    pub may_have_ownership: bool,
}

impl<'a> GVArray for GVMutableArrayImplForGMutableSpan<'a> {
    fn size(&self) -> usize {
        self.size
    }
    fn get(&self, index: usize, r_value: &mut [u8]) {
        r_value.copy_from_slice(&self.data[index * self.element_size..(index + 1) * self.element_size]);
    }
}

impl<'a> GVArrayImpl for GVMutableArrayImplForGMutableSpan<'a> {
    fn element_size(&self) -> usize {
        self.element_size
    }
    fn common_info(&self) -> CommonVArrayInfo {
        CommonVArrayInfo {
            ty: CommonVArrayInfoType::Span,
            may_have_ownership: self.may_have_ownership,
            data: self.data.as_ptr(),
        }
    }
}

impl<'a> GVMutableArrayImpl for GVMutableArrayImplForGMutableSpan<'a> {
    fn set_by_copy(&mut self, index: usize, value: &[u8]) {
        self.data[index * self.element_size..(index + 1) * self.element_size]
            .copy_from_slice(value);
    }
    fn set_by_relocate(&mut self, index: usize, value: &mut [u8]) {
        self.set_by_copy(index, value);
    }
    fn set_all(&mut self, src: &[u8]) {
        self.data[..self.size * self.element_size].copy_from_slice(src);
    }
}

/// `GVArrayImpl_For_SingleValueRef` — non-owning single-value impl.
pub struct GVArrayImplForSingleValueRef<'a> {
    pub element_size: usize,
    pub value: &'a [u8],
    pub size: usize,
}

impl<'a> GVArray for GVArrayImplForSingleValueRef<'a> {
    fn size(&self) -> usize {
        self.size
    }
    fn get(&self, _index: usize, r_value: &mut [u8]) {
        r_value.copy_from_slice(self.value);
    }
}

impl<'a> GVArrayImpl for GVArrayImplForSingleValueRef<'a> {
    fn element_size(&self) -> usize {
        self.element_size
    }
    fn common_info(&self) -> CommonVArrayInfo {
        CommonVArrayInfo {
            ty: CommonVArrayInfoType::Single,
            may_have_ownership: true,
            data: self.value.as_ptr(),
        }
    }
    /// `materialize` — `fill_assign_indices`/`fill_construct_indices`.
    fn materialize(&self, mask: &[i64], dst: &mut [u8], _uninit: bool) {
        let es = self.element_size;
        for &i in mask {
            dst[(i as usize) * es..(i as usize + 1) * es].copy_from_slice(self.value);
        }
    }
    fn materialize_compressed(&self, mask: &[i64], dst: &mut [u8], _uninit: bool) {
        let es = self.element_size;
        for pos in 0..mask.len() {
            dst[pos * es..(pos + 1) * es].copy_from_slice(self.value);
        }
    }
}

/// `GVArrayImpl_For_SingleValue` / `For_SmallTrivialSingleValue` —
/// owning variants (merged: an owned `Vec<u8>` covers both paths).
pub struct GVArrayImplForSingleValue {
    pub element_size: usize,
    pub value: Vec<u8>,
    pub size: usize,
}

impl GVArray for GVArrayImplForSingleValue {
    fn size(&self) -> usize {
        self.size
    }
    fn get(&self, _index: usize, r_value: &mut [u8]) {
        r_value.copy_from_slice(&self.value);
    }
}

impl GVArrayImpl for GVArrayImplForSingleValue {
    fn element_size(&self) -> usize {
        self.element_size
    }
    fn common_info(&self) -> CommonVArrayInfo {
        CommonVArrayInfo {
            ty: CommonVArrayInfoType::Single,
            may_have_ownership: true,
            data: self.value.as_ptr(),
        }
    }
    fn materialize(&self, mask: &[i64], dst: &mut [u8], _uninit: bool) {
        let es = self.element_size;
        for &i in mask {
            dst[(i as usize) * es..(i as usize + 1) * es].copy_from_slice(&self.value);
        }
    }
    fn materialize_compressed(&self, mask: &[i64], dst: &mut [u8], _uninit: bool) {
        let es = self.element_size;
        for pos in 0..mask.len() {
            dst[pos * es..(pos + 1) * es].copy_from_slice(&self.value);
        }
    }
}

/// `GVArrayImpl_For_SlicedGVArray` — `varray_.slice(start, size)` view.
pub struct GVArrayImplForSlicedGVArray {
    pub varray: Box<dyn GVArrayImpl>,
    pub offset: i64,
    pub slice_size: usize,
    pub element_size: usize,
}

impl GVArray for GVArrayImplForSlicedGVArray {
    fn size(&self) -> usize {
        self.slice_size
    }
    fn get(&self, index: usize, r_value: &mut [u8]) {
        self.varray.get(index + self.offset as usize, r_value);
    }
    fn get_to_uninitialized(&self, index: usize, r_value: &mut [u8]) {
        self.varray
            .get_to_uninitialized(index + self.offset as usize, r_value);
    }
}

impl GVArrayImpl for GVArrayImplForSlicedGVArray {
    fn element_size(&self) -> usize {
        self.element_size
    }
    fn common_info(&self) -> CommonVArrayInfo {
        let info = self.varray.common_info();
        match info.ty {
            CommonVArrayInfoType::Any => CommonVArrayInfo::default(),
            CommonVArrayInfoType::Span => CommonVArrayInfo {
                ty: CommonVArrayInfoType::Span,
                may_have_ownership: info.may_have_ownership,
                data: unsafe { info.data.add(self.element_size * self.offset as usize) },
            },
            CommonVArrayInfoType::Single => info,
        }
    }
    fn materialize(&self, mask: &[i64], dst: &mut [u8], uninit: bool) {
        // `mask.shift(offset_)` + `shifted_dst = dst - offset*es`.
        let es = self.element_size;
        let shifted: Vec<i64> = mask.iter().map(|&i| i + self.offset).collect();
        for (pos, &i) in shifted.iter().enumerate() {
            let d = &mut dst[(mask[pos] as usize) * es..(mask[pos] as usize + 1) * es];
            if uninit {
                self.varray.get_to_uninitialized(i as usize, d);
            } else {
                self.varray.get(i as usize, d);
            }
        }
    }
    fn materialize_compressed(&self, mask: &[i64], dst: &mut [u8], uninit: bool) {
        let shifted: Vec<i64> = mask.iter().map(|&i| i + self.offset).collect();
        self.varray.materialize_compressed(&shifted, dst, uninit);
    }
}

/// `GVArrayCommon` — owning handle methods (`materialize*`,
/// `is_span`, `get_internal_span`, `is_single`,
/// `get_internal_single`, `index_range`).
pub struct GVArrayCommon<T: GVArrayImpl + ?Sized> {
    pub impl_: T,
}

impl<T: GVArrayImpl + ?Sized> GVArrayCommon<T> {
    /// `materialize(dst)` over the full range.
    pub fn materialize(&self, dst: &mut [u8]) {
        let mask: Vec<i64> = (0..self.impl_.size() as i64).collect();
        self.impl_.materialize(&mask, dst, false);
    }
    pub fn materialize_masked(&self, mask: &[i64], dst: &mut [u8]) {
        self.impl_.materialize(mask, dst, false);
    }
    pub fn materialize_to_uninitialized(&self, dst: &mut [u8]) {
        let mask: Vec<i64> = (0..self.impl_.size() as i64).collect();
        self.impl_.materialize(&mask, dst, true);
    }
    pub fn materialize_compressed(&self, mask: &[i64], dst: &mut [u8]) {
        self.impl_.materialize_compressed(mask, dst, false);
    }
    pub fn materialize_compressed_to_uninitialized(&self, mask: &[i64], dst: &mut [u8]) {
        self.impl_.materialize_compressed(mask, dst, true);
    }
    pub fn is_span(&self) -> bool {
        self.impl_.common_info().ty == CommonVArrayInfoType::Span
    }
    pub fn is_single(&self) -> bool {
        self.impl_.common_info().ty == CommonVArrayInfoType::Single
    }
    pub fn index_range(&self) -> std::ops::Range<usize> {
        0..self.impl_.size()
    }
}

/// `GVArraySpan` — span view; materializes when not already a span.
pub struct GVArraySpan {
    pub size: usize,
    pub element_size: usize,
    /// Element bytes — either borrowed from the impl (raw) or owned.
    pub owned_data: Option<Vec<u8>>,
    pub data_ptr: *const u8,
    /// Keep the source impl alive while the span views its storage.
    _varray: Option<Box<dyn GVArrayImpl>>,
}
unsafe impl Send for GVArraySpan {}

impl GVArraySpan {
    /// `GVArraySpan(varray)` ctor.
    pub fn new(varray: Box<dyn GVArrayImpl>) -> Self {
        let size = varray.size();
        let element_size = varray.element_size();
        let info = varray.common_info();
        if info.ty == CommonVArrayInfoType::Span {
            GVArraySpan {
                size,
                element_size,
                owned_data: None,
                data_ptr: info.data,
                _varray: Some(varray),
            }
        } else {
            let mut owned = vec![0u8; element_size * size];
            let mask: Vec<i64> = (0..size as i64).collect();
            varray.materialize(&mask, &mut owned, true);
            let data_ptr = owned.as_ptr();
            GVArraySpan {
                size,
                element_size,
                owned_data: Some(owned),
                data_ptr,
                _varray: Some(varray),
            }
        }
    }
    /// Element bytes as a slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data_ptr, self.size * self.element_size) }
    }
}

/// `GMutableVArraySpan` — mutable span view with `save()`.
pub struct GMutableVArraySpan {
    pub size: usize,
    pub element_size: usize,
    pub owned_data: Option<Vec<u8>>,
    pub data_ptr: *mut u8,
    pub varray: Box<dyn GVMutableArrayImpl>,
    pub show_not_saved_warning: bool,
    pub save_has_been_called: bool,
}
unsafe impl Send for GMutableVArraySpan {}

impl GMutableVArraySpan {
    /// `GMutableVArraySpan(varray, copy_values_to_span)`.
    pub fn new(varray: Box<dyn GVMutableArrayImpl>, copy_values_to_span: bool) -> Self {
        let size = varray.size();
        let element_size = varray.element_size();
        let info = varray.common_info();
        if info.ty == CommonVArrayInfoType::Span {
            GMutableVArraySpan {
                size,
                element_size,
                owned_data: None,
                data_ptr: info.data as *mut u8,
                varray,
                show_not_saved_warning: true,
                save_has_been_called: false,
            }
        } else {
            let mut owned = vec![0u8; element_size * size];
            if copy_values_to_span {
                let mask: Vec<i64> = (0..size as i64).collect();
                varray.materialize(&mask, &mut owned, true);
            }
            let data_ptr = owned.as_mut_ptr();
            GMutableVArraySpan {
                size,
                element_size,
                owned_data: Some(owned),
                data_ptr,
                varray,
                show_not_saved_warning: true,
                save_has_been_called: false,
            }
        }
    }
    /// `save()` — write the span back through `varray_.set_all` when
    /// this span owns the data (writes to shared storage are direct).
    pub fn save(&mut self) {
        self.save_has_been_called = true;
        if let Some(owned) = &self.owned_data {
            let src = owned.clone();
            self.varray.set_all(&src);
        }
    }
    /// `disable_not_applied_warning`.
    pub fn disable_not_applied_warning(&mut self) {
        self.show_not_saved_warning = false;
    }
    /// Mutable element bytes.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data_ptr, self.size * self.element_size) }
    }
}

impl Drop for GMutableVArraySpan {
    fn drop(&mut self) {
        if self.show_not_saved_warning && !self.save_has_been_called && self.owned_data.is_some() {
            eprintln!("Warning: Call `save()` to make sure that changes persist in all cases.");
        }
    }
}
