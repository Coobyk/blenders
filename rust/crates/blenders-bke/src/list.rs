//! Port of `intern/list.cc` — `bke::GList` (generic-value list over
//! `CPPType` with implicit sharing).
//!
//! `GArray<>`/`GPointer`/`GVArray` aren't ported — element storage is
//! `Vec<u8>` inside `ImplicitSharingInfoAndData`; `GPointer` is
//! `(*const u8, &CppType)`; `varray`/`values` expose raw byte access.
//! All value ops go through `CppType`'s vtable verbatim.

use blenders_bli::cpp_type::CppType;
use blenders_bli::implicit_sharing::{ImplicitSharingInfo, ImplicitSharingInfoAndData};

/// `GPointer` — (type, value ptr).
#[derive(Clone, Copy)]
pub struct GPointer {
    pub type_: &'static CppType,
    pub value: *const u8,
}
unsafe impl Send for GPointer {}
unsafe impl Sync for GPointer {}
impl GPointer {
    pub fn get(&self) -> *const u8 { self.value }
}

/// `GList::ArrayData` — `data` lives in `sharing_info.data`.
pub struct ArrayData {
    /// `sharing_info` (owns `data`).
    pub sharing_info: Box<ImplicitSharingInfoAndData>,
}
/// `GList::SingleData` — `value` lives in `sharing_info.data`.
pub struct SingleData {
    /// `sharing_info`.
    pub sharing_info: Box<ImplicitSharingInfoAndData>,
}
/// `GList::DataVariant`.
pub enum DataVariant {
    Array(ArrayData),
    Single(SingleData),
}

/// `ImplicitSharedValue` byte-buffer arena — keeps sharing infos alive.
type SharingBox = Box<ImplicitSharingInfoAndData>;

/// `memory_is_zero`.
fn memory_is_zero(value: *const u8, size: usize) -> bool {
    (0..size).all(|i| unsafe { *value.add(i) } == 0)
}

/// `array_data_from_garray` — wrap an element buffer in a new
/// `ImplicitSharedValue` (mem-free owned).
fn array_data_from_vec(data: Vec<u8>) -> SharingBox {
    ImplicitSharingInfoAndData::new_mem_free(data)
}

fn alloc(type_: &CppType, size: usize) -> Vec<u8> {
    vec![0u8; type_.size * size]
}
fn alloc_uninit(type_: &CppType, size: usize) -> Vec<u8> {
    vec![0u8; type_.size * size]
}
/// `type.fill_construct_n(value, dst, n)`.
unsafe fn fill_construct_n(type_: &CppType, value: *const u8, dst: *mut u8, n: usize) {
    if let Some(f) = type_.fill_construct_n {
        f(value, dst, n);
    } else if type_.is_trivially_copyable() {
        for i in 0..n {
            std::ptr::copy_nonoverlapping(value, dst.add(i * type_.size), type_.size);
        }
    }
}
/// `type.copy_construct_n`.
unsafe fn copy_construct_n(type_: &CppType, src: *const u8, dst: *mut u8, n: usize) {
    if let Some(f) = type_.copy_construct_n {
        f(src, dst, n);
    } else if type_.is_trivially_copyable() {
        std::ptr::copy_nonoverlapping(src, dst, n * type_.size);
    }
}
/// `type.default_construct_n`.
unsafe fn default_construct_n(type_: &CppType, dst: *mut u8, n: usize) {
    if let Some(f) = type_.default_construct_n {
        f(dst, n);
    } else {
        std::ptr::write_bytes(dst, 0, n * type_.size);
    }
}
/// `type.copy_construct`.
unsafe fn copy_construct(type_: &CppType, src: *const u8, dst: *mut u8) {
    if let Some(f) = type_.copy_construct {
        f(src, dst);
    } else {
        std::ptr::copy_nonoverlapping(src, dst, type_.size);
    }
}

impl ArrayData {
    /// `ArrayData::ForValue` — zero check → zeroed buffer; else fill.
    pub fn for_value(type_: &'static CppType, value: *const u8, size: usize) -> Self {
        if value.is_null() || memory_is_zero(value, type_.size) {
            return Self { sharing_info: array_data_from_vec(alloc(type_, size)) };
        }
        let mut buf = alloc_uninit(type_, size);
        unsafe { fill_construct_n(type_, value, buf.as_mut_ptr(), size) };
        Self { sharing_info: array_data_from_vec(buf) }
    }
    /// `ForDefaultValue`.
    pub fn for_default_value(type_: &'static CppType, size: usize) -> Self {
        let mut dv = alloc(type_, 1);
        unsafe { (type_.default_value)(dv.as_mut_ptr()) };
        Self::for_value(type_, dv.as_ptr(), size)
    }
    /// `ForConstructed` — default-construct `size` elements.
    pub fn for_constructed(type_: &'static CppType, size: usize) -> Self {
        let mut buf = alloc_uninit(type_, size);
        unsafe { default_construct_n(type_, buf.as_mut_ptr(), size) };
        Self { sharing_info: array_data_from_vec(buf) }
    }
    /// `ForUninitialized`.
    pub fn for_uninitialized(type_: &'static CppType, size: usize) -> Self {
        Self { sharing_info: array_data_from_vec(alloc_uninit(type_, size)) }
    }
    /// `count_memory` → returns `type.size * size` bytes shared.
    pub fn count_memory(&self, type_: &CppType, size: usize) -> usize {
        type_.size * size
    }
    /// `span_for_write` — copy-on-write (sharing_info not mutable →
    /// fresh copy), `tag_ensured_mutable` afterwards.
    pub fn ensure_writable(&mut self, type_: &CppType, size: usize) {
        if !self.sharing_info.is_mutable() {
            let old = self.sharing_info.data.read().unwrap().clone().unwrap_or_default();
            let mut new_buf = alloc_uninit(type_, size);
            unsafe {
                copy_construct_n(type_, old.as_ptr(), new_buf.as_mut_ptr(), size.min(old.len() / type_.size));
            }
            self.sharing_info = array_data_from_vec(new_buf);
        }
        self.sharing_info.tag_ensured_mutable();
    }
    /// Read access to the element buffer.
    pub fn data(&self) -> std::sync::RwLockReadGuard<'_, Option<Vec<u8>>> {
        self.sharing_info.data.read().unwrap()
    }
}

impl SingleData {
    /// `SingleData::ForValue` — copy-construct one element.
    pub fn for_value(type_: &'static CppType, value: *const u8) -> Self {
        let mut buf = alloc(type_, 1);
        unsafe { copy_construct(type_, value, buf.as_mut_ptr()) };
        Self { sharing_info: array_data_from_vec(buf) }
    }
    /// `ForDefaultValue`.
    pub fn for_default_value(type_: &'static CppType) -> Self {
        let mut dv = alloc(type_, 1);
        unsafe { (type_.default_value)(dv.as_mut_ptr()) };
        Self::for_value(type_, dv.as_ptr())
    }
    /// `count_memory` → `type.size`.
    pub fn count_memory(&self, type_: &CppType) -> usize {
        type_.size
    }
    /// `value_for_write` — copy-on-write single value.
    pub fn ensure_writable(&mut self, type_: &'static CppType) {
        if !self.sharing_info.is_mutable() {
            let old = self.sharing_info.data.read().unwrap().clone().unwrap_or_default();
            *self = SingleData::for_value(type_, old.as_ptr());
        }
        self.sharing_info.tag_ensured_mutable();
    }
    /// Read access to the value bytes.
    pub fn data(&self) -> std::sync::RwLockReadGuard<'_, Option<Vec<u8>>> {
        self.sharing_info.data.read().unwrap()
    }
}

/// `GList`.
pub struct GList {
    /// `cpp_type_`.
    pub cpp_type: &'static CppType,
    /// `data_` — `std::variant<ArrayData, SingleData>`; `None` = moved-out.
    pub data: Option<DataVariant>,
    /// `size_` — element count.
    pub size: i64,
}

/// `GList::values()` — variant view of the data.
pub enum GListValues {
    /// `GSpan`.
    Span { data: Vec<u8>, elem_size: usize, size: i64 },
    /// `GPointer`.
    Pointer(GPointer),
}
/// `values_for_write`.
pub enum GListValuesMut {
    /// `GMutableSpan`.
    Span { data: Vec<u8>, elem_size: usize, size: i64 },
    /// `GMutablePointer`.
    Pointer { type_: &'static CppType, value: Vec<u8> },
}

impl GList {
    /// `GList(type, data, size)`.
    pub fn new(cpp_type: &'static CppType, data: DataVariant, size: i64) -> Self {
        Self { cpp_type, data: Some(data), size }
    }
    /// `GList(GArray)` — array data.
    pub fn from_vec(cpp_type: &'static CppType, buf: Vec<u8>, size: i64) -> Self {
        Self {
            cpp_type,
            data: Some(DataVariant::Array(ArrayData { sharing_info: array_data_from_vec(buf) })),
            size,
        }
    }
    /// `GList::from_single(value, size)`.
    pub fn from_single(value: GPointer, size: i64) -> Self {
        Self {
            cpp_type: value.type_,
            data: Some(DataVariant::Single(SingleData::for_value(value.type_, value.value))),
            size,
        }
    }

    /// `resize` — copy existing + default-construct the tail.
    pub fn resize(&mut self, new_size: i64) {
        if let Some(DataVariant::Array(array_data)) = &self.data {
            let type_ = self.cpp_type;
            let mut new_buf = alloc_uninit(type_, new_size as usize);
            let old = array_data.sharing_info.data.read().unwrap().clone().unwrap_or_default();
            unsafe {
                let keep = self.size.min(new_size) as usize;
                copy_construct_n(type_, old.as_ptr(), new_buf.as_mut_ptr(), keep);
                if self.size < new_size {
                    default_construct_n(type_, new_buf.as_mut_ptr().add(keep * type_.size), (new_size - self.size) as usize);
                }
            }
            self.data = Some(DataVariant::Array(ArrayData { sharing_info: array_data_from_vec(new_buf) }));
        }
        self.size = new_size;
    }

    /// `ensure_array_storage` — Single → Array.
    pub fn ensure_array_storage(&mut self) {
        if let Some(DataVariant::Single(single_data)) = &self.data {
            let old = single_data.sharing_info.data.read().unwrap().clone().unwrap_or_default();
            self.data = Some(DataVariant::Array(ArrayData::for_value(
                self.cpp_type,
                old.as_ptr(),
                self.size as usize,
            )));
        }
    }

    /// `varray` — byte snapshot of the storage.
    pub fn varray(&self) -> Vec<u8> {
        match &self.data {
            Some(DataVariant::Array(a)) => {
                a.sharing_info.data.read().unwrap().clone().unwrap_or_default()
            }
            Some(DataVariant::Single(s)) => {
                s.sharing_info.data.read().unwrap().clone().unwrap_or_default()
            }
            None => Vec::new(),
        }
    }

    /// `count_memory` — bytes to add.
    pub fn count_memory(&self) -> usize {
        match &self.data {
            Some(DataVariant::Array(a)) => a.count_memory(self.cpp_type, self.size as usize),
            Some(DataVariant::Single(s)) => s.count_memory(self.cpp_type),
            None => 0,
        }
    }

    /// `values` — variant view (snapshot).
    pub fn values(&self) -> GListValues {
        match &self.data {
            Some(DataVariant::Array(a)) => GListValues::Span {
                data: a.sharing_info.data.read().unwrap().clone().unwrap_or_default(),
                elem_size: self.cpp_type.size,
                size: self.size,
            },
            Some(DataVariant::Single(s)) => GListValues::Pointer(GPointer {
                type_: self.cpp_type,
                value: s.sharing_info.data.read().unwrap().as_ref().map(|v| v.as_ptr()).unwrap_or(std::ptr::null()),
            }),
            None => unreachable!("BLI_assert_unreachable"),
        }
    }

    /// `values_for_write` — copy-on-write then expose mutable data.
    pub fn values_for_write(&mut self) -> GListValuesMut {
        match self.data.as_mut().unwrap() {
            DataVariant::Array(a) => {
                a.ensure_writable(self.cpp_type, self.size as usize);
                GListValuesMut::Span {
                    data: a.sharing_info.data.read().unwrap().clone().unwrap_or_default(),
                    elem_size: self.cpp_type.size,
                    size: self.size,
                }
            }
            DataVariant::Single(s) => {
                s.ensure_writable(self.cpp_type);
                GListValuesMut::Pointer {
                    type_: self.cpp_type,
                    value: s.sharing_info.data.read().unwrap().clone().unwrap_or_default(),
                }
            }
        }
    }
}
