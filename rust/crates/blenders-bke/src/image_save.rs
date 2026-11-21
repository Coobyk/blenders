//! Port of `intern/image_save.cc` — `ImageSaveOptions`
//! (init/update/free), `image_save_update_filepath`,
//! `image_render_print_save_message`, `image_render_write_stamp_test`,
//! the filepath guessing logic of `BKE_image_save_options_init`, and
//! the EXR helper `image_exr_from_rgb_to_bw` /
//! `image_exr_opaque_alpha_buffer`.
//! `PORT_PENDING`: `ImBuf` write paths (`IMB_imb_write*`,
//! `BKE_image_acquire_ibuf`), `BKE_image_save`, `BKE_image_render_write`,
//! `add_exr_compositing_result` (OpenEXR), reports/colormanagement.

use blenders_dna::image_types::{Image, IMA_SRC_MOVIE, IMA_SRC_TILED, IMA_SRC_VIEWER};
use blenders_dna::scene_types::ImageFormatData;

/// `ImColorMode` values relevant to save.
pub mod im_color_mode {
    /// `ImColorMode::BW`.
    pub const BW: i32 = 0;
    /// `ImColorMode::RGB`.
    pub const RGB: i32 = 1;
    /// `ImColorMode::RGBA`.
    pub const RGBA: i32 = 2;
}

/// `R_IMF_COLOR_MANAGEMENT_FOLLOW_SCENE`.
pub const R_IMF_COLOR_MANAGEMENT_FOLLOW_SCENE: i8 = 2;

/// `FILE_MAX`.
pub const FILE_MAX: usize = 1024;
/// `MAX_COLORSPACE_NAME`.
pub const MAX_COLORSPACE_NAME: usize = 64;

/// `ImageSaveOptions`.
#[derive(Clone)]
pub struct ImageSaveOptions {
    /// `bmain` handle.
    pub bmain: usize,
    /// `scene` handle.
    pub scene: usize,
    /// `im_format`.
    pub im_format: ImageFormatData,
    /// `filepath`.
    pub filepath: String,
    /// `relative`.
    pub relative: bool,
    /// `save_copy`.
    pub save_copy: bool,
    /// `save_as_render`.
    pub save_as_render: bool,
    /// `do_newpath`.
    pub do_newpath: bool,
    /// `orig_imtype`.
    pub orig_imtype: i8,
    /// `orig_colorspace`.
    pub orig_colorspace: String,
    /// `prev_save_as_render`.
    pub prev_save_as_render: bool,
    /// `prev_imtype`.
    pub prev_imtype: i8,
}

impl Default for ImageSaveOptions {
    fn default() -> Self {
        Self {
            bmain: usize::MAX,
            scene: usize::MAX,
            im_format: ImageFormatData::default(),
            filepath: String::new(),
            relative: false,
            save_copy: false,
            save_as_render: false,
            do_newpath: false,
            orig_imtype: 0,
            orig_colorspace: String::new(),
            prev_save_as_render: false,
            prev_imtype: 0,
        }
    }
}

/// `BKE_image_save_options_init` — the path/type decision part.
/// ImBuf/colorspace access is injected:
/// - `acquire_ok`: whether an ibuf could be acquired
/// - `image_file_path`: `BKE_image_user_file_path_ex`
/// - `fileframe`: ibuf fileframe for movies
/// - `main_filepath`: `BKE_main_blendfile_path`
/// - `last_image_path`: `G.filepath_last_image`
#[allow(clippy::too_many_arguments)]
pub fn bke_image_save_options_init(
    opts: &mut ImageSaveOptions,
    ima: &Image,
    image_name: &str,
    guess_path: bool,
    save_as_render: bool,
    acquire_ok: bool,
    image_file_path: &dyn Fn() -> String,
    fileframe: i32,
    main_filepath: &str,
    last_image_path: &str,
) -> bool {
    *opts = ImageSaveOptions::default();
    opts.save_as_render = ima.source == IMA_SRC_VIEWER || save_as_render;

    if acquire_ok {
        opts.filepath = image_file_path();

        if ima.source == IMA_SRC_MOVIE {
            /* `foo.avi` → `foo_0001.png`-style per-frame name. */
            let stem = opts
                .filepath
                .rsplit_once('.')
                .map(|(s, _)| s.to_string())
                .unwrap_or_else(|| opts.filepath.clone());
            opts.filepath = format!("{}_{:04}", stem, fileframe);
        }

        if guess_path && opts.filepath.is_empty() {
            let is_prev_save = last_image_path != "//" && !last_image_path.is_empty();
            if opts.save_as_render {
                if is_prev_save {
                    opts.filepath = last_image_path.to_string();
                } else {
                    let mut p = String::from("//Untitled");
                    blenders_bli::path_utils::abs(&mut p, main_filepath);
                    opts.filepath = p;
                }
            } else {
                let mut safe = image_name.to_string();
                blenders_bli::path_utils::make_safe_filename(&mut safe);
                let mut joined = format!("//{}", safe);
                blenders_bli::path_utils::abs(
                    &mut joined,
                    if is_prev_save {
                        last_image_path
                    } else {
                        main_filepath
                    },
                );
                opts.filepath = joined;
            }
            if ima.source == IMA_SRC_TILED && !opts.filepath.contains("<UDIM>") {
                opts.filepath.push_str(".<UDIM>");
            }
        }
        opts.orig_imtype = opts.im_format.imtype;
        opts.prev_save_as_render = opts.save_as_render;
        opts.prev_imtype = opts.im_format.imtype;
        true
    } else {
        opts.orig_imtype = opts.im_format.imtype;
        opts.prev_save_as_render = opts.save_as_render;
        opts.prev_imtype = opts.im_format.imtype;
        false
    }
}

/// `BKE_image_save_options_update` — colorspace auto-update on
/// save-as-render / file-type changes (colorspace setters injected).
pub fn bke_image_save_options_update(
    opts: &mut ImageSaveOptions,
    image_type_render: bool,
    imtype_requires_linear_float: &dyn Fn(i8) -> bool,
    space_name_is_data: &dyn Fn(&str) -> bool,
    init_for_write: &mut dyn FnMut(&mut ImageFormatData),
    copy_from_scene: &mut dyn FnMut(&mut ImageFormatData),
    set_colorspace: &mut dyn FnMut(&mut ImageFormatData, &str),
    update_color_space_for_type: &mut dyn FnMut(&mut ImageFormatData),
) {
    if opts.save_as_render {
        if !opts.prev_save_as_render {
            if image_type_render {
                init_for_write(&mut opts.im_format);
            } else {
                copy_from_scene(&mut opts.im_format);
            }
        }
    } else if opts.prev_save_as_render
        || imtype_requires_linear_float(opts.im_format.imtype)
            != imtype_requires_linear_float(opts.prev_imtype)
    {
        if space_name_is_data(&opts.orig_colorspace) {
            /* Stays the same regardless of file format. */
        } else if imtype_requires_linear_float(opts.im_format.imtype)
            == imtype_requires_linear_float(opts.orig_imtype)
        {
            set_colorspace(&mut opts.im_format, &opts.orig_colorspace);
        } else {
            update_color_space_for_type(&mut opts.im_format);
        }
    }
    opts.prev_save_as_render = opts.save_as_render;
    opts.prev_imtype = opts.im_format.imtype;
}

/// `BKE_image_save_options_free`.
pub fn bke_image_save_options_free(opts: &mut ImageSaveOptions) {
    *opts = ImageSaveOptions::default();
}

/// `image_save_update_filepath` — apply the chosen filepath back to
/// the image (relative/absolute per `opts`).
pub fn image_save_update_filepath(
    ima: &mut Image,
    filepath: &str,
    opts: &ImageSaveOptions,
    path_to_relative: &dyn Fn(&str) -> String,
    path_to_absolute: &dyn Fn(&str) -> String,
    set_name: &mut dyn FnMut(&mut Image, &str),
) {
    let path = if opts.relative {
        path_to_relative(filepath)
    } else {
        path_to_absolute(filepath)
    };
    /* Store into `ima->filepath` via the injected setter (C writes the
     * DNA field directly; the name is derived in `image_save_single` —
     * keep both hooks). */
    let _ = &path;
    set_name(ima, &path);
}

/// `image_render_print_save_message` — log/report string on save.
pub fn image_render_print_save_message(ok: bool, err: i32, filepath: &str) -> String {
    if ok {
        format!("Saved: '{}'", filepath)
    } else {
        format!(
            "Render error ({}) cannot save: '{}'",
            std::io::Error::from_raw_os_error(err),
            filepath
        )
    }
}

/// `image_render_write_stamp_test` — write with/without stamp via the
/// injected writer, report result.
pub fn image_render_write_stamp_test(
    stamp: bool,
    write_stamp: &mut dyn FnMut(&str) -> bool,
    write_plain: &mut dyn FnMut(&str) -> bool,
    filepath: &str,
) -> (bool, String) {
    let ok = if stamp {
        write_stamp(filepath)
    } else {
        write_plain(filepath)
    };
    let err = if ok { 0 } else { 5 /*EIO*/ };
    (ok, image_render_print_save_message(ok, err, filepath))
}

/// `image_exr_from_rgb_to_bw` — float RGB(A) → luminance buffer.
pub fn image_exr_from_rgb_to_bw(rect_float: &[f32], channels: usize) -> Vec<f32> {
    rect_float
        .chunks(channels)
        .map(|px| {
            /* `BLI_color.rgb_to_grayscale` luminance weights. */
            0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2]
        })
        .collect()
}

/// `image_exr_opaque_alpha_buffer` — width*height buffer of 1.0f.
pub fn image_exr_opaque_alpha_buffer(width: usize, height: usize) -> Vec<f32> {
    vec![1.0f32; width * height]
}

/// `image_exr_from_scene_linear_to_output` — colorspace transform of
/// the rect via the injected converter (ImBuf colormanagement is
/// pending); returns `None` when the rect is `None` like the C.
pub fn image_exr_from_scene_linear_to_output(
    rect_float: Option<&[f32]>,
    convert: &dyn Fn(&[f32]) -> Vec<f32>,
) -> Option<Vec<f32>> {
    rect_float.map(convert)
}
