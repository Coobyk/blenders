//! Port of `src/blendthumb_png.cc` — minimal PNG
//! writer (IHDR/IDAT/IEND, filter-0 rows).

/// `eThumbStatus` verbatim.
pub const BT_OK: i32 = 0;
pub const BT_FILE_ERR: i32 = 1;
pub const BT_INVALID_FILE: i32 = 2;
pub const BT_INVALID_THUMB: i32 = 3;
pub const BT_ERROR: i32 = 4;
/// `PNG_CHUNK_EXTRA` verbatim.
pub const PNG_CHUNK_EXTRA: usize = 12;
/// `MAKE_ID` verbatim.
pub const fn make_id(a: u8, b: u8, c: u8, d: u8) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32)
}
/// PNG file signature verbatim.
pub const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// `Thumbnail` verbatim fields.
#[derive(Default)]
pub struct Thumbnail {
    /// `width`/`height`.
    pub width: i32,
    pub height: i32,
    /// `data` RGBA bytes.
    pub data: Vec<u8>,
}

/// `png_extend_native_int32` (verbatim — BE append).
pub fn png_extend_native_int32(output: &mut Vec<u8>, data: i32) {
    output.extend_from_slice(&data.to_be_bytes());
}
/// `png_chunk_create` (verbatim — len,tag,data,crc32).
pub fn png_chunk_create(output: &mut Vec<u8>, tag: u32, data: &[u8]) {
    let crc_tag = crc32(&tag.to_be_bytes());
    let crc_data = crc32(data);
    let crc = crc32_combine(crc_tag, crc_data, data.len() as u32);
    let _ = crc;
    let crc = crc32_two(&tag.to_be_bytes(), data);
    png_extend_native_int32(output, data.len() as i32);
    output.extend_from_slice(&tag.to_be_bytes());
    output.extend_from_slice(data);
    png_extend_native_int32(output, crc as i32);
}
fn crc32_two(a: &[u8], b: &[u8]) -> u32 {
    let mut crc = crc32(a);
    crc = crc32_continue(crc, b);
    crc
}
/// minimal CRC32 (PNG poly).
fn crc32(data: &[u8]) -> u32 {
    crc32_continue(0xFFFF_FFFF, data) ^ 0xFFFF_FFFF
}
fn crc32_continue(crc: u32, data: &[u8]) -> u32 {
    let mut c = crc;
    for &byte in data {
        c ^= byte as u32;
        for _ in 0..8 {
            c = if (c & 1) != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    c
}
fn crc32_combine(c1: u32, c2: u32, _len2: u32) -> u32 {
    c1 ^ c2
}
/// `filtered_rows_from_thumb` (verbatim — per-row
/// filter byte 0x00 + RGBA row).
pub fn filtered_rows_from_thumb(thumb: &Thumbnail) -> Vec<u8> {
    let line_size = (thumb.width * 4) as usize;
    let mut filtered = Vec::with_capacity(thumb.height as usize * (line_size + 1));
    for i in 0..thumb.height as usize {
        filtered.push(0x00);
        filtered.extend_from_slice(&thumb.data[i * line_size..(i + 1) * line_size]);
    }
    filtered
}
/// `blendthumb_create_png_data_from_thumb` (verbatim —
/// empty→None; IHDR(8,6) + IDAT + IEND).
pub fn blendthumb_create_png_data_from_thumb(
    thumb: &Thumbnail,
    zlib_compress: &dyn Fn(&[u8]) -> Option<Vec<u8>>,
) -> Option<Vec<u8>> {
    if thumb.data.is_empty() {
        return None;
    }
    let image_data = (zlib_compress)(&filtered_rows_from_thumb(thumb))?;
    let mut ihdr_data = Vec::with_capacity(4 + 4 + 5);
    png_extend_native_int32(&mut ihdr_data, thumb.width);
    png_extend_native_int32(&mut ihdr_data, thumb.height);
    ihdr_data.extend_from_slice(&[0x08, 0x06, 0x00, 0x00, 0x00]);
    let mut png_buf = Vec::new();
    png_buf.extend_from_slice(&PNG_SIGNATURE);
    png_chunk_create(&mut png_buf, make_id(b'I', b'H', b'D', b'R'), &ihdr_data);
    png_chunk_create(&mut png_buf, make_id(b'I', b'D', b'A', b'T'), &image_data);
    png_chunk_create(&mut png_buf, make_id(b'I', b'E', b'N', b'D'), &[]);
    Some(png_buf)
}
