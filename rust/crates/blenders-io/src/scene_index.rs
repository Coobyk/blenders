/// Port of `usd/hydra/scene_index.cc` —
/// Hydra scene-index root.
/// `UsdStageSceneIndex` (verbatim — prim
/// registry).
pub struct UsdStageSceneIndex {
    /// `prims` path→type.
    pub prims: Vec<(String, String)>,
}
