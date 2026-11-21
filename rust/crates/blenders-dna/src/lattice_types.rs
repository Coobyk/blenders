//! Port of `DNA_lattice_types.h` — `Lattice`, `EditLatt`.
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};
use crate::key_types::KEY_BSPLINE;
use blenders_bli::listbase::ListBase;

/// `eLattice_Flag`.
pub const LT_GRID: i16 = 1 << 0;
pub const LT_OUTSIDE: i16 = 1 << 1;
pub const LT_DS_EXPAND: i16 = 1 << 2;

pub const LT_ACTBP_NONE: i32 = -1;

/// `EditLatt`.
#[derive(Clone)]
pub struct EditLatt {
    /// `Lattice *latt`.
    pub latt: usize,
    pub shapenr: i32,
    pub needs_flush_to_id: i8,
}
impl Default for EditLatt {
    fn default() -> Self {
        Self {
            latt: NIL,
            shapenr: 0,
            needs_flush_to_id: 0,
        }
    }
}

/// `Lattice`.
#[derive(Clone)]
pub struct Lattice {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub pntsu: i16,
    pub pntsv: i16,
    pub pntsw: i16,
    pub flag: i16,
    pub opntsu: i16,
    pub opntsv: i16,
    pub opntsw: i16,
    /// `char typeu/v/w` — `KeyInterpolationType`.
    pub typeu: i8,
    pub typev: i8,
    pub typew: i8,
    pub actbp: i32,
    pub fu: f32,
    pub fv: f32,
    pub fw: f32,
    pub du: f32,
    pub dv: f32,
    pub dw: f32,
    /// `BPoint *def` → Vec.
    pub def: Vec<crate::curve_types::BPoint>,
    /// `Key *key`.
    pub key: usize,
    /// `MDeformVert *dvert` → Vec.
    pub dvert: Vec<crate::meshdata_types::MDeformVert>,
    /// `char vgroup[64]`.
    pub vgroup: [u8; 64],
    /// `ListBaseT<bDeformGroup>`.
    pub vertex_group_names: ListBase,
    pub vertex_group_active_index: i32,
    /// `EditLatt *editlatt`.
    pub editlatt: usize,
    /// `draw::LatticeBatchCache *batch_cache`.
    pub batch_cache: usize,
}
impl Default for Lattice {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            pntsu: 0,
            pntsv: 0,
            pntsw: 0,
            flag: LT_GRID,
            opntsu: 0,
            opntsv: 0,
            opntsw: 0,
            typeu: KEY_BSPLINE as i8,
            typev: KEY_BSPLINE as i8,
            typew: KEY_BSPLINE as i8,
            actbp: LT_ACTBP_NONE,
            fu: 0.0,
            fv: 0.0,
            fw: 0.0,
            du: 0.0,
            dv: 0.0,
            dw: 0.0,
            def: Vec::new(),
            key: NIL,
            dvert: Vec::new(),
            vgroup: [0; 64],
            vertex_group_names: ListBase::new(),
            vertex_group_active_index: 0,
            editlatt: NIL,
            batch_cache: NIL,
        }
    }
}
