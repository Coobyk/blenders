/// Port of `usd/intern/usd_writer_abstract.cc`
/// — base USD writer (time/frame sampling).
/// `USDAbstractWriter` (verbatim — frame write
/// dispatch).
pub struct UsdAbstractWriter {
    /// `frame_has_been_written`.
    pub frame_written: bool,
}
