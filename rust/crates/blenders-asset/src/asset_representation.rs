//! Port of `intern/asset_representation.cc` —
//! AssetRepresentation (local ID | external asset).

use crate::asset_library::AssetWeakReference;

/// `RemoteAssetFileStatus` verbatim ordinals.
pub const REMOTE_ASSET_FILE_UNSET: i32 = 0;
pub const REMOTE_ASSET_FILE_QUEUED: i32 = 1;
pub const REMOTE_ASSET_FILE_DOWNLOADING: i32 = 2;
pub const REMOTE_ASSET_FILE_AVAILABLE: i32 = 3;

/// `AssetRepresentation::ExternalAsset` verbatim.
#[derive(Default)]
pub struct ExternalAsset {
    /// `name`.
    pub name: String,
    /// `id_type`.
    pub id_type: i32,
    /// `preview_` handle.
    pub preview: Option<usize>,
    /// `remote_file_status`.
    pub remote_file_status: i32,
}
/// `AssetRepresentation` verbatim — either a local ID
/// (handle) or external file asset.
pub enum AssetKind {
    /// `std::get<ID *>(&asset_)`.
    LocalId(usize),
    /// `std::get<ExternalAsset>(&asset_)`.
    External(ExternalAsset),
}
impl Default for AssetKind {
    fn default() -> Self {
        AssetKind::External(ExternalAsset::default())
    }
}
/// `AssetRepresentation` verbatim.
#[derive(Default)]
pub struct AssetRepresentation {
    /// `relative_identifier_`.
    pub relative_identifier: String,
    /// `asset_` variant.
    pub asset: AssetKind,
}
impl AssetRepresentation {
    /// `local_id` (verbatim).
    pub fn local_id(&self) -> Option<usize> {
        match self.asset {
            AssetKind::LocalId(id) => Some(id),
            _ => None,
        }
    }
    /// `get_name` (verbatim — local: `id->name + 2`;
    /// external: stored name).
    pub fn get_name(&self, local_id_name: Option<&str>) -> String {
        if self.local_id().is_some() {
            return local_id_name.unwrap_or("").chars().skip(2).collect();
        }
        match &self.asset {
            AssetKind::External(e) => e.name.clone(),
            _ => String::new(),
        }
    }
    /// `make_weak_reference` (verbatim — library ref +
    /// relative identifier).
    pub fn make_weak_reference(
        &self,
        library_type: i32,
        custom_identifier: &str,
    ) -> AssetWeakReference {
        AssetWeakReference {
            asset_library_type: library_type,
            asset_library_custom_identifier: custom_identifier.to_string(),
            relative_asset_identifier: self.relative_identifier.clone(),
        }
    }
}
