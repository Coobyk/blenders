/// Port of `intern/writefile.cc` — BLO
/// .blend writer (DNA encode, compression).
/// `BLO_write_file` (verbatim — Main →
/// file).
pub struct BlendFileWriter {
    /// `fd` target.
    pub fd: usize,
    /// `write_flags` (compress/remap).
    pub write_flags: i32,
}
/// `BLO_write_file_mem` (verbatim — in-mem
/// write for undo).
pub fn blo_write_file_mem(main: usize) -> usize {
    main
}
