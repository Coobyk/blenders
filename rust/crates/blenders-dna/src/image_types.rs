//! Port of `DNA_image_types.h` — `Image`, `ImageUser`, `ImageTile`,
//! `ImagePackedFile`, `RenderSlot`, `ImageView`, `ImageAnim` + enums.
#![allow(non_camel_case_types)]
use crate::color_types::ColorManagedColorspaceSettings;
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `eImageUser_Flag`.
pub const IMA_ANIM_ALWAYS: i16 = 1 << 0;
pub const IMA_SHOW_SEQUENCER_SCENE: i16 = 1 << 1;
pub const IMA_NEED_FRAME_RECALC: i16 = 1 << 3;
pub const IMA_SHOW_STEREO: i16 = 1 << 4;
pub const IMA_USER_FRAME_IN_RANGE: i16 = 1 << 10;

/// `eImage_Flag`.
pub type ImageFlag = i32;
pub const IMA_HIGH_BITDEPTH: ImageFlag = 1 << 0;
pub const IMA_FLAG_UNUSED_1: ImageFlag = 1 << 1;
pub const IMA_DO_PREMUL: ImageFlag = 1 << 2;
pub const IMA_FLAG_UNUSED_4: ImageFlag = 1 << 4;
pub const IMA_NOCOLLECT: ImageFlag = 1 << 5;
pub const IMA_FLAG_UNUSED_6: ImageFlag = 1 << 6;
pub const IMA_OLD_PREMUL: ImageFlag = 1 << 7;
pub const IMA_FLAG_UNUSED_8: ImageFlag = 1 << 8;
pub const IMA_USED_FOR_RENDER: ImageFlag = 1 << 9;
pub const IMA_VIEW_AS_RENDER: ImageFlag = 1 << 11;
pub const IMA_FLAG_UNUSED_12: ImageFlag = 1 << 12;
pub const IMA_DEINTERLACE: ImageFlag = 1 << 13;
pub const IMA_USE_VIEWS: ImageFlag = 1 << 14;
pub const IMA_FLAG_UNUSED_15: ImageFlag = 1 << 15;
pub const IMA_FLAG_UNUSED_16: ImageFlag = 1 << 16;
pub const IMA_AUTOSAVE_TEMPPACK: ImageFlag = 1 << 17;

/// `eImageSource`.
pub type ImageSource = i16;
pub const IMA_SRC_FILE: ImageSource = 1;
pub const IMA_SRC_SEQUENCE: ImageSource = 2;
pub const IMA_SRC_MOVIE: ImageSource = 3;
pub const IMA_SRC_GENERATED: ImageSource = 4;
pub const IMA_SRC_VIEWER: ImageSource = 5;
pub const IMA_SRC_TILED: ImageSource = 6;

/// `eImageType`.
pub type ImageType = i16;
pub const IMA_TYPE_IMAGE: ImageType = 0;
pub const IMA_TYPE_MULTILAYER: ImageType = 1;
pub const IMA_TYPE_UV_TEST: ImageType = 2;
pub const IMA_TYPE_R_RESULT: ImageType = 4;
pub const IMA_TYPE_COMPOSITE: ImageType = 5;

/// `eImageGenType`.
pub type ImageGenType = i8;
pub const IMA_GENTYPE_BLANK: ImageGenType = 0;
pub const IMA_GENTYPE_GRID: ImageGenType = 1;
pub const IMA_GENTYPE_GRID_COLOR: ImageGenType = 2;

pub const IMA_MAX_RENDER_TEXT_SIZE: usize = 512;

/// `eImage_GenFlag`.
pub const IMA_GEN_FLOAT: i8 = 1 << 0;
pub const IMA_GEN_TILE: i8 = 1 << 1;

/// `eImageAlphaMode`.
pub type ImageAlphaMode = i8;
pub const IMA_ALPHA_STRAIGHT: ImageAlphaMode = 0;
pub const IMA_ALPHA_PREMUL: ImageAlphaMode = 1;
pub const IMA_ALPHA_CHANNEL_PACKED: ImageAlphaMode = 2;
pub const IMA_ALPHA_IGNORE: ImageAlphaMode = 3;

/// `eImageFormat_ViewsFormat` (DNA_scene_enums.h) placeholder.
pub type ImageFormatViewsFormat = i32;

/// `ImageUser`.
#[derive(Clone)]
pub struct ImageUser {
    /// `Scene *scene`.
    pub scene: usize,
    pub framenr: i32,
    pub frames: i32,
    pub offset: i32,
    pub sfra: i32,
    pub cycl: i8,
    pub multiview_eye: i8,
    pub pass: i16,
    pub tile: i32,
    pub multi_index: i16,
    pub view: i16,
    pub layer: i16,
    pub flag: i16,
}
impl Default for ImageUser {
    fn default() -> Self {
        Self {
            scene: NIL,
            framenr: 0,
            frames: 0,
            offset: 0,
            sfra: 0,
            cycl: 0,
            multiview_eye: 0,
            pass: 0,
            tile: 0,
            multi_index: 0,
            view: 0,
            layer: 0,
            flag: 0,
        }
    }
}

/// `ImageAnim`.
#[derive(Clone)]
pub struct ImageAnim {
    pub next: usize,
    pub prev: usize,
    /// `MovieReader *anim`.
    pub anim: usize,
}
impl Default for ImageAnim {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            anim: NIL,
        }
    }
}

/// `ImageView`.
#[derive(Clone)]
pub struct ImageView {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`, `filepath[1024]`.
    pub name: [u8; 64],
    pub filepath: [u8; 1024],
}
impl Default for ImageView {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            filepath: [0; 1024],
        }
    }
}

/// `ImagePackedFile`.
#[derive(Clone)]
pub struct ImagePackedFile {
    pub next: usize,
    pub prev: usize,
    /// `PackedFile *packedfile`.
    pub packedfile: usize,
    pub view: i32,
    pub tile_number: i32,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
}
impl Default for ImagePackedFile {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            packedfile: NIL,
            view: 0,
            tile_number: 0,
            filepath: [0; 1024],
        }
    }
}

/// `RenderSlot`.
#[derive(Clone)]
pub struct RenderSlot {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `RenderResult *render`.
    pub render: usize,
}
impl Default for RenderSlot {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            render: NIL,
        }
    }
}

/// `ImageTile_Runtime`.
#[derive(Clone, Copy, Default)]
pub struct ImageTileRuntime {
    pub tilearray_layer: i32,
    pub tilearray_offset: [i32; 2],
    pub tilearray_size: [i32; 2],
}

/// `ImageTile`.
#[derive(Clone)]
pub struct ImageTile {
    pub next: usize,
    pub prev: usize,
    pub runtime: ImageTileRuntime,
    pub tile_number: i32,
    pub gen_x: i32,
    pub gen_y: i32,
    pub gen_type: ImageGenType,
    pub gen_flag: i8,
    pub gen_depth: i16,
    pub gen_color: [f32; 4],
    /// `char label[64]`.
    pub label: [u8; 64],
}
impl Default for ImageTile {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            runtime: ImageTileRuntime::default(),
            tile_number: 0,
            gen_x: 0,
            gen_y: 0,
            gen_type: IMA_GENTYPE_BLANK,
            gen_flag: 0,
            gen_depth: 0,
            gen_color: [0.0; 4],
            label: [0; 64],
        }
    }
}

/// `Image`.
#[derive(Clone)]
pub struct Image {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    /// `ListBaseT<ImageAnim>`.
    pub anims: ListBase,
    /// `RenderResult *rr`.
    pub rr: usize,
    /// `ListBaseT<RenderSlot>`.
    pub renderslots: ListBase,
    pub render_slot: i16,
    pub last_render_slot: i16,
    pub flag: ImageFlag,
    pub source: ImageSource,
    pub type_: ImageType,
    pub lastframe: i32,
    pub seam_margin: i16,
    /// DNA_DEPRECATED `PackedFile *packedfile`.
    pub packedfile: usize,
    /// `ListBaseT<ImagePackedFile>`.
    pub packedfiles: ListBase,
    /// `PreviewImage *preview`.
    pub preview: usize,
    /// `ListBaseT<ImagePackedFile>` autosave_packedfiles.
    pub autosave_packedfiles: ListBase,
    /// DNA_DEPRECATED gen fields.
    pub gen_x: i32,
    pub gen_y: i32,
    pub gen_type: ImageGenType,
    pub gen_flag: i8,
    pub gen_depth: i16,
    pub gen_color: [f32; 4],
    pub aspx: f32,
    pub aspy: f32,
    pub colorspace_settings: ColorManagedColorspaceSettings,
    pub alpha_mode: ImageAlphaMode,
    pub eye: i8,
    pub views_format: ImageFormatViewsFormat,
    pub active_tile_index: i32,
    /// `ListBaseT<ImageTile>`.
    pub tiles: ListBase,
    /// `ListBaseT<ImageView>`.
    pub views: ListBase,
    /// `Stereo3dFormat *stereo3d_format`.
    pub stereo3d_format: usize,
    /// `bke::ImageRuntime *runtime`.
    pub runtime: usize,
}
impl Default for Image {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            filepath: [0; 1024],
            anims: ListBase::new(),
            rr: NIL,
            renderslots: ListBase::new(),
            render_slot: 0,
            last_render_slot: 0,
            flag: 0,
            source: 0,
            type_: IMA_TYPE_IMAGE,
            lastframe: 0,
            seam_margin: 8,
            packedfile: NIL,
            packedfiles: ListBase::new(),
            preview: NIL,
            autosave_packedfiles: ListBase::new(),
            gen_x: 1024,
            gen_y: 1024,
            gen_type: IMA_GENTYPE_GRID,
            gen_flag: 0,
            gen_depth: 0,
            gen_color: [0.0; 4],
            aspx: 1.0,
            aspy: 1.0,
            colorspace_settings: ColorManagedColorspaceSettings::default(),
            alpha_mode: IMA_ALPHA_STRAIGHT,
            eye: 0,
            views_format: 0,
            active_tile_index: 0,
            tiles: ListBase::new(),
            views: ListBase::new(),
            stereo3d_format: NIL,
            runtime: NIL,
        }
    }
}
