//! Port of `intern/lib_id_remapper.cc` — `IDRemapper`.
//! IDs are `usize` handles; refcount ops (`id_us_*`) are PORT_PENDING.

use std::collections::HashMap;
use blenders_dna::id::NIL;

/// `IDRemapperApplyResult` (BKE_lib_remap.hh).
pub const ID_REMAP_RESULT_SOURCE_NOT_MAPPABLE: i32 = 0;
pub const ID_REMAP_RESULT_SOURCE_UNAVAILABLE: i32 = 1;
pub const ID_REMAP_RESULT_SOURCE_UNASSIGNED: i32 = 2;
pub const ID_REMAP_RESULT_SOURCE_REMAPPED: i32 = 3;

/// `IDRemapperApplyOptions` bits.
pub const ID_REMAP_APPLY_UPDATE_REFCOUNT: u32 = 1 << 0;
pub const ID_REMAP_APPLY_ENSURE_REAL: u32 = 1 << 1;
pub const ID_REMAP_APPLY_UNMAP_WHEN_REMAPPING_TO_SELF: u32 = 1 << 2;
pub const ID_REMAP_APPLY_DEFAULT: u32 = ID_REMAP_APPLY_UPDATE_REFCOUNT;

/// `IDRemapper` — old→new ID mapping with per-type source filter.
#[derive(Default)]
pub struct IDRemapper {
    mappings: HashMap<usize, usize>,
    /// `source_types_` — OR of `BKE_idtype_idcode_to_idfilter` over added types.
    pub source_types: u64,
    /// `allow_idtype_mismatch` (BKE_lib_remap.hh flag).
    pub allow_idtype_mismatch: bool,
}

impl IDRemapper {
    /// `IDRemapper::add`.
    /// `old_id_filter` = `BKE_idtype_idcode_to_idfilter(GS(old_id->name))` —
    /// caller computes it (ID arena handles).
    pub fn add(&mut self, old_id: usize, new_id: usize, old_id_filter: u64) {
        debug_assert!(old_id != NIL);
        debug_assert!(old_id_filter != 0);
        self.mappings.entry(old_id).or_insert(new_id);
        self.source_types |= old_id_filter;
    }

    /// `IDRemapper::add_overwrite`.
    pub fn add_overwrite(&mut self, old_id: usize, new_id: usize, old_id_filter: u64) {
        debug_assert!(old_id != NIL);
        debug_assert!(old_id_filter != 0);
        self.mappings.insert(old_id, new_id);
        self.source_types |= old_id_filter;
    }

    /// `IDRemapper::get_mapping_result`.
    pub fn get_mapping_result(&self, id: usize, options: u32, id_self: usize) -> i32 {
        let Some(&new_id) = self.mappings.get(&id) else {
            return ID_REMAP_RESULT_SOURCE_UNAVAILABLE;
        };
        let mut new_id = new_id;
        if options & ID_REMAP_APPLY_UNMAP_WHEN_REMAPPING_TO_SELF != 0 && id_self == new_id {
            new_id = NIL;
        }
        if new_id == NIL {
            return ID_REMAP_RESULT_SOURCE_UNASSIGNED;
        }
        ID_REMAP_RESULT_SOURCE_REMAPPED
    }

    /// `IDRemapper::apply` — `r_id` is the pointer slot (usize) to rewrite.
    pub fn apply(&self, r_id: &mut usize, options: u32, id_self: usize) -> i32 {
        debug_assert!(options & ID_REMAP_APPLY_UNMAP_WHEN_REMAPPING_TO_SELF == 0 || id_self != NIL);
        if *r_id == NIL {
            return ID_REMAP_RESULT_SOURCE_NOT_MAPPABLE;
        }
        let Some(&new_id) = self.mappings.get(r_id) else {
            return ID_REMAP_RESULT_SOURCE_UNAVAILABLE;
        };
        // ID_REMAP_APPLY_UPDATE_REFCOUNT: id_us_min(*r_id_ptr) — PORT_PENDING.
        *r_id = new_id;
        if options & ID_REMAP_APPLY_UNMAP_WHEN_REMAPPING_TO_SELF != 0 && *r_id == id_self {
            *r_id = NIL;
        }
        if *r_id == NIL {
            return ID_REMAP_RESULT_SOURCE_UNASSIGNED;
        }
        // id_us_plus_no_lib / id_us_ensure_real — PORT_PENDING.
        ID_REMAP_RESULT_SOURCE_REMAPPED
    }

    /// `IDRemapper::result_to_string`.
    pub fn result_to_string(result: i32) -> &'static str {
        match result {
            ID_REMAP_RESULT_SOURCE_NOT_MAPPABLE => "not_mappable",
            ID_REMAP_RESULT_SOURCE_UNAVAILABLE => "unavailable",
            ID_REMAP_RESULT_SOURCE_UNASSIGNED => "unassigned",
            _ => "remapped",
        }
    }

    /// `IDRemapper::iter` — visit every (old, new) pair.
    pub fn iter(&self, mut f: impl FnMut(usize, usize)) {
        for (old_id, new_id) in &self.mappings {
            f(*old_id, *new_id);
        }
    }
}
