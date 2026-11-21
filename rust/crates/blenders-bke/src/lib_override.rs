//! Port of `intern/lib_override.cc` pure parts —
//! `BKE_lib_override_library_tag_set`, `is_user_edited`,
//! `is_system_defined`, `is_hierarchy_leaf`,
//! `BKE_lib_override_operation_as_string`.
//!
//! `PORT_PENDING`: `BKE_lib_override_library_create(_from_
//! tag)`/`resync`/`make_local`/`delete` (ID copying +
//! Main integration), `lib_override_library_property_*`
//! (RNA-path driven diff engine, `BKE_lib_override_rna_
//! property_find`, auto-apply of RNA ops), `status_check_*`,
//! `operations_(create|restore)`, `id_(reset|unused_cleanup)`,
//! `BKE_lib_override_library_update`.

use blenders_dna::id::{
    LibOverrideOp, liboverride_flag::LIBOVERRIDE_FLAG_SYSTEM_DEFINED,
    liboverride_op_flag::LIBOVERRIDE_OP_FLAG_IDPOINTER_MATCH_REFERENCE,
    liboverride_prop_tag::LIBOVERRIDE_PROP_OP_TAG_UNUSED,
};

/// `IDOverrideLibraryPropertyOperation` — runtime form
/// (same fields as the DNA struct, Vec-backed).
#[derive(Clone, Default)]
pub struct LibOverridePropertyOp {
    /// `operation`.
    pub operation: LibOverrideOp,
    /// `flag`.
    pub flag: i16,
    /// `tag`.
    pub tag: i16,
    /// `subitem_reference_name`.
    pub subitem_reference_name: Option<String>,
    /// `subitem_local_name`.
    pub subitem_local_name: Option<String>,
    /// `subitem_reference_index`.
    pub subitem_reference_index: i32,
    /// `subitem_local_index`.
    pub subitem_local_index: i32,
}

/// `IDOverrideLibraryProperty` — runtime form.
#[derive(Clone, Default)]
pub struct LibOverrideProperty {
    /// `rna_path`.
    pub rna_path: String,
    /// `operations`.
    pub operations: Vec<LibOverridePropertyOp>,
    /// `tag`.
    pub tag: i16,
    /// `rna_prop_type`.
    pub rna_prop_type: u32,
}

/// `IDOverrideLibraryTag` — runtime tag bits.
pub const LIBOVERRIDE_TAG_RESYNC: i32 = 1 << 0;

/// `BKE_lib_override_library_tag_set` (verbatim).
pub fn lib_override_library_tag_set(tag: &mut i32, bit: i32, value: bool) {
    if value {
        *tag |= bit;
    } else {
        *tag &= !bit;
    }
}

/// `BKE_lib_override_library_is_user_edited` (verbatim).
pub fn lib_override_library_is_user_edited(
    is_override_library: bool,
    is_override_library_real: bool,
    properties: &[LibOverrideProperty],
) -> bool {
    if !is_override_library {
        return false;
    }
    if !is_override_library_real {
        return false;
    }
    for op in properties {
        for opop in &op.operations {
            if (opop.flag & LIBOVERRIDE_OP_FLAG_IDPOINTER_MATCH_REFERENCE) != 0 {
                continue;
            }
            if opop.operation == LibOverrideOp::LIBOVERRIDE_OP_NOOP {
                continue;
            }
            return true;
        }
    }
    false
}

/// `BKE_lib_override_library_is_system_defined` (verbatim —
/// caller resolves `override_owner_id->override_library->
/// flag`).
pub fn lib_override_library_is_system_defined(owner_flag: u32) -> bool {
    (owner_flag & LIBOVERRIDE_FLAG_SYSTEM_DEFINED) != 0
}

/// `foreachid_is_hierarchy_leaf_fn` + `BKE_lib_override_
/// library_is_hierarchy_leaf` — leaf iff no linked ID inside
/// `hierarchy_root`'s hierarchy is a real override itself
/// (verbatim; caller supplies the linked-override check).
pub fn lib_override_library_is_hierarchy_leaf<'a>(
    is_override_library_real: bool,
    linked_ids_same_hierarchy_has_override: impl FnOnce() -> bool,
) -> bool {
    if !is_override_library_real {
        return false;
    }
    !linked_ids_same_hierarchy_has_override()
}

/// `BKE_lib_override_operation_as_string` (verbatim).
pub fn lib_override_operation_as_string(operation: LibOverrideOp) -> &'static str {
    match operation {
        LibOverrideOp::LIBOVERRIDE_OP_NOOP => "NoOp",
        LibOverrideOp::LIBOVERRIDE_OP_REPLACE => "Replace",
        LibOverrideOp::LIBOVERRIDE_OP_ADD => "Add",
        LibOverrideOp::LIBOVERRIDE_OP_SUBTRACT => "Subtract",
        LibOverrideOp::LIBOVERRIDE_OP_MULTIPLY => "Multiply",
        LibOverrideOp::LIBOVERRIDE_OP_INSERT_AFTER => "Insert After",
        LibOverrideOp::LIBOVERRIDE_OP_INSERT_BEFORE => "Insert Before",
        LibOverrideOp::LIBOVERRIDE_OP_CUSTOM => "Custom",
    }
}

/// `BKE_lib_override_debug_print` — verbatim formatting to
/// a `String`.
pub fn lib_override_debug_print(
    intro_txt: Option<&str>,
    properties: &[LibOverrideProperty],
    subitem_refs: &dyn Fn(&LibOverridePropertyOp) -> (Option<String>, usize, Option<String>, usize),
) -> String {
    let mut out = String::new();
    let mut line_prefix = "";
    if let Some(intro) = intro_txt {
        out.push_str(intro);
        out.push('\n');
        line_prefix = "\t";
    }
    for op in properties {
        out.push_str(&format!("{}{} [", line_prefix, op.rna_path));
        if op.tag & LIBOVERRIDE_PROP_OP_TAG_UNUSED != 0 {
            out.push_str(" UNUSED ");
        }
        out.push_str("]\n");
        for opop in &op.operations {
            out.push_str(&format!(
                "{}{}{} [",
                line_prefix,
                line_prefix,
                lib_override_operation_as_string(opop.operation)
            ));
            if opop.tag & LIBOVERRIDE_PROP_OP_TAG_UNUSED != 0 {
                out.push_str(" UNUSED ");
            }
            if opop.flag & LIBOVERRIDE_OP_FLAG_IDPOINTER_MATCH_REFERENCE != 0 {
                out.push_str(" MATCH_REF ");
            }
            out.push_str("] ");
            let (ref_name, ref_id, loc_name, loc_id) = subitem_refs(opop);
            if ref_name.is_some() || loc_name.is_some() {
                out.push_str(&format!(
                    "({:?} <{}> -> {:?} <{}>)",
                    ref_name.unwrap_or_default(),
                    ref_id,
                    loc_name.unwrap_or_default(),
                    loc_id
                ));
            } else if opop.subitem_reference_index >= 0 || opop.subitem_local_index >= 0 {
                out.push_str(&format!(
                    "({} -> {})",
                    opop.subitem_reference_index, opop.subitem_local_index
                ));
            }
            out.push('\n');
        }
    }
    out
}
