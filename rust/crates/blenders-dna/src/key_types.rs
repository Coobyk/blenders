//! Port of `DNA_key_types.h` — `Key`, `KeyBlock` (shape keys).
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `ShapekeyContainerType`.
pub const KEY_NORMAL: i8 = 0;
pub const KEY_RELATIVE: i8 = 1;

/// `ShapekeyContainerFlag`.
pub const KEY_DS_EXPAND: i16 = 1;

/// `KeyInterpolationType`.
pub type KeyInterpolationType = i16;
pub const KEY_LINEAR: KeyInterpolationType = 0;
pub const KEY_CARDINAL: KeyInterpolationType = 1;
pub const KEY_BSPLINE: KeyInterpolationType = 2;
pub const KEY_CATMULL_ROM: KeyInterpolationType = 3;

/// `KeyBlockFlag`.
pub const KEYBLOCK_MUTE: i16 = 1 << 0;
pub const KEYBLOCK_SEL: i16 = 1 << 1;
pub const KEYBLOCK_LOCKED: i16 = 1 << 2;
pub const KEYBLOCK_LOCKED_SHAPE: i16 = 1 << 3;

pub const KEYELEM_FLOAT_LEN_COORD: usize = 3;
pub const KEYELEM_ELEM_SIZE_CURVE: usize = 3;
pub const KEYELEM_ELEM_LEN_BPOINT: usize = 2;
pub const KEYELEM_FLOAT_LEN_BPOINT: usize = KEYELEM_ELEM_LEN_BPOINT * KEYELEM_ELEM_SIZE_CURVE;
pub const KEYELEM_ELEM_LEN_BEZTRIPLE: usize = 4;
pub const KEYELEM_FLOAT_LEN_BEZTRIPLE: usize = KEYELEM_ELEM_LEN_BEZTRIPLE * KEYELEM_ELEM_SIZE_CURVE;

/// `KeyBlock`.
#[derive(Clone)]
pub struct KeyBlock {
    pub next: usize,
    pub prev: usize,
    pub pos: f32,
    pub curval: f32,
    pub type_: KeyInterpolationType,
    pub relative: i16,
    pub flag: i16,
    pub totelem: i32,
    pub uid: i32,
    /// `void *data` → element data ref.
    pub data: usize,
    /// `char name[64]`, `vgroup[64]`.
    pub name: [u8; 64],
    pub vgroup: [u8; 64],
    pub slidermin: f32,
    pub slidermax: f32,
}
impl Default for KeyBlock {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            pos: 0.0,
            curval: 0.0,
            type_: KEY_LINEAR,
            relative: 0,
            flag: 0,
            totelem: 0,
            uid: 0,
            data: NIL,
            name: [0; 64],
            vgroup: [0; 64],
            slidermin: 0.0,
            slidermax: 0.0,
        }
    }
}

/// `Key`.
#[derive(Clone)]
pub struct Key {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `KeyBlock *refkey`.
    pub refkey: usize,
    /// `char elemstr[32]`.
    pub elemstr: [u8; 32],
    pub elemsize: i32,
    /// `ListBaseT<KeyBlock>`.
    pub block: ListBase,
    /// `ID *from`.
    pub from: usize,
    pub totkey: i32,
    pub flag: i16,
    pub type_: i8,
    pub ctime: f32,
    pub uidgen: i32,
}
impl Default for Key {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            refkey: NIL,
            elemstr: [0; 32],
            elemsize: 0,
            block: ListBase::new(),
            from: NIL,
            totkey: 0,
            flag: 0,
            type_: KEY_NORMAL,
            ctime: 0.0,
            uidgen: 0,
        }
    }
}
