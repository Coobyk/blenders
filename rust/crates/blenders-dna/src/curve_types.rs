//! Port of `DNA_curve_types.h` — `Curve`, `Nurb`, `BezTriple`, `BPoint`,
//! `BevList`, `EditNurb`, `CharInfo`, `TextBox`.
#![allow(non_camel_case_types)]
use crate::curve_enums::*;
use crate::id::{ID, NIL};
use crate::object_types::ObjectType;
use blenders_bli::listbase::ListBase;

/// `BevPoint`.
#[derive(Clone, Copy, Default)]
pub struct BevPoint {
    pub vec: [f32; 3],
    pub tilt: f32,
    pub radius: f32,
    pub weight: f32,
    pub offset: f32,
    pub sina: f32,
    pub cosa: f32,
    pub dir: [f32; 3],
    pub tan: [f32; 3],
    pub quat: [f32; 4],
    pub dupe_tag: i16,
}

/// `BevList`.
#[derive(Clone)]
pub struct BevList {
    pub next: usize,
    pub prev: usize,
    pub nr: i32,
    pub dupe_nr: i32,
    pub poly: i32,
    pub hole: i32,
    pub reversed: bool,
    pub charidx: i32,
    /// `int *segbevcount` → Vec.
    pub segbevcount: Vec<i32>,
    /// `float *seglen` → Vec.
    pub seglen: Vec<f32>,
    /// `BevPoint *bevpoints` → Vec.
    pub bevpoints: Vec<BevPoint>,
}
impl Default for BevList {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            nr: 0,
            dupe_nr: 0,
            poly: 0,
            hole: 0,
            reversed: false,
            charidx: 0,
            segbevcount: Vec::new(),
            seglen: Vec::new(),
            bevpoints: Vec::new(),
        }
    }
}

/// `BezTriple`.
#[derive(Clone, Copy)]
pub struct BezTriple {
    /// `float vec[3][3]` — h1, kp, h2.
    pub vec: [[f32; 3]; 3],
    pub tilt: f32,
    pub weight: f32,
    pub radius: f32,
    pub ipo: BezTripleInterpolation,
    pub h1: BezTripleHandle,
    pub h2: BezTripleHandle,
    pub f1: u8,
    pub f2: u8,
    pub f3: u8,
    /// `char hide` — doubles as `eBezTriple_KeyframeType` via `BEZKEYTYPE`.
    pub hide: i8,
    pub easing: BezTripleEasing,
    pub back: f32,
    pub amplitude: f32,
    pub period: f32,
    pub auto_handle_type: BezTripleAutoType,
}
impl Default for BezTriple {
    fn default() -> Self {
        Self {
            vec: [[0.0; 3]; 3],
            tilt: 0.0,
            weight: 0.0,
            radius: 0.0,
            ipo: 0,
            h1: 0,
            h2: 0,
            f1: 0,
            f2: 0,
            f3: 0,
            hide: 0,
            easing: 0,
            back: 0.0,
            amplitude: 0.0,
            period: 0.0,
            auto_handle_type: 0,
        }
    }
}
impl BezTriple {
    /// `BEZT_ISSEL_ANY`.
    pub fn issel_any(&self) -> bool {
        self.f2 & crate::object_types::SELECT as u8 != 0
            || self.f1 & crate::object_types::SELECT as u8 != 0
            || self.f3 & crate::object_types::SELECT as u8 != 0
    }
    /// `BEZT_ISSEL_ALL`.
    pub fn issel_all(&self) -> bool {
        self.f2 & crate::object_types::SELECT as u8 != 0
            && self.f1 & crate::object_types::SELECT as u8 != 0
            && self.f3 & crate::object_types::SELECT as u8 != 0
    }
    /// `BEZT_ISSEL_IDX`.
    pub fn issel_idx(&self, i: usize) -> bool {
        (i == 0 && self.f1 & crate::object_types::SELECT as u8 != 0)
            || (i == 1 && self.f2 & crate::object_types::SELECT as u8 != 0)
            || (i == 2 && self.f3 & crate::object_types::SELECT as u8 != 0)
    }
    /// `BEZT_IS_AUTOH`.
    pub fn is_autoh(&self) -> bool {
        (self.h1 == HD_AUTO || self.h1 == HD_AUTO_ANIM)
            && (self.h2 == HD_AUTO || self.h2 == HD_AUTO_ANIM)
    }
    /// `BEZKEYTYPE`.
    pub fn keytype(&self) -> BezTripleKeyframeType {
        self.hide
    }
    /// `BEZT_SEL_ALL`.
    pub fn sel_all(&mut self) {
        self.f1 |= BEZT_FLAG_SELECT;
        self.f2 |= BEZT_FLAG_SELECT;
        self.f3 |= BEZT_FLAG_SELECT;
    }
    /// `BEZT_DESEL_ALL`.
    pub fn desel_all(&mut self) {
        self.f1 &= !BEZT_FLAG_SELECT;
        self.f2 &= !BEZT_FLAG_SELECT;
        self.f3 &= !BEZT_FLAG_SELECT;
    }
    /// `BEZT_SEL_IDX`.
    pub fn sel_idx(&mut self, i: usize) {
        match i {
            0 => self.f1 |= BEZT_FLAG_SELECT,
            1 => self.f2 |= BEZT_FLAG_SELECT,
            2 => self.f3 |= BEZT_FLAG_SELECT,
            _ => {}
        }
    }
    /// `BEZT_DESEL_IDX`.
    pub fn desel_idx(&mut self, i: usize) {
        match i {
            0 => self.f1 &= !BEZT_FLAG_SELECT,
            1 => self.f2 &= !BEZT_FLAG_SELECT,
            2 => self.f3 &= !BEZT_FLAG_SELECT,
            _ => {}
        }
    }
}

/// `BPoint`.
#[derive(Clone, Copy)]
pub struct BPoint {
    pub vec: [f32; 4],
    pub tilt: f32,
    pub weight: f32,
    pub f1: u8,
    pub hide: i16,
    pub radius: f32,
}
impl Default for BPoint {
    fn default() -> Self {
        Self {
            vec: [0.0; 4],
            tilt: 0.0,
            weight: 0.0,
            f1: 0,
            hide: 0,
            radius: 0.0,
        }
    }
}

/// `Nurb`.
#[derive(Clone)]
pub struct Nurb {
    pub next: usize,
    pub prev: usize,
    pub type_: NurbType,
    pub mat_nr: i16,
    pub hide: i16,
    pub flag: i16,
    pub pntsu: i32,
    pub pntsv: i32,
    pub resolu: i16,
    pub resolv: i16,
    pub orderu: i16,
    pub orderv: i16,
    pub flagu: i16,
    pub flagv: i16,
    /// `float *knotsu`, `*knotsv` → Vec.
    pub knotsu: Vec<f32>,
    pub knotsv: Vec<f32>,
    /// `BPoint *bp` → Vec.
    pub bp: Vec<BPoint>,
    /// `BezTriple *bezt` → Vec.
    pub bezt: Vec<BezTriple>,
    pub tilt_interp: i16,
    pub radius_interp: i16,
    pub charidx: i32,
}
impl Default for Nurb {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: CU_POLY,
            mat_nr: 0,
            hide: 0,
            flag: 0,
            pntsu: 0,
            pntsv: 0,
            resolu: 0,
            resolv: 0,
            orderu: 0,
            orderv: 0,
            flagu: 0,
            flagv: 0,
            knotsu: Vec::new(),
            knotsv: Vec::new(),
            bp: Vec::new(),
            bezt: Vec::new(),
            tilt_interp: 0,
            radius_interp: 0,
            charidx: 0,
        }
    }
}

/// `CharInfo`.
#[derive(Clone, Copy, Default)]
pub struct CharInfo {
    pub kern: f32,
    pub mat_nr: i16,
    pub flag: i8,
}

/// `TextBox`.
#[derive(Clone, Copy, Default)]
pub struct TextBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// `CVKeyIndexMap` — `Map<const void *, CVKeyIndex *>` placeholder.
#[derive(Clone, Default)]
pub struct CvKeyIndexMap(pub usize);

/// `EditNurb`.
#[derive(Clone)]
pub struct EditNurb {
    /// `ListBaseT<Nurb>`.
    pub nurbs: ListBase,
    /// `CVKeyIndexMap *keyindex`.
    pub keyindex: usize,
    pub shapenr: i32,
    pub needs_flush_to_id: i8,
}
impl Default for EditNurb {
    fn default() -> Self {
        Self {
            nurbs: ListBase::new(),
            keyindex: NIL,
            shapenr: 0,
            needs_flush_to_id: 0,
        }
    }
}

/// `Curve` (legacy curve/font data-block).
#[derive(Clone)]
pub struct Curve {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `ListBaseT<Nurb>`.
    pub nurb: ListBase,
    /// `EditNurb *editnurb`.
    pub editnurb: usize,
    /// `Object *bevobj`, `*taperobj`, `*textoncurve`.
    pub bevobj: usize,
    pub taperobj: usize,
    pub textoncurve: usize,
    /// `Key *key`.
    pub key: usize,
    /// `Material **mat` → Vec of refs.
    pub mat: Vec<usize>,
    /// `CurveProfile *bevel_profile`.
    pub bevel_profile: usize,
    pub texspace_location: [f32; 3],
    pub texspace_size: [f32; 3],
    pub ob_type: ObjectType,
    pub texspace_flag: i8,
    pub twist_mode: CurveTwistMode,
    pub twist_smooth: f32,
    pub smallcaps_scale: f32,
    pub pathlen: i32,
    pub bevresol: i16,
    pub totcol: i16,
    pub flag: CurveFlag,
    pub offset: f32,
    pub extrude: f32,
    pub bevel_radius: f32,
    pub resolu: i16,
    pub resolv: i16,
    pub resolu_ren: i16,
    pub resolv_ren: i16,
    pub actnu: i32,
    pub actvert: i32,
    pub overflow: CurveOverflow,
    pub spacemode: CurveSpaceMode,
    pub align_y: CurveAlignY,
    pub bevel_mode: CurveBevelMode,
    pub taper_radius_mode: CurveTaperRadiusMode,
    pub fill_solver: CurveFillSolverType,
    pub fill_rule: CurveFillRuleType,
    pub spacing: f32,
    pub linedist: f32,
    pub shear: f32,
    pub fsize: f32,
    pub wordspace: f32,
    pub ulpos: f32,
    pub ulheight: f32,
    pub xof: f32,
    pub yof: f32,
    pub linewidth: f32,
    pub pos: i32,
    pub selstart: i32,
    pub selend: i32,
    pub len_char32: i32,
    pub len: i32,
    /// `char *str`.
    pub str_: Option<String>,
    /// `EditFont *editfont`.
    pub editfont: usize,
    /// `char family[64]`.
    pub family: [u8; 64],
    /// `VFont *vfont/b/i/bi`.
    pub vfont: usize,
    pub vfontb: usize,
    pub vfonti: usize,
    pub vfontbi: usize,
    /// `TextBox *tb` → Vec.
    pub tb: Vec<TextBox>,
    pub totbox: i32,
    pub actbox: i32,
    /// `CharInfo *strinfo` → Vec.
    pub strinfo: Vec<CharInfo>,
    pub curinfo: CharInfo,
    pub ctime: f32,
    pub bevfac1: f32,
    pub bevfac2: f32,
    pub bevfac1_mapping: CurveBevfacMapping,
    pub bevfac2_mapping: CurveBevfacMapping,
    pub edit_data_from_original: i8,
    /// `const Curves *curve_eval`.
    pub curve_eval: usize,
    /// `draw::CurveBatchCache *batch_cache`.
    pub batch_cache: usize,
}
impl Default for Curve {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            nurb: ListBase::new(),
            editnurb: NIL,
            bevobj: NIL,
            taperobj: NIL,
            textoncurve: NIL,
            key: NIL,
            mat: Vec::new(),
            bevel_profile: NIL,
            texspace_location: [0.0; 3],
            texspace_size: [1.0; 3],
            ob_type: ObjectType::ObCurvesLegacy,
            texspace_flag: CU_TEXSPACE_FLAG_AUTO,
            twist_mode: CU_TWIST_MINIMUM,
            twist_smooth: 0.0,
            smallcaps_scale: 0.75,
            pathlen: 100,
            bevresol: 4,
            totcol: 0,
            flag: CU_DEFORM_BOUNDS_OFF | CU_PATH_RADIUS,
            offset: 0.0,
            extrude: 0.0,
            bevel_radius: 0.0,
            resolu: 12,
            resolv: 12,
            resolu_ren: 0,
            resolv_ren: 0,
            actnu: 0,
            actvert: 0,
            overflow: CU_OVERFLOW_NONE,
            spacemode: CU_ALIGN_X_LEFT,
            align_y: CU_ALIGN_Y_TOP_BASELINE,
            bevel_mode: CU_BEV_MODE_ROUND,
            taper_radius_mode: CU_TAPER_RADIUS_OVERRIDE,
            fill_solver: CU_FILL_SOLVER_SWEEP_LINE,
            fill_rule: CU_FILL_RULE_EVEN_ODD,
            spacing: 1.0,
            linedist: 1.0,
            shear: 0.0,
            fsize: 1.0,
            wordspace: 1.0,
            ulpos: 0.0,
            ulheight: 0.05,
            xof: 0.0,
            yof: 0.0,
            linewidth: 0.0,
            pos: 0,
            selstart: 0,
            selend: 0,
            len_char32: 0,
            len: 0,
            str_: None,
            editfont: NIL,
            family: [0; 64],
            vfont: NIL,
            vfontb: NIL,
            vfonti: NIL,
            vfontbi: NIL,
            tb: Vec::new(),
            totbox: 0,
            actbox: 0,
            strinfo: Vec::new(),
            curinfo: CharInfo::default(),
            ctime: 0.0,
            bevfac1: 0.0,
            bevfac2: 1.0,
            bevfac1_mapping: CU_BEVFAC_MAP_RESOLU,
            bevfac2_mapping: CU_BEVFAC_MAP_RESOLU,
            edit_data_from_original: 0,
            curve_eval: NIL,
            batch_cache: NIL,
        }
    }
}
