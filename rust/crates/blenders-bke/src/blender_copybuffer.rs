//! Port of `intern/blender_copybuffer.cc` — `BKE_copybuffer_read` /
//! `BKE_copybuffer_paste` orchestration.
//!
//! The whole file sequences `BKE_blendfile_link_append_*`, `BKE_main_*`,
//! `DEG_relations_tag_update` — all injected as closures pending the
//! blendfile/link-append + Main ports.

/// `BLENDFILE_LINK_APPEND_INVALID` (BKE_blendfile_link_append.hh).
pub const BLENDFILE_LINK_APPEND_INVALID: i32 = -1;

/// `ID_TAG_PRE_EXISTING` / `ID_FLAG_CLIPBOARD_MARK` callers pass in.

/// The link/append + Main pipeline steps `copybuffer_append` and the
/// entry points call, in order (all PORT_PENDING systems).
pub struct CopybufferOps<'a> {
    /// `BKE_main_id_tag_all(bmain, ID_TAG_PRE_EXISTING, true/false)`.
    pub main_id_tag_all: &'a mut dyn FnMut(bool),
    /// `BKE_blendfile_link_append_context_init_done`.
    pub context_init_done: &'a mut dyn FnMut(),
    /// `BKE_blendfile_link`.
    pub link: &'a mut dyn FnMut(),
    /// `BKE_main_lib_objects_recalc_all`.
    pub lib_objects_recalc_all: &'a mut dyn FnMut(),
    /// `IMB_colormanagement_check_file_config`.
    pub colormanagement_check: &'a mut dyn FnMut(),
    /// `BKE_blendfile_append`.
    pub append: &'a mut dyn FnMut(),
    /// `BKE_blendfile_link_append_instantiate_loose`.
    pub instantiate_loose: &'a mut dyn FnMut(),
    /// `BKE_blendfile_link_append_context_finalize`.
    pub context_finalize: &'a mut dyn FnMut(),
    /// `DEG_relations_tag_update`.
    pub relations_tag_update: &'a mut dyn FnMut(),
}

/// `copybuffer_append`.
pub fn copybuffer_append(ops: &mut CopybufferOps) {
    (ops.main_id_tag_all)(true);
    (ops.context_init_done)();
    (ops.link)();
    (ops.lib_objects_recalc_all)();
    (ops.colormanagement_check)();
    (ops.append)();
    (ops.instantiate_loose)();
    (ops.context_finalize)();
    (ops.main_id_tag_all)(false);
    (ops.relations_tag_update)();
}

/// `BKE_copybuffer_read` — `add_items` returns the pasted count
/// (`BLENDFILE_LINK_APPEND_INVALID` on failure); `main_new`/
/// `main_init_from_reference`/`main_free` own the temp Main.
///
/// Returns the temp-main handle (usize) on success.
#[allow(clippy::too_many_arguments)]
pub fn bke_copybuffer_read(
    main_new: &mut dyn FnMut() -> usize,
    main_init_from_reference: &mut dyn FnMut(usize),
    context_library_add: &mut dyn FnMut(&str),
    add_items_from_library: &mut dyn FnMut(u64) -> i32,
    context_free: &mut dyn FnMut(),
    main_free: &mut dyn FnMut(usize),
    ops: &mut CopybufferOps,
    libname: &str,
    id_types_mask: u64,
) -> Option<usize> {
    let bmain_src = main_new();
    main_init_from_reference(bmain_src);
    context_library_add(libname);
    let num_pasted = add_items_from_library(id_types_mask);
    if num_pasted == BLENDFILE_LINK_APPEND_INVALID {
        context_free();
        main_free(bmain_src);
        return None;
    }
    copybuffer_append(ops);
    context_free();
    Some(bmain_src)
}

/// `BKE_copybuffer_paste` — same pipeline into the active Main plus a
/// deselect + clipboard-mark clear. Returns `num_pasted` (0 on fail).
#[allow(clippy::too_many_arguments)]
pub fn bke_copybuffer_paste(
    context_library_add: &mut dyn FnMut(&str),
    add_items_from_library: &mut dyn FnMut(u64) -> i32,
    base_deselect_all: &mut dyn FnMut(),
    main_id_flag_clipboard_mark_clear: &mut dyn FnMut(),
    context_free: &mut dyn FnMut(),
    ops: &mut CopybufferOps,
    libname: &str,
    id_types_mask: u64,
) -> i32 {
    /* `BLI_assert((flag & BLO_LIBLINK_APPEND_RECURSIVE) == 0)` — caller. */
    context_library_add(libname);
    let num_pasted = add_items_from_library(id_types_mask);
    if num_pasted == BLENDFILE_LINK_APPEND_INVALID {
        context_free();
        return 0;
    }
    base_deselect_all();
    copybuffer_append(ops);
    main_id_flag_clipboard_mark_clear();
    context_free();
    num_pasted
}
