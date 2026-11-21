//! Port of `intern/image_format.cc` — `imtype` predicates, valid
//! depths/channels, `BKE_imtype_from_arg`, extension tables and
//! `do_ensure_image_extension`, `imtype_best_depth` and the
//! `BKE_image_format_from_imbuf` ftype mapping.
//!
//! `ibuf` state is injected as plain fields; `ImBuf` write-back
//! stays with the caller.

use blenders_dna::scene_types::{
    ImageFormatData, R_IMF_CHAN_DEPTH_10, R_IMF_CHAN_DEPTH_12, R_IMF_CHAN_DEPTH_16,
    R_IMF_CHAN_DEPTH_32, R_IMF_CHAN_DEPTH_8, R_IMF_IMTYPE_AVIF, R_IMF_IMTYPE_BMP,
    R_IMF_IMTYPE_CINEON, R_IMF_IMTYPE_DDS, R_IMF_IMTYPE_DPX, R_IMF_IMTYPE_FFMPEG,
    R_IMF_IMTYPE_INVALID, R_IMF_IMTYPE_IRIS, R_IMF_IMTYPE_IRIZ, R_IMF_IMTYPE_JP2,
    R_IMF_IMTYPE_JPEG90, R_IMF_IMTYPE_MULTILAYER, R_IMF_IMTYPE_OPENEXR, R_IMF_IMTYPE_PNG,
    R_IMF_IMTYPE_PSD, R_IMF_IMTYPE_RADHDR, R_IMF_IMTYPE_RAWTGA, R_IMF_IMTYPE_TARGA,
    R_IMF_IMTYPE_TIFF, R_IMF_IMTYPE_WEBP, R_IMF_JP2_CODEC_J2K, R_IMF_JP2_CODEC_JP2,
    R_IMF_TIFF_CODEC_DEFLATE, R_IMF_TIFF_CODEC_LZW, R_IMF_TIFF_CODEC_NONE,
    R_IMF_TIFF_CODEC_PACKBITS,
};

/// `BKE_IMAGE_PATH_EXT_MAX`.
pub const BKE_IMAGE_PATH_EXT_MAX: usize = 8;

/// `IMA_CHAN_FLAG_*`.
pub const IMA_CHAN_FLAG_RGB: i8 = 1 << 0;
/// `IMA_CHAN_FLAG_RGBA`.
pub const IMA_CHAN_FLAG_RGBA: i8 = 1 << 1;
/// `IMA_CHAN_FLAG_BW`.
pub const IMA_CHAN_FLAG_BW: i8 = 1 << 2;

/// `BKE_image_format_init` — `Default` covers it; re-exported for parity.
pub fn bke_image_format_init(imf: &mut ImageFormatData) {
    *imf = ImageFormatData::default();
}

/// `BKE_image_format_copy`.
pub fn bke_image_format_copy(dst: &mut ImageFormatData, src: &ImageFormatData) {
    *dst = src.clone();
}

/// `BKE_imtype_is_image`.
pub fn bke_imtype_is_image(imtype: i8) -> bool {
    !bke_imtype_is_multi_layer_image(imtype) && !bke_imtype_is_movie(imtype)
}

/// `BKE_imtype_is_multi_layer_image`.
pub fn bke_imtype_is_multi_layer_image(imtype: i8) -> bool {
    imtype == R_IMF_IMTYPE_MULTILAYER
}

/// `BKE_imtype_is_movie`.
pub fn bke_imtype_is_movie(imtype: i8) -> bool {
    imtype == R_IMF_IMTYPE_FFMPEG
}

/// `BKE_imtype_supports_compress`.
pub fn bke_imtype_supports_compress(imtype: i8) -> bool {
    matches!(imtype, R_IMF_IMTYPE_PNG)
}

/// `BKE_imtype_supports_quality`.
pub fn bke_imtype_supports_quality(imtype: i8) -> bool {
    matches!(
        imtype,
        R_IMF_IMTYPE_JPEG90 | R_IMF_IMTYPE_JP2 | R_IMF_IMTYPE_WEBP | R_IMF_IMTYPE_AVIF
    )
}

/// `BKE_imtype_requires_linear_float`.
pub fn bke_imtype_requires_linear_float(imtype: i8) -> bool {
    matches!(
        imtype,
        R_IMF_IMTYPE_CINEON
            | R_IMF_IMTYPE_DPX
            | R_IMF_IMTYPE_RADHDR
            | R_IMF_IMTYPE_OPENEXR
            | R_IMF_IMTYPE_MULTILAYER
    )
}

/// `BKE_imtype_valid_channels`.
pub fn bke_imtype_valid_channels(imtype: i8) -> i8 {
    let mut chan_flag = IMA_CHAN_FLAG_RGB;
    match imtype {
        R_IMF_IMTYPE_BMP
        | R_IMF_IMTYPE_TARGA
        | R_IMF_IMTYPE_RAWTGA
        | R_IMF_IMTYPE_IRIS
        | R_IMF_IMTYPE_PNG
        | R_IMF_IMTYPE_TIFF
        | R_IMF_IMTYPE_OPENEXR
        | R_IMF_IMTYPE_MULTILAYER
        | R_IMF_IMTYPE_DDS
        | R_IMF_IMTYPE_JP2
        | R_IMF_IMTYPE_DPX
        | R_IMF_IMTYPE_WEBP
        | R_IMF_IMTYPE_AVIF => chan_flag |= IMA_CHAN_FLAG_RGBA,
        _ => {}
    }
    match imtype {
        R_IMF_IMTYPE_BMP
        | R_IMF_IMTYPE_PNG
        | R_IMF_IMTYPE_JPEG90
        | R_IMF_IMTYPE_TARGA
        | R_IMF_IMTYPE_RAWTGA
        | R_IMF_IMTYPE_TIFF
        | R_IMF_IMTYPE_IRIS
        | R_IMF_IMTYPE_OPENEXR
        | R_IMF_IMTYPE_JP2
        | R_IMF_IMTYPE_AVIF => chan_flag |= IMA_CHAN_FLAG_BW,
        _ => {}
    }
    chan_flag
}

/// `BKE_imtype_valid_depths` — bitmask of `R_IMF_CHAN_DEPTH_*`.
pub fn bke_imtype_valid_depths(imtype: i8) -> i8 {
    match imtype {
        R_IMF_IMTYPE_TIFF => {
            R_IMF_CHAN_DEPTH_8 | R_IMF_CHAN_DEPTH_16
        }
        R_IMF_IMTYPE_OPENEXR => {
            R_IMF_CHAN_DEPTH_16 | R_IMF_CHAN_DEPTH_32
        }
        R_IMF_IMTYPE_MULTILAYER => R_IMF_CHAN_DEPTH_16 | R_IMF_CHAN_DEPTH_32,
        R_IMF_IMTYPE_DPX => {
            R_IMF_CHAN_DEPTH_8 | R_IMF_CHAN_DEPTH_10 | R_IMF_CHAN_DEPTH_12 | R_IMF_CHAN_DEPTH_16
        }
        R_IMF_IMTYPE_CINEON => R_IMF_CHAN_DEPTH_10,
        R_IMF_IMTYPE_JP2 => R_IMF_CHAN_DEPTH_8 | R_IMF_CHAN_DEPTH_12 | R_IMF_CHAN_DEPTH_16,
        R_IMF_IMTYPE_PNG => R_IMF_CHAN_DEPTH_8 | R_IMF_CHAN_DEPTH_16,
        R_IMF_IMTYPE_AVIF => R_IMF_CHAN_DEPTH_8 | R_IMF_CHAN_DEPTH_10 | R_IMF_CHAN_DEPTH_12,
        _ => R_IMF_CHAN_DEPTH_8,
    }
}

/// `BKE_imtype_valid_depths_with_video` — `video_bit_depths` injects
/// `MOV_codec_valid_bit_depths` for scene FFMPEG output.
pub fn bke_imtype_valid_depths_with_video(
    imtype: i8,
    video_bit_depths: &dyn Fn() -> i8,
) -> i8 {
    let mut depths = bke_imtype_valid_depths(imtype);
    if imtype == R_IMF_IMTYPE_FFMPEG {
        depths |= video_bit_depths();
    }
    depths
}

/// `BKE_imtype_first_valid_depth`.
pub fn bke_imtype_first_valid_depth(valid_depths: i8) -> i8 {
    for d in [
        R_IMF_CHAN_DEPTH_32,
        R_IMF_CHAN_DEPTH_16,
        R_IMF_CHAN_DEPTH_12,
        R_IMF_CHAN_DEPTH_10,
        R_IMF_CHAN_DEPTH_8,
    ] {
        if valid_depths & d != 0 {
            return d;
        }
    }
    R_IMF_CHAN_DEPTH_8
}

/// `BKE_imtype_from_arg` — identifier → imtype.
pub fn bke_imtype_from_arg(imtype_arg: &str) -> i8 {
    match imtype_arg {
        "TGA" => R_IMF_IMTYPE_TARGA,
        "IRIS" => R_IMF_IMTYPE_IRIS,
        "JPEG" => R_IMF_IMTYPE_JPEG90,
        "RAWTGA" => R_IMF_IMTYPE_RAWTGA,
        "PNG" => R_IMF_IMTYPE_PNG,
        "BMP" => R_IMF_IMTYPE_BMP,
        "HDR" => R_IMF_IMTYPE_RADHDR,
        "TIFF" => R_IMF_IMTYPE_TIFF,
        "OPEN_EXR" | "EXR" | "MULTILAYER" | "OPEN_EXR_MULTILAYER" => {
            if imtype_arg == "EXR" || imtype_arg == "OPEN_EXR" {
                R_IMF_IMTYPE_OPENEXR
            } else {
                R_IMF_IMTYPE_MULTILAYER
            }
        }
        "FFMPEG" => R_IMF_IMTYPE_FFMPEG,
        "CINEON" => R_IMF_IMTYPE_CINEON,
        "DPX" => R_IMF_IMTYPE_DPX,
        "JP2" => R_IMF_IMTYPE_JP2,
        "WEBP" => R_IMF_IMTYPE_WEBP,
        "AVIF" => R_IMF_IMTYPE_AVIF,
        _ => R_IMF_IMTYPE_INVALID,
    }
}

/// `image_path_ext_from_imformat_impl`.
pub fn image_path_ext_from_imformat_impl(
    imtype: i8,
    im_format: Option<&ImageFormatData>,
) -> Vec<&'static str> {
    let mut ext = Vec::new();
    if imtype == R_IMF_IMTYPE_IRIS {
        ext.push(".rgb");
    } else if imtype == R_IMF_IMTYPE_IRIZ {
        ext.push(".rgb");
    } else if imtype == R_IMF_IMTYPE_RADHDR {
        ext.push(".hdr");
    } else if imtype == R_IMF_IMTYPE_PNG || imtype == R_IMF_IMTYPE_FFMPEG {
        ext.push(".png");
    } else if imtype == R_IMF_IMTYPE_DDS {
        ext.push(".dds");
    } else if imtype == R_IMF_IMTYPE_TARGA || imtype == R_IMF_IMTYPE_RAWTGA {
        ext.push(".tga");
    } else if imtype == R_IMF_IMTYPE_BMP {
        ext.push(".bmp");
    } else if imtype == R_IMF_IMTYPE_TIFF {
        ext.push(".tif");
        ext.push(".tiff");
    } else if imtype == R_IMF_IMTYPE_PSD {
        ext.push(".psd");
    } else if imtype == R_IMF_IMTYPE_OPENEXR || imtype == R_IMF_IMTYPE_MULTILAYER {
        ext.push(".exr");
    } else if imtype == R_IMF_IMTYPE_CINEON {
        ext.push(".cin");
    } else if imtype == R_IMF_IMTYPE_DPX {
        ext.push(".dpx");
    } else if imtype == R_IMF_IMTYPE_JP2 {
        match im_format {
            Some(f) => {
                if f.jp2_codec == R_IMF_JP2_CODEC_JP2 {
                    ext.push(".jp2");
                } else if f.jp2_codec == R_IMF_JP2_CODEC_J2K {
                    ext.push(".j2c");
                }
            }
            None => ext.push(".jp2"),
        }
    } else if imtype == R_IMF_IMTYPE_WEBP {
        ext.push(".webp");
    } else if imtype == R_IMF_IMTYPE_AVIF {
        ext.push(".avif");
    } else {
        ext.push(".jpg");
        ext.push(".jpeg");
    }
    debug_assert!(ext.len() < BKE_IMAGE_PATH_EXT_MAX);
    ext
}

/// `BKE_image_path_ext_from_imformat`.
pub fn bke_image_path_ext_from_imformat(im_format: &ImageFormatData) -> Vec<&'static str> {
    image_path_ext_from_imformat_impl(im_format.imtype, Some(im_format))
}

/// `BKE_image_path_ext_from_imtype`.
pub fn bke_image_path_ext_from_imtype(imtype: i8) -> Vec<&'static str> {
    image_path_ext_from_imformat_impl(imtype, None)
}

/// `imb_ext_image` — extensions recognized as images.
pub const IMB_EXT_IMAGE: &[&str] = &[
    ".png", ".tga", ".bmp", ".jpg", ".jpeg", ".tif", ".tiff", ".exr", ".cin", ".dpx",
    ".hdr", ".rgb", ".psd", ".dds", ".webp", ".jp2", ".j2c", ".avif",
];

fn ext_of(filepath: &str) -> Option<&str> {
    let name = filepath.rsplit('/').next()?;
    let dot = name.rfind('.')?;
    Some(&name[dot..])
}

/// `do_ensure_image_extension` — returns true when the path changed.
pub fn do_ensure_image_extension(
    filepath: &mut String,
    imtype: i8,
    im_format: Option<&ImageFormatData>,
) -> bool {
    let ext_array = image_path_ext_from_imformat_impl(imtype, im_format);
    if ext_array.is_empty() {
        return false;
    }
    let cur = ext_of(filepath).map(|e| e.to_lowercase());
    if let Some(c) = &cur {
        if ext_array.iter().any(|e| e == c) {
            return false;
        }
    }
    if let Some(c) = &cur {
        if IMB_EXT_IMAGE.iter().any(|e| e == c) {
            /* `BLI_path_extension_replace`. */
            let len = filepath.len() - c.len();
            filepath.truncate(len);
            filepath.push_str(ext_array[0]);
            return true;
        }
    }
    /* `BLI_path_extension_ensure`. */
    filepath.push_str(ext_array[0]);
    true
}

/// `BKE_image_path_ext_from_imformat_ensure`.
pub fn bke_image_path_ext_from_imformat_ensure(
    filepath: &mut String,
    im_format: &ImageFormatData,
) -> bool {
    do_ensure_image_extension(filepath, im_format.imtype, Some(im_format))
}

/// `BKE_image_path_ext_from_imtype_ensure`.
pub fn bke_image_path_ext_from_imtype_ensure(filepath: &mut String, imtype: i8) -> bool {
    do_ensure_image_extension(filepath, imtype, None)
}

/// ImBuf ftype/option summary the C reads (`ibuf->ftype`, `foptions`).
#[derive(Clone, Copy, Default)]
pub struct ImBufFormatSummary {
    /// `ibuf->ftype` — `eImbFileType`.
    pub ftype: i32,
    /// `foptions.flag`.
    pub flag: i32,
    /// `foptions.quality`.
    pub quality: i8,
    /// `foptions.compress`.
    pub compress: i8,
    /// `ibuf->float_data() != nullptr`.
    pub has_float_data: bool,
}

/// `eImbFileType` values used here.
pub const IMB_FTYPE_IRIS: i32 = 1;
/// `IMB_FTYPE_RADHDR`.
pub const IMB_FTYPE_RADHDR: i32 = 2;
/// `IMB_FTYPE_PNG`.
pub const IMB_FTYPE_PNG: i32 = 3;
/// `IMB_FTYPE_DDS`.
pub const IMB_FTYPE_DDS: i32 = 4;
/// `IMB_FTYPE_BMP`.
pub const IMB_FTYPE_BMP: i32 = 5;
/// `IMB_FTYPE_TIF`.
pub const IMB_FTYPE_TIF: i32 = 6;
/// `IMB_FTYPE_OPENEXR`.
pub const IMB_FTYPE_OPENEXR: i32 = 7;
/// `IMB_FTYPE_JP2`.
pub const IMB_FTYPE_JP2: i32 = 8;
/// `IMB_FTYPE_TGA`.
pub const IMB_FTYPE_TGA: i32 = 9;
/// `IMB_FTYPE_CINEON`.
pub const IMB_FTYPE_CINEON: i32 = 10;
/// `IMB_FTYPE_DPX`.
pub const IMB_FTYPE_DPX: i32 = 11;
/// `IMB_FTYPE_WEBP`.
pub const IMB_FTYPE_WEBP: i32 = 12;
/// `IMB_FTYPE_AVIF`.
pub const IMB_FTYPE_AVIF: i32 = 13;
/// `IMB_FTYPE_JPG`.
pub const IMB_FTYPE_JPG: i32 = 32;

/// `foptions.flag` bits.
pub const PNG_16BIT: i32 = 1 << 0;
/// `TIF_16BIT`.
pub const TIF_16BIT: i32 = 1 << 1;
/// `TIF_COMPRESS_NONE`.
pub const TIF_COMPRESS_NONE: i32 = 1 << 2;
/// `TIF_COMPRESS_DEFLATE`.
pub const TIF_COMPRESS_DEFLATE: i32 = 1 << 3;
/// `TIF_COMPRESS_LZW`.
pub const TIF_COMPRESS_LZW: i32 = 1 << 4;
/// `TIF_COMPRESS_PACKBITS`.
pub const TIF_COMPRESS_PACKBITS: i32 = 1 << 5;
/// `OPENEXR_HALF`.
pub const OPENEXR_HALF: i32 = 1 << 6;
/// `OPENEXR_CODEC_MASK`.
pub const OPENEXR_CODEC_MASK: i32 = 0xFF << 8;
/// `J2K_16BIT`.
pub const J2K_16BIT: i32 = 1 << 16;
/// `J2K_32BIT`.
pub const J2K_32BIT: i32 = 1 << 17;

/// `imtype_best_depth`.
pub fn imtype_best_depth(imbuf: &ImBufFormatSummary, imtype: i8) -> i8 {
    let depth_ok = bke_imtype_valid_depths(imtype);
    if imbuf.has_float_data {
        if depth_ok & R_IMF_CHAN_DEPTH_32 != 0 {
            return R_IMF_CHAN_DEPTH_32;
        }
        if depth_ok & R_IMF_CHAN_DEPTH_16 != 0 {
            return R_IMF_CHAN_DEPTH_16;
        }
        if depth_ok & R_IMF_CHAN_DEPTH_12 != 0 {
            return R_IMF_CHAN_DEPTH_12;
        }
        return R_IMF_CHAN_DEPTH_8;
    }
    for d in [
        R_IMF_CHAN_DEPTH_8,
        R_IMF_CHAN_DEPTH_12,
        R_IMF_CHAN_DEPTH_16,
        R_IMF_CHAN_DEPTH_32,
    ] {
        if depth_ok & d != 0 {
            return d;
        }
    }
    R_IMF_CHAN_DEPTH_8
}

/// `BKE_image_format_from_imbuf`.
pub fn bke_image_format_from_imbuf(
    im_format: &mut ImageFormatData,
    imbuf: &ImBufFormatSummary,
) {
    let custom_flags = imbuf.flag;
    bke_image_format_init(im_format);
    im_format.media_type = 0; /* MEDIA_TYPE_IMAGE */
    let mut is_depth_set = false;

    match imbuf.ftype {
        IMB_FTYPE_IRIS => im_format.imtype = R_IMF_IMTYPE_IRIS,
        IMB_FTYPE_RADHDR => im_format.imtype = R_IMF_IMTYPE_RADHDR,
        IMB_FTYPE_PNG => {
            im_format.imtype = R_IMF_IMTYPE_PNG;
            if custom_flags & PNG_16BIT != 0 {
                im_format.depth = R_IMF_CHAN_DEPTH_16;
                is_depth_set = true;
            }
            im_format.compress = imbuf.compress;
        }
        IMB_FTYPE_DDS => im_format.imtype = R_IMF_IMTYPE_DDS,
        IMB_FTYPE_BMP => im_format.imtype = R_IMF_IMTYPE_BMP,
        IMB_FTYPE_TIF => {
            im_format.imtype = R_IMF_IMTYPE_TIFF;
            if custom_flags & TIF_16BIT != 0 {
                im_format.depth = R_IMF_CHAN_DEPTH_16;
                is_depth_set = true;
            }
            for (flag, codec) in [
                (TIF_COMPRESS_NONE, R_IMF_TIFF_CODEC_NONE),
                (TIF_COMPRESS_DEFLATE, R_IMF_TIFF_CODEC_DEFLATE),
                (TIF_COMPRESS_LZW, R_IMF_TIFF_CODEC_LZW),
                (TIF_COMPRESS_PACKBITS, R_IMF_TIFF_CODEC_PACKBITS),
            ] {
                if custom_flags & flag != 0 {
                    im_format.tiff_codec = codec;
                }
            }
        }
        IMB_FTYPE_OPENEXR => {
            im_format.imtype = R_IMF_IMTYPE_OPENEXR;
            if custom_flags & OPENEXR_HALF != 0 {
                im_format.depth = R_IMF_CHAN_DEPTH_16;
                is_depth_set = true;
            }
            im_format.exr_codec = ((custom_flags & OPENEXR_CODEC_MASK) >> 8) as i8;
        }
        IMB_FTYPE_JP2 => {
            im_format.imtype = R_IMF_IMTYPE_JP2;
            if custom_flags & J2K_16BIT != 0 {
                im_format.depth = R_IMF_CHAN_DEPTH_16;
                is_depth_set = true;
            } else if custom_flags & J2K_32BIT != 0 {
                im_format.depth = R_IMF_CHAN_DEPTH_32;
                is_depth_set = true;
            }
            im_format.jp2_codec = R_IMF_JP2_CODEC_JP2;
        }
        IMB_FTYPE_TGA => im_format.imtype = R_IMF_IMTYPE_TARGA,
        IMB_FTYPE_CINEON => im_format.imtype = R_IMF_IMTYPE_CINEON,
        IMB_FTYPE_DPX => im_format.imtype = R_IMF_IMTYPE_DPX,
        IMB_FTYPE_WEBP => {
            im_format.imtype = R_IMF_IMTYPE_WEBP;
            im_format.quality = imbuf.quality;
        }
        IMB_FTYPE_AVIF => {
            im_format.imtype = R_IMF_IMTYPE_AVIF;
            im_format.quality = imbuf.quality;
        }
        _ => im_format.imtype = R_IMF_IMTYPE_JPEG90,
    }

    if !is_depth_set {
        im_format.depth = imtype_best_depth(imbuf, im_format.imtype);
    }
}

/// `BKE_image_format_is_byte`.
pub fn bke_image_format_is_byte(imf: &ImageFormatData) -> bool {
    imf.depth <= R_IMF_CHAN_DEPTH_8
}
