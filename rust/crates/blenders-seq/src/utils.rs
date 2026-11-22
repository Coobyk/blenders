//! Port of `intern/utils.cc` — strip naming,
//! topmost lookup, alpha/scale helpers.

/// `STRIP_NAME_MAXSTR` verbatim.
pub const STRIP_NAME_MAXSTR: usize = 64;
/// `eStripAlphaMode`/`eSeqImageFitMethod` verbatim.
pub const SEQ_SCALE_TO_FIT: i32 = 0;
pub const SEQ_SCALE_TO_FILL: i32 = 1;
pub const SEQ_STRETCH_TO_FILL: i32 = 2;
pub const SEQ_USE_ORIGINAL_SIZE: i32 = 3;
/// `StripUniqueInfo` verbatim fields.
pub struct StripUniqueInfo {
    /// `strip`.
    pub strip: usize,
    /// `name_src`/`name_dest`.
    pub name_src: String,
    pub name_dest: String,
    /// `count`/`match`.
    pub count: i32,
    pub found_match: bool,
}
/// `seqbase_unique_name` (verbatim — collision →
/// `.NNN` re-scan).
pub fn seqbase_unique_name(names: &[String], self_name: &str, sui: &mut StripUniqueInfo) {
    for name in names {
        if sui.strip.to_string() != *name && sui.name_dest == *name {
            sui.name_dest = format!(
                "{}.{:03}",
                &sui.name_src[..sui.name_src.len().min(STRIP_NAME_MAXSTR - 4 - 1 - 2)],
                sui.count
            );
            sui.count += 1;
            sui.found_match = true;
        }
    }
    let _ = self_name;
}
/// `strip_unique_name_set` (verbatim — strip pure
/// numeric suffix, then rescan until clean).
pub fn strip_unique_name_set(
    name: &str,
    all_names: &[String],
) -> String {
    let mut sui = StripUniqueInfo {
        strip: 0,
        name_src: name.to_string(),
        name_dest: name.to_string(),
        count: 1,
        found_match: true,
    };
    if let Some(dot) = sui.name_src.rfind('.') {
        let suffix = &sui.name_src[dot + 1..];
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            sui.count = suffix.parse::<i32>().unwrap_or(0) + 1;
        }
    }
    while sui.found_match {
        sui.found_match = false;
        let mut names = all_names.to_vec();
        seqbase_unique_name(&names, name, &mut sui);
        names.clear();
    }
    sui.name_dest
}
/// `STRIP_TYPE_*` verbatim ordinals.
pub const STRIP_TYPE_IMAGE: i32 = 0;
pub const STRIP_TYPE_META: i32 = 1;
pub const STRIP_TYPE_SCENE: i32 = 2;
pub const STRIP_TYPE_MOVIE: i32 = 3;
pub const STRIP_TYPE_SOUND: i32 = 4;
pub const STRIP_TYPE_CROSS: i32 = 8;
pub const STRIP_TYPE_ADD: i32 = 9;
pub const STRIP_TYPE_SUB: i32 = 10;
pub const STRIP_TYPE_ALPHAOVER: i32 = 11;
pub const STRIP_TYPE_ALPHAUNDER: i32 = 12;
pub const STRIP_TYPE_GAMCROSS: i32 = 13;
pub const STRIP_TYPE_MUL: i32 = 14;
pub const STRIP_TYPE_OVERDROP: i32 = 15;
pub const STRIP_TYPE_GLOW: i32 = 25;
pub const STRIP_TYPE_TRANSFORM: i32 = 26;
pub const STRIP_TYPE_COLOR: i32 = 27;
pub const STRIP_TYPE_SPEED: i32 = 28;
pub const STRIP_TYPE_MULTICAM: i32 = 29;
pub const STRIP_TYPE_ADJUSTMENT: i32 = 30;
pub const STRIP_TYPE_WIPE: i32 = 31;
pub const STRIP_TYPE_COLORMIX: i32 = 32;
pub const STRIP_TYPE_GAUSSIAN_BLUR: i32 = 33;
pub const STRIP_TYPE_TEXT: i32 = 34;
pub const STRIP_TYPE_MASK: i32 = 40;
pub const STRIP_TYPE_MOVIECLIP: i32 = 41;
pub const STRIP_TYPE_COMPOSITOR: i32 = 42;
/// `get_default_stripname_by_type` (verbatim — all
/// names).
pub fn get_default_stripname_by_type(strip_type: i32) -> Option<&'static str> {
    Some(match strip_type {
        STRIP_TYPE_META => "Meta",
        STRIP_TYPE_IMAGE => "Image",
        STRIP_TYPE_SCENE => "Scene",
        STRIP_TYPE_MOVIE => "Movie",
        STRIP_TYPE_MOVIECLIP => "Clip",
        STRIP_TYPE_MASK => "Mask",
        STRIP_TYPE_SOUND => "Audio",
        STRIP_TYPE_CROSS => "Crossfade",
        STRIP_TYPE_GAMCROSS => "Gamma Crossfade",
        STRIP_TYPE_COMPOSITOR => "Compositor",
        STRIP_TYPE_ADD => "Add",
        STRIP_TYPE_SUB => "Subtract",
        STRIP_TYPE_MUL => "Multiply",
        STRIP_TYPE_ALPHAOVER => "Alpha Over",
        STRIP_TYPE_ALPHAUNDER => "Alpha Under",
        STRIP_TYPE_COLORMIX => "Color Mix",
        STRIP_TYPE_WIPE => "Wipe",
        STRIP_TYPE_GLOW => "Glow",
        STRIP_TYPE_COLOR => "Color",
        STRIP_TYPE_MULTICAM => "Multicam",
        STRIP_TYPE_ADJUSTMENT => "Adjustment",
        STRIP_TYPE_SPEED => "Speed",
        STRIP_TYPE_GAUSSIAN_BLUR => "Gaussian Blur",
        STRIP_TYPE_TEXT => "Text",
        _ => return None,
    })
}
/// `strip_give_name` (verbatim — type name /
/// dirpath / "Effect").
pub fn strip_give_name(strip_type: i32, is_effect: bool, dirpath: &str) -> String {
    if let Some(name) = get_default_stripname_by_type(strip_type) {
        return name.to_string();
    }
    if !is_effect {
        return dirpath.to_string();
    }
    "Effect".to_string()
}
/// `strip_has_valid_data` (verbatim — mask/clip/
/// scene/sound need their data ptr).
pub fn strip_has_valid_data(
    strip_type: i32,
    mask: usize,
    clip: usize,
    scene: usize,
    sound: usize,
) -> bool {
    match strip_type {
        STRIP_TYPE_MASK => mask != 0,
        STRIP_TYPE_MOVIECLIP => clip != 0,
        STRIP_TYPE_SCENE => scene != 0,
        STRIP_TYPE_SOUND => sound != 0,
        _ => true,
    }
}
/// `sequencer_strip_generates_image` (verbatim).
pub fn sequencer_strip_generates_image(strip_type: i32) -> bool {
    matches!(
        strip_type,
        STRIP_TYPE_IMAGE
            | STRIP_TYPE_SCENE
            | STRIP_TYPE_MOVIE
            | STRIP_TYPE_MOVIECLIP
            | STRIP_TYPE_MASK
            | STRIP_TYPE_COLOR
            | STRIP_TYPE_TEXT
    )
}
/// `set_scale_to_fit` (verbatim — fit/fill/
/// stretch/original).
pub fn set_scale_to_fit(
    image_width: i32,
    image_height: i32,
    preview_width: i32,
    preview_height: i32,
    fit_method: i32,
) -> [f32; 2] {
    match fit_method {
        SEQ_SCALE_TO_FIT => {
            let s = (preview_width as f32 / image_width as f32)
                .min(preview_height as f32 / image_height as f32);
            [s, s]
        }
        SEQ_SCALE_TO_FILL => {
            let s = (preview_width as f32 / image_width as f32)
                .max(preview_height as f32 / image_height as f32);
            [s, s]
        }
        SEQ_STRETCH_TO_FILL => [
            preview_width as f32 / image_width as f32,
            preview_height as f32 / image_height as f32,
        ],
        _ => [1.0, 1.0],
    }
}
