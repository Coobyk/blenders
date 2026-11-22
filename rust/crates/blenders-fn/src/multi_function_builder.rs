//! Port of `intern/multi_function_builder.cc` —
//! SI/SO/MF signature builder helpers.

/// `MFParamType` query (verbatim — is input/output).
pub fn mf_param_type_is_input(t: crate::multi_function::ParamType) -> bool {
    matches!(
        t,
        crate::multi_function::ParamType::ForSingleInput
            | crate::multi_function::ParamType::ForVectorInput
    )
}
/// `mf_param_type_is_output`.
pub fn mf_param_type_is_output(t: crate::multi_function::ParamType) -> bool {
    !mf_param_type_is_input(t)
}
