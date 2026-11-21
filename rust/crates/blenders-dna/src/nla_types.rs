//! Port of `DNA_nla_types.h` — legacy `bActionStrip`/`bActionModifier`.
#![allow(non_camel_case_types)]
use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eActStrip_Mode`.
pub const ACTSTRIPMODE_BLEND: i16 = 0;
pub const ACTSTRIPMODE_ADD: i16 = 1;

/// `eActStrip_Flag`.
pub const ACTSTRIP_SELECT: i16 = 1 << 0;
pub const ACTSTRIP_USESTRIDE: i16 = 1 << 1;
pub const ACTSTRIP_HOLDLASTFRAME: i16 = 1 << 3;
pub const ACTSTRIP_ACTIVE: i16 = 1 << 4;
pub const ACTSTRIP_LOCK_ACTION: i16 = 1 << 5;
pub const ACTSTRIP_MUTE: i16 = 1 << 6;
pub const ACTSTRIP_REVERSE: i16 = 1 << 7;
pub const ACTSTRIP_AUTO_BLENDS: i16 = 1 << 11;

/// `bActionModifier`.
#[derive(Clone)]
pub struct BActionModifier {
    pub next: usize,
    pub prev: usize,
    pub type_: i16,
    pub flag: i16,
    /// `char channel[32]`.
    pub channel: [u8; 32],
    pub noisesize: f32,
    pub turbul: f32,
    pub channels: i16,
    pub no_rot_axis: i16,
    /// `Object *ob`.
    pub ob: usize,
}
impl Default for BActionModifier {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: 0,
            flag: 0,
            channel: [0; 32],
            noisesize: 0.0,
            turbul: 0.0,
            channels: 0,
            no_rot_axis: 0,
            ob: NIL,
        }
    }
}

/// `bActionStrip`.
#[derive(Clone)]
pub struct BActionStrip {
    pub next: usize,
    pub prev: usize,
    pub flag: i16,
    pub mode: i16,
    pub stride_axis: i16,
    pub curmod: i16,
    /// `bAction *act`.
    pub act: usize,
    /// `Object *object`.
    pub object: usize,
    pub start: f32,
    pub end: f32,
    pub actstart: f32,
    pub actend: f32,
    pub actoffs: f32,
    pub stridelen: f32,
    pub repeat: f32,
    pub scale: f32,
    pub blendin: f32,
    pub blendout: f32,
    /// `char stridechannel[32]`, `offs_bone[32]`.
    pub stridechannel: [u8; 32],
    pub offs_bone: [u8; 32],
    /// `ListBaseT<bActionModifier>`.
    pub modifiers: ListBase,
}
impl Default for BActionStrip {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            flag: 0,
            mode: ACTSTRIPMODE_BLEND,
            stride_axis: 0,
            curmod: 0,
            act: NIL,
            object: NIL,
            start: 0.0,
            end: 0.0,
            actstart: 0.0,
            actend: 0.0,
            actoffs: 0.0,
            stridelen: 0.0,
            repeat: 0.0,
            scale: 0.0,
            blendin: 0.0,
            blendout: 0.0,
            stridechannel: [0; 32],
            offs_bone: [0; 32],
            modifiers: ListBase::new(),
        }
    }
}
