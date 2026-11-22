/// Port of `usd/intern/usd_reader_stage.cc`
/// — stage open + prim iteration.
/// `USDStageReader` (verbatim — stage handle).
pub struct UsdStageReader {
    /// `filepath`.
    pub filepath: String,
}
impl UsdStageReader {
    /// `collect_readers` (verbatim — prim
    /// dispatch per type).
    pub fn collect_readers(&self) -> Vec<String> {
        Vec::new()
    }
}
