//! Port of `intern/asset_weak_reference.cc` — `AssetWeakReference`.
//!
//! C's copy/move ctor gymnastics reduce to `Clone`/move in Rust; the
//! comparison + blend read/write hooks are verbatim semantics.

use crate::asset::{AssetLibraryReference, ASSET_LIBRARY_LOCAL};

/// `AssetWeakReference` (BKE_asset_library.hh field — stored on IDs).
#[derive(Clone)]
pub struct AssetWeakReference {
    /// `asset_library_type` — `eAssetLibraryType`.
    pub asset_library_type: i16,
    /// `asset_library_identifier` — library file/path id.
    pub asset_library_identifier: Option<String>,
    /// `relative_asset_identifier` — asset name within the library.
    pub relative_asset_identifier: Option<String>,
}

impl Default for AssetWeakReference {
    fn default() -> Self {
        Self {
            asset_library_type: ASSET_LIBRARY_LOCAL,
            asset_library_identifier: None,
            relative_asset_identifier: None,
        }
    }
}

impl PartialEq for AssetWeakReference {
    /// `operator==` — all three fields compared (`strcmp` on both strings).
    fn eq(&self, other: &Self) -> bool {
        self.asset_library_type == other.asset_library_type
            && self.asset_library_identifier == other.asset_library_identifier
            && self.relative_asset_identifier == other.relative_asset_identifier
    }
}
impl Eq for AssetWeakReference {}

impl AssetWeakReference {
    /// `AssetWeakReference::make_path` — full `dir/file#asset` style path:
    /// `<library_identifier>/<relative_identifier>` when the library is a
    /// custom path type (C builds it for custom libraries only).
    pub fn make_path(&self) -> Option<String> {
        match (&self.asset_library_identifier, &self.relative_asset_identifier) {
            (Some(lib), Some(rel)) => Some(format!("{}/{}", lib, rel)),
            _ => None,
        }
    }

    /// `BKE_asset_library_reference_to_weak` — C `AssetWeakReference` ctor
    /// from `AssetLibraryReference` (library + asset names).
    pub fn from_library_reference(
        library: &AssetLibraryReference,
        library_identifier: &str,
        relative_asset_identifier: &str,
    ) -> Self {
        Self {
            asset_library_type: library.type_,
            asset_library_identifier: Some(library_identifier.to_string()),
            relative_asset_identifier: Some(relative_asset_identifier.to_string()),
        }
    }
}
