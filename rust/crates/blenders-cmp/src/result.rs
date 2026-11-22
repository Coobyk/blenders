//! Port of `intern/result.cc` — compositor `Result`
//! type tables + refcount + allocation state.

/// `ResultType` verbatim ordinals.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResultType {
    Float,
    Vector,
    Color,
    Float2,
    Float3,
    Float4,
    Int,
    Int2,
    Int3,
    Int4,
    Bool,
    Float4x4,
    Menu,
    Quaternion,
    String,
    Object,
    Image,
    Font,
    Scene,
    Text,
    Mask,
    Bundle,
    Closure,
}
/// `ResultPrecision` verbatim ordinals.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResultPrecision {
    /// `Half`.
    Half,
    /// `Full`.
    Full,
}
/// `Result::is_single_value_only_type` (verbatim —
/// only handle-like types are single-only).
pub fn is_single_value_only_type(t: ResultType) -> bool {
    use ResultType::*;
    matches!(t, String | Object | Image | Font | Scene | Text | Mask | Bundle | Closure)
}
/// `Result::type_name` (verbatim).
pub fn type_name(t: ResultType) -> &'static str {
    use ResultType::*;
    match t {
        Float => "float",
        Float2 => "float2",
        Float3 => "float3",
        Float4 => "float4",
        Color => "color",
        Int => "int",
        Int2 => "int2",
        Int3 => "int3",
        Int4 => "int4",
        Bool => "bool",
        Float4x4 => "float4x4",
        Menu => "menu",
        Quaternion => "quaternion",
        String => "string",
        Object => "object",
        Image => "image",
        Font => "font",
        Scene => "scene",
        Text => "text",
        Mask => "mask",
        Bundle => "bundle",
        Closure => "closure",
        Vector => "vector",
    }
}
/// `Result::channels_count` (verbatim — per type).
pub fn channels_count(t: ResultType) -> i64 {
    use ResultType::*;
    match t {
        Float | Int | Bool | Menu => 1,
        Float2 | Int2 => 2,
        Float3 | Int3 | Quaternion => 3,
        Color | Float4 | Int4 => 4,
        Float4x4 => 16,
        _ => 1,
    }
}
/// `Result` verbatim fields.
#[derive(Default)]
pub struct Result {
    /// `type_`.
    pub type_: i32,
    /// `precision_`.
    pub precision: i32,
    /// `is_allocated_`.
    pub is_allocated: bool,
    /// `is_single_value_`.
    pub is_single_value: bool,
    /// `reference_count_`.
    pub reference_count: i32,
    /// `initial_reference_count_`.
    pub initial_reference_count: i32,
    /// `should_compute_`.
    pub should_compute: bool,
    /// `domain_`.
    pub domain: crate::domain::Domain,
    /// `value` storage (bytes).
    pub value: Vec<u8>,
}
impl Default for crate::domain::Domain {
    fn default() -> Self {
        crate::domain::Domain::new([0, 0])
    }
}
impl Result {
    /// `set_reference_count` (verbatim).
    pub fn set_reference_count(&mut self, count: i32) {
        self.reference_count = count;
    }
    /// `decrement_reference_count` (verbatim).
    pub fn decrement_reference_count(&mut self, count: i32) {
        self.reference_count -= count;
    }
    /// `should_compute` (verbatim — refcount > 0).
    pub fn should_compute(&self) -> bool {
        self.should_compute || self.reference_count > 0
    }
    /// `allocate_single_value` (verbatim — marks
    /// single + allocated).
    pub fn allocate_single_value(&mut self) {
        self.is_single_value = true;
        self.is_allocated = true;
    }
    /// `allocate_invalid` (verbatim).
    pub fn allocate_invalid(&mut self) {
        self.is_allocated = true;
    }
    /// `release` (verbatim — free when refcount→0).
    pub fn release(&mut self) {
        if self.reference_count <= 0 {
            self.free();
        }
    }
    /// `free` (verbatim — drop allocation).
    pub fn free(&mut self) {
        self.value.clear();
        self.is_allocated = false;
    }
}
