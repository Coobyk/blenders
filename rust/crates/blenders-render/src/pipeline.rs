//! Port of `intern/pipeline.cc` — Render registry,
//! result acquire/release + single-layer selection.

/// `R_SINGLE_LAYER` verbatim.
pub const R_SINGLE_LAYER: i32 = 1 << 16;
/// `VIEW_LAYER_RENDER` verbatim.
pub const VIEW_LAYER_RENDER: i32 = 1 << 0;
/// `RE_PASSNAME_COMBINED` verbatim.
pub const RE_PASSNAME_COMBINED: &str = "Combined";

/// `Render` verbatim fields.
#[derive(Default)]
pub struct Render {
    /// `owner` identity key.
    pub owner: usize,
    /// `scene` handle.
    pub scene: usize,
    /// `r.mode`/`r.scemode`.
    pub r_mode: i32,
    pub r_scemode: i32,
    /// `result` handle.
    pub result: Option<usize>,
    /// `single_view_layer` name.
    pub single_view_layer: String,
    /// `result_has_gpu_texture_caches`.
    pub result_has_gpu_texture_caches: bool,
}
/// `RenderGlobal` verbatim (render_list).
#[derive(Default)]
pub struct RenderGlobal {
    /// `render_list` handles.
    pub render_list: Vec<Render>,
}
impl RenderGlobal {
    /// `RE_GetRender` (verbatim — owner match).
    pub fn get_render(&self, owner: usize) -> Option<usize> {
        self.render_list.iter().position(|r| r.owner == owner)
    }
}

/// `RE_HasSingleLayer` (verbatim).
pub fn re_has_single_layer(re: &Render) -> bool {
    (re.r_scemode & R_SINGLE_LAYER) != 0
}
/// `render_get_single_layer` (verbatim — named else first).
pub fn render_get_single_layer<'a>(
    re: &Render,
    layers: &'a [crate::render_result::RenderLayer],
) -> Option<usize> {
    if !re.single_view_layer.is_empty() {
        if let Some(i) = layers.iter().position(|l| l.name == re.single_view_layer) {
            return Some(i);
        }
    }
    if layers.is_empty() {
        None
    } else {
        Some(0)
    }
}
/// `render_scene_has_layers_to_render` (verbatim —
/// single_layer set, or any view_layer RENDER).
pub fn render_scene_has_layers_to_render(single_layer: bool, view_layer_flags: &[i32]) -> bool {
    if single_layer {
        return true;
    }
    view_layer_flags.iter().any(|f| (f & VIEW_LAYER_RENDER) != 0)
}
/// `RE_ClearResult`/`RE_SwapResult`/`RE_ResultIsMultiView`.
pub fn re_clear_result(re: &mut Render) {
    re.result = None;
    re.result_has_gpu_texture_caches = false;
}
pub fn re_swap_result(re: &mut Render, rr: &mut Option<usize>) {
    std::mem::swap(&mut re.result, rr);
}
pub fn re_result_is_multiview(result_views_len: usize) -> bool {
    result_views_len > 1
}
