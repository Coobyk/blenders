//! Port of `DNA_ID.h` part 1: IDProperty UI data, IDProperty, ID base
//! structs, and the ID-type constants. Pointer members become `usize`
//! (`BM_NIL`-style NIL = `usize::MAX`) or `Option<Box<..>>`.
use blenders_bli::listbase::ListBase;

pub const NIL: usize = usize::MAX;

/// `IDPropertyUIData` — common header of all prop UI payloads.
#[derive(Clone, Default)]
pub struct IDPropertyUIData {
    /// `char *description`.
    pub description: Option<String>,
    /// `int value`.
    pub value: i32,
    /// `int icon`.
    pub icon: i32,
}

/// `IDPropertyUIDataEnumItem`.
#[derive(Clone, Default)]
pub struct IDPropertyUIDataEnumItem {
    /// `char *identifier`.
    pub identifier: Option<String>,
    /// `char *name`.
    pub name: Option<String>,
    /// `char *description`.
    pub description: Option<String>,
    /// `int value`.
    pub value: i32,
    /// `int icon`.
    pub icon: i32,
}

/// `IDPropertyUIDataInt`.
#[derive(Clone, Default)]
pub struct IDPropertyUIDataInt {
    pub base: IDPropertyUIData,
    /// `int *default_array` → Vec.
    pub default_array: Option<Vec<i32>>,
    pub default_array_len: i32,
    pub min: i32,
    pub max: i32,
    pub soft_min: i32,
    pub soft_max: i32,
    pub step: i32,
    pub default_value: i32,
    pub enum_items_num: i32,
    /// `IDPropertyUIDataEnumItem *enum_items` → Vec.
    pub enum_items: Option<Vec<IDPropertyUIDataEnumItem>>,
}

/// `IDPropertyUIDataBool`.
#[derive(Clone, Default)]
pub struct IDPropertyUIDataBool {
    pub base: IDPropertyUIData,
    /// `int8_t *default_array`.
    pub default_array: Option<Vec<i8>>,
    pub default_array_len: i32,
    pub default_value: i8,
}

/// `IDPropertyUIDataFloat`.
#[derive(Clone, Default)]
pub struct IDPropertyUIDataFloat {
    pub base: IDPropertyUIData,
    /// `double *default_array`.
    pub default_array: Option<Vec<f64>>,
    pub default_array_len: i32,
    pub step: f32,
    pub precision: i32,
    pub min: f64,
    pub max: f64,
    pub soft_min: f64,
    pub soft_max: f64,
    pub default_value: f64,
}

/// `IDPropertyUIDataString`.
#[derive(Clone, Default)]
pub struct IDPropertyUIDataString {
    pub base: IDPropertyUIData,
    /// `char *default_value`.
    pub default_value: Option<String>,
}

/// `IDPropertyUIDataID`.
#[derive(Clone, Default)]
pub struct IDPropertyUIDataID {
    pub base: IDPropertyUIData,
    pub id_type: i16,
}

/// `IDPropertyData` — the union payload; union → enum.
#[derive(Clone)]
pub enum IDPropertyDataPayload {
    /// `void *pointer` — opaque.
    Pointer(usize),
    /// `ListBaseT<IDProperty> group` — nested prop list (indices).
    Group(ListBase),
    /// `double *array`/`float *`/`int *`/… — array buffer + len.
    Array(Vec<u8>),
    /// `char *string`.
    Str(Option<String>),
    /// scalar.
    None,
}

impl Default for IDPropertyDataPayload {
    fn default() -> Self {
        Self::None
    }
}

/// `IDPropertyData`.
#[derive(Clone, Default)]
pub struct IDPropertyData {
    pub payload: IDPropertyDataPayload,
    /// `int type` (`IDP_*`).
    pub type_: i32,
    /// `int len` — array length.
    pub len: i32,
}
