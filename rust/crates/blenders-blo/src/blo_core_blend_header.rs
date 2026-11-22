//! Port of `intern/blo_core_blend_header.cc` —
//! `BLENDER` magic + legacy vs v1 header decode.

/// `MIN_SIZEOFBLENDERHEADER`/`MAX_SIZEOFBLENDERHEADER`.
pub const MIN_SIZEOFBLENDERHEADER: usize = 12;
pub const MAX_SIZEOFBLENDERHEADER: usize = 17;
/// `L_ENDIAN`/`B_ENDIAN` verbatim.
pub const L_ENDIAN: i32 = 0;
pub const B_ENDIAN: i32 = 1;
/// `BLEND_FILE_FORMAT_VERSION_0`/`_1` verbatim.
pub const BLEND_FILE_FORMAT_VERSION_0: i32 = 0;
pub const BLEND_FILE_FORMAT_VERSION_1: i32 = 1;

/// `BlenderHeader` verbatim.
#[derive(Clone, Copy)]
pub struct BlenderHeader {
    /// `file_format_version` (0 legacy / 1 new).
    pub file_format_version: i32,
    /// `pointer_size` (4 or 8).
    pub pointer_size: i32,
    /// `endian` (L/B_ENDIAN).
    pub endian: i32,
    /// `file_version` (e.g. 500).
    pub file_version: i32,
}
/// `BlenderHeaderVariant` verbatim.
pub enum BlenderHeaderVariant {
    /// valid header.
    Header(BlenderHeader),
    /// `BlenderHeaderInvalid`.
    Invalid,
    /// `BlenderHeaderUnknown`.
    Unknown,
}
impl BlenderHeader {
    /// `bhead_type` (verbatim — 4→BHead4, v0→Small8,
    /// else Large8).
    pub fn bhead_type(&self) -> crate::blo_core_bhead::BHeadType {
        use crate::blo_core_bhead::BHeadType;
        if self.pointer_size == 4 {
            return BHeadType::BHead4;
        }
        if self.file_format_version == BLEND_FILE_FORMAT_VERSION_0 {
            return BHeadType::SmallBHead8;
        }
        BHeadType::LargeBHead8
    }
}

fn is_digit(b: u8) -> bool {
    b.is_ascii_digit()
}
/// `BLO_readfile_blender_header_decode` (verbatim —
/// "BLENDER" then `_`/`v` legacy or `NN-0v` v1).
pub fn blo_readfile_blender_header_decode(header_bytes: &[u8]) -> BlenderHeaderVariant {
    if header_bytes.len() < MIN_SIZEOFBLENDERHEADER {
        return BlenderHeaderVariant::Invalid;
    }
    if &header_bytes[..7] != b"BLENDER" {
        return BlenderHeaderVariant::Invalid;
    }
    let mut header = BlenderHeader {
        file_format_version: 0,
        pointer_size: 0,
        endian: 0,
        file_version: 0,
    };
    let is_legacy = header_bytes[7] == b'_' || header_bytes[7] == b'-';
    if is_legacy {
        header.file_format_version = 0;
        match header_bytes[7] {
            b'_' => header.pointer_size = 4,
            b'-' => header.pointer_size = 8,
            _ => return BlenderHeaderVariant::Unknown,
        }
        match header_bytes[8] {
            b'v' => header.endian = L_ENDIAN,
            b'V' => header.endian = B_ENDIAN,
            _ => return BlenderHeaderVariant::Unknown,
        }
        if !is_digit(header_bytes[9]) || !is_digit(header_bytes[10]) || !is_digit(header_bytes[11]) {
            return BlenderHeaderVariant::Unknown;
        }
        header.file_version = std::str::from_utf8(&header_bytes[9..12])
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        return BlenderHeaderVariant::Header(header);
    }
    if !is_digit(header_bytes[7]) || !is_digit(header_bytes[8]) {
        return BlenderHeaderVariant::Unknown;
    }
    let header_size: i32 = std::str::from_utf8(&header_bytes[7..9])
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if header_size != MAX_SIZEOFBLENDERHEADER as i32 {
        return BlenderHeaderVariant::Unknown;
    }
    if header_bytes.len() < MAX_SIZEOFBLENDERHEADER {
        return BlenderHeaderVariant::Unknown;
    }
    if header_bytes[9] != b'-' {
        return BlenderHeaderVariant::Unknown;
    }
    header.pointer_size = 8;
    if !is_digit(header_bytes[10]) || !is_digit(header_bytes[11]) {
        return BlenderHeaderVariant::Unknown;
    }
    header.file_format_version = std::str::from_utf8(&header_bytes[10..12])
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if header.file_format_version != 1 {
        return BlenderHeaderVariant::Unknown;
    }
    if header_bytes[12] != b'v' {
        return BlenderHeaderVariant::Unknown;
    }
    header.endian = L_ENDIAN;
    if !(13..17).all(|i| is_digit(header_bytes[i])) {
        return BlenderHeaderVariant::Unknown;
    }
    header.file_version = std::str::from_utf8(&header_bytes[13..17])
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    BlenderHeaderVariant::Header(header)
}
