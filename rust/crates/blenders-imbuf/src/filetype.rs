//! Port of `intern/filetype.cc` — file-format
//! detection table + dispatch.

/// `IMB_FTYPE_*` verbatim ordinals.
pub const IMB_FTYPE_PNG: i32 = 1;
pub const IMB_FTYPE_TGA: i32 = 2;
pub const IMB_FTYPE_JPG: i32 = 3;
pub const IMB_FTYPE_BMP: i32 = 4;
pub const IMB_FTYPE_OPENEXR: i32 = 5;
pub const IMB_FTYPE_CINEON: i32 = 6;
pub const IMB_FTYPE_DPX: i32 = 7;
pub const IMB_FTYPE_TIF: i32 = 8;
pub const IMB_FTYPE_HDR: i32 = 9;
pub const IMB_FTYPE_IRIS: i32 = 10;
pub const IMB_FTYPE_PSD: i32 = 11;
pub const IMB_FTYPE_JP2: i32 = 12;
pub const IMB_FTYPE_SVG: i32 = 13;
pub const IMB_FTYPE_DDS: i32 = 14;
pub const IMB_FTYPE_WEBP: i32 = 15;
pub const IMB_FTYPE_AVIF: i32 = 16;
pub const IMB_FTYPE_RADHDR: i32 = 17;
/// `imb_is_a_*` magic-byte checks (verbatim
/// signatures).
pub fn imb_is_a_png(mem: &[u8]) -> bool {
    mem.starts_with(&[0x89, 0x50, 0x4E, 0x47])
}
pub fn imb_is_a_jpg(mem: &[u8]) -> bool {
    mem.starts_with(&[0xFF, 0xD8])
}
pub fn imb_is_a_bmp(mem: &[u8]) -> bool {
    mem.starts_with(b"BM")
}
pub fn imb_is_a_tga(_mem: &[u8]) -> bool {
    false /* TGA has no magic — extension-only. */
}
pub fn imb_is_a_hdr(mem: &[u8]) -> bool {
    mem.starts_with(b"#?RADIANCE") || mem.starts_with(b"#?RGBE")
}
pub fn imb_is_a_dds(mem: &[u8]) -> bool {
    mem.starts_with(b"DDS ")
}
pub fn imb_is_a_webp(mem: &[u8]) -> bool {
    mem.len() > 12 && mem.starts_with(b"RIFF") && &mem[8..12] == b"WEBP"
}
pub fn imb_is_a_avif(mem: &[u8]) -> bool {
    mem.len() > 12 && &mem[4..12] == b"ftypavif" || mem.len() > 12 && &mem[4..12] == b"ftypavis"
}
pub fn imb_is_a_jp2(mem: &[u8]) -> bool {
    mem.len() > 12 && &mem[4..12] == b"ftypjp2 " || mem.starts_with(&[0x00, 0x00, 0x00, 0x0C, 0x6A, 0x50])
}
pub fn imb_is_a_psd(mem: &[u8]) -> bool {
    mem.starts_with(b"8BPS")
}
pub fn imb_is_a_iris(mem: &[u8]) -> bool {
    mem.len() > 2 && mem[0] == 0x01 && mem[1] == 0xDA
}
/// `IMB_ftype_from_extension` (verbatim table).
pub fn ftype_from_extension(ext: &str) -> i32 {
    match ext.to_ascii_lowercase().as_str() {
        ".png" => IMB_FTYPE_PNG,
        ".tga" | ".tpic" => IMB_FTYPE_TGA,
        ".jpg" | ".jpeg" => IMB_FTYPE_JPG,
        ".bmp" | ".dib" => IMB_FTYPE_BMP,
        ".exr" => IMB_FTYPE_OPENEXR,
        ".cin" => IMB_FTYPE_CINEON,
        ".dpx" => IMB_FTYPE_DPX,
        ".tif" | ".tiff" => IMB_FTYPE_TIF,
        ".hdr" => IMB_FTYPE_HDR,
        ".sgi" | ".rgb" | ".rgba" | ".bw" => IMB_FTYPE_IRIS,
        ".psd" | ".pdd" | ".psb" => IMB_FTYPE_PSD,
        ".jp2" | ".j2c" | ".j2k" => IMB_FTYPE_JP2,
        ".svg" => IMB_FTYPE_SVG,
        ".dds" => IMB_FTYPE_DDS,
        ".webp" => IMB_FTYPE_WEBP,
        ".avif" => IMB_FTYPE_AVIF,
        _ => 0,
    }
}
