//! Port of `source/blender/blenloader`
//! — .blend read/write, undo memfile,
//! versioning, BHead core.

pub mod blend_validate;
pub mod blo_core_bhead;
pub mod blo_core_blend_header;
pub mod blo_core_file_reader;
pub mod readblenentry;
pub mod readfile;
pub mod readfile_tempload;
pub mod undofile;
pub mod versioning_250;
pub mod versioning_260;
pub mod versioning_270;
pub mod versioning_280;
pub mod versioning_290;
pub mod versioning_300;
pub mod versioning_400;
pub mod versioning_401;
pub mod versioning_402;
pub mod versioning_403;
pub mod versioning_404;
pub mod versioning_405;
pub mod versioning_500;
pub mod versioning_501;
pub mod versioning_502;
pub mod versioning_503;
pub mod versioning_504;
pub mod versioning_common;
pub mod versioning_defaults;
pub mod versioning_dna;
pub mod versioning_legacy;
pub mod versioning_userdef;
pub mod versioning_xxx_template;
pub mod writefile;
