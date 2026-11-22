//! Port of `intern/imageprocess.cc` — under/over
//! sampling + blit/polypoint hooks.

/// `imageprocess` semantics (verbatim — hires
/// buffers + row ops). Shell — actual pixel ops
/// live in transform/scaling/rectop.
pub const IMAGEPROCESS_UNDER_SAMPLE: i32 = 0;
pub const IMAGEPROCESS_OVER_SAMPLE: i32 = 1;
