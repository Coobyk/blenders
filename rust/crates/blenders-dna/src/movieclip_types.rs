//! Port of `DNA_movieclip_types.h` — `MovieClip` ID and helpers.
//! `MovieTracking`/`MovieTrackingMarker` come from `tracking_types` (stub pending).

use crate::color_types::ColorManagedColorspaceSettings;
use crate::id::{ID, NIL};
use crate::tracking_types::{MovieTracking, MovieTrackingMarker};
use blenders_bli::listbase::ListBase;

/// `eMovieClipProxy_Size` (short bitflags).
pub type MovieClipProxySize = i16;
pub const MCLIP_PROXY_SIZE_25: MovieClipProxySize = 1 << 0;
pub const MCLIP_PROXY_SIZE_50: MovieClipProxySize = 1 << 1;
pub const MCLIP_PROXY_SIZE_75: MovieClipProxySize = 1 << 2;
pub const MCLIP_PROXY_SIZE_100: MovieClipProxySize = 1 << 3;
pub const MCLIP_PROXY_UNDISTORTED_SIZE_25: MovieClipProxySize = 1 << 4;
pub const MCLIP_PROXY_UNDISTORTED_SIZE_50: MovieClipProxySize = 1 << 5;
pub const MCLIP_PROXY_UNDISTORTED_SIZE_75: MovieClipProxySize = 1 << 6;
pub const MCLIP_PROXY_UNDISTORTED_SIZE_100: MovieClipProxySize = 1 << 7;

/// `MovieClipSource`.
pub type MovieClipSource = i32;
pub const MCLIP_SRC_SEQUENCE: MovieClipSource = 1;
pub const MCLIP_SRC_MOVIE: MovieClipSource = 2;

/// `MovieClipFlag`.
pub type MovieClipFlag = i32;
pub const MCLIP_USE_PROXY: MovieClipFlag = 1 << 0;
pub const MCLIP_USE_PROXY_CUSTOM_DIR: MovieClipFlag = 1 << 1;
pub const MCLIP_DATA_EXPAND: MovieClipFlag = 1 << 3;
pub const MCLIP_PROXY_FLAGS: MovieClipFlag = MCLIP_USE_PROXY | MCLIP_USE_PROXY_CUSTOM_DIR;

/// `eMovieClipProxy_RenderSize`.
pub type MovieClipProxyRenderSize = i16;
pub const MCLIP_PROXY_RENDER_SIZE_FULL: MovieClipProxyRenderSize = 0;
pub const MCLIP_PROXY_RENDER_SIZE_25: MovieClipProxyRenderSize = 1;
pub const MCLIP_PROXY_RENDER_SIZE_50: MovieClipProxyRenderSize = 2;
pub const MCLIP_PROXY_RENDER_SIZE_75: MovieClipProxyRenderSize = 3;
pub const MCLIP_PROXY_RENDER_SIZE_100: MovieClipProxyRenderSize = 4;

/// `eMovieClipProxy_RenderFlag`.
pub type MovieClipProxyRenderFlag = i16;
pub const MCLIP_PROXY_RENDER_UNDISTORT: MovieClipProxyRenderFlag = 1;
pub const MCLIP_PROXY_RENDER_USE_FALLBACK_RENDER: MovieClipProxyRenderFlag = 2;

/// `MovieClipUser`.
#[derive(Clone)]
pub struct MovieClipUser {
    pub framenr: i32,
    pub render_size: MovieClipProxyRenderSize,
    pub render_flag: MovieClipProxyRenderFlag,
}
impl Default for MovieClipUser {
    fn default() -> Self {
        Self {
            framenr: 1,
            render_size: MCLIP_PROXY_RENDER_SIZE_FULL,
            render_flag: 0,
        }
    }
}

/// `MovieClipProxy`.
#[derive(Clone)]
pub struct MovieClipProxy {
    /// `char dir[768]`.
    pub dir: [u8; 768],
    pub quality: i16,
    pub build_size_flag: MovieClipProxySize,
}
impl Default for MovieClipProxy {
    fn default() -> Self {
        Self {
            dir: [0; 768],
            quality: 50,
            build_size_flag: MCLIP_PROXY_SIZE_25,
        }
    }
}

/// `MovieClip_RuntimeGPUTexture`.
#[derive(Clone, Default)]
pub struct MovieClipRuntimeGpuTexture {
    /// `void *next`, `*prev` (ListBase links).
    pub next: usize,
    pub prev: usize,
    pub user: MovieClipUser,
    /// `gpu::Texture *gputexture`.
    pub gputexture: usize,
}

/// `MovieClip_Runtime`.
#[derive(Clone, Default)]
pub struct MovieClipRuntime {
    /// `ListBaseT<MovieClip_RuntimeGPUTexture>`.
    pub gputextures: ListBase,
    pub last_update: u64,
}

/// `MovieClip` — movie clip ID block.
#[derive(Clone)]
pub struct MovieClip {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    pub source: MovieClipSource,
    pub lastsize: [i32; 2],
    pub aspx: f32,
    pub aspy: f32,
    /// `MovieReader *anim`.
    pub anim: usize,
    /// `MovieClipCache *cache`.
    pub cache: usize,
    /// `bGPdata *gpd`.
    pub gpd: usize,
    pub tracking: MovieTracking,
    /// `void *tracking_context`.
    pub tracking_context: usize,
    pub proxy: MovieClipProxy,
    pub flag: MovieClipFlag,
    pub len: i32,
    pub start_frame: i32,
    pub frame_offset: i32,
    pub colorspace_settings: ColorManagedColorspaceSettings,
    pub runtime: MovieClipRuntime,
}
impl Default for MovieClip {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            filepath: [0; 1024],
            source: 0,
            lastsize: [0; 2],
            aspx: 1.0,
            aspy: 1.0,
            anim: NIL,
            cache: NIL,
            gpd: NIL,
            tracking: MovieTracking::default(),
            tracking_context: NIL,
            proxy: MovieClipProxy::default(),
            flag: 0,
            len: 0,
            start_frame: 1,
            frame_offset: 0,
            colorspace_settings: ColorManagedColorspaceSettings::default(),
            runtime: MovieClipRuntime::default(),
        }
    }
}

/// `MovieClipScopes`.
#[derive(Clone)]
pub struct MovieClipScopes {
    pub ok: i16,
    pub use_track_mask: i16,
    pub track_preview_height: i32,
    pub frame_width: i32,
    pub frame_height: i32,
    pub undist_marker: MovieTrackingMarker,
    /// `ImBuf *track_search`, `*track_preview`.
    pub track_search: usize,
    pub track_preview: usize,
    pub track_pos: [f32; 2],
    pub track_disabled: i16,
    pub track_locked: i16,
    pub scene_framenr: i32,
    /// `MovieTrackingTrack *track`, `MovieTrackingMarker *marker`.
    pub track: usize,
    pub marker: usize,
    pub slide_scale: [f32; 2],
}
impl Default for MovieClipScopes {
    fn default() -> Self {
        Self {
            ok: 0,
            use_track_mask: 0,
            track_preview_height: 120,
            frame_width: 0,
            frame_height: 0,
            undist_marker: MovieTrackingMarker::default(),
            track_search: NIL,
            track_preview: NIL,
            track_pos: [0.0; 2],
            track_disabled: 0,
            track_locked: 0,
            scene_framenr: 0,
            track: NIL,
            marker: NIL,
            slide_scale: [0.0; 2],
        }
    }
}
