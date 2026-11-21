//! Port of `DNA_userdef_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `eDupli_ID_Flags`.
pub type DupliIdFlags = i32;
pub const USER_DUP_MESH: DupliIdFlags = 1 << 0;
pub const USER_DUP_CURVE: DupliIdFlags = 1 << 1;
pub const USER_DUP_SURF: DupliIdFlags = 1 << 2;
pub const USER_DUP_FONT: DupliIdFlags = 1 << 3;
pub const USER_DUP_MBALL: DupliIdFlags = 1 << 4;
pub const USER_DUP_LAMP: DupliIdFlags = 1 << 5;
pub const USER_DUP_MAT: DupliIdFlags = 1 << 7;
pub const USER_DUP_ARM: DupliIdFlags = 1 << 9;
pub const USER_DUP_ACT: DupliIdFlags = 1 << 10;
pub const USER_DUP_PSYS: DupliIdFlags = 1 << 11;
pub const USER_DUP_LIGHTPROBE: DupliIdFlags = 1 << 12;
pub const USER_DUP_GPENCIL: DupliIdFlags = 1 << 13;
pub const USER_DUP_CURVES: DupliIdFlags = 1 << 14;
pub const USER_DUP_POINTCLOUD: DupliIdFlags = 1 << 15;
pub const USER_DUP_VOLUME: DupliIdFlags = 1 << 16;
pub const USER_DUP_LATTICE: DupliIdFlags = 1 << 17;
pub const USER_DUP_CAMERA: DupliIdFlags = 1 << 18;
pub const USER_DUP_SPEAKER: DupliIdFlags = 1 << 19;
pub const USER_DUP_NTREE: DupliIdFlags = 1 << 20;
pub const USER_DUP_OBDATA: DupliIdFlags = (!0) & ((1 << 24) - 1);
pub const USER_DUP_OBJECT: DupliIdFlags = 1 << 24;
pub const USER_DUP_LINKED_ID: DupliIdFlags = 1 << 30;

