//! Port of `intern/multi_function.cc` —
//! MultiFunction signature + call math.

/// `MultiFunction::Signature` verbatim.
#[derive(Default)]
pub struct Signature {
    /// `param_names`.
    pub param_names: Vec<String>,
    /// `param_types` (in/out per param).
    pub param_types: Vec<ParamType>,
}
/// `ParamType` verbatim.
#[derive(Clone, Copy, PartialEq)]
pub enum ParamType {
    /// `ParamType::ForSingleInput`/`ForVectorInput`.
    ForSingleInput,
    ForVectorInput,
    /// `ForSingleOutput`/`ForMutable`/`ForVectorOutput`.
    ForSingleOutput,
    ForMutable,
    ForVectorOutput,
}
/// `MultiFunction` verbatim fields.
#[derive(Default)]
pub struct MultiFunction {
    /// `signature_`.
    pub signature: Signature,
    /// `name_`.
    pub name: String,
}
impl MultiFunction {
    /// `param_amount`.
    pub fn param_amount(&self) -> usize {
        self.signature.param_types.len()
    }
}
