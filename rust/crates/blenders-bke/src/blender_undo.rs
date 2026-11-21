//! Port of `intern/blender_undo.cc` — `BKE_memfile_undo_*`.
//!
//! Memfile undo = "read the whole blend state from memory" — the read is
//! `BKE_blendfile_read_mem` on the stored memfile. `Main`/`bContext` are
//! `usize` handles; the file flags juggling is ported verbatim.

/// `UNDO_DISK` — always 0 in C (disk undo was removed).
pub const UNDO_DISK: bool = false;

/// `MemFileUndoData` (BKE_blender_undo.hh).
#[derive(Default)]
pub struct MemFileUndoData {
    /// `mfu->memfile` — serialized blend bytes.
    pub memfile: Vec<u8>,
    /// `mfu->undo_str` / filepath.
    pub filepath: String,
    /// `mfu->ident` — undo index.
    pub ident: i32,
}

/// `BKE_memfile_undo_decode` — reload state from the memfile.
/// `read_setup` performs `BKE_blendfile_read_mem` +
/// `BKE_blendfile_read_setup_undo` (blendfile reader PORT_PENDING —
/// callers plug it in).
pub fn bke_memfile_undo_decode(
    mfu: &MemFileUndoData,
    undo_direction: i8,      /* eUndoStepDir */
    use_old_bmain_data: bool,
    file_flags: &mut u32,    /* G.fileflags */
    read_setup: &mut dyn FnMut(&[u8], bool) -> bool,
) -> bool {
    let _ = undo_direction;
    const G_FILE_NO_UI: u32 = 1 << 0; /* subset needed here */
    *file_flags |= G_FILE_NO_UI;
    let mut success = false;
    if UNDO_DISK {
        /* filepath path — dead code, kept for faithfulness. */
    } else {
        success = read_setup(&mfu.memfile, use_old_bmain_data);
    }
    *file_flags &= !G_FILE_NO_UI;
    success
}

/// `BKE_memfile_undo_encode` — write current Main into `mfu` via
/// `BKE_blendfile_memfile_write` (PORT_PENDING writer → caller fn).
pub fn bke_memfile_undo_encode(
    mfu: &mut MemFileUndoData,
    write: &mut dyn FnMut() -> Vec<u8>,
) {
    mfu.memfile = write();
}

/// `BKE_memfile_undo_free`.
pub fn bke_memfile_undo_free(mfu: MemFileUndoData) {
    drop(mfu);
}
