//! Port of `intern/render.cc` — RenderData +
//! transform/crop/scale-factor + early-out.

/// `SEQ_DEINTERLACE`/`SEQ_FLIPX`/`SEQ_FLIPY`/`SEQ_MAKE_FLOAT`/`SEQ_MUTE`.
pub const SEQ_DEINTERLACE: i32 = 1 << 8;
pub const SEQ_MAKE_FLOAT: i32 = 1 << 9;
pub const SEQ_MUTE: i32 = 1 << 2;
use crate::strip_transform::{SEQ_FLIPX, SEQ_FLIPY};
/// `STRIP_BLEND_REPLACE`/`STRIP_BLEND_SCREEN` etc.
pub const STRIP_BLEND_REPLACE: i32 = 9;
/// `SEQ_RENDER_SIZE_SCENE` verbatim.
pub const SEQ_RENDER_SIZE_SCENE: i32 = 3;
/// `GpuContextState` verbatim ordinals.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GpuContextState {
    AlreadyActive,
    Success,
    Unsupported,
}
/// `StripEarlyOut` verbatim ordinals.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StripEarlyOut {
    DoEffect,
    UseInput1,
    UseInput2,
    NoInput,
}
/// `RenderData` verbatim subset.
#[derive(Default)]
pub struct RenderData {
    /// `preview_render_size`/`scene_render_scale`/
    /// `use_proxies`/`motion_blur`/`rectx`/`recty`.
    pub preview_render_size: i32,
    pub scene_render_scale: i32,
    pub use_proxies: bool,
    pub rectx: i32,
    pub recty: i32,
    /// `skip_cache`/`is_prefetch_render`/`view_id`.
    pub skip_cache: bool,
    pub view_id: i32,
}
/// `StripTransform`/`StripCrop` verbatim fields.
#[derive(Clone, Default)]
pub struct StripTransform {
    /// `xofs`/`yofs`/`scale_x`/`scale_y`/`rotation`/`origin`.
    pub xofs: f32,
    pub yofs: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub origin: [f32; 2],
}
#[derive(Clone, Default)]
pub struct StripCrop {
    /// `left`/`right`/`top`/`bottom`.
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}
/// `StripScreenQuad` verbatim — 4 corners.
#[derive(Clone, Copy, Default)]
pub struct StripScreenQuad {
    /// `v0..v3` corners (x,y).
    pub v: [[f32; 2]; 4],
}
/// `sequencer_use_transform` (verbatim).
pub fn sequencer_use_transform(t: &StripTransform) -> bool {
    t.xofs != 0.0 || t.yofs != 0.0 || t.scale_x != 1.0 || t.scale_y != 1.0 || t.rotation != 0.0
}
/// `sequencer_use_crop` (verbatim).
pub fn sequencer_use_crop(c: &StripCrop) -> bool {
    c.left > 0 || c.right > 0 || c.top > 0 || c.bottom > 0
}
/// `seq_input_have_to_preprocess` (verbatim —
/// deinterlace/flip/float/crop/transform/mul/sat/
/// modifiers).
pub fn seq_input_have_to_preprocess(
    flag: i32,
    has_crop: bool,
    has_transform: bool,
    mul: f32,
    sat: f32,
    has_modifiers: bool,
    blend_mode: i32,
    blend_opacity: f32,
) -> bool {
    if (flag & (SEQ_DEINTERLACE | SEQ_FLIPX | SEQ_FLIPY | SEQ_MAKE_FLOAT)) != 0
        || has_crop
        || has_transform
    {
        return true;
    }
    let mut mul = mul;
    if blend_mode == STRIP_BLEND_REPLACE {
        mul *= blend_opacity / 100.0;
    }
    if mul != 1.0 {
        return true;
    }
    if sat != 1.0 {
        return true;
    }
    if has_modifiers {
        return true;
    }
    false
}
/// `seq_need_scale_to_render_size` (verbatim —
/// effects except color, mask, meta, scene-strips
/// → false; proxy → false).
pub fn seq_need_scale_to_render_size(
    is_proxy_image: bool,
    is_effect: bool,
    strip_type: i32,
    flag: i32,
) -> bool {
    if is_proxy_image {
        return false;
    }
    if (is_effect && strip_type != crate::utils::STRIP_TYPE_COLOR)
        || strip_type == crate::utils::STRIP_TYPE_MASK
        || strip_type == crate::utils::STRIP_TYPE_META
        || (strip_type == crate::utils::STRIP_TYPE_SCENE
            && (flag & crate::strip_relations::SEQ_SCENE_STRIPS) != 0)
    {
        return false;
    }
    true
}
/// `seq_must_swap_input_in_blend_mode` (verbatim).
pub fn seq_must_swap_input_in_blend_mode(blend_mode: i32) -> bool {
    matches!(
        blend_mode,
        crate::sequencer::STRIP_BLEND_ALPHAOVER | crate::sequencer::STRIP_BLEND_ALPHAUNDER
    )
}
/// `strip_get_early_out_for_blend_mode` (verbatim —
/// swap UseInput1/2 when alpha over/under).
pub fn strip_get_early_out_for_blend_mode(
    blend_mode: i32,
    early_out: StripEarlyOut,
) -> StripEarlyOut {
    if matches!(early_out, StripEarlyOut::DoEffect | StripEarlyOut::NoInput) {
        return early_out;
    }
    if seq_must_swap_input_in_blend_mode(blend_mode) {
        match early_out {
            StripEarlyOut::UseInput2 => return StripEarlyOut::UseInput1,
            StripEarlyOut::UseInput1 => return StripEarlyOut::UseInput2,
            _ => {}
        }
    }
    early_out
}
/// `render_is_muted` (verbatim — SEQ_MUTE or
/// channel muted).
pub fn render_is_muted(flag: i32, channel_muted: bool) -> bool {
    (flag & SEQ_MUTE) != 0 || channel_muted
}
/// `get_render_scale_factor` (verbatim — SCENE→
/// scene%/100, else proxy scale).
pub fn get_render_scale_factor(render_size: i32, scene_render_scale: i32) -> f32 {
    if render_size == SEQ_RENDER_SIZE_SCENE {
        scene_render_scale as f32 / 100.0
    } else {
        crate::proxy::rendersize_to_scale_factor(render_size)
    }
}
/// `sequencer_image_crop_init` (verbatim —
/// scale crop sides, rctf left..x-right ×
/// bottom..y-top).
pub fn sequencer_image_crop_init(
    c: &StripCrop,
    in_x: i32,
    in_y: i32,
    crop_scale_factor: f32,
) -> [f32; 4] {
    let left = c.left as f32 * crop_scale_factor;
    let right = c.right as f32 * crop_scale_factor;
    let top = c.top as f32 * crop_scale_factor;
    let bottom = c.bottom as f32 * crop_scale_factor;
    [left, in_x as f32 - right, bottom, in_y as f32 - top]
}
/// `multiply_ibuf` factor (verbatim — fmul for
/// non-alpha, 1.0 for alpha channel when
/// !multiply_alpha).
pub fn multiply_ibuf_factor(fmul: f32, multiply_alpha: bool, is_alpha_channel: bool) -> f32 {
    if is_alpha_channel && !multiply_alpha {
        1.0
    } else {
        fmul
    }
}
