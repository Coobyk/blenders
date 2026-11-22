//! Port of `intern/asset_catalog_definition_file.cc`
//! — `.cats.txt` file parse/write.

use crate::asset_catalog::{AssetCatalog, CatalogId};
use crate::asset_catalog_path::AssetCatalogPath;

/// `SUPPORTED_VERSION`/`VERSION_MARKER` verbatim.
pub const SUPPORTED_VERSION: i32 = 1;
pub const VERSION_MARKER: &str = "VERSION ";
/// `HEADER` verbatim.
pub const HEADER: &str = "# This is an Asset Catalog Definition file for Blender.\n\
#\n\
# Empty lines and lines starting with `#` will be ignored.\n\
# The first non-ignored line should be the version indicator.\n\
# Other lines are of the format \"UUID:catalog/path/for/assets:simple catalog name\"\n";

/// `AssetCatalogDefinitionFile` verbatim.
#[derive(Default)]
pub struct AssetCatalogDefinitionFile {
    /// `file_path`.
    pub file_path: String,
    /// `catalogs_` non-owning id→catalog snapshot.
    pub catalogs: std::collections::HashMap<CatalogId, AssetCatalog>,
}
impl AssetCatalogDefinitionFile {
    /// `contains`.
    pub fn contains(&self, catalog_id: CatalogId) -> bool {
        self.catalogs.contains_key(&catalog_id)
    }
    /// `forget`.
    pub fn forget(&mut self, catalog_id: CatalogId) {
        self.catalogs.remove(&catalog_id);
    }
    /// `parse_version_line` (verbatim — "VERSION " + ==
    /// SUPPORTED_VERSION).
    pub fn parse_version_line(line: &str) -> bool {
        if !line.starts_with(VERSION_MARKER) {
            return false;
        }
        let version_string = &line[VERSION_MARKER.len()..];
        let file_version: i32 = version_string.trim().parse().unwrap_or(0);
        file_version == SUPPORTED_VERSION
    }
    /// `parse_catalog_line` (verbatim —
    /// `UUID:path:simple_name`; empty path or bad uuid
    /// → None; missing 2nd delim → path only).
    pub fn parse_catalog_line(line: &str, parse_uuid: &dyn Fn(&str) -> Option<CatalogId>) -> Option<AssetCatalog> {
        let delim = ':';
        let first_delim = line.find(delim)?;
        let id_as_string = line[..first_delim].trim();
        let catalog_id = (parse_uuid)(id_as_string)?;
        let path_and_simple_name = &line[first_delim + 1..];
        let (path_in_file, simple_name) = match path_and_simple_name.find(delim) {
            Some(0) => return None,
            None => (path_and_simple_name.to_string(), String::new()),
            Some(i) => (
                path_and_simple_name[..i].to_string(),
                path_and_simple_name[i + 1..].trim().to_string(),
            ),
        };
        let catalog_path = AssetCatalogPath::new(path_in_file).cleanup();
        let mut cat = AssetCatalog::from_path(&catalog_path, catalog_id);
        cat.simple_name = if simple_name.is_empty() {
            AssetCatalog::sensible_simple_name_for_path(&catalog_path)
        } else {
            simple_name
        };
        Some(cat)
    }
    /// `parse_catalog_file` (verbatim — skip
    /// empty/`#`; first line must be version; keep via
    /// callback).
    pub fn parse_catalog_file(
        text: &str,
        parse_uuid: &dyn Fn(&str) -> Option<CatalogId>,
        mut catalog_loaded_callback: impl FnMut(AssetCatalog) -> bool,
    ) -> Vec<AssetCatalog> {
        let mut kept = Vec::new();
        let mut seen_version_number = false;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if !seen_version_number {
                if !Self::parse_version_line(line) {
                    break;
                }
                seen_version_number = true;
                continue;
            }
            if let Some(catalog) = Self::parse_catalog_line(line, parse_uuid) {
                if (catalog_loaded_callback)(catalog.clone()) {
                    kept.push(catalog);
                }
            }
        }
        kept
    }
    /// `write_to_disk` body (verbatim — HEADER +
    /// VERSION n + `id:path:name` per catalog sorted).
    pub fn write_to_disk_text(&self) -> String {
        let mut out = String::from(HEADER);
        out.push_str(VERSION_MARKER);
        out.push_str(&SUPPORTED_VERSION.to_string());
        out.push('\n');
        let mut cats: Vec<&AssetCatalog> = self.catalogs.values().collect();
        cats.sort_by(|a, b| a.path.cmp(&b.path));
        for c in cats {
            out.push_str(&format!(
                "{:032x}:{}:{}\n",
                c.catalog_id, c.path.path, c.simple_name
            ));
        }
        out
    }
}
