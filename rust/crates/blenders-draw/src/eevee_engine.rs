/// Port of `engines/eevee/eevee_engine.cc`
/// — EEVEE-next DrawEngine.
/// `EeveeEngine` (verbatim — pipeline list).
pub struct EeveeEngine {
    /// `pipelines` (surface/volume/
    /// shadow/composite).
    pub pipelines: Vec<usize>,
}
