//! Port of `intern/blender_copybuffer.cc` — `BKE_copybuffer_read` /
//! `BKE_copybuffer_paste` (copy/paste via partial blend-files).
//!
//! The link-append pipeline (`BKE_blendfile_link_append_*`) and the temp
//! file I/O are foreign systems — kept as `usize` handles with the call
//! order documented verbatim (PORT_PENDING the pipeline itself).

/// `eFileSelAction` — copybuffer writes a session temp file.
pub struct CopyBufferContext {
    /// `BlendfileLinkAppendContext` handle.
    pub lapp_context: usize,
    /// `Main` handles (dst bmain, library bmain).
    pub bmain: usize,
    /// ReportList handle.
    pub reports: usize,
}

/// `copybuffer_append` — the full link→append→instantiate sequence.
/// Steps, in order (each PORT_PENDING):
/// 1. `BKE_main_id_tag_all(bmain, ID_TAG_PRE_EXISTING, true)`
/// 2. `BKE_blendfile_link_append_context_init_done`
/// 3. `BKE_blendfile_link`
/// 4. `BKE_main_lib_objects_recalc_all` +
///    `IMB_colormanagement_check_file_config`
/// 5. `BKE_blendfile_append`
/// 6. `BKE_blendfile_link_append_instantiate_loose`
/// 7. `BKE_blendfile_link_append_context_finalize`
/// 8. `BKE_main_id_tag_all(bmain, ID_TAG_PRE_EXISTING, false)`
pub fn copybuffer_append(_ctx: &mut CopyBufferContext) {
    // PORT_PENDING: BKE_blendfile_link_append pipeline (BLO + main).
}

/// `BKE_copybuffer_read` — create a `Main` from the copybuffer temp file
/// (PORT_PENDING `BKE_blendfile_read`).
pub fn bke_copybuffer_read(_reference_bmain: usize, _copypaste_buf: usize, _reports: usize) -> usize {
    blenders_dna::id::NIL
}

/// `BKE_copybuffer_paste` — link-append the copybuffer file into `bmain`
/// (returns `OPERATOR_FINISHED`/`OPERATOR_CANCELLED` semantics as bool).
pub fn bke_copybuffer_paste(_ctx: &mut CopyBufferContext) -> bool {
    false
}
