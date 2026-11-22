//! Port of `intern/closure.cc` — Closure
//! (single-vertex source: buffer or value).

/// `Closure` verbatim fields.
#[derive(Default)]
pub struct Closure {
    /// `type` (ConstantValue | Buffer | Image).
    pub type_: i32,
    /// `value` (constant) or buffer bytes.
    pub value: Vec<f32>,
    /// `width`/`height`/`elem_size`.
    pub width: i32,
    pub height: i32,
    pub elem_size: i32,
}
/// `ClosureType` verbatim ordinals.
pub const CLOSURE_TYPE_CONSTANT_VALUE: i32 = 0;
pub const CLOSURE_TYPE_BUFFER: i32 = 1;
pub const CLOSURE_TYPE_IMAGE: i32 = 2;
