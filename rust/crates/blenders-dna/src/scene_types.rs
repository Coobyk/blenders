//! Port of `DNA_scene_types.h` — `Scene`, `RenderData`, `ToolSettings`,
//! `Paint` family, image-format/bake/EEVEE settings + all scene enums.
#![allow(non_camel_case_types)]
use crate::customdata_types::CustomDataMeshMasks;
use crate::id::{ID, NIL};
use crate::layer_types::ViewLayer;
use blenders_bli::listbase::ListBase;

/* Foreign-type placeholders (ported in their own headers when reached). */
/// `CurveMapping` (DNA_color_types.h).
pub type CurveMappingRef = usize;
/// `ImageUser` (DNA_image_types.h).
#[derive(Clone, Default)]
pub struct ImageUser(pub u64);
/// `ColorManagedViewSettings` / `ColorManagedDisplaySettings` /
/// `ColorManagedColorspaceSettings` (DNA_color_types.h).
#[derive(Clone, Default)]
pub struct ColorManagedViewSettings(pub u64);
#[derive(Clone, Default)]
pub struct ColorManagedDisplaySettings(pub u64);
#[derive(Clone, Default)]
pub struct ColorManagedColorspaceSettings(pub u64);
/// `View3DCursor` (DNA_view3d_types.h).
#[derive(Clone, Default)]
pub struct View3DCursor {
    pub location: [f32; 3],
    pub rotation: [f32; 4],
    pub rotation_mode: i16,
    pub rotation_axis: [f32; 3],
    pub rotation_angle: f32,
}
/// `View3DShading` (DNA_view3d_types.h) — placeholder blob.
#[derive(Clone, Default)]
pub struct View3DShading(pub [u64; 10]);
/// `AssetWeakReference` → arena index.
pub type AssetWeakReferenceRef = usize;

/* ===== eFFMpeg enums ===== */
pub const FFM_PRESET_NONE: i32 = 0;
pub const FFM_PRESET_ULTRAFAST: i32 = 1;
pub const FFM_PRESET_SUPERFAST: i32 = 2;
pub const FFM_PRESET_VERYFAST: i32 = 3;
pub const FFM_PRESET_FASTER: i32 = 4;
pub const FFM_PRESET_FAST: i32 = 5;
pub const FFM_PRESET_MEDIUM: i32 = 6;
pub const FFM_PRESET_SLOW: i32 = 7;
pub const FFM_PRESET_SLOWER: i32 = 8;
pub const FFM_PRESET_VERYSLOW: i32 = 9;
pub const FFM_PRESET_GOOD: i32 = 10;
pub const FFM_PRESET_BEST: i32 = 11;
pub const FFM_PRESET_REALTIME: i32 = 12;

/// `eFFMpegCrf`.
pub const FFM_CRF_NONE: i32 = -1;
pub const FFM_CRF_LOSSLESS: i32 = 0;
pub const FFM_CRF_PERC_LOSSLESS: i32 = 17;
pub const FFM_CRF_HIGH: i32 = 20;
pub const FFM_CRF_MEDIUM: i32 = 23;
pub const FFM_CRF_LOW: i32 = 26;
pub const FFM_CRF_VERYLOW: i32 = 29;
pub const FFM_CRF_LOWEST: i32 = 32;
pub const FFM_CRF_CUSTOM: i32 = 128;

/// `eFFMpegAudioChannels`.
pub const FFM_CHANNELS_MONO: i32 = 1;
pub const FFM_CHANNELS_STEREO: i32 = 2;
pub const FFM_CHANNELS_SURROUND4: i32 = 4;
pub const FFM_CHANNELS_SURROUND51: i32 = 6;
pub const FFM_CHANNELS_SURROUND71: i32 = 8;

/// `eFFMpegProresProfile`.
pub const FFM_PRORES_PROFILE_422_PROXY: i32 = 0;
pub const FFM_PRORES_PROFILE_422_LT: i32 = 1;
pub const FFM_PRORES_PROFILE_422_STD: i32 = 2;
pub const FFM_PRORES_PROFILE_422_HQ: i32 = 3;
pub const FFM_PRORES_PROFILE_4444: i32 = 4;
pub const FFM_PRORES_PROFILE_4444_XQ: i32 = 5;

/// `IMB_Ffmpeg_Codec_ID`.
pub const FFMPEG_CODEC_ID_NONE: i32 = 0;
pub const FFMPEG_CODEC_ID_MPEG1VIDEO: i32 = 1;
pub const FFMPEG_CODEC_ID_MPEG2VIDEO: i32 = 2;
pub const FFMPEG_CODEC_ID_MPEG4: i32 = 12;
pub const FFMPEG_CODEC_ID_FLV1: i32 = 21;
pub const FFMPEG_CODEC_ID_DVVIDEO: i32 = 24;
pub const FFMPEG_CODEC_ID_HUFFYUV: i32 = 25;
pub const FFMPEG_CODEC_ID_H264: i32 = 27;
pub const FFMPEG_CODEC_ID_THEORA: i32 = 30;
pub const FFMPEG_CODEC_ID_FFV1: i32 = 33;
pub const FFMPEG_CODEC_ID_QTRLE: i32 = 55;
pub const FFMPEG_CODEC_ID_PNG: i32 = 61;
pub const FFMPEG_CODEC_ID_DNXHD: i32 = 99;
pub const FFMPEG_CODEC_ID_VP9: i32 = 167;
pub const FFMPEG_CODEC_ID_H265: i32 = 173;
pub const FFMPEG_CODEC_ID_AV1: i32 = 226;
pub const FFMPEG_CODEC_ID_PRORES: i32 = 147;
pub const FFMPEG_CODEC_ID_PCM_S16LE: i32 = 65536;
pub const FFMPEG_CODEC_ID_MP2: i32 = 86016;
pub const FFMPEG_CODEC_ID_MP3: i32 = 86017;
pub const FFMPEG_CODEC_ID_AAC: i32 = 86018;
pub const FFMPEG_CODEC_ID_AC3: i32 = 86019;
pub const FFMPEG_CODEC_ID_VORBIS: i32 = 86021;
pub const FFMPEG_CODEC_ID_FLAC: i32 = 86028;
pub const FFMPEG_CODEC_ID_OPUS: i32 = 86076;

/// `eFFMpegCodec_Flag`.
pub const FFMPEG_MULTIPLEX_AUDIO: i32 = 1 << 0;
pub const FFMPEG_AUTOSPLIT_OUTPUT: i32 = 1 << 1;
pub const FFMPEG_LOSSLESS_OUTPUT: i32 = 1 << 2;
pub const FFMPEG_USE_MAX_B_FRAMES: i32 = 1 << 3;

/// `FFMpegCodecData`.
#[derive(Clone)]
pub struct FfMpegCodecData {
    pub type_: i32,
    pub codec: i32,
    pub audio_codec: i32,
    pub video_bitrate: i32,
    pub audio_bitrate: i32,
    pub audio_mixrate: i32,
    pub audio_channels: i32,
    pub audio_volume: f32,
    pub gop_size: i32,
    pub max_b_frames: i32,
    pub flags: i32,
    pub constant_rate_factor: i32,
    pub custom_constant_rate_factor: i32,
    pub ffmpeg_preset: i32,
    pub ffmpeg_prores_profile: i32,
    pub rc_min_rate: i32,
    pub rc_max_rate: i32,
    pub rc_buffer_size: i32,
    pub mux_packet_size: i32,
    pub mux_rate: i32,
}
impl Default for FfMpegCodecData {
    fn default() -> Self {
        Self {
            type_: 0,
            codec: 0,
            audio_codec: 0,
            video_bitrate: 0,
            audio_bitrate: 192,
            audio_mixrate: 48000,
            audio_channels: FFM_CHANNELS_STEREO,
            audio_volume: 1.0,
            gop_size: 0,
            max_b_frames: 0,
            flags: 0,
            constant_rate_factor: FFM_CRF_LOSSLESS,
            custom_constant_rate_factor: 23,
            ffmpeg_preset: FFM_PRESET_NONE,
            ffmpeg_prores_profile: FFM_PRORES_PROFILE_422_PROXY,
            rc_min_rate: 0,
            rc_max_rate: 0,
            rc_buffer_size: 0,
            mux_packet_size: 0,
            mux_rate: 0,
        }
    }
}
impl FfMpegCodecData {
    pub fn codec_id_get(&self) -> i32 {
        self.codec
    }
    pub fn audio_codec_id_get(&self) -> i32 {
        self.audio_codec
    }
    pub fn codec_id_set(&mut self, codec_id: i32) {
        self.codec = codec_id;
    }
    pub fn audio_codec_id_set(&mut self, codec_id: i32) {
        self.audio_codec = codec_id;
    }
}

/// `eAudio_Flag`.
pub const AUDIO_MUTE: i16 = 1 << 0;
pub const AUDIO_SYNC: i16 = 1 << 1;
pub const AUDIO_SCRUB: i16 = 1 << 2;
pub const AUDIO_VOLUME_ANIMATED: i16 = 1 << 3;

/// `AudioData`.
#[derive(Clone)]
pub struct AudioData {
    pub mixrate: i32,
    pub main: f32,
    pub speed_of_sound: f32,
    pub doppler_factor: f32,
    pub distance_model: i32,
    pub flag: i16,
    pub volume: f32,
}
impl Default for AudioData {
    fn default() -> Self {
        Self {
            mixrate: 0,
            main: 0.0,
            speed_of_sound: 343.3,
            doppler_factor: 1.0,
            distance_model: 2,
            flag: AUDIO_SYNC,
            volume: 1.0,
        }
    }
}

/// `SceneRenderLayer` (DNA_DEPRECATED).
#[derive(Clone)]
pub struct SceneRenderLayer {
    pub next: usize,
    pub prev: usize,
    /// DNA_DEPRECATED `char name[64]`.
    pub name: [u8; 64],
    /// DNA_DEPRECATED `Material *mat_override`, `World *world_override`.
    pub mat_override: usize,
    pub world_override: usize,
    /// DNA_DEPRECATED `lay`, `lay_zmask`, `lay_exclude`.
    pub lay: u32,
    pub lay_zmask: u32,
    pub lay_exclude: u32,
    /// DNA_DEPRECATED `eSceneLayer_Flag layflag`.
    pub layflag: i16,
    /// DNA_DEPRECATED.
    pub passflag: i32,
    pub pass_xor: i32,
    pub samples: i32,
    pub pass_alpha_threshold: f32,
    /// DNA_DEPRECATED `IDProperty *prop`.
    pub prop: usize,
    /// DNA_DEPRECATED `FreestyleConfig`.
    pub freestyle_config: crate::layer_types::FreestyleConfig,
}
impl Default for SceneRenderLayer {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            mat_override: NIL,
            world_override: NIL,
            lay: 0,
            lay_zmask: 0,
            lay_exclude: 0,
            layflag: 0,
            passflag: 0,
            pass_xor: 0,
            samples: 0,
            pass_alpha_threshold: 0.0,
            prop: NIL,
            freestyle_config: crate::layer_types::FreestyleConfig::default(),
        }
    }
}

/// `eSceneView_Flag` (DNA_scene_enums.h) placeholder type.
pub type SceneViewFlag = u8;
/// `SceneRenderView`.
#[derive(Clone)]
pub struct SceneRenderView {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`, `char suffix[64]`.
    pub name: [u8; 64],
    pub suffix: [u8; 64],
    pub viewflag: SceneViewFlag,
}
impl Default for SceneRenderView {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            suffix: [0; 64],
            viewflag: 0,
        }
    }
}

pub const STEREO_RIGHT_NAME: &str = "right";
pub const STEREO_LEFT_NAME: &str = "left";
pub const STEREO_RIGHT_SUFFIX: &str = "_R";
pub const STEREO_LEFT_SUFFIX: &str = "_L";

/// `eStereo3dFlag`/`eStereoDisplayMode`/`eStereo3dAnaglyphType`/`eStereo3dInterlaceType` placeholders.
pub const S3D_DISPLAY_ANAGLYPH: i16 = 0;
pub const S3D_ANAGLYPH_REDCYAN: i16 = 0;
pub const S3D_INTERLACE_ROW: i16 = 0;

/// `Stereo3dFormat`.
#[derive(Clone, Default)]
pub struct Stereo3dFormat {
    pub flag: i16,
    pub display_mode: i16,
    pub anaglyph_type: i16,
    pub interlace_type: i16,
}
impl Stereo3dFormat {
    pub fn new() -> Self {
        Self {
            flag: 0,
            display_mode: S3D_DISPLAY_ANAGLYPH,
            anaglyph_type: S3D_ANAGLYPH_REDCYAN,
            interlace_type: S3D_INTERLACE_ROW,
        }
    }
}

/// `MediaType`.
pub const MEDIA_TYPE_IMAGE: i8 = 0;
pub const MEDIA_TYPE_MULTI_LAYER_IMAGE: i8 = 1;
pub const MEDIA_TYPE_VIDEO: i8 = 2;

/// `eMediaType_Legacy`.
pub const R_IMF_IMTYPE_TARGA: i8 = 0;
pub const R_IMF_IMTYPE_IRIS: i8 = 1;
pub const R_IMF_IMTYPE_JPEG90: i8 = 4;
pub const R_IMF_IMTYPE_IRIZ: i8 = 7;
pub const R_IMF_IMTYPE_RAWTGA: i8 = 14;
pub const R_IMF_IMTYPE_PNG: i8 = 17;
pub const R_IMF_IMTYPE_BMP: i8 = 20;
pub const R_IMF_IMTYPE_RADHDR: i8 = 21;
pub const R_IMF_IMTYPE_TIFF: i8 = 22;
pub const R_IMF_IMTYPE_OPENEXR: i8 = 23;
pub const R_IMF_IMTYPE_FFMPEG: i8 = 24;
pub const R_IMF_IMTYPE_CINEON: i8 = 26;
pub const R_IMF_IMTYPE_DPX: i8 = 27;
pub const R_IMF_IMTYPE_MULTILAYER: i8 = 28;
pub const R_IMF_IMTYPE_DDS: i8 = 29;
pub const R_IMF_IMTYPE_JP2: i8 = 30;
pub const R_IMF_IMTYPE_PSD: i8 = 34;
pub const R_IMF_IMTYPE_WEBP: i8 = 35;
pub const R_IMF_IMTYPE_AVIF: i8 = 37;
pub const R_IMF_IMTYPE_INVALID: i8 = -1;

/// `eMediaType_Flag`.
pub const R_IMF_FLAG_PREVIEW_JPG: i8 = 1 << 1;

/// `eImageFormatDepth`.
pub const R_IMF_CHAN_DEPTH_8: i8 = 1 << 1;
pub const R_IMF_CHAN_DEPTH_10: i8 = 1 << 2;
pub const R_IMF_CHAN_DEPTH_12: i8 = 1 << 3;
pub const R_IMF_CHAN_DEPTH_16: i8 = 1 << 4;
pub const R_IMF_CHAN_DEPTH_32: i8 = 1 << 6;

/// `eMediaType_ExrCodec`.
pub const R_IMF_EXR_CODEC_NONE: i8 = 0;
pub const R_IMF_EXR_CODEC_PXR24: i8 = 1;
pub const R_IMF_EXR_CODEC_ZIP: i8 = 2;
pub const R_IMF_EXR_CODEC_PIZ: i8 = 3;
pub const R_IMF_EXR_CODEC_RLE: i8 = 4;
pub const R_IMF_EXR_CODEC_ZIPS: i8 = 5;
pub const R_IMF_EXR_CODEC_B44: i8 = 6;
pub const R_IMF_EXR_CODEC_B44A: i8 = 7;
pub const R_IMF_EXR_CODEC_DWAA: i8 = 8;
pub const R_IMF_EXR_CODEC_DWAB: i8 = 9;
pub const R_IMF_EXR_CODEC_HTJ2K: i8 = 10;
pub const R_IMF_EXR_CODEC_MAX: i8 = 11;

/// `eMediaType_ExrFlag`.
pub const R_IMF_EXR_FLAG_MULTIPART: i8 = 1 << 0;

/// `eMediaType_Jp2Flag`.
pub const R_IMF_JP2_FLAG_YCC: i8 = 1 << 0;
pub const R_IMF_JP2_FLAG_CINE_PRESET: i8 = 1 << 1;
pub const R_IMF_JP2_FLAG_CINE_48: i8 = 1 << 2;

/// `eMediaType_Jp2Codec`.
pub const R_IMF_JP2_CODEC_JP2: i8 = 0;
pub const R_IMF_JP2_CODEC_J2K: i8 = 1;

/// `eMediaType_CineonFlag`.
pub const R_IMF_CINEON_FLAG_LOG: i8 = 1 << 0;

/// `eMediaType_TiffCodec`.
pub const R_IMF_TIFF_CODEC_DEFLATE: i8 = 0;
pub const R_IMF_TIFF_CODEC_LZW: i8 = 1;
pub const R_IMF_TIFF_CODEC_PACKBITS: i8 = 2;
pub const R_IMF_TIFF_CODEC_NONE: i8 = 3;

/// `eMediaType_ColorManagement`.
pub const R_IMF_COLOR_MANAGEMENT_FOLLOW_SCENE: i8 = 0;
pub const R_IMF_COLOR_MANAGEMENT_OVERRIDE: i8 = 1;

/// `eImageFormat_ViewsFormat`/`eSceneViews_Format` placeholders.
pub type ImageFormatViewsFormat = i16;
pub type SceneViewsFormat = i16;

/// `ImageFormatData`.
#[derive(Clone)]
pub struct ImageFormatData {
    pub media_type: i8,
    pub imtype: i8,
    pub depth: i8,
    /// `ImColorMode`.
    pub color_mode: i8,
    pub flag: i8,
    pub quality: i8,
    pub compress: i8,
    pub exr_codec: i8,
    pub exr_flag: i8,
    pub jp2_flag: i8,
    pub jp2_codec: i8,
    pub tiff_codec: i8,
    pub cineon_flag: i8,
    pub cineon_white: i16,
    pub cineon_black: i16,
    pub cineon_gamma: f32,
    pub stereo3d_format: Stereo3dFormat,
    pub views_format: ImageFormatViewsFormat,
    pub color_management: i8,
    pub view_settings: ColorManagedViewSettings,
    pub display_settings: ColorManagedDisplaySettings,
    pub linear_colorspace_settings: ColorManagedColorspaceSettings,
}
impl Default for ImageFormatData {
    fn default() -> Self {
        Self {
            media_type: MEDIA_TYPE_IMAGE,
            imtype: R_IMF_IMTYPE_PNG,
            depth: R_IMF_CHAN_DEPTH_8,
            color_mode: crate::small_types::ImColorMode::Rgba as i8,
            flag: 0,
            quality: 90,
            compress: 15,
            exr_codec: 0,
            exr_flag: R_IMF_EXR_FLAG_MULTIPART,
            jp2_flag: 0,
            jp2_codec: 0,
            tiff_codec: 0,
            cineon_flag: 0,
            cineon_white: 0,
            cineon_black: 0,
            cineon_gamma: 0.0,
            stereo3d_format: Stereo3dFormat::new(),
            views_format: 0,
            color_management: 0,
            view_settings: ColorManagedViewSettings::default(),
            display_settings: ColorManagedDisplaySettings::default(),
            linear_colorspace_settings: ColorManagedColorspaceSettings::default(),
        }
    }
}

/// `eBakeType`.
pub const R_BAKE_NORMALS: i32 = 0;
pub const R_BAKE_DISPLACEMENT: i32 = 1;
pub const R_BAKE_AO: i32 = 2;
pub const R_BAKE_VECTOR_DISPLACEMENT: i32 = 3;

/// `eBake_Flag`.
pub const R_BAKE_CLEAR: i16 = 1 << 0;
pub const R_BAKE_TO_ACTIVE: i16 = 1 << 2;
pub const R_BAKE_MULTIRES: i16 = 1 << 4;
pub const R_BAKE_LORES_MESH: i16 = 1 << 5;
pub const R_BAKE_CAGE: i16 = 1 << 8;
pub const R_BAKE_SPLIT_MAT: i16 = 1 << 9;
pub const R_BAKE_AUTO_NAME: i16 = 1 << 10;

/// `eBakeMarginType`.
pub const R_BAKE_ADJACENT_FACES: i8 = 0;
pub const R_BAKE_EXTEND: i8 = 1;

/// `eBakeNormalSwizzle`.
pub const R_BAKE_POSX: i8 = 0;
pub const R_BAKE_POSY: i8 = 1;
pub const R_BAKE_POSZ: i8 = 2;
pub const R_BAKE_NEGX: i8 = 3;
pub const R_BAKE_NEGY: i8 = 4;
pub const R_BAKE_NEGZ: i8 = 5;

/// `eBakeTarget`.
pub const R_BAKE_TARGET_IMAGE_TEXTURES: i8 = 0;
pub const R_BAKE_TARGET_VERTEX_COLORS: i8 = 1;

/// `eBakeSaveMode`.
pub const R_BAKE_SAVE_INTERNAL: i8 = 0;
pub const R_BAKE_SAVE_EXTERNAL: i8 = 1;

/// `eBakeViewFrom`.
pub const R_BAKE_VIEW_FROM_ABOVE_SURFACE: i8 = 0;
pub const R_BAKE_VIEW_FROM_ACTIVE_CAMERA: i8 = 1;

/// `eBakePassFilter`.
pub const R_BAKE_PASS_FILTER_NONE: i32 = 0;
pub const R_BAKE_PASS_FILTER_UNUSED: i32 = 1 << 0;
pub const R_BAKE_PASS_FILTER_EMIT: i32 = 1 << 1;
pub const R_BAKE_PASS_FILTER_DIFFUSE: i32 = 1 << 2;
pub const R_BAKE_PASS_FILTER_GLOSSY: i32 = 1 << 3;
pub const R_BAKE_PASS_FILTER_TRANSM: i32 = 1 << 4;
pub const R_BAKE_PASS_FILTER_SUBSURFACE: i32 = 1 << 5;
pub const R_BAKE_PASS_FILTER_DIRECT: i32 = 1 << 6;
pub const R_BAKE_PASS_FILTER_INDIRECT: i32 = 1 << 7;
pub const R_BAKE_PASS_FILTER_COLOR: i32 = 1 << 8;
pub const R_BAKE_PASS_FILTER_ALL: i32 = !0;

/// `eBakeSpace`.
pub const R_BAKE_SPACE_CAMERA: i8 = 0;
pub const R_BAKE_SPACE_WORLD: i8 = 1;
pub const R_BAKE_SPACE_OBJECT: i8 = 2;
pub const R_BAKE_SPACE_TANGENT: i8 = 3;

/// `BakeData`.
#[derive(Clone)]
pub struct BakeData {
    pub im_format: ImageFormatData,
    /// `char filepath[FILE_MAX]`.
    pub filepath: [u8; 1024],
    pub type_: i32,
    pub width: i16,
    pub height: i16,
    pub margin: i16,
    pub flag: i16,
    pub cage_extrusion: f32,
    pub max_ray_distance: f32,
    pub pass_filter: i32,
    pub normal_swizzle: [i8; 3],
    pub normal_space: i8,
    pub displacement_space: i8,
    pub target: i8,
    pub save_mode: i8,
    pub margin_type: i8,
    pub view_from: i8,
    /// `Object *cage_object`.
    pub cage_object: usize,
}
impl Default for BakeData {
    fn default() -> Self {
        let mut filepath = [0u8; 1024];
        filepath[..2].copy_from_slice(b"//");
        Self {
            im_format: ImageFormatData::default(),
            filepath,
            type_: R_BAKE_NORMALS,
            width: 512,
            height: 512,
            margin: 16,
            flag: R_BAKE_CLEAR,
            cage_extrusion: 0.0,
            max_ray_distance: 0.0,
            pass_filter: R_BAKE_PASS_FILTER_ALL,
            normal_swizzle: [R_BAKE_POSX, R_BAKE_POSY, R_BAKE_POSZ],
            normal_space: R_BAKE_SPACE_TANGENT,
            displacement_space: R_BAKE_SPACE_OBJECT,
            target: R_BAKE_TARGET_IMAGE_TEXTURES,
            save_mode: R_BAKE_SAVE_INTERNAL,
            margin_type: R_BAKE_ADJACENT_FACES,
            view_from: R_BAKE_VIEW_FROM_ABOVE_SURFACE,
            cage_object: NIL,
        }
    }
}

/// `eQualityOption`.
pub const SCE_PERF_HQ_NORMALS: i16 = 1 << 0;

/// `eHairType`.
pub const SCE_HAIR_SHAPE_STRAND: i16 = 0;
pub const SCE_HAIR_SHAPE_STRIP: i16 = 1;
pub const SCE_HAIR_SHAPE_CYLINDER: i16 = 2;

/// `eRender_MotionBlurPosition`.
pub const SCE_MB_CENTER: i32 = 0;
pub const SCE_MB_START: i32 = 1;
pub const SCE_MB_END: i32 = 2;

/// `eCompositorDevice`.
pub const SCE_COMPOSITOR_DEVICE_CPU: i32 = 0;
pub const SCE_COMPOSITOR_DEVICE_GPU: i32 = 1;

/// `eCompositorCacheFlags`.
pub const SCE_COMPOSITOR_CACHE_NONE: u8 = 0;
pub const SCE_COMPOSITOR_CACHE_FRAMES: u8 = 1 << 0;

/// `CompositorProxyFlags`.
pub const COMPOSITOR_PROXY_NONE: u8 = 0;
pub const COMPOSITOR_PROXY_USE_PREVIEW_SCALE: u8 = 1 << 0;
pub const COMPOSITOR_PROXY_ADAPT_TO_RENDER_SCALE: u8 = 1 << 1;

/// `eCompositorPrecision`.
pub const SCE_COMPOSITOR_PRECISION_AUTO: i32 = 0;
pub const SCE_COMPOSITOR_PRECISION_FULL: i32 = 1;

/// `eCompositorDenoiseDevice`.
pub const SCE_COMPOSITOR_DENOISE_DEVICE_AUTO: i32 = 0;
pub const SCE_COMPOSITOR_DENOISE_DEVICE_CPU: i32 = 1;
pub const SCE_COMPOSITOR_DENOISE_DEVICE_GPU: i32 = 2;

/// `eCompositorDenoiseQuality`.
pub const SCE_COMPOSITOR_DENOISE_HIGH: i32 = 0;
pub const SCE_COMPOSITOR_DENOISE_BALANCED: i32 = 1;
pub const SCE_COMPOSITOR_DENOISE_FAST: i32 = 2;

/// `eRenderOutputMode`.
pub const R_SAVE_MODE_DEFAULT: i32 = 0;
pub const R_SAVE_MODE_DISABLED: i32 = 1;

/// `eRender_TimeJumpUnit`.
pub const SCE_TIME_JUMP_FRAME: i32 = 0;
pub const SCE_TIME_JUMP_SECOND: i32 = 1;

/// `eRender_Flag`.
pub const SCER_PRV_RANGE: i16 = 1 << 0;
pub const SCER_LOCK_FRAME_SELECTION: i16 = 1 << 1;
pub const SCER_ALLOW_PREROLL: i16 = 1 << 2;
pub const SCER_SHOW_SUBFRAME: i16 = 1 << 3;
pub const SCER_WRAP_TIMELINE_NAVIGATION: i16 = 1 << 4;

/// `eRender_Mode`.
pub const R_SIMPLIFY_NORMALS: i32 = 1 << 1;
pub const R_BORDER: i32 = 1 << 9;
pub const R_CROP: i32 = 1 << 11;
pub const R_NO_CAMERA_SWITCH: i32 = 1 << 12;
pub const R_MBLUR: i32 = 1 << 14;
pub const R_FIXED_THREADS: i32 = 1 << 19;
pub const R_NO_OVERWRITE: i32 = 1 << 22;
pub const R_TOUCH: i32 = 1 << 23;
pub const R_SIMPLIFY: i32 = 1 << 24;
pub const R_EDGE_FRS: i32 = 1 << 25;
pub const R_PERSISTENT_DATA: i32 = 1 << 26;
pub const R_SAVE_OUTPUT: i32 = 1 << 28;

/// `eRender_SeqFlag`.
pub const R_SEQ_OVERRIDE_SCENE_SETTINGS: i8 = 1 << 5;

/// `eRender_SceMode`.
pub const R_DOSEQ: i32 = 1 << 0;
pub const R_BG_RENDER: i32 = 1 << 1;
pub const R_PASSEPARTOUT: i32 = 1 << 2;
pub const R_BUTS_PREVIEW: i32 = 1 << 3;
pub const R_EXTENSION: i32 = 1 << 4;
pub const R_MATNODE_PREVIEW: i32 = 1 << 5;
pub const R_DOCOMP: i32 = 1 << 6;
pub const R_COMP_CROP: i32 = 1 << 7;
pub const R_SINGLE_LAYER: i32 = 1 << 9;
pub const R_NO_IMAGE_LOAD: i32 = 1 << 12;
pub const R_NO_FRAME_UPDATE: i32 = 1 << 14;
pub const R_TEXNODE_PREVIEW: i32 = 1 << 18;
pub const R_EXR_CACHE_FILE: i32 = 1 << 20;
pub const R_MULTIVIEW: i32 = 1 << 21;
pub const R_USE_TEXTURE_CACHE: i32 = 1 << 22;
pub const R_TEXTURE_CACHE_AUTO_GENERATE: i32 = 1 << 23;

/// `eRender_Stamp`.
pub const R_STAMP_TIME: i32 = 1 << 0;
pub const R_STAMP_FRAME: i32 = 1 << 1;
pub const R_STAMP_DATE: i32 = 1 << 2;
pub const R_STAMP_CAMERA: i32 = 1 << 3;
pub const R_STAMP_SCENE: i32 = 1 << 4;
pub const R_STAMP_NOTE: i32 = 1 << 5;
pub const R_STAMP_DRAW: i32 = 1 << 6;
pub const R_STAMP_MARKER: i32 = 1 << 7;
pub const R_STAMP_FILENAME: i32 = 1 << 8;
pub const R_STAMP_SEQSTRIP: i32 = 1 << 9;
pub const R_STAMP_RENDERTIME: i32 = 1 << 10;
pub const R_STAMP_CAMERALENS: i32 = 1 << 11;
pub const R_STAMP_STRIPMETA: i32 = 1 << 12;
pub const R_STAMP_MEMORY: i32 = 1 << 13;
pub const R_STAMP_HIDE_LABELS: i32 = 1 << 14;
pub const R_STAMP_FRAME_RANGE: i32 = 1 << 15;
pub const R_STAMP_HOSTNAME: i32 = 1 << 16;
pub const R_STAMP_ALL: i32 = R_STAMP_TIME
    | R_STAMP_FRAME
    | R_STAMP_DATE
    | R_STAMP_CAMERA
    | R_STAMP_SCENE
    | R_STAMP_NOTE
    | R_STAMP_MARKER
    | R_STAMP_FILENAME
    | R_STAMP_SEQSTRIP
    | R_STAMP_RENDERTIME
    | R_STAMP_CAMERALENS
    | R_STAMP_MEMORY
    | R_STAMP_HIDE_LABELS
    | R_STAMP_FRAME_RANGE
    | R_STAMP_HOSTNAME;

/// `eRender_AlphaMode`.
pub const R_ADDSKY: i8 = 0;
pub const R_ALPHAPREMUL: i8 = 1;

/// `eRender_ColorMgtFlag`.
pub const R_COLOR_MANAGEMENT: i32 = 1 << 0;

/// `eRender_LineThicknessMode`.
pub const R_LINE_THICKNESS_ABSOLUTE: i32 = 1;
pub const R_LINE_THICKNESS_RELATIVE: i32 = 2;

/// `RenderData`.
#[derive(Clone)]
pub struct RenderData {
    pub im_format: ImageFormatData,
    pub ffcodecdata: FfMpegCodecData,
    pub cfra: i32,
    pub sfra: i32,
    pub efra: i32,
    pub subframe: f32,
    pub psfra: i32,
    pub pefra: i32,
    pub images: i32,
    pub framapto: i32,
    pub flag: i16,
    pub threads: i16,
    pub framelen: f32,
    pub frame_step: i32,
    pub dimensionspreset: i16,
    pub size: i16,
    pub xsch: i32,
    pub ysch: i32,
    /// DNA_DEPRECATED `tilex`, `tiley`, `color_mode`, `imtype`, `subimtype`, `quality`.
    pub tilex: i32,
    pub tiley: i32,
    pub color_mode: i16,
    pub imtype: i16,
    pub subimtype: i16,
    pub quality: i16,
    pub use_lock_interface: i8,
    pub scemode: i32,
    pub mode: i32,
    pub frs_sec: i16,
    pub alphamode: i8,
    pub border: crate::vec_types::Rctf,
    /// `ListBaseT<SceneRenderLayer>`.
    pub layers: ListBase,
    /// DNA_DEPRECATED `actlay`.
    pub actlay: i16,
    pub xasp: f32,
    pub yasp: f32,
    pub ppm_factor: f32,
    pub ppm_base: f32,
    pub frs_sec_base: f32,
    pub gauss: f32,
    pub color_mgt_flag: i32,
    pub dither_intensity: f32,
    /// DNA_DEPRECATED `bake_*`.
    pub bake_mode: i16,
    pub bake_flag: i16,
    pub bake_margin: i16,
    pub bake_margin_type: i16,
    /// `char pic[FILE_MAX]`.
    pub pic: [u8; 1024],
    pub stamp: i32,
    pub stamp_font_id: i16,
    /// `char stamp_udata[768]`.
    pub stamp_udata: [u8; 768],
    pub fg_stamp: [f32; 4],
    pub bg_stamp: [f32; 4],
    pub seq_prev_type: i8,
    pub seq_flag: i8,
    pub simplify_subsurf: i16,
    pub simplify_subsurf_render: i16,
    pub simplify_gpencil: i16,
    pub simplify_particles: f32,
    pub simplify_particles_render: f32,
    pub simplify_volumes: f32,
    pub line_thickness_mode: i32,
    pub unit_line_thickness: f32,
    /// `char engine[32]`.
    pub engine: [u8; 32],
    pub perf_flag: i16,
    pub anisotropic_filter: i16,
    pub bake: BakeData,
    pub preview_pixel_size: i16,
    /// `ListBaseT<SceneRenderView>`.
    pub views: ListBase,
    pub actview: i16,
    pub views_format: SceneViewsFormat,
    pub hair_type: i16,
    pub hair_subdiv: i16,
    pub motion_blur_shutter: f32,
    pub motion_blur_position: i32,
    /// `CurveMapping mblur_shutter_curve` → owned curve ref.
    pub mblur_shutter_curve: CurveMappingRef,
    pub compositor_device: i32,
    pub compositor_cache_flags: u8,
    pub compositor_proxy_flags: u8,
    pub compositor_proxy_preview_scale: i8,
    pub compositor_precision: i32,
    pub compositor_denoise_device: i32,
    pub compositor_denoise_preview_quality: i32,
    pub compositor_denoise_final_quality: i32,
    pub time_jump_delta: f32,
    pub time_jump_unit: i32,
}
impl Default for RenderData {
    fn default() -> Self {
        let mut pic = [0u8; 1024];
        pic[..2].copy_from_slice(b"//");
        Self {
            im_format: ImageFormatData::default(),
            ffcodecdata: FfMpegCodecData::default(),
            cfra: 1,
            sfra: 1,
            efra: 250,
            subframe: 0.0,
            psfra: 0,
            pefra: 0,
            images: 100,
            framapto: 100,
            flag: 0,
            threads: 1,
            framelen: 1.0,
            frame_step: 1,
            dimensionspreset: 0,
            size: 100,
            xsch: 1920,
            ysch: 1080,
            tilex: 256,
            tiley: 256,
            color_mode: 0,
            imtype: 0,
            subimtype: 0,
            quality: 0,
            use_lock_interface: 0,
            scemode: R_DOCOMP | R_DOSEQ | R_EXTENSION | R_USE_TEXTURE_CACHE,
            mode: R_SAVE_OUTPUT,
            frs_sec: 24,
            alphamode: 0,
            border: crate::vec_types::Rctf {
                xmin: 0.0,
                xmax: 1.0,
                ymin: 0.0,
                ymax: 1.0,
            },
            layers: ListBase::new(),
            actlay: 0,
            xasp: 1.0,
            yasp: 1.0,
            ppm_factor: 72.0,
            ppm_base: 0.0254,
            frs_sec_base: 1.0,
            gauss: 1.5,
            color_mgt_flag: R_COLOR_MANAGEMENT,
            dither_intensity: 1.0,
            bake_mode: 0,
            bake_flag: 0,
            bake_margin: 0,
            bake_margin_type: 0,
            pic,
            stamp: R_STAMP_TIME
                | R_STAMP_FRAME
                | R_STAMP_DATE
                | R_STAMP_CAMERA
                | R_STAMP_SCENE
                | R_STAMP_FILENAME
                | R_STAMP_RENDERTIME,
            stamp_font_id: 12,
            stamp_udata: [0; 768],
            fg_stamp: [0.8, 0.8, 0.8, 1.0],
            bg_stamp: [0.0, 0.0, 0.0, 0.25],
            seq_prev_type: crate::object_types::OB_SOLID,
            seq_flag: 0,
            simplify_subsurf: 6,
            simplify_subsurf_render: 0,
            simplify_gpencil: 0,
            simplify_particles: 1.0,
            simplify_particles_render: 0.0,
            simplify_volumes: 1.0,
            line_thickness_mode: R_LINE_THICKNESS_ABSOLUTE,
            unit_line_thickness: 1.0,
            engine: [0; 32],
            perf_flag: 0,
            anisotropic_filter: 2,
            bake: BakeData::default(),
            preview_pixel_size: 0,
            views: ListBase::new(),
            actview: 0,
            views_format: 0,
            hair_type: SCE_HAIR_SHAPE_STRAND,
            hair_subdiv: 0,
            motion_blur_shutter: 0.5,
            motion_blur_position: SCE_MB_CENTER,
            mblur_shutter_curve: NIL,
            compositor_device: SCE_COMPOSITOR_DEVICE_GPU,
            compositor_cache_flags: SCE_COMPOSITOR_CACHE_FRAMES,
            compositor_proxy_flags: COMPOSITOR_PROXY_ADAPT_TO_RENDER_SCALE,
            compositor_proxy_preview_scale: 50,
            compositor_precision: SCE_COMPOSITOR_PRECISION_AUTO,
            compositor_denoise_device: SCE_COMPOSITOR_DENOISE_DEVICE_AUTO,
            compositor_denoise_preview_quality: SCE_COMPOSITOR_DENOISE_BALANCED,
            compositor_denoise_final_quality: SCE_COMPOSITOR_DENOISE_HIGH,
            time_jump_delta: 1.0,
            time_jump_unit: SCE_TIME_JUMP_SECOND,
        }
    }
}

/// `TimeMarker`.
#[derive(Clone)]
pub struct TimeMarker {
    pub next: usize,
    pub prev: usize,
    pub frame: i32,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub flag: u32,
    /// `Object *camera`.
    pub camera: usize,
    /// `IDProperty *prop`.
    pub prop: usize,
}
impl Default for TimeMarker {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            frame: 0,
            name: [0; 64],
            flag: 0,
            camera: NIL,
            prop: NIL,
        }
    }
}

/// `TimeMarkerFlag`.
pub const TIME_MARKER_ELEVATED_TEMP: u32 = 1 << 1;

/// `eUnifiedPaintSettingsFlags` placeholder.
pub type UnifiedPaintSettingsFlags = i32;

/// `UnifiedPaintSettings`.
#[derive(Clone)]
pub struct UnifiedPaintSettings {
    pub size: i32,
    pub unprojected_size: f32,
    pub alpha: f32,
    pub weight: f32,
    pub color: [f32; 3],
    pub secondary_color: [f32; 3],
    /// DNA_DEPRECATED `rgb`, `secondary_rgb`.
    pub rgb: [f32; 3],
    pub secondary_rgb: [f32; 3],
    pub color_jitter_flag: i32,
    pub hsv_jitter: [f32; 3],
    /// `CurveMapping *curve_rand_hue` etc.
    pub curve_rand_hue: CurveMappingRef,
    pub curve_rand_saturation: CurveMappingRef,
    pub curve_rand_value: CurveMappingRef,
    pub input_samples: i32,
    pub flag: UnifiedPaintSettingsFlags,
}
impl Default for UnifiedPaintSettings {
    fn default() -> Self {
        Self {
            size: 100,
            unprojected_size: 0.58,
            alpha: 0.5,
            weight: 0.5,
            color: [0.0; 3],
            secondary_color: [1.0; 3],
            rgb: [0.0; 3],
            secondary_rgb: [1.0; 3],
            color_jitter_flag: 0,
            hsv_jitter: [0.0; 3],
            curve_rand_hue: NIL,
            curve_rand_saturation: NIL,
            curve_rand_value: NIL,
            input_samples: 1,
            flag: 0,
        }
    }
}
pub const PAINT_MAX_INPUT_SAMPLES: i32 = 64;

/// `NamedBrushAssetReference`.
#[derive(Clone, Default)]
pub struct NamedBrushAssetReference {
    pub next: usize,
    pub prev: usize,
    /// `const char *name`.
    pub name: Option<String>,
    /// `AssetWeakReference *brush_asset_reference`.
    pub brush_asset_reference: AssetWeakReferenceRef,
}
impl NamedBrushAssetReference {
    pub fn new() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            ..Self::default()
        }
    }
}

/// `ToolSystemBrushBindings`.
#[derive(Clone)]
pub struct ToolSystemBrushBindings {
    /// `AssetWeakReference *main_brush_asset_reference`.
    pub main_brush_asset_reference: AssetWeakReferenceRef,
    /// `ListBaseT<NamedBrushAssetReference>`.
    pub active_brush_per_brush_type: ListBase,
}
impl Default for ToolSystemBrushBindings {
    fn default() -> Self {
        Self {
            main_brush_asset_reference: NIL,
            active_brush_per_brush_type: ListBase::new(),
        }
    }
}

/// `ePaintFlags`.
pub const PAINT_SHOW_BRUSH: i32 = 1 << 0;
pub const PAINT_FAST_NAVIGATE: i32 = 1 << 1;
pub const PAINT_SHOW_BRUSH_ON_SURFACE: i32 = 1 << 2;
pub const PAINT_USE_CAVITY_MASK: i32 = 1 << 3;
pub const PAINT_SCULPT_DELAY_UPDATES: i32 = 1 << 4;

/// `ePaintDebugFlags`.
pub const PAINT_DEBUG_SHOW_BVH_NODES: i32 = 1 << 0;

/// `ePaintCanvasSource`.
pub const PAINT_CANVAS_SOURCE_MATERIAL: i8 = 0;
pub const PAINT_CANVAS_SOURCE_IMAGE: i8 = 1;
pub const PAINT_CANVAS_SOURCE_COLOR_ATTRIBUTE: i8 = 2;

/// `eAutomasking_flag` / `ePaintSymmetryFlags` / `PaintCurveVisibilityFlags` placeholders.
pub type AutomaskingFlag = i32;
pub type PaintSymmetryFlags = i32;
pub type PaintCurveVisibilityFlags = i32;
pub const PAINT_SYMMETRY_FEATHER: i32 = 1 << 0;

/// `MeshAutomaskingSettings`.
#[derive(Clone)]
pub struct MeshAutomaskingSettings {
    pub flags: AutomaskingFlag,
    pub boundary_edges_propagation_steps: i32,
    pub cavity_blur_steps: i32,
    pub cavity_factor: f32,
    pub start_normal_limit: f32,
    pub start_normal_falloff: f32,
    pub view_normal_limit: f32,
    pub view_normal_falloff: f32,
    /// `CurveMapping *cavity_curve`, `*cavity_curve_op`.
    pub cavity_curve: CurveMappingRef,
    pub cavity_curve_op: CurveMappingRef,
}
impl Default for MeshAutomaskingSettings {
    fn default() -> Self {
        Self {
            flags: 0,
            boundary_edges_propagation_steps: 1,
            cavity_blur_steps: 2,
            cavity_factor: 0.5,
            start_normal_limit: 0.34906585,
            start_normal_falloff: 0.25,
            view_normal_limit: 1.570796,
            view_normal_falloff: 0.25,
            cavity_curve: NIL,
            cavity_curve_op: NIL,
        }
    }
}

/// `Paint`.
#[derive(Clone)]
pub struct Paint {
    /// `Brush *brush`.
    pub brush: usize,
    /// `AssetWeakReference *brush_asset_reference`.
    pub brush_asset_reference: AssetWeakReferenceRef,
    pub tool_brush_bindings: ToolSystemBrushBindings,
    /// `Palette *palette`.
    pub palette: usize,
    /// `CurveMapping *cavity_curve`.
    pub cavity_curve: CurveMappingRef,
    pub flags: i32,
    pub debug_flags: i32,
    /// DNA_DEPRECATED `num_input_samples`.
    pub num_input_samples_deprecated: i32,
    pub symmetry_flags: PaintSymmetryFlags,
    pub curve_visibility_flags: PaintCurveVisibilityFlags,
    pub tile_offset: [f32; 3],
    pub unified_paint_settings: UnifiedPaintSettings,
    /// `MeshAutomaskingSettings *`.
    pub mesh_automasking_settings: usize,
    /// `bke::PaintRuntime *runtime`.
    pub runtime: usize,
}
impl Default for Paint {
    fn default() -> Self {
        Self {
            brush: NIL,
            brush_asset_reference: NIL,
            tool_brush_bindings: ToolSystemBrushBindings::default(),
            palette: NIL,
            cavity_curve: NIL,
            flags: PAINT_SHOW_BRUSH,
            debug_flags: 0,
            num_input_samples_deprecated: 0,
            symmetry_flags: PAINT_SYMMETRY_FEATHER,
            curve_visibility_flags: 0,
            tile_offset: [1.0; 3],
            unified_paint_settings: UnifiedPaintSettings::default(),
            mesh_automasking_settings: NIL,
            runtime: NIL,
        }
    }
}

pub const IMAGEPAINT_MODE_MATERIAL: i8 = PAINT_CANVAS_SOURCE_MATERIAL;
pub const IMAGEPAINT_MODE_IMAGE: i8 = PAINT_CANVAS_SOURCE_IMAGE;

/// `eImagePaint_Interpolation`.
pub const IMAGEPAINT_INTERP_LINEAR: i32 = 0;
pub const IMAGEPAINT_INTERP_CLOSEST: i32 = 1;

/// `eImagePaint_Flag`.
pub const IMAGEPAINT_DRAWING: i16 = 1 << 0;
pub const IMAGEPAINT_PROJECT_XRAY: i16 = 1 << 4;
pub const IMAGEPAINT_PROJECT_BACKFACE: i16 = 1 << 5;
pub const IMAGEPAINT_PROJECT_FLAT: i16 = 1 << 6;
pub const IMAGEPAINT_PROJECT_LAYER_CLONE: i16 = 1 << 7;
pub const IMAGEPAINT_PROJECT_LAYER_STENCIL: i16 = 1 << 8;
pub const IMAGEPAINT_PROJECT_LAYER_STENCIL_INV: i16 = 1 << 9;

/// `eImagePaint_MissingData`.
pub const IMAGEPAINT_MISSING_UVS: i16 = 1 << 0;
pub const IMAGEPAINT_MISSING_MATERIAL: i16 = 1 << 1;
pub const IMAGEPAINT_MISSING_TEX: i16 = 1 << 2;
pub const IMAGEPAINT_MISSING_STENCIL: i16 = 1 << 3;

/// `ImagePaintSettings`.
#[derive(Clone)]
pub struct ImagePaintSettings {
    pub paint: Paint,
    pub flag: i16,
    pub missing_data: i16,
    pub seam_bleed: i16,
    pub normal_angle: i16,
    pub screen_grab_size: [i16; 2],
    pub mode: i32,
    /// `Image *stencil`, `*clone`, `*canvas`.
    pub stencil: usize,
    pub clone: usize,
    pub canvas: usize,
    pub stencil_col: [f32; 3],
    pub dither: f32,
    pub interp: i32,
    pub clone_offset: [f32; 2],
    pub clone_alpha: f32,
}
impl Default for ImagePaintSettings {
    fn default() -> Self {
        Self {
            paint: Paint::default(),
            flag: 0,
            missing_data: 0,
            seam_bleed: 2,
            normal_angle: 80,
            screen_grab_size: [0; 2],
            mode: 0,
            stencil: NIL,
            clone: NIL,
            canvas: NIL,
            stencil_col: [0.0; 3],
            dither: 0.0,
            interp: IMAGEPAINT_INTERP_LINEAR,
            clone_offset: [0.0; 2],
            clone_alpha: 0.5,
        }
    }
}

/// `PaintModeSettings`.
#[derive(Clone)]
pub struct PaintModeSettings {
    pub canvas_source: i8,
    /// `Image *canvas_image`.
    pub canvas_image: usize,
    pub image_user: ImageUser,
}
impl Default for PaintModeSettings {
    fn default() -> Self {
        Self {
            canvas_source: PAINT_CANVAS_SOURCE_MATERIAL,
            canvas_image: NIL,
            image_user: ImageUser::default(),
        }
    }
}

/// `eParticleEdit_BrushType`.
pub const PE_BRUSH_NONE: i16 = -1;
pub const PE_BRUSH_COMB: i16 = 0;
pub const PE_BRUSH_CUT: i16 = 1;
pub const PE_BRUSH_LENGTH: i16 = 2;
pub const PE_BRUSH_PUFF: i16 = 3;
pub const PE_BRUSH_ADD: i16 = 4;
pub const PE_BRUSH_SMOOTH: i16 = 5;
pub const PE_BRUSH_WEIGHT: i16 = 6;

/// `eParticleEdit_BrushFlag`.
pub const PE_BRUSH_DATA_PUFF_VOLUME: i32 = 1 << 0;

/// `eParticleEdit_EditType`.
pub const PE_TYPE_PARTICLES: i32 = 0;
pub const PE_TYPE_SOFTBODY: i32 = 1;
pub const PE_TYPE_CLOTH: i32 = 2;

/// `eParticleEdit_Flag`.
pub const PE_KEEP_LENGTHS: i16 = 1 << 0;
pub const PE_LOCK_FIRST: i16 = 1 << 1;
pub const PE_DEFLECT_EMITTER: i16 = 1 << 2;
pub const PE_INTERPOLATE_ADDED: i16 = 1 << 3;
pub const PE_DRAW_PART: i16 = 1 << 4;
pub const PE_FADE_TIME: i16 = 1 << 7;
pub const PE_AUTO_VELOCITY: i16 = 1 << 8;

/// `ParticleBrushData`.
#[derive(Clone, Copy)]
pub struct ParticleBrushData {
    pub size: i16,
    pub step: i16,
    pub invert: i16,
    pub count: i16,
    pub flag: i32,
    pub strength: f32,
}
impl Default for ParticleBrushData {
    fn default() -> Self {
        Self {
            size: 50,
            step: 10,
            invert: 0,
            count: 10,
            flag: 0,
            strength: 0.5,
        }
    }
}

/// `eParticleEdit_SelectMode`.
pub const SCE_SELECT_PATH: i32 = 1 << 0;
pub const SCE_SELECT_POINT: i32 = 1 << 1;
pub const SCE_SELECT_END: i32 = 1 << 2;

/// `ParticleEditSettings`.
#[derive(Clone)]
pub struct ParticleEditSettings {
    pub flag: i16,
    pub totrekey: i16,
    pub totaddkey: i16,
    pub brushtype: i16,
    /// `ParticleBrushData brush[7]`.
    pub brush: [ParticleBrushData; 7],
    /// `void *paintcursor`.
    pub paintcursor: usize,
    pub emitterdist: f32,
    pub selectmode: i32,
    pub edittype: i32,
    pub draw_step: i32,
    pub fade_frames: i32,
    /// `Scene *scene`, `Object *object`, `Object *shape_object`.
    pub scene: usize,
    pub object: usize,
    pub shape_object: usize,
}
impl Default for ParticleEditSettings {
    fn default() -> Self {
        Self {
            flag: PE_KEEP_LENGTHS | PE_LOCK_FIRST | PE_DEFLECT_EMITTER | PE_AUTO_VELOCITY,
            totrekey: 5,
            totaddkey: 5,
            brushtype: PE_BRUSH_COMB,
            brush: [ParticleBrushData::default(); 7],
            paintcursor: NIL,
            emitterdist: 0.25,
            selectmode: SCE_SELECT_PATH,
            edittype: 0,
            draw_step: 2,
            fade_frames: 2,
            scene: NIL,
            object: NIL,
            shape_object: NIL,
        }
    }
}

/// `eSculptFlags`.
pub const SCULPT_LOCK_X: i32 = 1 << 3;
pub const SCULPT_LOCK_Y: i32 = 1 << 4;
pub const SCULPT_LOCK_Z: i32 = 1 << 5;
pub const SCULPT_ONLY_DEFORM: i32 = 1 << 8;
pub const SCULPT_DYNTOPO_SUBDIVIDE: i32 = 1 << 12;
pub const SCULPT_DYNTOPO_COLLAPSE: i32 = 1 << 11;
pub const SCULPT_DYNTOPO_DETAIL_CONSTANT: i32 = 1 << 13;
pub const SCULPT_DYNTOPO_DETAIL_BRUSH: i32 = 1 << 14;
pub const SCULPT_DYNTOPO_DETAIL_MANUAL: i32 = 1 << 16;

/// `eSculptTransformMode`.
pub const SCULPT_TRANSFORM_MODE_ALL_VERTICES: i32 = 0;
pub const SCULPT_TRANSFORM_MODE_RADIUS_ELASTIC: i32 = 1;

/// `Sculpt`.
#[derive(Clone)]
pub struct Sculpt {
    pub paint: Paint,
    pub flags: i32,
    pub transform_mode: i32,
    /// DNA_DEPRECATED `automasking_flags`.
    pub automasking_flags: i32,
    /// DNA_DEPRECATED `radial_symm_legacy`.
    pub radial_symm_legacy: [i32; 3],
    pub detail_size: f32,
    pub symmetrize_direction: i32,
    pub gravity_factor: f32,
    pub constant_detail: f32,
    pub detail_percent: f32,
    /// DNA_DEPRECATED automasking fields.
    pub automasking_boundary_edges_propagation_steps: i32,
    pub automasking_cavity_blur_steps: i32,
    pub automasking_cavity_factor: f32,
    pub automasking_start_normal_limit: f32,
    pub automasking_start_normal_falloff: f32,
    pub automasking_view_normal_limit: f32,
    pub automasking_view_normal_falloff: f32,
    /// DNA_DEPRECATED `CurveMapping *automasking_cavity_curve[_op]`.
    pub automasking_cavity_curve: CurveMappingRef,
    pub automasking_cavity_curve_op: CurveMappingRef,
    /// `Object *gravity_object`.
    pub gravity_object: usize,
}
impl Default for Sculpt {
    fn default() -> Self {
        Self {
            paint: Paint::default(),
            flags: SCULPT_DYNTOPO_SUBDIVIDE | SCULPT_DYNTOPO_COLLAPSE,
            transform_mode: SCULPT_TRANSFORM_MODE_ALL_VERTICES,
            automasking_flags: 0,
            radial_symm_legacy: [0; 3],
            detail_size: 12.0,
            symmetrize_direction: 0,
            gravity_factor: 0.0,
            constant_detail: 3.0,
            detail_percent: 25.0,
            automasking_boundary_edges_propagation_steps: 1,
            automasking_cavity_blur_steps: 0,
            automasking_cavity_factor: 0.0,
            automasking_start_normal_limit: 0.34906585,
            automasking_start_normal_falloff: 0.25,
            automasking_view_normal_limit: 1.570796,
            automasking_view_normal_falloff: 0.25,
            automasking_cavity_curve: NIL,
            automasking_cavity_curve_op: NIL,
            gravity_object: NIL,
        }
    }
}

/// `CurvesSculpt`.
#[derive(Clone, Default)]
pub struct CurvesSculpt {
    pub paint: Paint,
}

/// `BRUSH_CURVE_SMOOTH` (DNA_brush_enums.h).
pub const BRUSH_CURVE_SMOOTH: i8 = 2;

/// `UvSculpt`.
#[derive(Clone)]
pub struct UvSculpt {
    /// `CurveMapping *curve_distance_falloff`.
    pub curve_distance_falloff: CurveMappingRef,
    pub size: i32,
    pub strength: f32,
    pub curve_distance_falloff_preset: i8,
}
impl Default for UvSculpt {
    fn default() -> Self {
        Self {
            curve_distance_falloff: NIL,
            size: 100,
            strength: 1.0,
            curve_distance_falloff_preset: BRUSH_CURVE_SMOOTH,
        }
    }
}

/// `GpPaint` / `GpVertexPaint` / `GpSculptPaint` / `GpWeightPaint` / `VPaint`.
#[derive(Clone, Default)]
pub struct GpPaint {
    pub paint: Paint,
    pub flag: i32,
    pub mode: i32,
}
#[derive(Clone, Default)]
pub struct GpVertexPaint {
    pub paint: Paint,
    pub flag: i32,
}
#[derive(Clone, Default)]
pub struct GpSculptPaint {
    pub paint: Paint,
    pub flag: i32,
}
#[derive(Clone, Default)]
pub struct GpWeightPaint {
    pub paint: Paint,
    pub flag: i32,
}
#[derive(Clone, Default)]
pub struct VPaint {
    pub paint: Paint,
    pub flag: i8,
}

/// `eGP_Lockaxis_Types`.
pub const GP_LOCKAXIS_VIEW: i32 = 0;
pub const GP_LOCKAXIS_X: i32 = 1;
pub const GP_LOCKAXIS_Y: i32 = 2;
pub const GP_LOCKAXIS_Z: i32 = 3;
pub const GP_LOCKAXIS_CURSOR: i32 = 4;

/// `eGP_Sculpt_SettingsFlag`.
pub const GP_SCULPT_SETT_FLAG_FRAME_FALLOFF: i32 = 1 << 0;
pub const GP_SCULPT_SETT_FLAG_PRIMITIVE_CURVE: i32 = 1 << 1;
pub const GP_SCULPT_SETT_FLAG_SCALE_THICKNESS: i32 = 1 << 3;
pub const GP_SCULPT_SETT_FLAG_AUTOMASK_STROKE: i32 = 1 << 4;
pub const GP_SCULPT_SETT_FLAG_AUTOMASK_LAYER_STROKE: i32 = 1 << 5;
pub const GP_SCULPT_SETT_FLAG_AUTOMASK_MATERIAL_STROKE: i32 = 1 << 6;
pub const GP_SCULPT_SETT_FLAG_AUTOMASK_LAYER_ACTIVE: i32 = 1 << 7;
pub const GP_SCULPT_SETT_FLAG_AUTOMASK_MATERIAL_ACTIVE: i32 = 1 << 8;

/// `eGP_Sculpt_SelectMaskFlag`.
pub const GP_SCULPT_MASK_SELECTMODE_POINT: i8 = 1 << 0;
pub const GP_SCULPT_MASK_SELECTMODE_STROKE: i8 = 1 << 1;
pub const GP_SCULPT_MASK_SELECTMODE_SEGMENT: i8 = 1 << 2;

/// `eGP_Vertex_SelectMaskFlag`.
pub const GP_VERTEX_MASK_SELECTMODE_POINT: i8 = 1 << 0;
pub const GP_VERTEX_MASK_SELECTMODE_STROKE: i8 = 1 << 1;
pub const GP_VERTEX_MASK_SELECTMODE_SEGMENT: i8 = 1 << 2;

/// `eGP_Interpolate_SettingsFlag`.
pub const GP_TOOLFLAG_INTERPOLATE_ALL_LAYERS: i16 = 1 << 0;
pub const GP_TOOLFLAG_INTERPOLATE_ONLY_SELECTED: i16 = 1 << 1;
pub const GP_TOOLFLAG_INTERPOLATE_EXCLUDE_BREAKDOWNS: i16 = 1 << 2;

/// `eGP_Interpolate_Type`.
pub const GP_IPO_LINEAR: i16 = 0;
pub const GP_IPO_CURVEMAP: i16 = 1;
pub const GP_IPO_BACK: i16 = 3;
pub const GP_IPO_BOUNCE: i16 = 4;
pub const GP_IPO_CIRC: i16 = 5;
pub const GP_IPO_CUBIC: i16 = 6;
pub const GP_IPO_ELASTIC: i16 = 7;
pub const GP_IPO_EXPO: i16 = 8;
pub const GP_IPO_QUAD: i16 = 9;
pub const GP_IPO_QUART: i16 = 10;
pub const GP_IPO_QUINT: i16 = 11;
pub const GP_IPO_SINE: i16 = 12;

/// `GP_Sculpt_Guide`.
#[derive(Clone)]
pub struct GpSculptGuide {
    pub use_guide: i8,
    pub use_snapping: i8,
    pub reference_point: i8,
    pub type_: i8,
    pub angle: f32,
    pub angle_snap: f32,
    pub spacing: f32,
    pub location: [f32; 3],
    /// `Object *reference_object`.
    pub reference_object: usize,
}
impl Default for GpSculptGuide {
    fn default() -> Self {
        Self {
            use_guide: 0,
            use_snapping: 0,
            reference_point: 0,
            type_: 0,
            angle: 0.0,
            angle_snap: 0.0,
            spacing: 20.0,
            location: [0.0; 3],
            reference_object: NIL,
        }
    }
}

/// `GP_Sculpt_Settings`.
#[derive(Clone)]
pub struct GpSculptSettings {
    /// `void *paintcursor`.
    pub paintcursor: usize,
    pub flag: i32,
    pub lock_axis: i32,
    pub isect_threshold: f32,
    /// `CurveMapping *cur_falloff`, `*cur_primitive`.
    pub cur_falloff: CurveMappingRef,
    pub cur_primitive: CurveMappingRef,
    pub guide: GpSculptGuide,
}
impl Default for GpSculptSettings {
    fn default() -> Self {
        Self {
            paintcursor: NIL,
            flag: 0,
            lock_axis: GP_LOCKAXIS_VIEW,
            isect_threshold: 0.0,
            cur_falloff: NIL,
            cur_primitive: NIL,
            guide: GpSculptGuide::default(),
        }
    }
}

/// `GP_Interpolate_Settings`.
#[derive(Clone)]
pub struct GpInterpolateSettings {
    /// `CurveMapping *custom_ipo`.
    pub custom_ipo: CurveMappingRef,
}
impl Default for GpInterpolateSettings {
    fn default() -> Self {
        Self { custom_ipo: NIL }
    }
}

/// `eCurvePaint_Flag`.
pub const CURVE_PAINT_FLAG_CORNERS_DETECT: i8 = 1 << 0;
pub const CURVE_PAINT_FLAG_PRESSURE_RADIUS: i8 = 1 << 1;
pub const CURVE_PAINT_FLAG_DEPTH_STROKE_ENDPOINTS: i8 = 1 << 2;
pub const CURVE_PAINT_FLAG_DEPTH_STROKE_OFFSET_ABS: i8 = 1 << 3;
pub const CURVE_PAINT_FLAG_DEPTH_ONLY_SELECTED: i8 = 1 << 4;

/// `eCurvePaint_FitMethod`.
pub const CURVE_PAINT_FIT_METHOD_REFIT: i8 = 0;
pub const CURVE_PAINT_FIT_METHOD_SPLIT: i8 = 1;

/// `eCurvePaint_DepthMode`.
pub const CURVE_PAINT_PROJECT_CURSOR: i8 = 0;
pub const CURVE_PAINT_PROJECT_SURFACE: i8 = 1;

/// `eCurvePaint_SurfacePlane`.
pub const CURVE_PAINT_SURFACE_PLANE_NORMAL_VIEW: i8 = 0;
pub const CURVE_PAINT_SURFACE_PLANE_NORMAL_SURFACE: i8 = 1;
pub const CURVE_PAINT_SURFACE_PLANE_VIEW: i8 = 2;

/// `eCurvePaint_AutoMerge`.
pub const AUTO_MERGE: i8 = 1 << 0;
pub const AUTO_MERGE_AND_SPLIT: i8 = 1 << 1;

/// `CU_BEZIER` (DNA_curve_enums.h).
pub const CU_BEZIER: i8 = 0;
/// `DEG2RADF(70)` corner_angle default.
pub const CURVE_PAINT_CORNER_ANGLE_DEFAULT: f32 = 70.0 * (std::f32::consts::PI / 180.0);

/// `CurvePaintSettings`.
#[derive(Clone)]
pub struct CurvePaintSettings {
    pub curve_type: i8,
    pub flag: i8,
    pub depth_mode: i8,
    pub surface_plane: i8,
    pub fit_method: i8,
    pub error_threshold: i16,
    pub radius_min: f32,
    pub radius_max: f32,
    pub radius_taper_start: f32,
    pub radius_taper_end: f32,
    pub surface_offset: f32,
    pub corner_angle: f32,
}
impl Default for CurvePaintSettings {
    fn default() -> Self {
        Self {
            curve_type: CU_BEZIER,
            flag: CURVE_PAINT_FLAG_CORNERS_DETECT,
            depth_mode: 0,
            surface_plane: 0,
            fit_method: 0,
            error_threshold: 8,
            radius_min: 0.0,
            radius_max: 1.0,
            radius_taper_start: 0.0,
            radius_taper_end: 0.0,
            surface_offset: 0.0,
            corner_angle: CURVE_PAINT_CORNER_ANGLE_DEFAULT,
        }
    }
}

/// `MeshStatVis`.
#[derive(Clone)]
pub struct MeshStatVis {
    pub type_: i8,
    pub overhang_axis: i8,
    pub overhang_min: f32,
    pub overhang_max: f32,
    pub thickness_min: f32,
    pub thickness_max: f32,
    pub thickness_samples: i8,
    pub distort_min: f32,
    pub distort_max: f32,
    pub sharp_min: f32,
    pub sharp_max: f32,
}
impl Default for MeshStatVis {
    fn default() -> Self {
        let d = |v: f32| v * (std::f32::consts::PI / 180.0);
        Self {
            type_: 0,
            overhang_axis: crate::object_types::OB_NEGZ as i8,
            overhang_min: 0.0,
            overhang_max: d(45.0),
            thickness_min: 0.0,
            thickness_max: 0.1,
            thickness_samples: 1,
            distort_min: d(5.0),
            distort_max: d(45.0),
            sharp_min: d(90.0),
            sharp_max: d(180.0),
        }
    }
}

/// `eSeqOverlapMode`.
pub const SEQ_OVERLAP_RIPPLE: i32 = 0;
pub const SEQ_OVERLAP_OVERWRITE: i32 = 1;
pub const SEQ_OVERLAP_SHUFFLE: i32 = 2;

/// `eSeqRippleFlag`.
pub const SEQ_RIPPLE_ALL_CHANNELS: i32 = 1 << 0;
pub const SEQ_RIPPLE_MARKERS: i32 = 1 << 1;
pub const SEQ_RIPPLE_INSERT: i32 = 1 << 2;
pub const SEQ_RIPPLE_CLEAR_RANGES: i32 = 1 << 3;

/// `eSequencerSnapMode`.
pub const SEQ_SNAP_TO_STRIPS: i16 = 1 << 0;
pub const SEQ_SNAP_TO_CURRENT_FRAME: i16 = 1 << 1;
pub const SEQ_SNAP_TO_STRIP_HOLD: i16 = 1 << 2;
pub const SEQ_SNAP_TO_MARKERS: i16 = 1 << 3;
pub const SEQ_SNAP_TO_PREVIEW_BORDERS: i16 = 1 << 4;
pub const SEQ_SNAP_TO_PREVIEW_CENTER: i16 = 1 << 5;
pub const SEQ_SNAP_TO_STRIPS_PREVIEW: i16 = 1 << 6;
pub const SEQ_SNAP_TO_RETIMING: i16 = 1 << 7;
pub const SEQ_SNAP_TO_INCREMENT: i16 = 1 << 8;
pub const SEQ_SNAP_TO_FRAME_RANGE: i16 = 1 << 9;

/// `eSequencerSnapFlag`.
pub const SEQ_SNAP_IGNORE_MUTED: i16 = 1 << 0;
pub const SEQ_SNAP_IGNORE_SOUND: i16 = 1 << 1;
pub const SEQ_SNAP_CURRENT_FRAME_TO_STRIPS: i16 = 1 << 2;
pub const SEQ_SNAP_TO_ALL_CHANNEL_STRIPS: i16 = 1 << 3;

/// `eSeqImageFitMethod`/`V3D_AROUND_CENTER_MEDIAN` placeholders.
pub type SeqImageFitMethod = i16;
pub const SEQ_SCALE_TO_FIT: i16 = 0;
pub const V3D_AROUND_CENTER_MEDIAN: i32 = 1;

/// `SequencerToolSettings`.
#[derive(Clone)]
pub struct SequencerToolSettings {
    pub fit_method: SeqImageFitMethod,
    pub snap_mode: i16,
    pub snap_flag: i16,
    pub overlap_mode: i32,
    pub ripple_flag: i32,
    pub snap_distance: i32,
    pub pivot_point: i32,
}
impl Default for SequencerToolSettings {
    fn default() -> Self {
        Self {
            fit_method: SEQ_SCALE_TO_FIT,
            snap_mode: 0,
            snap_flag: 0,
            overlap_mode: SEQ_OVERLAP_RIPPLE,
            ripple_flag: SEQ_RIPPLE_ALL_CHANNELS
                | SEQ_RIPPLE_MARKERS
                | SEQ_RIPPLE_CLEAR_RANGES,
            snap_distance: 0,
            pivot_point: V3D_AROUND_CENTER_MEDIAN,
        }
    }
}

/// `eTool_TransformFlag`.
pub const SCE_XFORM_AXIS_ALIGN: i32 = 1 << 0;
pub const SCE_XFORM_DATA_ORIGIN: i32 = 1 << 1;
pub const SCE_XFORM_SKIP_CHILDREN: i32 = 1 << 2;
pub const SCE_XFORM_SCULPT_PIVOT: i32 = 1 << 3;

/// `eTool_ObjectFlag`.
pub const SCE_OBJECT_MODE_LOCK: i32 = 1 << 0;

/// `eTool_WorkspaceToolFlag`.
pub const SCE_WORKSPACE_TOOL_FALLBACK: i32 = 0;
pub const SCE_WORKSPACE_TOOL_DEFAULT: i32 = 1;

/// `eSnapFlag`.
pub const SCE_SNAP: i16 = 1 << 0;
pub const SCE_SNAP_ROTATE: i16 = 1 << 1;
pub const SCE_SNAP_PEEL_OBJECT: i16 = 1 << 2;
pub const SCE_SNAP_NOT_TO_ACTIVE: i16 = 1 << 4;
pub const SCE_SNAP_ABS_GRID: i16 = 1 << 5;
pub const SCE_SNAP_ABS_TIME_STEP: i16 = 1 << 5;
pub const SCE_SNAP_BACKFACE_CULLING: i16 = 1 << 6;
pub const SCE_SNAP_KEEP_ON_SAME_OBJECT: i16 = 1 << 7;
pub const SCE_SNAP_TO_INCLUDE_EDITED: i16 = 1 << 8;
pub const SCE_SNAP_TO_INCLUDE_NONEDITED: i16 = 1 << 9;
pub const SCE_SNAP_TO_ONLY_SELECTABLE: i16 = 1 << 10;

/// `eSnapSourceOP`.
pub const SCE_SNAP_SOURCE_CLOSEST: i8 = 0;
pub const SCE_SNAP_SOURCE_CENTER: i8 = 1;
pub const SCE_SNAP_SOURCE_MEDIAN: i8 = 2;
pub const SCE_SNAP_SOURCE_ACTIVE: i8 = 3;

/// `eSnapTargetOP`.
pub const SCE_SNAP_TARGET_ALL: i16 = 0;
pub const SCE_SNAP_TARGET_NOT_SELECTED: i16 = 1 << 0;
pub const SCE_SNAP_TARGET_NOT_ACTIVE: i16 = 1 << 1;
pub const SCE_SNAP_TARGET_NOT_EDITED: i16 = 1 << 2;
pub const SCE_SNAP_TARGET_ONLY_SELECTABLE: i16 = 1 << 3;
pub const SCE_SNAP_TARGET_NOT_NONEDITED: i16 = 1 << 4;

/// `eSnapMode`.
pub const SCE_SNAP_TO_NONE: i16 = 0;
pub const SCE_SNAP_TO_FRAME: i16 = 1 << 0;
pub const SCE_SNAP_TO_SECOND: i16 = 1 << 1;
pub const SCE_SNAP_TO_KEYS: i16 = 1 << 3;
pub const SCE_SNAP_TO_STRIPS: i16 = 1 << 4;
pub const SCE_SNAP_TO_POINT: i16 = 1 << 0;
pub const SCE_SNAP_TO_EDGE_MIDPOINT: i16 = 1 << 1;
pub const SCE_SNAP_TO_EDGE_ENDPOINT: i16 = 1 << 2;
pub const SCE_SNAP_TO_EDGE_PERPENDICULAR: i16 = 1 << 3;
pub const SCE_SNAP_TO_EDGE: i16 = 1 << 4;
pub const SCE_SNAP_TO_FACE: i16 = 1 << 5;
pub const SCE_SNAP_TO_VOLUME: i16 = 1 << 6;
pub const SCE_SNAP_TO_GRID: i16 = 1 << 7;
pub const SCE_SNAP_TO_INCREMENT: i16 = 1 << 8;
pub const SCE_SNAP_INDIVIDUAL_NEAREST: i16 = 1 << 9;
pub const SCE_SNAP_INDIVIDUAL_PROJECT: i16 = 1 << 10;
pub const SCE_SNAP_TO_FACE_MIDPOINT: i16 = 1 << 11;
pub const SCE_SNAP_TO_ORIGIN: i16 = 1 << 12;
pub const SCE_SNAP_TO_VERTEX: i16 = SCE_SNAP_TO_POINT | SCE_SNAP_TO_EDGE_ENDPOINT;
pub const SCE_SNAP_TO_GEOM: i16 = SCE_SNAP_TO_VERTEX
    | SCE_SNAP_TO_EDGE
    | SCE_SNAP_TO_FACE
    | SCE_SNAP_TO_FACE_MIDPOINT
    | SCE_SNAP_TO_EDGE_MIDPOINT
    | SCE_SNAP_TO_EDGE_PERPENDICULAR;

/// `eTool_UvSculptFlag`.
pub const UV_SCULPT_LOCK_BORDERS: i32 = 1;
pub const UV_SCULPT_ALL_ISLANDS: i32 = 2;

/// `eGpPaint_Flag`.
pub const GPPAINT_FLAG_USE_MATERIAL: i32 = 0;
pub const GPPAINT_FLAG_USE_VERTEXCOLOR: i32 = 1;

/// `eVPaint_Flag`.
pub const VP_FLAG_VGROUP_RESTRICT: i8 = 1 << 7;

/// `eSnapTransformMode`.
pub const SCE_SNAP_TRANSFORM_MODE_TRANSLATE: i8 = 1 << 0;
pub const SCE_SNAP_TRANSFORM_MODE_ROTATE: i8 = 1 << 1;
pub const SCE_SNAP_TRANSFORM_MODE_SCALE: i8 = 1 << 2;

/// `eTool_SelectMode`.
pub const SCE_SELECT_VERTEX: i32 = 1 << 0;
pub const SCE_SELECT_EDGE: i32 = 1 << 1;
pub const SCE_SELECT_FACE: i32 = 1 << 2;

/// `eMeshStatVis_Type`.
pub const SCE_STATVIS_OVERHANG: i8 = 0;
pub const SCE_STATVIS_THICKNESS: i8 = 1;
pub const SCE_STATVIS_INTERSECT: i8 = 2;
pub const SCE_STATVIS_DISTORT: i8 = 3;
pub const SCE_STATVIS_SHARP: i8 = 4;

/// `eTool_ProportionalFalloff`.
pub const PROP_SMOOTH: i8 = 0;
pub const PROP_SPHERE: i8 = 1;
pub const PROP_ROOT: i8 = 2;
pub const PROP_SHARP: i8 = 3;
pub const PROP_LIN: i8 = 4;
pub const PROP_CONST: i8 = 5;
pub const PROP_RANDOM: i8 = 6;
pub const PROP_INVSQUARE: i8 = 7;
pub const PROP_MODE_MAX: i8 = 8;

/// `eTool_ProportionalEdit`.
pub const PROP_EDIT_USE: i8 = 1 << 0;
pub const PROP_EDIT_CONNECTED: i8 = 1 << 1;
pub const PROP_EDIT_PROJECTED: i8 = 1 << 2;

/// `eTool_WeightUser`.
pub const OB_DRAW_GROUPUSER_NONE: i8 = 0;
pub const OB_DRAW_GROUPUSER_ACTIVE: i8 = 1;
pub const OB_DRAW_GROUPUSER_ALL: i8 = 2;

/// `eTool_Unwrapper`.
pub const UVCALC_UNWRAP_METHOD_ANGLE: i8 = 0;
pub const UVCALC_UNWRAP_METHOD_CONFORMAL: i8 = 1;
pub const UVCALC_UNWRAP_METHOD_MINIMUM_STRETCH: i8 = 2;

/// `eTool_UvCalcFlag`.
pub const UVCALC_FILLHOLES: i8 = 1 << 0;
pub const UVCALC_NO_ASPECT_CORRECT: i8 = 1 << 1;
pub const UVCALC_TRANSFORM_CORRECT_SLIDE: i8 = 1 << 2;
pub const UVCALC_USESUBSURF: i8 = 1 << 3;
pub const UVCALC_TRANSFORM_CORRECT: i8 = 1 << 4;
pub const UVCALC_TRANSFORM_CORRECT_KEEP_CONNECTED: i8 = 1 << 5;
pub const UVCALC_UNWRAP_NO_FLIP: i8 = 1 << 6;
pub const UVCALC_UNWRAP_USE_WEIGHTS: i8 = 1 << 7;

/// `eTool_UvFlag`.
pub const UV_FLAG_SELECT_SYNC: i8 = 1 << 0;
pub const UV_FLAG_SHOW_SAME_IMAGE: i8 = 1 << 1;
pub const UV_FLAG_SELECT_ISLAND: i8 = 1 << 2;
pub const UV_FLAG_CUSTOM_REGION: i8 = 1 << 3;

/// `eTool_UvSelectMode`.
pub const UV_SELECT_VERT: i8 = 1 << 0;
pub const UV_SELECT_EDGE: i8 = 1 << 1;
pub const UV_SELECT_FACE: i8 = 1 << 2;

/// `eTool_UvSticky`.
pub const UV_STICKY_LOCATION: i8 = 0;
pub const UV_STICKY_DISABLE: i8 = 1;
pub const UV_STICKY_VERT: i8 = 2;

/// `eGPencil_Flags`.
pub const GP_USE_MULTI_FRAME_EDITING: i8 = 1 << 0;
pub const GP_TOOL_FLAG_RETAIN_LAST: i8 = 1 << 1;
pub const GP_TOOL_FLAG_PAINT_ONBACK: i8 = 1 << 2;
pub const GP_TOOL_FLAG_THUMBNAIL_LIST: i8 = 1 << 3;
pub const GP_TOOL_FLAG_CREATE_WEIGHTS: i8 = 1 << 4;
pub const GP_TOOL_FLAG_AUTOMERGE_STROKE: i8 = 1 << 5;

/// `eGPencil_SimplifyFlags`.
pub const SIMPLIFY_GPENCIL_ENABLE: i16 = 1 << 0;
pub const SIMPLIFY_GPENCIL_ON_PLAY: i16 = 1 << 1;
pub const SIMPLIFY_GPENCIL_FILL: i16 = 1 << 2;
pub const SIMPLIFY_GPENCIL_MODIFIER: i16 = 1 << 3;
pub const SIMPLIFY_GPENCIL_FX: i16 = 1 << 5;
pub const SIMPLIFY_GPENCIL_TINT: i16 = 1 << 7;
pub const SIMPLIFY_GPENCIL_AA: i16 = 1 << 8;

/// `eGPencil_Placement_Flags`.
pub const GP_PROJECT_VIEWSPACE: i8 = 1 << 0;
pub const GP_PROJECT_DEPTH_VIEW: i8 = 1 << 2;
pub const GP_PROJECT_DEPTH_STROKE: i8 = 1 << 3;
pub const GP_PROJECT_DEPTH_STROKE_ENDPOINTS: i8 = 1 << 4;
pub const GP_PROJECT_CURSOR: i8 = 1 << 5;
pub const GP_PROJECT_DEPTH_STROKE_FIRST: i8 = 1 << 6;
pub const GP_PROJECT_DEPTH_ONLY_SELECTED: i8 = 1 << 7;

/// `eGPencil_Selectmode_types`.
pub const GP_SELECTMODE_POINT: i8 = 0;
pub const GP_SELECTMODE_STROKE: i8 = 1;
pub const GP_SELECTMODE_SEGMENT: i8 = 2;

/// `eGPencil_GuideTypes`.
pub const GP_GUIDE_CIRCULAR: i8 = 0;
pub const GP_GUIDE_RADIAL: i8 = 1;
pub const GP_GUIDE_PARALLEL: i8 = 2;
pub const GP_GUIDE_GRID: i8 = 3;
pub const GP_GUIDE_ISO: i8 = 4;

/// `eGPencil_Guide_Reference`.
pub const GP_GUIDE_REF_CURSOR: i8 = 0;
pub const GP_GUIDE_REF_CUSTOM: i8 = 1;
pub const GP_GUIDE_REF_OBJECT: i8 = 2;

/// `eAutokey_Mode`, `eVGroupSelect`, `eV3DPlaceDepth`, `eV3DPlaceOrient`,
/// `FIX_TO_CAM_FLAG_*` placeholders (foreign headers).
pub type AutokeyMode = i16;
pub const AUTOKEY_MODE_NORMAL: i16 = 0;
pub type VGroupSelect = i8;
pub const WT_VGROUP_ALL: i8 = 0;
pub type V3DPlaceDepth = i8;
pub const V3D_PLACE_DEPTH_SURFACE: i8 = 0;
pub type V3DPlaceOrient = i8;
pub const V3D_PLACE_ORIENT_SURFACE: i8 = 0;
pub const FIX_TO_CAM_FLAG_USE_LOC: u8 = 1 << 0;
pub const FIX_TO_CAM_FLAG_USE_ROT: u8 = 1 << 1;
pub const FIX_TO_CAM_FLAG_USE_SCALE: u8 = 1 << 2;

/// `TransformOrientation` placeholder type (foreign).
pub type TransformOrientation = u64;

/// `ToolSettings`.
#[derive(Clone)]
pub struct ToolSettings {
    /// `VPaint *vpaint`, `*wpaint`.
    pub vpaint: usize,
    pub wpaint: usize,
    /// `Sculpt *sculpt`.
    pub sculpt: usize,
    pub uvsculpt: UvSculpt,
    /// `GpPaint *gp_paint`.
    pub gp_paint: usize,
    /// `GpVertexPaint *gp_vertexpaint`.
    pub gp_vertexpaint: usize,
    /// `GpSculptPaint *gp_sculptpaint`.
    pub gp_sculptpaint: usize,
    /// `GpWeightPaint *gp_weightpaint`.
    pub gp_weightpaint: usize,
    /// `CurvesSculpt *curves_sculpt`.
    pub curves_sculpt: usize,
    pub vgroup_weight: f32,
    pub doublimit: f32,
    pub automerge: i8,
    pub object_flag: i8,
    pub selectmode: i8,
    pub unwrapper: i8,
    pub uvcalc_flag: i8,
    pub uv_flag: i8,
    pub uv_selectmode: i8,
    pub uv_sticky: i8,
    pub uv_custom_region: crate::vec_types::Rctf,
    pub uvcalc_margin: f32,
    pub uvcalc_iterations: i32,
    pub uvcalc_weight_factor: f32,
    /// `char uvcalc_weight_group[64]`.
    pub uvcalc_weight_group: [u8; 64],
    pub autoik_chainlen: i16,
    pub gpencil_flags: i8,
    pub gpencil_v3d_align: i8,
    pub gpencil_v2d_align: i8,
    pub annotate_v3d_align: i8,
    pub annotate_thickness: i16,
    pub gpencil_surface_offset: f32,
    pub gpencil_selectmode_edit: i8,
    pub gpencil_selectmode_sculpt: i8,
    pub gp_sculpt: GpSculptSettings,
    pub gp_interpolate: GpInterpolateSettings,
    pub imapaint: ImagePaintSettings,
    pub paint_mode: PaintModeSettings,
    pub particle: ParticleEditSettings,
    pub proportional_size: f32,
    pub select_thresh: f32,
    pub keying_flag: i16,
    pub autokey_mode: AutokeyMode,
    pub keyframe_type: i8,
    pub edge_mode: i8,
    pub edge_mode_live_unwrap: i8,
    pub transform_pivot_point: i8,
    pub transform_flag: i8,
    pub snap_node_mode: i8,
    pub snap_mode: i16,
    pub snap_uv_mode: i16,
    pub snap_anim_mode: i16,
    pub snap_playhead_mode: i16,
    pub snap_flag: i16,
    pub snap_flag_node: i16,
    pub snap_flag_seq: i16,
    pub snap_flag_anim: i16,
    pub snap_flag_driver: i16,
    pub snap_flag_playhead: i16,
    pub snap_uv_flag: i16,
    pub snap_target: i8,
    pub snap_transform_mode_flag: i8,
    pub snap_face_nearest_steps: i16,
    pub proportional_edit: i8,
    pub prop_mode: i8,
    pub proportional_objects: i8,
    pub proportional_mask: i8,
    pub proportional_action: i8,
    pub proportional_fcurve: i8,
    pub lock_markers: i8,
    pub auto_normalize: i8,
    pub wpaint_lock_relative: i8,
    pub multipaint: i8,
    pub weightuser: i8,
    pub vgroupsubset: VGroupSelect,
    pub gpencil_selectmode_vertex: i8,
    pub uv_sculpt_settings: i8,
    pub workspace_tool_type: i8,
    /// DNA_DEPRECATED `sculpt_paint_*` + `unified_paint_settings`.
    pub sculpt_paint_settings: i16,
    pub sculpt_paint_unified_size: i32,
    pub sculpt_paint_unified_unprojected_radius: f32,
    pub sculpt_paint_unified_alpha: f32,
    pub unified_paint_settings: UnifiedPaintSettings,
    pub curve_paint_settings: CurvePaintSettings,
    pub statvis: MeshStatVis,
    pub normal_vector: [f32; 3],
    /// `CurveProfile *custom_bevel_profile_preset`.
    pub custom_bevel_profile_preset: usize,
    /// `SequencerToolSettings *sequencer_tool_settings`.
    pub sequencer_tool_settings: usize,
    pub snap_mode_tools: i16,
    pub plane_axis: i8,
    pub plane_depth: V3DPlaceDepth,
    pub plane_orient: V3DPlaceOrient,
    pub use_plane_axis_auto: i8,
    pub snap_angle_increment_2d: f32,
    pub snap_angle_increment_2d_precision: f32,
    pub snap_angle_increment_3d: f32,
    pub snap_angle_increment_3d_precision: f32,
    pub snap_step_seconds: i16,
    pub snap_step_frames: i16,
    pub playhead_snap_distance: i32,
    /// `Object *anim_mirror_object`, `*anim_relative_object`.
    pub anim_mirror_object: usize,
    pub anim_relative_object: usize,
    /// `char anim_mirror_bone[64]`.
    pub anim_mirror_bone: [u8; 64],
    pub fix_to_cam_flag: u8,
}
impl Default for ToolSettings {
    fn default() -> Self {
        let d = |v: f32| v * (std::f32::consts::PI / 180.0);
        let mut uvcalc_weight_group = [0u8; 64];
        uvcalc_weight_group[..13].copy_from_slice(b"uv_importance");
        Self {
            vpaint: NIL,
            wpaint: NIL,
            sculpt: NIL,
            uvsculpt: UvSculpt::default(),
            gp_paint: NIL,
            gp_vertexpaint: NIL,
            gp_sculptpaint: NIL,
            gp_weightpaint: NIL,
            curves_sculpt: NIL,
            vgroup_weight: 1.0,
            doublimit: 0.001,
            automerge: 0,
            object_flag: SCE_OBJECT_MODE_LOCK as i8,
            selectmode: SCE_SELECT_VERTEX as i8,
            unwrapper: UVCALC_UNWRAP_METHOD_CONFORMAL,
            uvcalc_flag: UVCALC_TRANSFORM_CORRECT_SLIDE,
            uv_flag: UV_FLAG_SELECT_SYNC,
            uv_selectmode: UV_SELECT_VERT,
            uv_sticky: UV_STICKY_LOCATION,
            uv_custom_region: crate::vec_types::Rctf::default(),
            uvcalc_margin: 0.001,
            uvcalc_iterations: 10,
            uvcalc_weight_factor: 1.0,
            uvcalc_weight_group,
            autoik_chainlen: 0,
            gpencil_flags: 0,
            gpencil_v3d_align: GP_PROJECT_VIEWSPACE,
            gpencil_v2d_align: GP_PROJECT_VIEWSPACE,
            annotate_v3d_align: GP_PROJECT_VIEWSPACE | GP_PROJECT_CURSOR,
            annotate_thickness: 3,
            gpencil_surface_offset: 0.0,
            gpencil_selectmode_edit: GP_SELECTMODE_POINT,
            gpencil_selectmode_sculpt: 0,
            gp_sculpt: GpSculptSettings::default(),
            gp_interpolate: GpInterpolateSettings::default(),
            imapaint: ImagePaintSettings::default(),
            paint_mode: PaintModeSettings::default(),
            particle: ParticleEditSettings::default(),
            proportional_size: 1.0,
            select_thresh: 0.01,
            keying_flag: 0,
            autokey_mode: AUTOKEY_MODE_NORMAL,
            keyframe_type: 0,
            edge_mode: 0,
            edge_mode_live_unwrap: 0,
            transform_pivot_point: V3D_AROUND_CENTER_MEDIAN as i8,
            transform_flag: 0,
            snap_node_mode: SCE_SNAP_TO_GRID as i8,
            snap_mode: SCE_SNAP_TO_INCREMENT,
            snap_uv_mode: SCE_SNAP_TO_INCREMENT,
            snap_anim_mode: SCE_SNAP_TO_FRAME,
            snap_playhead_mode: SCE_SNAP_TO_KEYS | SCE_SNAP_TO_STRIPS,
            snap_flag: SCE_SNAP_TO_INCLUDE_EDITED | SCE_SNAP_TO_INCLUDE_NONEDITED,
            snap_flag_node: 0,
            snap_flag_seq: SCE_SNAP,
            snap_flag_anim: SCE_SNAP,
            snap_flag_driver: 0,
            snap_flag_playhead: 0,
            snap_uv_flag: 0,
            snap_target: SCE_SNAP_SOURCE_CLOSEST,
            snap_transform_mode_flag: SCE_SNAP_TRANSFORM_MODE_TRANSLATE,
            snap_face_nearest_steps: 1,
            proportional_edit: 0,
            prop_mode: 0,
            proportional_objects: 0,
            proportional_mask: 0,
            proportional_action: 0,
            proportional_fcurve: 0,
            lock_markers: 0,
            auto_normalize: 1,
            wpaint_lock_relative: 0,
            multipaint: 1,
            weightuser: OB_DRAW_GROUPUSER_ACTIVE,
            vgroupsubset: WT_VGROUP_ALL,
            gpencil_selectmode_vertex: 0,
            uv_sculpt_settings: 0,
            workspace_tool_type: 0,
            sculpt_paint_settings: 0,
            sculpt_paint_unified_size: 0,
            sculpt_paint_unified_unprojected_radius: 0.0,
            sculpt_paint_unified_alpha: 0.0,
            unified_paint_settings: UnifiedPaintSettings::default(),
            curve_paint_settings: CurvePaintSettings::default(),
            statvis: MeshStatVis::default(),
            normal_vector: [0.0; 3],
            custom_bevel_profile_preset: NIL,
            sequencer_tool_settings: NIL,
            snap_mode_tools: SCE_SNAP_TO_GEOM,
            plane_axis: 2,
            plane_depth: V3D_PLACE_DEPTH_SURFACE,
            plane_orient: V3D_PLACE_ORIENT_SURFACE,
            use_plane_axis_auto: 0,
            snap_angle_increment_2d: d(5.0),
            snap_angle_increment_2d_precision: d(1.0),
            snap_angle_increment_3d: d(5.0),
            snap_angle_increment_3d_precision: d(1.0),
            snap_step_seconds: 1,
            snap_step_frames: 2,
            playhead_snap_distance: 20,
            anim_mirror_object: NIL,
            anim_relative_object: NIL,
            anim_mirror_bone: [0; 64],
            fix_to_cam_flag: FIX_TO_CAM_FLAG_USE_LOC
                | FIX_TO_CAM_FLAG_USE_ROT
                | FIX_TO_CAM_FLAG_USE_SCALE,
        }
    }
}

/// `USER_UNIT_ADAPTIVE`.
pub const USER_UNIT_ADAPTIVE: u8 = 0xFF;

/// `eUnit_System`.
pub const USER_UNIT_NONE: i8 = 0;
pub const USER_UNIT_METRIC: i8 = 1;
pub const USER_UNIT_IMPERIAL: i8 = 2;

/// `eUnit_Flag`.
pub const USER_UNIT_OPT_SPLIT: i8 = 1;
pub const USER_UNIT_ROT_RADIANS_DEPRECATED: i8 = 2;

/// `eUnit_RotationSystem`.
pub const USER_UNIT_ROT_DEGREES: i8 = 0;
pub const USER_UNIT_ROT_RADIANS: i8 = 2;

/// `UnitSettings`.
#[derive(Clone, Default)]
pub struct UnitSettings {
    pub scale_length: f32,
    pub system: i8,
    pub system_rotation: i8,
    pub flag: i16,
    pub length_unit: i8,
    pub mass_unit: i8,
    pub time_unit: i8,
    pub temperature_unit: i8,
}

/// `ePhysics_Flag`.
pub const PHYS_GLOBAL_GRAVITY: i32 = 1;

/// `PhysicsSettings`.
#[derive(Clone)]
pub struct PhysicsSettings {
    pub gravity: [f32; 3],
    pub flag: i32,
    pub quick_cache_step: i32,
}
impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: [0.0, 0.0, -9.81],
            flag: PHYS_GLOBAL_GRAVITY,
            quick_cache_step: 0,
        }
    }
}

/// `DisplaySafeAreas`.
#[derive(Clone)]
pub struct DisplaySafeAreas {
    pub title: [f32; 2],
    pub action: [f32; 2],
    pub title_center: [f32; 2],
    pub action_center: [f32; 2],
}
impl Default for DisplaySafeAreas {
    fn default() -> Self {
        Self {
            title: [10.0 / 100.0, 5.0 / 100.0],
            action: [3.5 / 100.0, 3.5 / 100.0],
            title_center: [17.5 / 100.0, 5.0 / 100.0],
            action_center: [15.0 / 100.0, 3.5 / 100.0],
        }
    }
}

/// `eSceneDisplay_AA`.
pub const SCE_DISPLAY_AA_OFF: i8 = 0;
pub const SCE_DISPLAY_AA_FXAA: i8 = 1;
pub const SCE_DISPLAY_AA_SAMPLES_5: i8 = 5;
pub const SCE_DISPLAY_AA_SAMPLES_8: i8 = 8;
pub const SCE_DISPLAY_AA_SAMPLES_11: i8 = 11;
pub const SCE_DISPLAY_AA_SAMPLES_16: i8 = 16;
pub const SCE_DISPLAY_AA_SAMPLES_32: i8 = 32;

/// `M_SQRT1_3`.
pub const M_SQRT1_3: f32 = 0.577_350_269_19;

/// `SceneDisplay`.
#[derive(Clone)]
pub struct SceneDisplay {
    pub light_direction: [f32; 3],
    pub shadow_shift: f32,
    pub shadow_focus: f32,
    pub matcap_ssao_distance: f32,
    pub matcap_ssao_attenuation: f32,
    pub matcap_ssao_samples: i32,
    pub viewport_aa: i8,
    pub render_aa: i8,
    pub shading: View3DShading,
}
impl Default for SceneDisplay {
    fn default() -> Self {
        Self {
            light_direction: [M_SQRT1_3; 3],
            shadow_shift: 0.1,
            shadow_focus: 0.0,
            matcap_ssao_distance: 0.2,
            matcap_ssao_attenuation: 1.0,
            matcap_ssao_samples: 16,
            viewport_aa: SCE_DISPLAY_AA_FXAA,
            render_aa: SCE_DISPLAY_AA_SAMPLES_8,
            shading: View3DShading::default(),
        }
    }
}

/// `RaytraceEEVEE_Flag`.
pub const RAYTRACE_EEVEE_USE_DENOISE: i32 = 1 << 0;
pub const RAYTRACE_EEVEE_USE_BACKFACE: i32 = 1 << 1;

/// `RaytraceEEVEE_DenoiseStages`.
pub const RAYTRACE_EEVEE_DENOISE_SPATIAL: i32 = 1 << 0;
pub const RAYTRACE_EEVEE_DENOISE_TEMPORAL: i32 = 1 << 1;
pub const RAYTRACE_EEVEE_DENOISE_BILATERAL: i32 = 1 << 2;

/// `RaytraceEEVEE_Method`.
pub const RAYTRACE_EEVEE_METHOD_PROBE: i32 = 0;
pub const RAYTRACE_EEVEE_METHOD_SCREEN: i32 = 1;

/// `RaytraceEEVEE`.
#[derive(Clone)]
pub struct RaytraceEevee {
    pub screen_trace_quality: f32,
    pub screen_trace_thickness: f32,
    pub trace_max_roughness: f32,
    pub resolution_scale: i32,
    pub flag: i32,
    pub denoise_stages: i32,
    pub backface_radiance_scale: f32,
}
impl Default for RaytraceEevee {
    fn default() -> Self {
        Self {
            screen_trace_quality: 0.25,
            screen_trace_thickness: 0.1,
            trace_max_roughness: 0.5,
            resolution_scale: 2,
            flag: RAYTRACE_EEVEE_USE_DENOISE | RAYTRACE_EEVEE_USE_BACKFACE,
            denoise_stages: RAYTRACE_EEVEE_DENOISE_SPATIAL
                | RAYTRACE_EEVEE_DENOISE_TEMPORAL
                | RAYTRACE_EEVEE_DENOISE_BILATERAL,
            backface_radiance_scale: 0.25,
        }
    }
}

/// `eSceneEEVEE_Flag`.
pub const SCE_EEVEE_VOLUMETRIC_SHADOWS: i32 = 1 << 2;
pub const SCE_EEVEE_GTAO_ENABLED: i32 = 1 << 4;
pub const SCE_EEVEE_MOTION_BLUR_ENABLED_DEPRECATED: i32 = 1 << 9;
pub const SCE_EEVEE_TAA_REPROJECTION: i32 = 1 << 11;
pub const SCE_EEVEE_SSR_ENABLED: i32 = 1 << 14;
pub const SCE_EEVEE_GI_AUTOBAKE: i32 = 1 << 19;
pub const SCE_EEVEE_OVERSCAN: i32 = 1 << 21;
pub const SCE_EEVEE_DOF_JITTER: i32 = 1 << 23;
pub const SCE_EEVEE_SHADOW_ENABLED: i32 = 1 << 24;
pub const SCE_EEVEE_RAYTRACE_OPTIONS_SPLIT: i32 = 1 << 25;
pub const SCE_EEVEE_SHADOW_JITTERED_VIEWPORT: i32 = 1 << 26;
pub const SCE_EEVEE_VOLUME_CUSTOM_RANGE: i32 = 1 << 27;
pub const SCE_EEVEE_FAST_GI_ENABLED: i32 = 1 << 28;

/// `FastGI_Method`.
pub const FAST_GI_FULL: i8 = 0;
pub const FAST_GI_AO_ONLY: i8 = 1;

/// `SceneEEVEE`.
#[derive(Clone)]
pub struct SceneEevee {
    pub flag: i32,
    pub gi_diffuse_bounces: i32,
    pub gi_cubemap_resolution: i32,
    pub gi_visibility_resolution: i32,
    pub gi_glossy_clamp: f32,
    pub gi_irradiance_pool_size: i32,
    pub taa_samples: i32,
    pub taa_render_samples: i32,
    pub volumetric_start: f32,
    pub volumetric_end: f32,
    pub volumetric_tile_size: i32,
    pub volumetric_samples: i32,
    pub volumetric_sample_distribution: f32,
    pub volumetric_light_clamp: f32,
    pub volumetric_shadow_samples: i32,
    pub volumetric_ray_depth: i32,
    /// DNA_DEPRECATED `gtao_distance`, `gtao_thickness`.
    pub gtao_distance: f32,
    pub gtao_thickness: f32,
    pub fast_gi_bias: f32,
    pub fast_gi_resolution: i32,
    pub fast_gi_step_count: i32,
    pub fast_gi_ray_count: i32,
    pub fast_gi_quality: f32,
    pub fast_gi_distance: f32,
    pub fast_gi_thickness_near: f32,
    pub fast_gi_method: i8,
    pub bokeh_overblur: f32,
    pub bokeh_max_size: f32,
    pub bokeh_threshold: f32,
    pub bokeh_neighbor_max: f32,
    /// DNA_DEPRECATED `motion_blur_samples`.
    pub motion_blur_samples: i32,
    pub motion_blur_max: i32,
    pub motion_blur_steps: i32,
    /// DNA_DEPRECATED `motion_blur_position_deprecated`, `motion_blur_shutter_deprecated`.
    pub motion_blur_position_deprecated: i32,
    pub motion_blur_shutter_deprecated: f32,
    pub motion_blur_depth_scale: f32,
    /// DNA_DEPRECATED `shadow_cube_size_deprecated`.
    pub shadow_cube_size_deprecated: i32,
    pub shadow_pool_size: i32,
    pub shadow_ray_count: i32,
    pub shadow_step_count: i32,
    pub shadow_resolution_scale: f32,
    pub clamp_surface_direct: f32,
    pub clamp_surface_indirect: f32,
    pub clamp_volume_direct: f32,
    pub clamp_volume_indirect: f32,
    pub direct_light_intensity: f32,
    pub indirect_light_intensity: f32,
    pub ray_tracing_method: i32,
    pub ray_tracing_options: RaytraceEevee,
    pub overscan: f32,
    pub light_threshold: f32,
    pub time_limit: f32,
}
impl Default for SceneEevee {
    fn default() -> Self {
        Self {
            flag: SCE_EEVEE_TAA_REPROJECTION | SCE_EEVEE_SHADOW_ENABLED,
            gi_diffuse_bounces: 3,
            gi_cubemap_resolution: 512,
            gi_visibility_resolution: 32,
            gi_glossy_clamp: 0.0,
            gi_irradiance_pool_size: 16,
            taa_samples: 16,
            taa_render_samples: 64,
            volumetric_start: 0.1,
            volumetric_end: 100.0,
            volumetric_tile_size: 8,
            volumetric_samples: 64,
            volumetric_sample_distribution: 0.8,
            volumetric_light_clamp: 0.0,
            volumetric_shadow_samples: 16,
            volumetric_ray_depth: 16,
            gtao_distance: 0.0,
            gtao_thickness: 0.0,
            fast_gi_bias: 0.05,
            fast_gi_resolution: 2,
            fast_gi_step_count: 8,
            fast_gi_ray_count: 2,
            fast_gi_quality: 0.25,
            fast_gi_distance: 0.0,
            fast_gi_thickness_near: 0.1,
            fast_gi_method: FAST_GI_FULL,
            bokeh_overblur: 5.0,
            bokeh_max_size: 100.0,
            bokeh_threshold: 1.0,
            bokeh_neighbor_max: 10.0,
            motion_blur_samples: 0,
            motion_blur_max: 32,
            motion_blur_steps: 1,
            motion_blur_position_deprecated: 0,
            motion_blur_shutter_deprecated: 0.0,
            motion_blur_depth_scale: 100.0,
            shadow_cube_size_deprecated: 0,
            shadow_pool_size: 512,
            shadow_ray_count: 1,
            shadow_step_count: 6,
            shadow_resolution_scale: 1.0,
            clamp_surface_direct: 0.0,
            clamp_surface_indirect: 10.0,
            clamp_volume_direct: 0.0,
            clamp_volume_indirect: 0.0,
            direct_light_intensity: 1.0,
            indirect_light_intensity: 1.0,
            ray_tracing_method: RAYTRACE_EEVEE_METHOD_SCREEN,
            ray_tracing_options: RaytraceEevee::default(),
            overscan: 3.0,
            light_threshold: 0.01,
            time_limit: 0.0,
        }
    }
}

/// `SceneGpencil`.
#[derive(Clone)]
pub struct SceneGpencil {
    pub smaa_threshold: f32,
    pub smaa_threshold_render: f32,
    pub aa_samples: i32,
    pub motion_blur_steps: i32,
}
impl Default for SceneGpencil {
    fn default() -> Self {
        Self {
            smaa_threshold: 1.0,
            smaa_threshold_render: 0.25,
            aa_samples: 8,
            motion_blur_steps: 8,
        }
    }
}

/// `eSceneHydra_ExportMethod`.
pub const SCE_HYDRA_EXPORT_HYDRA: i32 = 0;
pub const SCE_HYDRA_EXPORT_USD: i32 = 1;

/// `SceneHydra`.
#[derive(Clone, Default)]
pub struct SceneHydra {
    pub export_method: i32,
}

/// `eScene_OrientationSlot`.
pub const SCE_ORIENT_DEFAULT: i32 = 0;
pub const SCE_ORIENT_TRANSLATE: i32 = 1;
pub const SCE_ORIENT_ROTATE: i32 = 2;
pub const SCE_ORIENT_SCALE: i32 = 3;

/// `TransformOrientationSlot`.
#[derive(Clone, Copy, Default)]
pub struct TransformOrientationSlot {
    pub type_: i32,
    pub index_custom: i32,
    pub flag: i8,
}

/// `SceneCompositorEffectFlags`.
pub const SCENE_COMPOSITOR_EFFECT_NONE: u8 = 0;
pub const SCENE_COMPOSITOR_EFFECT_ENABLE_FOR_RENDER: u8 = 1 << 0;
pub const SCENE_COMPOSITOR_EFFECT_ENABLE_FOR_PREVIEW: u8 = 1 << 1;
pub const SCENE_COMPOSITOR_EFFECT_IS_ACTIVE: u8 = 1 << 2;
pub const SCENE_COMPOSITOR_EFFECT_SHOW_NODE_GROUP_SELECTOR: u8 = 1 << 3;

/// `uiPanelDataExpansion` / `UI_PANEL_DATA_EXPAND_ROOT` placeholders.
pub type UiPanelDataExpansion = u8;
pub const UI_PANEL_DATA_EXPAND_ROOT: u8 = 1;

/// `SceneCompositorEffect`.
#[derive(Clone)]
pub struct SceneCompositorEffect {
    pub next: usize,
    pub previous: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `bNodeTree *node_group`.
    pub node_group: usize,
    /// `IDProperty *system_properties`.
    pub system_properties: usize,
    pub flags: u8,
    pub ui_panel_data_expansion: UiPanelDataExpansion,
}
impl Default for SceneCompositorEffect {
    fn default() -> Self {
        Self {
            next: NIL,
            previous: NIL,
            name: [0; 64],
            node_group: NIL,
            system_properties: NIL,
            flags: SCENE_COMPOSITOR_EFFECT_ENABLE_FOR_RENDER
                | SCENE_COMPOSITOR_EFFECT_ENABLE_FOR_PREVIEW
                | SCENE_COMPOSITOR_EFFECT_SHOW_NODE_GROUP_SELECTOR,
            ui_panel_data_expansion: UI_PANEL_DATA_EXPAND_ROOT,
        }
    }
}

/// `eScene_Flag`.
pub const SCE_DS_SELECTED: i32 = 1 << 0;
pub const SCE_DS_COLLAPSED: i32 = 1 << 1;
pub const SCE_NLA_EDIT_ON: i32 = 1 << 2;
pub const SCE_FRAME_DROP: i32 = 1 << 3;
pub const SCE_KEYS_NO_SELONLY: i32 = 1 << 4;
pub const SCE_READFILE_LIBLINK_NEED_SETSCENE_CHECK: i32 = 1 << 5;
pub const SCE_CUSTOM_SIMULATION_RANGE: i32 = 1 << 6;

/// `eScenePlaybackLoopMode`.
pub const SCE_LOOP_MODE_INFINITE: u8 = 0;
pub const SCE_LOOP_MODE_STOP_END_FRAME: u8 = 1;
pub const SCE_LOOP_MODE_STOP_START_FRAME: u8 = 2;
pub const SCE_LOOP_MODE_RESTORE: u8 = 3;
pub const SCE_LOOP_MODE_BOUNCE: u8 = 4;

/// `eScene_IterFlag`.
pub const F_START: i32 = 0;
pub const F_SCENE: i32 = 1;
pub const F_DUPLI: i32 = 3;

/// `SceneDepsgraphsMap` placeholder → opaque.
pub type SceneDepsgraphsMap = usize;
/// `RigidBodyWorld`/`MovieClip`/`bGPdata`/`KeyingSet`/`Editing` → arena refs.

/// `Scene` — the scene data-block.
#[derive(Clone)]
pub struct Scene {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `Object *camera`.
    pub camera: usize,
    /// `World *world`.
    pub world: usize,
    /// `Scene *set`.
    pub set: usize,
    /// `ListBaseT<Base>` (legacy; view_layers hold bases).
    pub base: ListBase,
    /// DNA_DEPRECATED `Base *basact`.
    pub basact: usize,
    pub cursor: View3DCursor,
    /// DNA_DEPRECATED `lay`, `layact`.
    pub lay: u32,
    pub layact: i32,
    pub flag: i16,
    /// DNA_DEPRECATED `use_nodes`.
    pub use_nodes: i8,
    /// DNA_DEPRECATED `bNodeTree *nodetree`, `*compositing_node_group`.
    pub nodetree: usize,
    pub compositing_node_group: usize,
    /// `ListBaseT<SceneCompositorEffect>`.
    pub compositor_effects: ListBase,
    /// `Editing *ed`.
    pub ed: usize,
    /// `ToolSettings *toolsettings`.
    pub toolsettings: usize,
    pub safe_areas: DisplaySafeAreas,
    pub r: RenderData,
    pub audio: AudioData,
    /// `ListBaseT<TimeMarker>`.
    pub markers: ListBase,
    /// `ListBaseT<TransformOrientation>`.
    pub transform_spaces: ListBase,
    /// `TransformOrientationSlot orientation_slots[4]`.
    pub orientation_slots: [TransformOrientationSlot; 4],
    /// `void *fps_info`.
    pub fps_info: usize,
    /// `SceneDepsgraphsMap *depsgraph_hash`.
    pub depsgraph_hash: usize,
    pub playback_loop_mode: u8,
    pub active_keyingset: i32,
    /// `ListBaseT<KeyingSet>`.
    pub keyingsets: ListBase,
    pub unit: UnitSettings,
    /// `bGPdata *gpd`.
    pub gpd: usize,
    /// `MovieClip *clip`.
    pub clip: usize,
    pub physics_settings: PhysicsSettings,
    pub customdata_mask: CustomDataMeshMasks,
    pub customdata_mask_modal: CustomDataMeshMasks,
    pub view_settings: ColorManagedViewSettings,
    pub display_settings: ColorManagedDisplaySettings,
    pub sequencer_colorspace_settings: ColorManagedColorspaceSettings,
    /// `RigidBodyWorld *rigidbody_world`.
    pub rigidbody_world: usize,
    /// `PreviewImage *preview`.
    pub preview: usize,
    /// `ListBaseT<ViewLayer>`.
    pub view_layers: ListBase,
    /// `Collection *master_collection`.
    pub master_collection: usize,
    /// `IDProperty *layer_properties`.
    pub layer_properties: usize,
    pub simulation_frame_start: i32,
    pub simulation_frame_end: i32,
    pub display: SceneDisplay,
    pub eevee: SceneEevee,
    pub grease_pencil_settings: SceneGpencil,
    pub hydra: SceneHydra,
    /// `bke::SceneRuntime *runtime`.
    pub runtime: usize,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            camera: NIL,
            world: NIL,
            set: NIL,
            base: ListBase::new(),
            basact: NIL,
            cursor: View3DCursor::default(),
            lay: 0,
            layact: 0,
            flag: 0,
            use_nodes: 0,
            nodetree: NIL,
            compositing_node_group: NIL,
            compositor_effects: ListBase::new(),
            ed: NIL,
            toolsettings: NIL,
            safe_areas: DisplaySafeAreas::default(),
            r: RenderData::default(),
            audio: AudioData::default(),
            markers: ListBase::new(),
            transform_spaces: ListBase::new(),
            orientation_slots: [TransformOrientationSlot::default(); 4],
            fps_info: NIL,
            depsgraph_hash: NIL,
            playback_loop_mode: SCE_LOOP_MODE_INFINITE,
            active_keyingset: 0,
            keyingsets: ListBase::new(),
            unit: UnitSettings::default(),
            gpd: NIL,
            clip: NIL,
            physics_settings: PhysicsSettings::default(),
            customdata_mask: CustomDataMeshMasks::default(),
            customdata_mask_modal: CustomDataMeshMasks::default(),
            view_settings: ColorManagedViewSettings::default(),
            display_settings: ColorManagedDisplaySettings::default(),
            sequencer_colorspace_settings: ColorManagedColorspaceSettings::default(),
            rigidbody_world: NIL,
            preview: NIL,
            view_layers: ListBase::new(),
            master_collection: NIL,
            layer_properties: NIL,
            simulation_frame_start: 1,
            simulation_frame_end: 250,
            display: SceneDisplay::default(),
            eevee: SceneEevee::default(),
            grease_pencil_settings: SceneGpencil::default(),
            hydra: SceneHydra::default(),
            runtime: NIL,
        }
    }
}
impl Scene {
    /// `frames_per_second()` — `r.frs_sec / r.frs_sec_base`.
    pub fn frames_per_second(&self) -> f64 {
        self.r.frs_sec as f64 / self.r.frs_sec_base as f64
    }
    /// `playback_start()` — honors `SCER_PRV_RANGE`.
    pub fn playback_start(&self) -> i32 {
        if self.r.flag & SCER_PRV_RANGE != 0 {
            self.r.psfra
        } else {
            self.r.sfra
        }
    }
    /// `playback_end()`.
    pub fn playback_end(&self) -> i32 {
        if self.r.flag & SCER_PRV_RANGE != 0 {
            self.r.pefra
        } else {
            self.r.efra
        }
    }
}

/// Render engine ids (`RE_engine_id_*`).
pub const RE_ENGINE_ID_BLENDER_EEVEE: &str = "BLENDER_EEVEE_NEXT";
pub const RE_ENGINE_ID_BLENDER_WORKBENCH: &str = "BLENDER_WORKBENCH";
pub const RE_ENGINE_ID_CYCLES: &str = "CYCLES";

pub const MAXFRAME: i32 = 1_048_574;
pub const MAXFRAMEF: f32 = 1_048_574.0;
pub const MINFRAME: i32 = 0;
pub const MINFRAMEF: f32 = 0.0;
pub const MINAFRAME: i32 = -1_048_574;
pub const MINAFRAMEF: f32 = -1_048_574.0;
