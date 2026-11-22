/// Port of `intern/draw_select_buffer.cc` —
/// select-ID buffer readback.
/// `DRW_select_buffer` (verbatim — read
/// uint buffer + de-dup).
pub fn drw_select_buffer_read(buf: &[u32]) -> Vec<u32> {
    let mut out: Vec<u32> = buf.to_vec();
    out.sort_unstable();
    out.dedup();
    out
}
