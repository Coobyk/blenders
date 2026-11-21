//! Port of `DNA_pointcache_types.h` — `PointCache` + mem-cache entries.

use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `ePointCache_DataType` (short).
pub type PointCacheDataType = i16;
pub const BPHYS_DATA_INDEX: PointCacheDataType = 0;
pub const BPHYS_DATA_LOCATION: PointCacheDataType = 1;
pub const BPHYS_DATA_SMOKE_LOW: PointCacheDataType = 1;
pub const BPHYS_DATA_VELOCITY: PointCacheDataType = 2;
pub const BPHYS_DATA_SMOKE_HIGH: PointCacheDataType = 2;
pub const BPHYS_DATA_ROTATION: PointCacheDataType = 3;
pub const BPHYS_DATA_DYNAMICPAINT: PointCacheDataType = 3;
pub const BPHYS_DATA_AVELOCITY: PointCacheDataType = 4;
pub const BPHYS_DATA_XCONST: PointCacheDataType = 4;
pub const BPHYS_DATA_SIZE: PointCacheDataType = 5;
pub const BPHYS_DATA_TIMES: PointCacheDataType = 6;
pub const BPHYS_DATA_BOIDS: PointCacheDataType = 7;
pub const BPHYS_TOT_DATA: usize = 8;

/// `ePointCache_ExtraDataType` (short).
pub type PointCacheExtraDataType = i16;
pub const BPHYS_EXTRA_FLUID_SPRINGS: PointCacheExtraDataType = 1;
pub const BPHYS_EXTRA_CLOTH_ACCELERATION: PointCacheExtraDataType = 2;

/// `ePointCache_Flag`.
pub type PointCacheFlag = i32;
pub const PTCACHE_BAKED: PointCacheFlag = 1 << 0;
pub const PTCACHE_OUTDATED: PointCacheFlag = 1 << 1;
pub const PTCACHE_SIMULATION_VALID: PointCacheFlag = 1 << 2;
pub const PTCACHE_BAKING: PointCacheFlag = 1 << 3;
pub const PTCACHE_DISK_CACHE: PointCacheFlag = 1 << 6;
pub const PTCACHE_FRAMES_SKIPPED: PointCacheFlag = 1 << 8;
pub const PTCACHE_EXTERNAL: PointCacheFlag = 1 << 9;
pub const PTCACHE_READ_INFO: PointCacheFlag = 1 << 10;
pub const PTCACHE_IGNORE_LIBPATH: PointCacheFlag = 1 << 11;
pub const PTCACHE_FAKE_SMOKE: PointCacheFlag = 1 << 12;
pub const PTCACHE_IGNORE_CLEAR: PointCacheFlag = 1 << 13;
pub const PTCACHE_FLAG_INFO_DIRTY: PointCacheFlag = 1 << 14;
pub const PTCACHE_REDO_NEEDED: PointCacheFlag = PTCACHE_OUTDATED | PTCACHE_FRAMES_SKIPPED;
pub const PTCACHE_FLAGS_COPY: PointCacheFlag =
    PTCACHE_DISK_CACHE | PTCACHE_EXTERNAL | PTCACHE_IGNORE_LIBPATH;

/// `PointCacheCompression` (short).
pub type PointCacheCompression = i16;
pub const PTCACHE_COMPRESS_NO: PointCacheCompression = 0;
pub const PTCACHE_COMPRESS_LZO_DEPRECATED: PointCacheCompression = 1;
pub const PTCACHE_COMPRESS_LZMA_DEPRECATED: PointCacheCompression = 2;
pub const PTCACHE_COMPRESS_ZSTD_FILTERED: PointCacheCompression = 3;
pub const PTCACHE_COMPRESS_ZSTD_FAST_DEPRECATED: PointCacheCompression = 4;
pub const PTCACHE_COMPRESS_ZSTD_SLOW_DEPRECATED: PointCacheCompression = 8;

/// `PTCacheExtra`.
#[derive(Clone)]
pub struct PtCacheExtra {
    pub next: usize,
    pub prev: usize,
    pub type_: PointCacheExtraDataType,
    pub totdata: u32,
    /// `void *data` → opaque blob ref.
    pub data: usize,
}
impl Default for PtCacheExtra {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: 0,
            totdata: 0,
            data: NIL,
        }
    }
}

/// `PTCacheMem`.
#[derive(Clone)]
pub struct PtCacheMem {
    pub next: usize,
    pub prev: usize,
    pub frame: u32,
    pub totpoint: u32,
    pub data_types: u32,
    pub flag: u32,
    /// `void *data[BPHYS_TOT_DATA]` — per-type buffer refs.
    pub data: [usize; 8],
    /// `ListBaseT<PTCacheExtra>`.
    pub extradata: ListBase,
}
impl Default for PtCacheMem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            frame: 0,
            totpoint: 0,
            data_types: 0,
            flag: 0,
            data: [NIL; 8],
            extradata: ListBase::default(),
        }
    }
}

/// `PointCache`.
#[derive(Clone)]
pub struct PointCache {
    pub next: usize,
    pub prev: usize,
    pub flag: PointCacheFlag,
    pub step: i32,
    pub simframe: i32,
    pub startframe: i32,
    pub endframe: i32,
    pub editframe: i32,
    pub last_exact: i32,
    pub last_valid: i32,
    pub totpoint: i32,
    pub index: i32,
    pub compression: PointCacheCompression,
    /// `char name[64]`, `prev_name[64]`, `info[128]`, `path[1024]`.
    pub name: [u8; 64],
    pub prev_name: [u8; 64],
    pub info: [u8; 128],
    pub path: [u8; 1024],
    /// `char *cached_frames` → Vec.
    pub cached_frames: Vec<u8>,
    pub cached_frames_len: i32,
    /// `ListBaseT<PTCacheMem>`.
    pub mem_cache: ListBase,
    /// `PTCacheEdit *edit`.
    pub edit: usize,
    /// `void (*free_edit)(PTCacheEdit*)` → function ref (unused in Rust port).
    pub free_edit: usize,
}
impl Default for PointCache {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            flag: 0,
            step: 0,
            simframe: 0,
            startframe: 0,
            endframe: 0,
            editframe: 0,
            last_exact: 0,
            last_valid: 0,
            totpoint: 0,
            index: 0,
            compression: PTCACHE_COMPRESS_NO,
            name: [0; 64],
            prev_name: [0; 64],
            info: [0; 128],
            path: [0; 1024],
            cached_frames: Vec::new(),
            cached_frames_len: 0,
            mem_cache: ListBase::default(),
            edit: NIL,
            free_edit: NIL,
        }
    }
}
