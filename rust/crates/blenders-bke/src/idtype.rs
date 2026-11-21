//! Port of `BKE_idtype.hh` — `IDTypeInfo` vtable for ID datablocks.
//!
//! Callback members that need not-yet-ported machinery (RNA, LibraryForeachIDData,
//! BlendWriter/BlendDataReader, AssetTypeInfo) are kept as `Option<fn>`-shaped
//! opaque markers: they exist so the per-type `IDType_ID_*` tables can be ported
//! verbatim; the signatures will become real as their dependencies land.

/// `IDTypeInfo` — per-ID-type callback table (see BKE_idtype.hh).
///
/// Function-pointer members are `usize` placeholders for now (PORT_PENDING):
/// Blender stores `void *`-style callbacks here and the runtime types they
/// mention (`LibraryForeachIDData`, `BlendWriter`, `BlendDataReader`,
/// `BlendLibReader`, `BlendExpander`, `BlendReadData`) are not ported yet.
#[derive(Clone)]
pub struct IDTypeInfo {
    /// `short id_code` — e.g. `ID_SPK`.
    pub id_code: i16,
    /// `uint64_t id_filter` — `FILTER_ID_*` for linking UI.
    pub id_filter: u64,
    /// `uint64_t dependencies_id_types` — `FILTER_ID_*` this type can point at.
    pub dependencies_id_types: u64,
    /// `int main_listbase_index` — `INDEX_ID_*`.
    pub main_listbase_index: i32,
    /// `uint32_t struct_size`.
    pub struct_size: u32,
    /// `const char *name`.
    pub name: &'static str,
    /// `const char *name_plural`.
    pub name_plural: &'static str,
    /// `const char *translation_context`.
    pub translation_context: &'static str,
    /// `uint32_t flags` — `IDTYPE_FLAGS_*`.
    pub flags: u32,
    /// `AssetTypeInfo *asset_type_info` — PORT_PENDING.
    pub asset_type_info: usize,

    /// `IDTypeInitFunc init_data`.
    pub init_data: usize,
    /// `IDTypeCopyFunc copy_data`.
    pub copy_data: usize,
    /// `IDTypeFreeFunc free_data`.
    pub free_data: usize,
    /// `IDTypeMakeLocalFunc make_local`.
    pub make_local: usize,
    /// `IDTypeForeachIDFunctionCallback foreach_id`.
    pub foreach_id: usize,
    /// `IDTypeForeachCacheFunctionCallback foreach_cache`.
    pub foreach_cache: usize,
    /// `IDTypeForeachPathFunctionCallback foreach_path`.
    pub foreach_path: usize,
    /// `IDTypeForeachWorkingSpaceColorFunc foreach_working_space_color`.
    pub foreach_working_space_color: usize,
    /// `IDTypeForeachAssetWeakRefFunc foreach_asset_weak_reference`.
    pub foreach_asset_weak_reference: usize,
    /// `IDTypeOwnerGet owner_pointer_get`.
    pub owner_pointer_get: usize,

    /// `IDTypeBlendWriteFunction blend_write`.
    pub blend_write: usize,
    /// `IDTypeBlendReadDataFunction blend_read_data`.
    pub blend_read_data: usize,
    /// `IDTypeBlendReadAfterLiblinkFunction blend_read_after_liblink`.
    pub blend_read_after_liblink: usize,
    /// `IDTypeBlendReadUndoPreserve blend_read_undo_preserve`.
    pub blend_read_undo_preserve: usize,
    /// `IDTypeLibOverrideApplyPostFunc lib_override_apply_post`.
    pub lib_override_apply_post: usize,
}

/// `IDTYPE_FLAGS_*` from BKE_idtype.hh.
pub const IDTYPE_FLAGS_NO_COPY: u32 = 1 << 0;
pub const IDTYPE_FLAGS_NO_LIBLINK: u32 = 1 << 1;
pub const IDTYPE_FLAGS_APPEND_IS_REUSABLE: u32 = 1 << 2;
pub const IDTYPE_FLAGS_NO_ANIMDATA: u32 = 1 << 3;

const NIL: usize = usize::MAX;

impl IDTypeInfo {
    /// Base constructor matching Blender's aggregate-init defaults (all callbacks null).
    pub const fn new(
        id_code: i16,
        id_filter: u64,
        dependencies_id_types: u64,
        main_listbase_index: i32,
        struct_size: u32,
        name: &'static str,
        name_plural: &'static str,
        translation_context: &'static str,
        flags: u32,
    ) -> Self {
        Self {
            id_code,
            id_filter,
            dependencies_id_types,
            main_listbase_index,
            struct_size,
            name,
            name_plural,
            translation_context,
            flags,
            asset_type_info: 0,
            init_data: 0,
            copy_data: 0,
            free_data: 0,
            make_local: 0,
            foreach_id: 0,
            foreach_cache: 0,
            foreach_path: 0,
            foreach_working_space_color: 0,
            foreach_asset_weak_reference: 0,
            owner_pointer_get: 0,
            blend_write: 0,
            blend_read_data: 0,
            blend_read_after_liblink: 0,
            blend_read_undo_preserve: 0,
            lib_override_apply_post: 0,
        }
    }
}

/// `BKE_idtype_get_info_from_idcode` — lookup by `ID_SPK`-style code; PORT_PENDING
/// until the global `IDTypeInfo` registry table exists.
pub fn idtype_get_info_from_idcode(_id_code: i16) -> Option<&'static IDTypeInfo> {
    None
}
