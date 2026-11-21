//! Port of `DNA_cachefile_types.h` — `CacheFile` ID (Alembic/USD cache).

use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `eCacheFileType` (char).
pub type CacheFileType = i8;
pub const CACHEFILE_TYPE_ALEMBIC: CacheFileType = 1;
pub const CACHEFILE_TYPE_USD: CacheFileType = 2;
pub const CACHE_FILE_TYPE_INVALID: CacheFileType = 0;

/// `eCacheFile_Flag` (short).
pub type CacheFileFlag = i16;
pub const CACHEFILE_DS_EXPAND: CacheFileFlag = 1 << 0;
pub const CACHEFILE_UNUSED_0: CacheFileFlag = 1 << 1;

/// `eCacheFileLayer_Flag` (int).
pub type CacheFileLayerFlag = i32;
pub const CACHEFILE_LAYER_HIDDEN: CacheFileLayerFlag = 1 << 0;

/// `eCacheFile_VelocityUnit` (char).
pub type CacheFileVelocityUnit = i8;
pub const CACHEFILE_VELOCITY_UNIT_FRAME: CacheFileVelocityUnit = 0;
pub const CACHEFILE_VELOCITY_UNIT_SECOND: CacheFileVelocityUnit = 1;

/// `CacheObjectPath`.
#[derive(Clone)]
pub struct CacheObjectPath {
    pub next: usize,
    pub prev: usize,
    /// `char path[4096]`.
    pub path: [u8; 4096],
}
impl Default for CacheObjectPath {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            path: [0; 4096],
        }
    }
}

/// `CacheFileLayer`.
#[derive(Clone)]
pub struct CacheFileLayer {
    pub next: usize,
    pub prev: usize,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    pub flag: CacheFileLayerFlag,
}
impl Default for CacheFileLayer {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            filepath: [0; 1024],
            flag: 0,
        }
    }
}

/// `CacheFileHandleReaderSet` — `Set<CacheReader **>` → Vec of reader refs.
pub type CacheFileHandleReaderSet = Vec<usize>;

/// `CacheFile` — cache file ID block.
#[derive(Clone)]
pub struct CacheFile {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `ListBaseT<CacheObjectPath>`.
    pub object_paths: ListBase,
    /// `ListBaseT<CacheFileLayer>`.
    pub layers: ListBase,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    pub is_sequence: i8,
    pub forward_axis: i8,
    pub up_axis: i8,
    pub override_frame: i8,
    pub scale: f32,
    pub frame: f32,
    pub frame_offset: f32,
    pub flag: CacheFileFlag,
    pub type_: CacheFileType,
    pub active_layer: i32,
    pub velocity_unit: CacheFileVelocityUnit,
    /// `char velocity_name[64]`.
    pub velocity_name: [u8; 64],
    /// `CacheArchiveHandle *handle`.
    pub handle: usize,
    /// `char handle_filepath[1024]`.
    pub handle_filepath: [u8; 1024],
    /// `CacheFileHandleReaderSet *handle_readers`.
    pub handle_readers: usize,
}
impl Default for CacheFile {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            object_paths: ListBase::default(),
            layers: ListBase::default(),
            filepath: [0; 1024],
            is_sequence: 0,
            forward_axis: 0,
            up_axis: 0,
            override_frame: 0,
            scale: 1.0,
            frame: 0.0,
            frame_offset: 0.0,
            flag: 0,
            type_: CACHE_FILE_TYPE_INVALID,
            active_layer: 0,
            velocity_unit: 0,
            velocity_name: [0; 64],
            handle: NIL,
            handle_filepath: [0; 1024],
            handle_readers: NIL,
        }
    }
}
