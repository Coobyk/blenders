//! Port of `intern/scene_strip.cc` — scene strip
//! rendering entry points + input flag decode.

/// `seq_scene_strip_render` parameter flags.
pub const SEQ_SCENE_STRIP_FLAG_NONE: i32 = 0;
/// `scene_strip_render` hook args (verbatim —
//! camera choice + scene override).
#[derive(Default)]
pub struct SceneStripRenderArgs {
    /// `scene` strip source scene handle.
    pub scene: usize,
    /// `use_gpencil`/etc render flags.
    pub render_flags: i32,
}
/// `scene_strip_get` — strip→scene resolution
/// (verbatim — strip->scene or default).
pub fn scene_strip_get(strip_scene: Option<usize>, default_scene: usize) -> usize {
    strip_scene.unwrap_or(default_scene)
}
