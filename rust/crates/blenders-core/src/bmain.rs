//! Port of `Main` (BKE_main / DNA `struct Main`) — the top-level
//! datablock container holding per-type lists.
//!
//! Blender keeps a `ListBase` per `ID` type (`bmain->objects`, `meshes`, …)
//! plus a `GHash` map `name → ID`. Rust uses `HashMap<IdName, Box<Id>>`
//! per list until the typed datablock structs are ported.

use std::collections::HashMap;

use crate::id::{Id, IdKind, IdName};

/// Port of `struct Main` — every datablock list in the .blend file.
#[derive(Debug, Default)]
pub struct Main {
    /// `bmain->filepath[1024]` — current .blend path.
    pub filepath: String,
    /// `bmain->name[256]` — file display name.
    pub name: String,
    /// `bmain->versionfile` — file version number.
    pub version_file: i16,
    /// `bmain->subversionfile`.
    pub subversion_file: i16,
    /// `bmain->minversionfile`.
    pub minversion_file: i16,
    /// `bmain->minsubversionfile`.
    pub minsubversion_file: i16,
    /// `bmain->lock` flags.
    pub lock: i16,
    /// `bmain->is_global_main`.
    pub is_global_main: bool,

    /// Per-kind datablock lists (`ListBase` each, keyed by `ID.name`).
    blocks: HashMap<IdKind, HashMap<IdName, Id>>,
}

impl Main {
    /// `BKE_main_new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BKE_libblock_alloc` — insert a new datablock of `kind` named `name`,
    /// ensuring unique name via `BKE_id_new_name_validate` semantics.
    /// Returns the stored name.
    pub fn add_block(&mut self, kind: IdKind, name: &str) -> IdName {
        let list = self.blocks.entry(kind).or_default();
        // Ensure uniqueness like `BKE_id_new_name_validate` ("name", "name.001", …).
        let mut candidate = IdName::new(kind, name);
        if list.contains_key(&candidate) {
            for n in 1..999 {
                candidate = IdName::new(kind, &format!("{name}.{n:03}"));
                if !list.contains_key(&candidate) {
                    break;
                }
            }
        }
        let mut id = Id::new(kind, "");
        // Keep the full unique name (kind code already inside `candidate`).
        id.name = candidate.clone();
        list.insert(candidate.clone(), id);
        candidate
    }

    /// `BKE_libblock_find_name` — lookup by kind and full name string.
    pub fn find(&self, kind: IdKind, name: &IdName) -> Option<&Id> {
        self.blocks.get(&kind)?.get(name)
    }

    /// `BLI_listbase_count` per kind.
    pub fn count(&self, kind: IdKind) -> usize {
        self.blocks.get(&kind).map(|l| l.len()).unwrap_or(0)
    }

    /// Iterate blocks of a kind (`FOREACH_MAIN_ID` equivalent).
    pub fn iter(&self, kind: IdKind) -> impl Iterator<Item = (&IdName, &Id)> {
        self.blocks
            .get(&kind)
            .into_iter()
            .flat_map(|m| m.iter())
    }

    /// `BKE_main_free` — drop everything.
    pub fn clear(&mut self) {
        self.blocks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_names() {
        let mut main = Main::new();
        let a = main.add_block(IdKind::Object, "Cube");
        let b = main.add_block(IdKind::Object, "Cube");
        assert_ne!(a.to_string(), b.to_string());
        assert_eq!(main.count(IdKind::Object), 2);
    }

    #[test]
    fn find_block() {
        let mut main = Main::new();
        let n = main.add_block(IdKind::Mesh, "Grid");
        assert!(main.find(IdKind::Mesh, &n).is_some());
        assert!(main.find(IdKind::Object, &n).is_none());
    }
}
