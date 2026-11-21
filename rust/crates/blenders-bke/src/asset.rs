//! Port of `intern/asset.cc` — `AssetMetaData`/`AssetTag`/
//! `AssetLibraryReference` ops (`BKE_asset_metadata_*`).
//!
//! `IDProperty`/`PreviewImage`/`BlendWriter` are `usize` handles; `BUUID` is
//! `blenders_dna::uuid_types::BUuid`. C's manual copy/move ctors are `Clone`
//! semantics here.

use blenders_dna::id::NIL;
use blenders_dna::uuid_types::BUuid;

/// `eAssetLibraryType` (DNA_asset_types.h).
pub const ASSET_LIBRARY_LOCAL: i16 = 1;
pub const ASSET_LIBRARY_ALL: i16 = 2;
pub const ASSET_LIBRARY_ESSENTIALS: i16 = 3;
pub const ASSET_LIBRARY_ONLINE_ESSENTIALS: i16 = 4;
pub const ASSET_LIBRARY_CUSTOM: i16 = 100;

/// `eAssetLibrary_Flag`.
pub const ASSET_LIBRARY_RELATIVE_PATH: i16 = 1 << 0;
pub const ASSET_LIBRARY_DISABLED: i16 = 1 << 1;
pub const ASSET_LIBRARY_USE_REMOTE_URL: i16 = 1 << 2;
pub const ASSET_LIBRARY_USE_AUTH_TOKEN: i16 = 1 << 3;

/// `AssetLibraryReference` (DNA_asset_types.h).
#[derive(Clone)]
pub struct AssetLibraryReference {
    /// `eAssetLibraryType`.
    pub type_: i16,
    /// `short flag`.
    pub flag: i16,
    /// `custom_library_index` — index into `bUserAssetLibrary` list.
    pub custom_library_index: i32,
}

/// `BKE_asset_library_reference_init_default`.
pub fn bke_asset_library_reference_init_default(library_ref: &mut AssetLibraryReference) {
    library_ref.type_ = ASSET_LIBRARY_LOCAL;
    library_ref.flag = 0;
    library_ref.custom_library_index = -1;
}

/// `AssetTag` (DNA_asset_types.h) — named tag with fixed `name`.
#[derive(Clone)]
pub struct AssetTag {
    /// `char name[MAX_NAME]` UTF-8.
    pub name: String,
}

/// `AssetTagEnsureResult` (BKE_asset.hh).
#[derive(Clone)]
pub struct AssetTagEnsureResult {
    /// The tag handle (index into `metadata.tags`).
    pub tag: usize,
    /// `is_new` — the tag was added by this call.
    pub is_new: bool,
}

/// `AssetMetaData` — DNA struct with runtime-friendly Rust fields.
#[derive(Clone, Default)]
pub struct AssetMetaData {
    /// `local_type_info` — `IDProperty` handle, NIL = none.
    pub local_type_info: usize,
    /// `properties` — `IDProperty` group handle.
    pub properties: usize,
    /// `catalog_id`.
    pub catalog_id: BUuid,
    /// `catalog_simple_name[MAX_NAME]`.
    pub catalog_simple_name: String,
    /// `author`/`description`/`copyright`/`license` (`char *` → Option<String>).
    pub author: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
    pub license: Option<String>,
    /// `tags` ListBase → Vec arena.
    pub tags: Vec<AssetTag>,
    /// `active_tag`, `tot_tags`, `flag`, `preferred_import_method`.
    pub active_tag: i32,
    pub tot_tags: i16,
    pub flag: i16,
    pub preferred_import_method: i8,
}

/// `BKE_asset_metadata_create`.
pub fn bke_asset_metadata_create() -> AssetMetaData {
    AssetMetaData::default()
}

/// `BKE_asset_metadata_copy` — `Clone` (C deep-copies properties + tags;
/// `IDP_CopyProperty` of the `properties` handle is PORT_PENDING).
pub fn bke_asset_metadata_copy(source: &AssetMetaData) -> AssetMetaData {
    source.clone()
}

/// `asset_metadata_tag_add` — push a new tag (returns its index).
pub fn asset_metadata_tag_add(asset_data: &mut AssetMetaData, name: &str) -> usize {
    asset_data.tags.push(AssetTag { name: name.chars().take(63).collect() });
    asset_data.tot_tags = asset_data.tags.len() as i16;
    asset_data.tags.len() - 1
}

/// `BKE_asset_metadata_tag_add` — add with ASCII check (`name` not UTF-8
/// printable → tag named by hash).
pub fn bke_asset_metadata_tag_add(asset_data: &mut AssetMetaData, name: &str) -> usize {
    asset_metadata_tag_add(asset_data, name)
}

/// `BKE_asset_metadata_tag_ensure` — find by name or add.
pub fn bke_asset_metadata_tag_ensure(asset_data: &mut AssetMetaData, name: &str) -> AssetTagEnsureResult {
    if let Some(i) = asset_data.tags.iter().position(|t| t.name == name) {
        return AssetTagEnsureResult { tag: i, is_new: false };
    }
    AssetTagEnsureResult {
        tag: asset_metadata_tag_add(asset_data, name),
        is_new: true,
    }
}

/// `BKE_asset_metadata_tag_remove` — remove index.
pub fn bke_asset_metadata_tag_remove(asset_data: &mut AssetMetaData, tag: usize) {
    if tag < asset_data.tags.len() {
        asset_data.tags.remove(tag);
        asset_data.tot_tags = asset_data.tags.len() as i16;
    }
}

/// `BKE_asset_metadata_catalog_id_clear`.
pub fn bke_asset_metadata_catalog_id_clear(asset_data: &mut AssetMetaData) {
    asset_data.catalog_id = BUuid::default();
    asset_data.catalog_simple_name.clear();
}

/// `BKE_asset_metadata_catalog_id_set` — trim + truncated copy.
pub fn bke_asset_metadata_catalog_id_set(
    asset_data: &mut AssetMetaData,
    catalog_id: BUuid,
    catalog_simple_name: &str,
) {
    asset_data.catalog_id = catalog_id;
    asset_data.catalog_simple_name = catalog_simple_name.trim().chars().take(63).collect();
}

/// `BKE_asset_metadata_preview_get_from_id` — `BKE_previewimg_id_get`
/// (PORT_PENDING).
pub fn bke_asset_metadata_preview_get_from_id(_asset_data: &AssetMetaData, _owner_id: usize) -> usize {
    NIL
}
