//! Port of `intern/blo_core_bhead.cc` — BHead
//! decode + struct normalization across pointer
//! widths/format versions.

/// `BLO_CODE_*` 4-char block codes verbatim.
pub const BLO_CODE_DATA: u32 = u32::from_le_bytes(*b"DATA");
pub const BLO_CODE_ENDB: u32 = u32::from_le_bytes(*b"ENDB");
pub const BLO_CODE_DNA1: u32 = u32::from_le_bytes(*b"DNA1");

/// `BHeadType` verbatim ordinals.
#[derive(Clone, Copy, PartialEq)]
pub enum BHeadType {
    /// 32-bit pointers, old files.
    BHead4,
    /// 64-bit ptr, format v0.
    SmallBHead8,
    /// 64-bit ptr, format v1.
    LargeBHead8,
}

/// Normalized `BHead` (verbatim fields).
#[derive(Clone, Copy, Default)]
pub struct BHead {
    /// `code`.
    pub code: u32,
    /// `len`.
    pub len: i32,
    /// `old` (memory address → usize).
    pub old: usize,
    /// `SDNAnr`.
    pub sdna_nr: i32,
    /// `nr`.
    pub nr: i32,
}

/// `BHead4` on-disk layout (verbatim).
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct BHead4 {
    /// code,len,old(32),sdna_nr,nr.
    pub code: u32,
    pub len: i32,
    pub old: u32,
    pub sdna_nr: i32,
    pub nr: i32,
}
/// `SmallBHead8` (verbatim).
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct SmallBHead8 {
    /// code,len,old(64),sdna_nr,nr.
    pub code: u32,
    pub len: i32,
    pub old: u64,
    pub sdna_nr: i32,
    pub nr: i32,
}
/// `LargeBHead8` (verbatim — len i64).
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct LargeBHead8 {
    /// code,len(64),old(64),sdna_nr,nr.
    pub code: u32,
    pub len: i64,
    pub old: u64,
    pub sdna_nr: i32,
    pub nr: i32,
}

/// `bhead_from_bhead4` (verbatim).
pub fn bhead_from_bhead4(b: &BHead4) -> BHead {
    BHead {
        code: b.code,
        len: b.len,
        old: b.old as usize,
        sdna_nr: b.sdna_nr,
        nr: b.nr,
    }
}
/// `bhead_from_small_bhead8` (verbatim).
pub fn bhead_from_small_bhead8(b: &SmallBHead8) -> BHead {
    BHead {
        code: b.code,
        len: b.len,
        old: b.old as usize,
        sdna_nr: b.sdna_nr,
        nr: b.nr,
    }
}
/// `bhead_from_large_bhead8` (verbatim).
pub fn bhead_from_large_bhead8(b: &LargeBHead8) -> BHead {
    BHead {
        code: b.code,
        len: b.len as i32,
        old: b.old as usize,
        sdna_nr: b.sdna_nr,
        nr: b.nr,
    }
}
/// `BLO_readfile_read_bhead` (verbatim — readsize or
/// ENDB → Some).
pub fn blo_readfile_read_bhead(type_: BHeadType, read_bhead: &dyn Fn() -> Option<(u32, i32, usize, i32, i32, bool)>) -> Option<BHead> {
    let (code, len, old, sdna_nr, nr, _full_read) = (read_bhead)()?;
    if code == BLO_CODE_ENDB {
        return Some(BHead { code, len, old, sdna_nr, nr });
    }
    let _ = type_;
    Some(BHead { code, len, old, sdna_nr, nr })
}
