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

/* ===== DNA_ID.h part 2: IDProperty, overrides, ID, Library, enums ===== */

pub const MAX_IDPROP_NAME: usize = 64;
pub const DEFAULT_ALLOC_FOR_NULL_STRINGS: usize = 64;
pub const MAX_ID_NAME: usize = 258;
pub const FILE_MAX: usize = 1024;

/// `eIDPropertyType`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(i8)]
pub enum IDPType {
    IDP_STRING = 0,
    IDP_INT = 1,
    IDP_FLOAT = 2,
    IDP_ARRAY = 5,
    IDP_GROUP = 6,
    IDP_DOUBLE = 9,
    IDP_IDPARRAY = 10,
    IDP_BOOLEAN = 11,
    IDP_ENUM = 12,
    IDP_BYTES = 13,
    #[default]
    IDP_NONE = 14,
}

/// `eIDPropertyFlag`.
pub mod idp_flag {
    pub const IDP_FLAG_GHOST: u8 = 1 << 7;
}

/// `eID_OverrideLib_Op`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(i16)]
pub enum LibOverrideOp {
    #[default]
    LIBOVERRIDE_OP_NOOP = 0,
    LIBOVERRIDE_OP_REPLACE = 1,
    LIBOVERRIDE_OP_ADD = 101,
    LIBOVERRIDE_OP_SUBTRACT = 102,
    LIBOVERRIDE_OP_MULTIPLY = 103,
    LIBOVERRIDE_OP_INSERT_AFTER = 201,
    LIBOVERRIDE_OP_INSERT_BEFORE = 202,
    LIBOVERRIDE_OP_CUSTOM = 255,
}

/// `eID_OverrideLib_OpFlag`.
pub mod liboverride_op_flag {
    pub const LIBOVERRIDE_OP_FLAG_MANDATORY: i16 = 1 << 0;
    pub const LIBOVERRIDE_OP_FLAG_LOCKED: i16 = 1 << 1;
    pub const LIBOVERRIDE_OP_FLAG_IDPOINTER_MATCH_REFERENCE: i16 = 1 << 8;
    pub const LIBOVERRIDE_OP_FLAG_IDPOINTER_ITEM_USE_ID: i16 = 1 << 9;
}

/// `eID_OverrideLib_PropTag`.
pub mod liboverride_prop_tag {
    pub const LIBOVERRIDE_PROP_OP_TAG_UNUSED: i16 = 1 << 0;
    pub const LIBOVERRIDE_PROP_TAG_NEEDS_RETORE: i16 = 1 << 1;
}

/// `eID_OverrideLib_Flag`.
pub mod liboverride_flag {
    pub const LIBOVERRIDE_FLAG_NO_HIERARCHY: u32 = 1 << 0;
    pub const LIBOVERRIDE_FLAG_SYSTEM_DEFINED: u32 = 1 << 1;
}

/// `IDOverrideLibraryPropertyOperation`.
#[derive(Clone, Default)]
pub struct IDOverrideLibraryPropertyOperation {
    /// `next`/`prev` — arena index into an ops list.
    pub next: usize,
    pub prev: usize,
    pub operation: LibOverrideOp,
    pub flag: i16,
    pub tag: i16,
    /// `char *subitem_reference_name`.
    pub subitem_reference_name: Option<String>,
    /// `char *subitem_local_name`.
    pub subitem_local_name: Option<String>,
    pub subitem_reference_index: i32,
    pub subitem_local_index: i32,
    /// `ID *subitem_reference_id`.
    pub subitem_reference_id: usize,
    /// `ID *subitem_local_id`.
    pub subitem_local_id: usize,
    /// `char *label`.
    pub label: Option<String>,
    /// `char *tooltip`.
    pub tooltip: Option<String>,
}

/// `IDOverrideLibraryProperty`.
#[derive(Clone, Default)]
pub struct IDOverrideLibraryProperty {
    pub next: usize,
    pub prev: usize,
    /// `char *rna_path`.
    pub rna_path: Option<String>,
    /// `ListBaseT<IDOverrideLibraryPropertyOperation>` — arena indices.
    pub operations: ListBase,
    pub tag: i16,
    /// `unsigned int rna_prop_type`.
    pub rna_prop_type: u32,
}

/// `IDOverrideLibrary`.
#[derive(Clone, Default)]
pub struct IDOverrideLibrary {
    /// `ID *reference`.
    pub reference: usize,
    /// `ListBaseT<IDOverrideLibraryProperty>`.
    pub properties: ListBase,
    /// `ID *hierarchy_root`.
    pub hierarchy_root: usize,
    /// `IDOverrideLibraryRuntime *runtime`.
    pub runtime: usize,
    pub flag: u32,
}

/// `eID_RemapStatus`.
pub mod id_remap {
    pub const ID_REMAP_IS_LINKED_DIRECT: i8 = 1 << 0;
    pub const ID_REMAP_IS_USER_ONE_SKIPPED: i8 = 1 << 1;
}

/// `IDHash` — 16-byte hash; first 8 bytes are the u64 short-hash.
#[derive(Clone, Copy, Default)]
pub struct IDHash {
    pub data: [u8; 16],
}

impl IDHash {
    /// `IDHash::hash()`.
    pub fn hash(&self) -> u64 {
        u64::from_le_bytes(self.data[0..8].try_into().unwrap())
    }
    /// `IDHash::get_null()`.
    pub fn get_null() -> Self {
        Self::default()
    }
    /// `IDHash::is_null()`.
    pub fn is_null(&self) -> bool {
        self.data == [0; 16]
    }
}

impl PartialEq for IDHash {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for IDHash {}

/// `ID_Type` — two-char code as u16 (little-endian first-char-low).
pub type IDType = u16;

/// `GS(name)` — build an `ID_Type` from the first two chars of an ID name.
pub const fn gs(name: &[u8; 2]) -> IDType {
    name[0] as u16 | ((name[1] as u16) << 8)
}

/// `ID` — the data-block base struct. ID pointers/arena refs are `usize`.
#[derive(Clone)]
pub struct ID {
    /// `void *next, *prev` — list slots in `Main` id list.
    pub next: usize,
    pub prev: usize,
    /// `ID *newid`.
    pub newid: usize,
    /// `Library *lib`.
    pub lib: usize,
    /// `AssetMetaData *asset_data`.
    pub asset_data: usize,
    /// `char name[MAX_ID_NAME]` — [0..2] = id type, [2..] = display name.
    pub name: [u8; MAX_ID_NAME],
    pub flag: i16,
    pub tag: i32,
    /// `int us` — user count.
    pub us: i32,
    pub icon_id: i32,
    pub recalc: u32,
    pub recalc_up_to_undo_push: u32,
    pub recalc_after_undo_push: u32,
    /// `unsigned int session_uid`.
    pub session_uid: u32,
    pub deep_hash: IDHash,
    /// `IDProperty *properties` — generic user props (list head idx).
    pub properties: usize,
    /// `IDProperty *system_properties`.
    pub system_properties: usize,
    /// `IDOverrideLibrary *override_library`.
    pub override_library: usize,
    /// `ID *orig_id` — copy-on-write original.
    pub orig_id: usize,
    /// `void *py_instance`.
    pub py_instance: usize,
    /// `LibraryWeakReference *library_weak_reference`.
    pub library_weak_reference: usize,
    /// `bke::id::ID_Runtime *runtime`.
    pub runtime: usize,
}

impl Default for ID {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            newid: NIL,
            lib: NIL,
            asset_data: NIL,
            name: [0; MAX_ID_NAME],
            flag: 0,
            tag: 0,
            us: 0,
            icon_id: 0,
            recalc: 0,
            recalc_up_to_undo_push: 0,
            recalc_after_undo_push: 0,
            session_uid: 0,
            deep_hash: IDHash::default(),
            properties: NIL,
            system_properties: NIL,
            override_library: NIL,
            orig_id: NIL,
            py_instance: NIL,
            library_weak_reference: NIL,
            runtime: NIL,
        }
    }
}

impl ID {
    /// `ID::id_type()` — `GS(name)`.
    pub fn id_type(&self) -> IDType {
        gs(&[self.name[0], self.name[1]])
    }
}

/// `Library`.
#[derive(Clone)]
pub struct Library {
    pub id: ID,
    /// `char filepath[FILE_MAX]`.
    pub filepath: [u8; FILE_MAX],
    pub flag: u16,
    pub undo_runtime_tag: u16,
    /// `Library *archive_parent_library`.
    pub archive_parent_library: usize,
    /// `PackedFile *packedfile`.
    pub packedfile: usize,
    /// `bke::library::LibraryRuntime *runtime`.
    pub runtime: usize,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            id: ID::default(),
            filepath: [0; FILE_MAX],
            flag: 0,
            undo_runtime_tag: 0,
            archive_parent_library: NIL,
            packedfile: NIL,
            runtime: NIL,
        }
    }
}

/// `LibraryWeakReference`.
#[derive(Clone)]
pub struct LibraryWeakReference {
    /// `char library_filepath[FILE_MAX]`.
    pub library_filepath: [u8; FILE_MAX],
    /// `char library_id_name[MAX_ID_NAME]`.
    pub library_id_name: [u8; MAX_ID_NAME],
}

impl Default for LibraryWeakReference {
    fn default() -> Self {
        Self {
            library_filepath: [0; FILE_MAX],
            library_id_name: [0; MAX_ID_NAME],
        }
    }
}

/// `ePreviewImage_Flag`.
pub mod preview_image_flag {
    pub const PRV_CHANGED: i32 = 1 << 0;
    pub const PRV_USER_EDITED: i32 = 1 << 1;
    pub const PRV_RENDERING: i32 = 1 << 2;
}

/// `ePreviewImage_Tag`.
pub mod preview_image_tag {
    pub const PRV_TAG_DEFERRED_RENDERING: i16 = 1 << 1;
    pub const PRV_TAG_DEFERRED_DELETE: i16 = 1 << 2;
    pub const PRV_TAG_DEFERRED_INVALID: i16 = 1 << 3;
    pub const PRV_TAG_RESTART_RENDERING: i16 = 1 << 4;
}

/// `PreviewImage` — two sizes (icon + large).
#[derive(Clone)]
pub struct PreviewImage {
    pub w: [u32; 2],
    pub h: [u32; 2],
    pub flag: [i16; 2],
    pub changed_timestamp: [i16; 2],
    /// `unsigned int *rect[2]` — pixel buffers.
    pub rect: [Option<Vec<u32>>; 2],
    /// `bke::PreviewImageRuntime *runtime`.
    pub runtime: usize,
}

impl Default for PreviewImage {
    fn default() -> Self {
        Self {
            w: [0; 2],
            h: [0; 2],
            flag: [0; 2],
            changed_timestamp: [0; 2],
            rect: [None, None],
            runtime: NIL,
        }
    }
}

/// `ID_FAKE_USERS(id)`.
pub fn id_fake_users(id: &ID) -> i32 {
    if id.flag & ID_FLAG_FAKEUSER != 0 {
        1
    } else {
        0
    }
}
/// `ID_EXTRA_USERS(id)`.
pub fn id_extra_users(id: &ID) -> i32 {
    if id.tag & ID_TAG_EXTRAUSER != 0 {
        1
    } else {
        0
    }
}
/// `ID_EXTRA_REAL_USERS(id)`.
pub fn id_extra_real_users(id: &ID) -> i32 {
    if id.tag & ID_TAG_EXTRAUSER_SET != 0 {
        1
    } else {
        0
    }
}
/// `ID_REAL_USERS(id)`.
pub fn id_real_users(id: &ID) -> i32 {
    id.us - id_fake_users(id)
}
/// `ID_REFCOUNTING_USERS(id)`.
pub fn id_refcounting_users(id: &ID) -> i32 {
    id_real_users(id) - id_extra_real_users(id)
}
/// `ID_MISSING`.
pub fn id_missing(id: &ID) -> bool {
    id.tag & ID_TAG_MISSING != 0
}
/// `ID_IS_LINKED`.
pub fn id_is_linked(id: &ID) -> bool {
    id.lib != NIL
}
/// `ID_IS_ASSET`.
pub fn id_is_asset(id: &ID) -> bool {
    id.asset_data != NIL
}
/// `ID_IS_OVERRIDE_LIBRARY_REAL`.
pub fn id_is_override_library_real(id: &ID) -> bool {
    id.override_library != NIL
}
/// `ID_IS_OVERRIDE_LIBRARY_VIRTUAL`.
pub fn id_is_override_library_virtual(id: &ID) -> bool {
    id.flag & ID_FLAG_EMBEDDED_DATA_LIB_OVERRIDE != 0
}
/// `ID_IS_OVERRIDE_LIBRARY`.
pub fn id_is_override_library(id: &ID) -> bool {
    id_is_override_library_real(id) || id_is_override_library_virtual(id)
}

/// `eID_Flag`.
pub const ID_FLAG_FAKEUSER: i16 = 1 << 9;
pub const ID_FLAG_EMBEDDED_DATA: i16 = 1 << 10;
pub const ID_FLAG_INDIRECT_WEAK_LINK: i16 = 1 << 11;
pub const ID_FLAG_EMBEDDED_DATA_LIB_OVERRIDE: i16 = 1 << 12;
pub const ID_FLAG_LIB_OVERRIDE_RESYNC_LEFTOVER: i16 = 1 << 13;
pub const ID_FLAG_CLIPBOARD_MARK: i16 = 1 << 14;
pub const ID_FLAG_LINKED_AND_PACKED: i16 = 1 << 15;

/// `eID_Tag`.
pub const ID_TAG_LOCAL: i32 = 0;
pub const ID_TAG_EXTERN: i32 = 1 << 0;
pub const ID_TAG_INDIRECT: i32 = 1 << 1;
pub const ID_TAG_RUNTIME: i32 = 1 << 2;
pub const ID_TAG_MISSING: i32 = 1 << 3;
pub const ID_TAG_EXTRAUSER: i32 = 1 << 4;
pub const ID_TAG_EXTRAUSER_SET: i32 = 1 << 5;
pub const ID_TAG_LIBOVERRIDE_REFOK: i32 = 1 << 6;
pub const ID_TAG_LIBOVERRIDE_AUTOREFRESH: i32 = 1 << 7;
pub const ID_TAG_LIBOVERRIDE_NEED_RESYNC: i32 = 1 << 8;
pub const ID_TAG_NEW: i32 = 1 << 12;
pub const ID_TAG_PRE_EXISTING: i32 = 1 << 13;
pub const ID_TAG_UNDO_OLD_ID_REUSED_UNCHANGED: i32 = 1 << 17;
pub const ID_TAG_UNDO_OLD_ID_REUSED_NOUNDO: i32 = 1 << 18;
pub const ID_TAG_UNDO_OLD_ID_REREAD_IN_PLACE: i32 = 1 << 19;
pub const ID_TAG_TEMP_MAIN: i32 = 1 << 20;
pub const ID_TAG_NO_MAIN: i32 = 1 << 21;
pub const ID_TAG_LOCALIZED: i32 = 1 << 22;
pub const ID_TAG_COPIED_ON_EVAL: i32 = 1 << 23;
pub const ID_TAG_NO_USER_REFCOUNT: i32 = 1 << 25;
pub const ID_TAG_NOT_ALLOCATED: i32 = 1 << 26;
pub const ID_TAG_DOIT: i32 = 1 << 31;

/// `ID_TAG_KEEP_ON_UNDO`.
pub const ID_TAG_KEEP_ON_UNDO: i32 =
    ID_TAG_EXTRAUSER | ID_TAG_MISSING | ID_TAG_RUNTIME | ID_TAG_LOCAL | ID_TAG_EXTERN | ID_TAG_INDIRECT;

/// `IDRecalcFlag`.
pub const ID_RECALC_TRANSFORM: u32 = 1 << 0;
pub const ID_RECALC_GEOMETRY: u32 = 1 << 1;
pub const ID_RECALC_ANIMATION: u32 = 1 << 2;
pub const ID_RECALC_PSYS_REDO: u32 = 1 << 3;
pub const ID_RECALC_PSYS_RESET: u32 = 1 << 4;
pub const ID_RECALC_PSYS_CHILD: u32 = 1 << 5;
pub const ID_RECALC_PSYS_PHYS: u32 = 1 << 6;
pub const ID_RECALC_SHADING: u32 = 1 << 7;
pub const ID_RECALC_SELECT: u32 = 1 << 9;
pub const ID_RECALC_BASE_FLAGS: u32 = 1 << 10;
pub const ID_RECALC_POINT_CACHE: u32 = 1 << 11;
pub const ID_RECALC_EDITORS: u32 = 1 << 12;
pub const ID_RECALC_SYNC_TO_EVAL: u32 = 1 << 13;
pub const ID_RECALC_SEQUENCER_STRIPS: u32 = 1 << 14;
pub const ID_RECALC_FRAME_CHANGE: u32 = 1 << 15;
pub const ID_RECALC_AUDIO_FPS: u32 = 1 << 16;
pub const ID_RECALC_AUDIO_VOLUME: u32 = 1 << 17;
pub const ID_RECALC_AUDIO_MUTE: u32 = 1 << 18;
pub const ID_RECALC_AUDIO_LISTENER: u32 = 1 << 19;
pub const ID_RECALC_AUDIO: u32 = 1 << 20;
pub const ID_RECALC_PARAMETERS: u32 = 1 << 21;
pub const ID_RECALC_SOURCE: u32 = 1 << 23;
pub const ID_RECALC_TAG_FOR_UNDO: u32 = 1 << 24;
pub const ID_RECALC_NTREE_OUTPUT: u32 = 1 << 25;
pub const ID_RECALC_HIERARCHY: u32 = 1 << 26;
pub const ID_RECALC_COMPOSITOR: u32 = 1 << 27;
pub const ID_RECALC_PROVISION_28: u32 = 1 << 28;
pub const ID_RECALC_PROVISION_29: u32 = 1 << 29;
pub const ID_RECALC_PROVISION_30: u32 = 1 << 30;
pub const ID_RECALC_PROVISION_31: u32 = 1 << 31;
pub const ID_RECALC_ANIMATION_NO_FLUSH: u32 = ID_RECALC_SYNC_TO_EVAL;
pub const ID_RECALC_GEOMETRY_ALL_MODES: u32 = ID_RECALC_GEOMETRY | ID_RECALC_SYNC_TO_EVAL;
pub const ID_RECALC_ALL: u32 = 0xffffffff;
pub const ID_RECALC_PSYS_ALL: u32 =
    ID_RECALC_PSYS_REDO | ID_RECALC_PSYS_RESET | ID_RECALC_PSYS_CHILD | ID_RECALC_PSYS_PHYS;

/// `FILTER_ID_*` bitmask consts (id-type filter for e.g. outliner).
pub const FILTER_ID_AC: u64 = 1 << 0;
pub const FILTER_ID_AR: u64 = 1 << 1;
pub const FILTER_ID_BR: u64 = 1 << 2;
pub const FILTER_ID_CA: u64 = 1 << 3;
pub const FILTER_ID_CU_LEGACY: u64 = 1 << 4;
pub const FILTER_ID_GD_LEGACY: u64 = 1 << 5;
pub const FILTER_ID_GR: u64 = 1 << 6;
pub const FILTER_ID_IM: u64 = 1 << 7;
pub const FILTER_ID_LA: u64 = 1 << 8;
pub const FILTER_ID_LS: u64 = 1 << 9;
pub const FILTER_ID_LT: u64 = 1 << 10;
pub const FILTER_ID_MA: u64 = 1 << 11;
pub const FILTER_ID_MB: u64 = 1 << 12;
pub const FILTER_ID_MC: u64 = 1 << 13;
pub const FILTER_ID_ME: u64 = 1 << 14;
pub const FILTER_ID_MSK: u64 = 1 << 15;
pub const FILTER_ID_NT: u64 = 1 << 16;
pub const FILTER_ID_OB: u64 = 1 << 17;
pub const FILTER_ID_PAL: u64 = 1 << 18;
pub const FILTER_ID_PC: u64 = 1 << 19;
pub const FILTER_ID_SCE: u64 = 1 << 20;
pub const FILTER_ID_SPK: u64 = 1 << 21;
pub const FILTER_ID_SO: u64 = 1 << 22;
pub const FILTER_ID_TE: u64 = 1 << 23;
pub const FILTER_ID_TXT: u64 = 1 << 24;
pub const FILTER_ID_VF: u64 = 1 << 25;
pub const FILTER_ID_WO: u64 = 1 << 26;
pub const FILTER_ID_PA: u64 = 1 << 27;
pub const FILTER_ID_CF: u64 = 1 << 28;
pub const FILTER_ID_WS: u64 = 1 << 29;
pub const FILTER_ID_LP: u64 = 1 << 31;
pub const FILTER_ID_CV: u64 = 1 << 32;
pub const FILTER_ID_PT: u64 = 1 << 33;
pub const FILTER_ID_VO: u64 = 1 << 34;
pub const FILTER_ID_SIM: u64 = 1 << 35;
pub const FILTER_ID_KE: u64 = 1 << 36;
pub const FILTER_ID_SCR: u64 = 1 << 37;
pub const FILTER_ID_WM: u64 = 1 << 38;
pub const FILTER_ID_LI: u64 = 1 << 39;
pub const FILTER_ID_GP: u64 = 1 << 40;
pub const FILTER_ID_IP: u64 = 1 << 41;
pub const FILTER_ID_ALL: u64 = FILTER_ID_AC | FILTER_ID_AR | FILTER_ID_BR | FILTER_ID_CA
    | FILTER_ID_CU_LEGACY | FILTER_ID_GD_LEGACY | FILTER_ID_GR | FILTER_ID_IM | FILTER_ID_LA
    | FILTER_ID_LS | FILTER_ID_LT | FILTER_ID_MA | FILTER_ID_MB | FILTER_ID_MC | FILTER_ID_ME
    | FILTER_ID_MSK | FILTER_ID_NT | FILTER_ID_OB | FILTER_ID_PA | FILTER_ID_PAL | FILTER_ID_PC
    | FILTER_ID_SCE | FILTER_ID_SPK | FILTER_ID_SO | FILTER_ID_TE | FILTER_ID_TXT | FILTER_ID_VF
    | FILTER_ID_WO | FILTER_ID_CF | FILTER_ID_WS | FILTER_ID_LP | FILTER_ID_CV | FILTER_ID_PT
    | FILTER_ID_VO | FILTER_ID_SIM | FILTER_ID_KE | FILTER_ID_SCR | FILTER_ID_WM | FILTER_ID_LI
    | FILTER_ID_GP | FILTER_ID_IP;

/// `eID_Index` — slot order of `Main->curlb[]`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum IndexID {
    INDEX_ID_LI = 0,
    INDEX_ID_AC,
    INDEX_ID_GD_LEGACY,
    INDEX_ID_NT,
    INDEX_ID_VF,
    INDEX_ID_TXT,
    INDEX_ID_SO,
    INDEX_ID_MSK,
    INDEX_ID_IM,
    INDEX_ID_MC,
    INDEX_ID_TE,
    INDEX_ID_MA,
    INDEX_ID_LS,
    INDEX_ID_WO,
    INDEX_ID_CF,
    INDEX_ID_PA,
    INDEX_ID_KE,
    INDEX_ID_AR,
    INDEX_ID_ME,
    INDEX_ID_CU_LEGACY,
    INDEX_ID_MB,
    INDEX_ID_CV,
    INDEX_ID_PT,
    INDEX_ID_VO,
    INDEX_ID_LT,
    INDEX_ID_LA,
    INDEX_ID_CA,
    INDEX_ID_SPK,
    INDEX_ID_LP,
    INDEX_ID_GP,
    INDEX_ID_OB,
    INDEX_ID_GR,
    INDEX_ID_PAL,
    INDEX_ID_PC,
    INDEX_ID_BR,
    INDEX_ID_SCE,
    INDEX_ID_SCR,
    INDEX_ID_WS,
    INDEX_ID_WM,
    INDEX_ID_NULL,
}
pub const INDEX_ID_MAX: i32 = IndexID::INDEX_ID_NULL as i32 + 1;

/* `ID_Type` codes — the 2-char prefixes from `DNA_ID_enums.h` live there;
   the ones needed for GS comparisons are emitted in `id_enums`. */
