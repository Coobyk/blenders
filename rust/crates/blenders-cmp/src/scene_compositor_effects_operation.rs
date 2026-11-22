//! Port of `intern/scene_compositor_effects_operation.cc`
//! — scene-level compositor effects entry.

/// `SceneCompositorEffectsOperation` verbatim.
#[derive(Default)]
pub struct SceneCompositorEffectsOperation {
    /// `scene`/`depsgraph`/`camera` handles.
    pub scene: usize,
    pub camera: usize,
}
