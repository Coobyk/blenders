//! Port of `DNA_meta_types.h` — `MetaBall`, `MetaElem`.
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `eMetaBall_TexSpaceFlag`.
pub const MB_TEXSPACE_FLAG_AUTO: i8 = 1 << 0;

/// `eMetaBall_Flag`.
pub const MB_UPDATE_ALWAYS: i8 = 0;
pub const MB_UPDATE_HALFRES: i8 = 1;
pub const MB_UPDATE_FAST: i8 = 2;
pub const MB_UPDATE_NEVER: i8 = 3;

/// `eMetaBall_Flag2`.
pub const MB_DS_EXPAND: i8 = 1 << 0;

/// `eMetaElem_Type`.
pub const MB_BALL: i16 = 0;
pub const MB_TUBEX: i16 = 1;
pub const MB_TUBEY: i16 = 2;
pub const MB_TUBEZ: i16 = 3;
pub const MB_TUBE: i16 = 4;
pub const MB_PLANE: i16 = 5;
pub const MB_ELLIPSOID: i16 = 6;
pub const MB_CUBE: i16 = 7;
/// `MB_TYPE_SIZE_SQUARED`.
pub fn mb_type_size_squared(type_: i16) -> bool {
    type_ == MB_ELLIPSOID
}

/// `eMetaElem_Flag`.
pub const MB_SELECT: i16 = 1 << 0;
pub const MB_NEGATIVE: i16 = 1 << 1;
pub const MB_HIDE: i16 = 1 << 3;
pub const MB_SCALE_RAD: i16 = 1 << 4;

/// `MetaElem`.
#[derive(Clone)]
pub struct MetaElem {
    pub next: usize,
    pub prev: usize,
    /// `BoundBox *bb`.
    pub bb: usize,
    pub type_: i16,
    pub flag: i16,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub quat: [f32; 4],
    pub expx: f32,
    pub expy: f32,
    pub expz: f32,
    pub rad: f32,
    pub rad2: f32,
    pub s: f32,
    pub len: f32,
    /// `float *mat`, `*imat` → owned 4x4 matrices.
    pub mat: Option<Box<[[f32; 4]; 4]>>,
    pub imat: Option<Box<[[f32; 4]; 4]>>,
}
impl Default for MetaElem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            bb: NIL,
            type_: MB_BALL,
            flag: 0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            quat: [0.0; 4],
            expx: 0.0,
            expy: 0.0,
            expz: 0.0,
            rad: 0.0,
            rad2: 0.0,
            s: 0.0,
            len: 0.0,
            mat: None,
            imat: None,
        }
    }
}

/// `MetaBall`.
#[derive(Clone)]
pub struct MetaBall {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `ListBaseT<MetaElem>`.
    pub elems: ListBase,
    /// `ListBaseT<MetaElem> *editelems`.
    pub editelems: usize,
    /// `Material **mat` → Vec.
    pub mat: Vec<usize>,
    pub flag: i8,
    pub flag2: i8,
    pub totcol: i16,
    pub texspace_flag: i8,
    pub needs_flush_to_id: i8,
    pub texspace_location: [f32; 3],
    pub texspace_size: [f32; 3],
    pub wiresize: f32,
    pub rendersize: f32,
    pub thresh: f32,
    /// `MetaElem *lastelem`.
    pub lastelem: usize,
}
impl Default for MetaBall {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            elems: ListBase::new(),
            editelems: NIL,
            mat: Vec::new(),
            flag: MB_UPDATE_ALWAYS,
            flag2: 0,
            totcol: 0,
            texspace_flag: MB_TEXSPACE_FLAG_AUTO,
            needs_flush_to_id: 0,
            texspace_location: [0.0; 3],
            texspace_size: [1.0; 3],
            wiresize: 0.4,
            rendersize: 0.2,
            thresh: 0.6,
            lastelem: NIL,
        }
    }
}
