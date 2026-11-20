//! Port of `cpp_type.cc` + `cpp_types.cc` — `blender::CPPType` runtime
//! type info and the built-in type registry (`register_cpp_types`,
//! `CPPTypeFlags`, `to_string`, `print_or_default`).

/// `CPPTypeFlags` — capability bits on a registered type.
pub mod cpp_type_flags {
    /// `CPPTypeFlags::None`.
    pub const NONE: u32 = 0;
    /// `CPPTypeFlags::BasicType` — the type has print/hash/move support.
    pub const BASIC_TYPE: u32 = 1 << 0;
    /// `CPPTypeFlags::IdentityDefaultValue` — default != all-zero.
    pub const IDENTITY_DEFAULT_VALUE: u32 = 1 << 1;
    /// `CPPTypeFlags::SupportsGenericInterface` (get/set any field by name).
    pub const SUPPORTS_GENERIC_INTERFACE: u32 = 1 << 2;
}

/// `UniqueHashBytes` — accumulated hash bytes for `hash_unique_default`.
#[derive(Default)]
pub struct UniqueHashBytes {
    pub data: Vec<u8>,
}

/// `hash_unique_default<std::string>` — append raw bytes.
pub fn hash_unique_default_str(value: &str, hash: &mut UniqueHashBytes) {
    hash.data.extend_from_slice(value.as_bytes());
}

/// `CPPType` — per-type vtable of value operations.
/// `value`/`other` are raw pointers to `size` bytes as in C.
pub struct CppType {
    pub name: &'static str,
    pub size: usize,
    pub alignment: usize,
    pub flags: u32,
    /// `default_value` — writes `size` bytes to `r_value`.
    pub default_value: unsafe fn(*mut u8),
    /// `default_value_name` — e.g. "0" or a printable default.
    pub default_value_name: &'static str,
    /// `default_value_name` for indexing (arrays).
    pub default_value_indices: &'static str,
    /// `print` — stringize value into `ss` (only when `is_printable()`).
    pub print: Option<unsafe fn(*const u8, &mut String)>,
    /// `hash` — stable hash of a value (MM3-compatible seed in C).
    pub hash: Option<unsafe fn(*const u8) -> u64>,
    /// `is_equal` — deep equality.
    pub is_equal: Option<unsafe fn(*const u8, *const u8) -> bool>,
    /// `is_equal_to` / `is_uniquely_represented` checks.
    pub is_uniquely_represented: bool,
    /// `default_construct` — placement default init.
    pub default_construct: Option<unsafe fn(*mut u8)>,
    /// `default_construct_n`.
    pub default_construct_n: Option<unsafe fn(*mut u8, usize)>,
    /// `default_construct_indices` — init `n` elements at `indexes`.
    pub default_construct_indices: Option<unsafe fn(*mut u8, *const i64, usize)>,
    /// `destruct` / `destruct_n` / `destruct_indices`.
    pub destruct: Option<unsafe fn(*mut u8)>,
    pub destruct_n: Option<unsafe fn(*mut u8, usize)>,
    pub destruct_indices: Option<unsafe fn(*mut u8, *const i64, usize)>,
    /// `copy_assign` / `copy_assign_n` / `copy_assign_indices` / `copy_assign_compressed`.
    pub copy_assign: Option<unsafe fn(*const u8, *mut u8)>,
    pub copy_assign_n: Option<unsafe fn(*const u8, *mut u8, usize)>,
    pub copy_assign_indices: Option<unsafe fn(*const u8, *mut u8, *const i64, usize)>,
    /// `copy_construct` / `copy_construct_n` / `copy_construct_indices`.
    pub copy_construct: Option<unsafe fn(*const u8, *mut u8)>,
    pub copy_construct_n: Option<unsafe fn(*const u8, *mut u8, usize)>,
    pub copy_construct_indices: Option<unsafe fn(*const u8, *mut u8, *const i64, usize)>,
    /// `move_assign` / `move_construct` / `relocate_*`.
    pub move_assign: Option<unsafe fn(*mut u8, *mut u8)>,
    pub move_assign_indices: Option<unsafe fn(*mut u8, *mut u8, *const i64, usize)>,
    pub move_construct: Option<unsafe fn(*mut u8, *mut u8)>,
    pub move_construct_indices: Option<unsafe fn(*mut u8, *mut u8, *const i64, usize)>,
    pub relocate_assign: Option<unsafe fn(*mut u8, *mut u8)>,
    pub relocate_assign_indices: Option<unsafe fn(*mut u8, *mut u8, *const i64, usize)>,
    pub relocate_assign_n: Option<unsafe fn(*mut u8, *mut u8, usize)>,
    pub relocate_assign_compressed: Option<unsafe fn(*mut u8, *mut u8, *const i64, usize)>,
    pub relocate_construct: Option<unsafe fn(*mut u8, *mut u8)>,
    pub relocate_construct_indices: Option<unsafe fn(*mut u8, *mut u8, *const i64, usize)>,
    pub relocate_construct_compressed: Option<unsafe fn(*mut u8, *mut u8, *const i64, usize)>,
    /// `fill_assign` / `fill_construct` / `fill_assign_indices` / `fill_construct_indices`.
    pub fill_assign_n: Option<unsafe fn(*const u8, *mut u8, usize)>,
    pub fill_assign_indices: Option<unsafe fn(*const u8, *mut u8, *const i64, usize)>,
    pub fill_construct_n: Option<unsafe fn(*const u8, *mut u8, usize)>,
    pub fill_construct_indices: Option<unsafe fn(*const u8, *mut u8, *const i64, usize)>,
    /// `add_default` / `add_unique_hash` — fold value bytes into `UniqueHashBytes`.
    pub hash_unique_default: Option<unsafe fn(*const u8, *mut UniqueHashBytes)>,
    /// `destruct_value` — also used by `value_destruct`.
    pub value_destruct: Option<unsafe fn(*mut u8)>,
}

impl CppType {
    /// `type.is_printable()`.
    pub fn is_printable(&self) -> bool {
        self.print.is_some()
    }
    /// `type.has_special_member_functions()`.
    pub fn has_special_member_functions(&self) -> bool {
        self.flags & cpp_type_flags::BASIC_TYPE != 0
    }
    /// `type.default_value_is_zero()`.
    pub fn default_value_is_zero(&self) -> bool {
        self.flags & cpp_type_flags::IDENTITY_DEFAULT_VALUE == 0
    }
    /// `type.value_has_default()` — always true (C: any type can default-construct).
    pub fn value_has_default(&self) -> bool {
        true
    }
    /// `type.is_trivially_destructible()`.
    pub fn is_trivially_destructible(&self) -> bool {
        self.destruct.is_none()
    }
    /// `type.has_implicit_hash()`.
    pub fn has_implicit_hash(&self) -> bool {
        self.hash.is_some()
    }
    /// `type.is_trivially_copyable()`.
    pub fn is_trivially_copyable(&self) -> bool {
        self.flags & cpp_type_flags::BASIC_TYPE != 0
    }
    /// `type.to_string(value)`.
    pub unsafe fn to_string(&self, value: *const u8) -> String {
        let mut ss = String::new();
        if let Some(p) = self.print {
            unsafe { p(value, &mut ss) };
        }
        ss
    }
    /// `type.print_or_default(value, ss, default_value)`.
    pub unsafe fn print_or_default(&self, value: *const u8, ss: &mut String, default_value: &str) {
        if let Some(p) = self.print {
            unsafe { p(value, ss) };
        } else {
            ss.push_str(default_value);
        }
    }
    /// `type.sizeof()` (name parity with `type.size()`).
    pub fn size_of(&self) -> usize {
        self.size
    }
}

/// `CPPType::for_custom_type` — register a custom type handle.
/// Global registry replacing the C `BLI_CPP_TYPE_REGISTER` macro.
pub fn register_cpp_types() -> std::sync::RwLockWriteGuard<'static, Vec<&'static CppType>> {
    static REGISTRY: std::sync::OnceLock<std::sync::RwLock<Vec<&'static CppType>>> =
        std::sync::OnceLock::new();
    REGISTRY
        .get_or_init(|| std::sync::RwLock::new(Vec::new()))
        .write()
        .unwrap()
}
