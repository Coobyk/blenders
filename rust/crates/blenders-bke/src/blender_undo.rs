//! Port of `intern/blender_undo.cc` — `BKE_memfile_undo_*` memfile
//! encode/decode. `#define UNDO_DISK 0` — the disk path is dead code
//! in C too; the memfile path is ported, with `BKE_blendfile_read*` /
//! `BLO_memfile_*` injected (PORT_PENDING blendfile read/write).

/// `UNDO_DISK` — set to 0 in the C source (disk-undo path disabled).
pub const UNDO_DISK: bool = false;

/// `MemFile` (BLO_undofile) — the serialized undo buffer.
#[derive(Clone, Default)]
pub struct MemFile {
    pub data: Vec<u8>,
    pub size: usize,
}

/// `MemFileUndoData` (BKE_undo_system.hh).
#[derive(Default)]
pub struct MemFileUndoData {
    /// `char filepath[FILE_MAX]` — disk path (UNDO_DISK only).
    pub filepath: String,
    pub memfile: MemFile,
    /// `size_t undo_size`.
    pub undo_size: usize,
}

/// `BKE_memfile_undo_decode` — returns success. `read_from_memfile` +
/// `read_setup_undo` are the blendfile-reader pair (PORT_PENDING);
/// `tag_on_visible_update` is `DEG_tag_on_visible_update`.
#[allow(clippy::too_many_arguments)]
pub fn bke_memfile_undo_decode(
    mfu: &MemFileUndoData,
    undo_direction: i32,
    use_old_bmain_data: bool,
    bmain_filepath: &mut String,
    current_filepath: &str,
    read_from_memfile: &mut dyn FnMut(&MemFile, i32, bool) -> Option<usize>,
    read_setup_undo: &mut dyn FnMut(usize, i32),
    tag_on_visible_update: &mut dyn FnMut(),
) -> bool {
    let mainstr = bmain_filepath.clone(); /* temporal store */
    /* `G.fileflags |= G_FILE_NO_UI` — caller manages the flag. */
    let mut success = false;
    /* !UNDO_DISK branch: */
    if let Some(bfd) = read_from_memfile(&mfu.memfile, undo_direction, use_old_bmain_data) {
        read_setup_undo(bfd, undo_direction);
        success = true;
    }
    let _ = current_filepath;
    *bmain_filepath = mainstr;
    if success {
        tag_on_visible_update();
    }
    success
}

/// `BKE_memfile_undo_encode` — `clear_future` drops redo chunks of the
/// previous memfile (`BLO_memfile_clear_future`, PORT_PENDING); the
/// write itself is `BLO_memfile_write` in the caller.
pub fn bke_memfile_undo_encode(
    memfile_write: &mut dyn FnMut() -> MemFile,
    mfu_prev: Option<&mut MemFileUndoData>,
) -> MemFileUndoData {
    let mut mfu = MemFileUndoData::default();
    /* !UNDO_DISK: */
    if let Some(prev) = mfu_prev {
        prev.memfile.data.clear(); /* BLO_memfile_clear_future */
    }
    mfu.memfile = memfile_write();
    mfu.undo_size = mfu.memfile.size;
    mfu
}

/// `BKE_memfile_undo_free`.
pub fn bke_memfile_undo_free(mfu: MemFileUndoData) {
    drop(mfu);
}
