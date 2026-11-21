//! Port of `source/blender/blenkernel/BKE_displist.h` types +
//! `intern/displist.cc` (partial — geometry builders arrive with the
//! curve/surface tessellation passes; `PORT_PENDING`).

/// `#DispList.type` (`eDispList_Type`).
pub const DL_POLY: i16 = 0;
/// Line segment.
pub const DL_SEGM: i16 = 1;
/// A grid surface that respects `DL_CYCL_U` & `DL_CYCL_V`.
pub const DL_SURF: i16 = 2;
/// (3) is `DL_UNUSED` upstream.
pub const DL_INDEX3: i16 = 4;
/// (5) is `DL_INDEX4` upstream.
/// DL_VERTCOL = 6 — unused.
pub const DL_VERTS: i16 = 7;

/// `#DispList.flag` (`eDispList_Flag`).
pub const DL_CYCL_U: i16 = 1 << 0;
pub const DL_CYCL_V: i16 = 1 << 1;
pub const DL_FRONT_CURVE: i16 = 1 << 2;
pub const DL_BACK_CURVE: i16 = 1 << 3;
pub const DL_REVERSED: i16 = 1 << 4;

/// `DispList` — display-list element (`next`/`prev` replaced by
/// `Vec<DispList>` storage at call sites; kept for drop-in struct
/// parity with the C).
#[derive(Clone, Default)]
pub struct DispList {
    pub next: usize,
    pub prev: usize,
    pub type_: i16,
    pub flag: i16,
    pub parts: i32,
    pub nr: i32,
    /// `col`, `rt` — currently only used for smooth flag.
    pub col: i16,
    pub rt: i16,
    /// `float *verts` — `nr * parts` xyz triples.
    pub verts: Vec<f32>,
    /// `float *nors` — optional per-vert normals.
    pub nors: Vec<f32>,
    /// `int *index`.
    pub index: Vec<i32>,
    pub charidx: i32,
    /// `totindex` — indexed array drawing surfaces.
    pub totindex: i32,
}

/// `BLI_addtail(disp, dl)` — arena-friendly append.
pub fn displist_addtail(disp: &mut Vec<DispList>, dl: DispList) {
    disp.push(dl);
}

/// `BKE_displist_find` — first element of `type`.
pub fn bke_displist_find(lb: &[DispList], type_: i16) -> Option<usize> {
    lb.iter().position(|dl| dl.type_ == type_)
}

/* PORT_PENDING: `BKE_displist_fill`, `BKE_displist_minmax`,
 * `BKE_displist_normals_add`, surf/topo helpers — arrive with
 * `intern/displist.cc` surface tessellation port. */
