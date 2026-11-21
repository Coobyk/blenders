//! Port of `DNA_asset_types.h` — asset library/metadata structs.

use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eAssetLibraryType` (short).
pub type AssetLibraryType = i16;
pub const ASSET_LIBRARY_LOCAL: AssetLibraryType = 1;
pub const ASSET_LIBRARY_ALL: AssetLibraryType = 2;
pub const ASSET_LIBRARY_ESSENTIALS: AssetLibraryType = 3;
pub const ASSET_LIBRARY_ONLINE_ESSENTIALS: AssetLibraryType = 4;
pub const ASSET_LIBRARY_CUSTOM: AssetLibraryType = 100;

/// `eAssetImportMethod`.
pub type AssetImportMethod = i32;
pub const ASSET_IMPORT_LINK: AssetImportMethod = 0;
pub const ASSET_IMPORT_APPEND: AssetImportMethod = 1;
pub const ASSET_IMPORT_APPEND_REUSE: AssetImportMethod = 2;
pub const ASSET_IMPORT_PACK: AssetImportMethod = 3;

/// `eAssetLibrary_Flag`.
pub type AssetLibraryFlag = i32;
pub const ASSET_LIBRARY_RELATIVE_PATH: AssetLibraryFlag = 1 << 0;
pub const ASSET_LIBRARY_DISABLED: AssetLibraryFlag = 1 << 1;
pub const ASSET_LIBRARY_USE_REMOTE_URL: AssetLibraryFlag = 1 << 2;
pub const ASSET_LIBRARY_USE_AUTH_TOKEN: AssetLibraryFlag = 1 << 3;
pub const ASSET_LIBRARY_PROJECT_DEFINED: AssetLibraryFlag = 1 << 4;

/// `enum class AssetAccess : int8_t`.
pub type AssetAccess = i8;
pub const ASSET_ACCESS_ONLINE_AND_OFFLINE: AssetAccess = 0;
pub const ASSET_ACCESS_ONLY_ONLINE: AssetAccess = 1;
pub const ASSET_ACCESS_ONLY_OFFLINE: AssetAccess = 2;

/// `AssetTag`.
#[derive(Clone)]
pub struct AssetTag {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
}
impl Default for AssetTag {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
        }
    }
}

/// `AssetMetaDataFlag`.
pub type AssetMetaDataFlag = i32;
pub const ASSETDATA_USE_OWN_IMPORT_METHOD: AssetMetaDataFlag = 1 << 0;

/// `bUUID` — 128-bit UUID (DNA_uuid_types.h).
#[derive(Clone, Copy, Default)]
pub struct BUuid {
    pub time_low: u32,
    pub time_mid: u16,
    pub time_hi_and_version: u16,
    pub clock_seq_hi_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

/// `AssetMetaData`.
#[derive(Clone)]
pub struct AssetMetaData {
    /// `AssetTypeInfo *local_type_info`.
    pub local_type_info: usize,
    /// `IDProperty *properties`.
    pub properties: usize,
    pub catalog_id: BUuid,
    /// `char catalog_simple_name[64]`.
    pub catalog_simple_name: [u8; 64],
    /// `char *author`, `*description`, `*copyright`, `*license`.
    pub author: Option<String>,
    pub description: Option<String>,
    pub copyright: Option<String>,
    pub license: Option<String>,
    /// `ListBaseT<AssetTag>`.
    pub tags: ListBase,
    pub active_tag: i16,
    pub tot_tags: i16,
    pub flag: AssetMetaDataFlag,
    pub preferred_import_method: AssetImportMethod,
}
impl Default for AssetMetaData {
    fn default() -> Self {
        Self {
            local_type_info: NIL,
            properties: NIL,
            catalog_id: BUuid::default(),
            catalog_simple_name: [0; 64],
            author: None,
            description: None,
            copyright: None,
            license: None,
            tags: ListBase::default(),
            active_tag: 0,
            tot_tags: 0,
            flag: 0,
            preferred_import_method: ASSET_IMPORT_APPEND,
        }
    }
}

/// `AssetImportSettings`.
#[derive(Clone, Default)]
pub struct AssetImportSettings {
    pub method: AssetImportMethod,
    pub use_instance_collections: bool,
}

/// `AssetLibraryReference`.
#[derive(Clone)]
pub struct AssetLibraryReference {
    pub type_: AssetLibraryType,
    pub custom_library_index: i32,
}
impl Default for AssetLibraryReference {
    fn default() -> Self {
        Self {
            type_: ASSET_LIBRARY_LOCAL,
            custom_library_index: -1,
        }
    }
}

/// `AssetWeakReference`.
#[derive(Clone, Default)]
pub struct AssetWeakReference {
    pub asset_library_type: AssetLibraryType,
    /// `const char *asset_library_identifier`.
    pub asset_library_identifier: Option<String>,
    /// `const char *relative_asset_identifier`.
    pub relative_asset_identifier: Option<String>,
}
impl AssetWeakReference {
    /// `operator==`.
    pub fn eq_ref(&self, other: &Self) -> bool {
        self.asset_library_type == other.asset_library_type
            && self.asset_library_identifier == other.asset_library_identifier
            && self.relative_asset_identifier == other.relative_asset_identifier
    }
}

/// `AssetCatalogPathLink`.
#[derive(Clone, Default)]
pub struct AssetCatalogPathLink {
    pub next: usize,
    pub prev: usize,
    /// `char *path`.
    pub path: Option<String>,
}
