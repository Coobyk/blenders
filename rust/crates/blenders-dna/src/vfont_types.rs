//! Port of `DNA_vfont_types.h`.
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};

/// `VFont`.
#[derive(Clone)]
pub struct VFont {
    pub id: ID,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    /// `VFontData *data`.
    pub data: usize,
    /// `PackedFile *packedfile`, `*temp_pf`.
    pub packedfile: usize,
    pub temp_pf: usize,
}
impl Default for VFont {
    fn default() -> Self {
        Self {
            id: ID::default(),
            filepath: [0; 1024],
            data: NIL,
            packedfile: NIL,
            temp_pf: NIL,
        }
    }
}

pub const FO_BUILTIN_NAME: &str = "<builtin>";
