//! Port of `DNA_effect_types.h` — legacy effect system (build/wave/legacy particles).

use crate::id::NIL;

/// `ePartEff_Flag` (short).
pub type PartEffFlag = i16;
pub const PAF_BSPLINE: PartEffFlag = 1 << 1;
pub const PAF_STATIC: PartEffFlag = 1 << 2;
pub const PAF_FACE: PartEffFlag = 1 << 3;
pub const PAF_ANIMATED: PartEffFlag = 1 << 4;
pub const PAF_UNBORN: PartEffFlag = 1 << 5;
pub const PAF_OFACE: PartEffFlag = 1 << 6;
pub const PAF_SHOWE: PartEffFlag = 1 << 7;
pub const PAF_TRAND: PartEffFlag = 1 << 8;
pub const PAF_EDISTR: PartEffFlag = 1 << 9;
pub const PAF_DIED: PartEffFlag = 1 << 11;

/// `ePartEff_Flag2` (short).
pub type PartEffFlag2 = i16;
pub const PAF_TEXTIME: PartEffFlag2 = 1;

/// `eEffect_Type` (short).
pub type EffectType = i16;
pub const EFF_BUILD: EffectType = 0;
pub const EFF_PARTICLE: EffectType = 1;
pub const EFF_WAVE: EffectType = 2;

/// `eEffect_Flag` (short).
pub type EffectFlag = i16;
pub const EFF_SELECT: EffectFlag = 1;

/// `ePartEff_SType` (short).
pub type PartEffSType = i16;
pub const PAF_NORMAL: PartEffSType = 0;
pub const PAF_VECT: PartEffSType = 1;

/// `ePartEff_TexMap` (short).
pub type PartEffTexMap = i16;
pub const PAF_TEXINT: PartEffTexMap = 0;
pub const PAF_TEXRGB: PartEffTexMap = 1;
pub const PAF_TEXGRAD: PartEffTexMap = 2;

/// `Effect` — effect list element base.
#[derive(Clone)]
pub struct Effect {
    pub next: usize,
    pub prev: usize,
    pub type_: EffectType,
    pub flag: EffectFlag,
    pub buttype: i16,
}
impl Default for Effect {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: EFF_BUILD,
            flag: 0,
            buttype: 0,
        }
    }
}

/// `BuildEff`.
#[derive(Clone)]
pub struct BuildEff {
    pub next: usize,
    pub prev: usize,
    pub type_: EffectType,
    pub flag: EffectFlag,
    pub buttype: i16,
    pub len: f32,
    pub sfra: f32,
}
impl Default for BuildEff {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: EFF_BUILD,
            flag: 0,
            buttype: 0,
            len: 0.0,
            sfra: 0.0,
        }
    }
}

/// `Particle` — legacy particle.
#[derive(Clone, Default)]
pub struct Particle {
    pub co: [f32; 3],
    pub no: [f32; 3],
    pub time: f32,
    pub lifetime: f32,
    pub mat_nr: i16,
}

/// `PartEff` — legacy particle effect.
#[derive(Clone)]
pub struct PartEff {
    pub next: usize,
    pub prev: usize,
    pub type_: EffectType,
    pub flag: PartEffFlag,
    pub buttype: i16,
    pub stype: PartEffSType,
    pub vertgroup: i16,
    pub userjit: i16,
    pub sta: f32,
    pub end: f32,
    pub lifetime: f32,
    pub totpart: i32,
    pub totkey: i32,
    pub seed: i32,
    pub normfac: f32,
    pub obfac: f32,
    pub randfac: f32,
    pub texfac: f32,
    pub randlife: f32,
    pub force: [f32; 3],
    pub damp: f32,
    pub nabla: f32,
    pub vectsize: f32,
    pub maxlen: f32,
    pub defvec: [f32; 3],
    pub mult: [f32; 4],
    pub life: [f32; 4],
    pub child: [i16; 4],
    pub mat: [i16; 4],
    pub texmap: PartEffTexMap,
    pub curmult: i16,
    pub staticstep: i16,
    pub omat: i16,
    pub timetex: i16,
    pub speedtex: i16,
    pub flag2: PartEffFlag2,
    pub flag2neg: PartEffFlag2,
    pub disp: i16,
    pub vertgroup_v: i16,
    /// `char vgroupname[64]`, `vgroupname_v[64]`.
    pub vgroupname: [u8; 64],
    pub vgroupname_v: [u8; 64],
    pub imat: [[f32; 4]; 4],
    /// `Particle *keys` → Vec.
    pub keys: Vec<Particle>,
    /// `Collection *group`.
    pub group: usize,
}
impl Default for PartEff {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: EFF_BUILD,
            flag: 0,
            buttype: 0,
            stype: PAF_NORMAL,
            vertgroup: 0,
            userjit: 0,
            sta: 0.0,
            end: 0.0,
            lifetime: 0.0,
            totpart: 0,
            totkey: 0,
            seed: 0,
            normfac: 0.0,
            obfac: 0.0,
            randfac: 0.0,
            texfac: 0.0,
            randlife: 0.0,
            force: [0.0; 3],
            damp: 0.0,
            nabla: 0.0,
            vectsize: 0.0,
            maxlen: 0.0,
            defvec: [0.0; 3],
            mult: [0.0; 4],
            life: [0.0; 4],
            child: [0; 4],
            mat: [0; 4],
            texmap: PAF_TEXINT,
            curmult: 0,
            staticstep: 0,
            omat: 0,
            timetex: 0,
            speedtex: 0,
            flag2: 0,
            flag2neg: 0,
            disp: 0,
            vertgroup_v: 0,
            vgroupname: [0; 64],
            vgroupname_v: [0; 64],
            imat: [[0.0; 4]; 4],
            keys: Vec::new(),
            group: NIL,
        }
    }
}

/// `WaveEff`.
#[derive(Clone)]
pub struct WaveEff {
    pub next: usize,
    pub prev: usize,
    pub type_: EffectType,
    pub flag: EffectFlag,
    pub buttype: i16,
    pub stype: i16,
    pub startx: f32,
    pub starty: f32,
    pub height: f32,
    pub width: f32,
    pub narrow: f32,
    pub speed: f32,
    pub minfac: f32,
    pub damp: f32,
    pub timeoffs: f32,
    pub lifetime: f32,
}
impl Default for WaveEff {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: EFF_BUILD,
            flag: 0,
            buttype: 0,
            stype: 0,
            startx: 0.0,
            starty: 0.0,
            height: 0.0,
            width: 0.0,
            narrow: 0.0,
            speed: 0.0,
            minfac: 0.0,
            damp: 0.0,
            timeoffs: 0.0,
            lifetime: 0.0,
        }
    }
}
