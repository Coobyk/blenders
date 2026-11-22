//! Port of `intern/conversion_operation.cc` —
//! implicit type conversion op.

/// `ConversionOperation` verbatim.
#[derive(Default)]
pub struct ConversionOperation {
    /// `input_type`/`expected_type` ResultTypes.
    pub input_type: i32,
    pub expected_type: i32,
}
/// `construct_if_needed` (verbatim — only when
/// types differ and conversion not skipped).
pub fn construct_if_needed(
    skip_type_conversion: bool,
    result_type: i32,
    expected_type: i32,
) -> Option<ConversionOperation> {
    if skip_type_conversion {
        return None;
    }
    if result_type != expected_type {
        return Some(ConversionOperation {
            input_type: result_type,
            expected_type,
        });
    }
    None
}
/// `execute` decision (verbatim — convertible →
/// convert else default fill).
pub enum ConversionPlan {
    /// not convertible → `allocate_default_remaining_outputs`.
    AllocateDefault,
    /// single value path.
    SingleValue,
    /// GPU texture path.
    Gpu,
    /// CPU path.
    Cpu,
}
/// `execute` (verbatim — is_convertible gate →
/// single/gpu/cpu).
pub fn conversion_operation_plan(
    is_convertible: bool,
    is_single_value: bool,
    use_gpu: bool,
) -> ConversionPlan {
    if !is_convertible {
        return ConversionPlan::AllocateDefault;
    }
    if is_single_value {
        return ConversionPlan::SingleValue;
    }
    if use_gpu {
        ConversionPlan::Gpu
    } else {
        ConversionPlan::Cpu
    }
}
/// shader name (verbatim —
/// `compositor_convert_{input}_to_{output}`).
pub fn conversion_shader_name(input_type_name: &str, output_type_name: &str) -> String {
    format!("compositor_convert_{}_to_{}", input_type_name, output_type_name)
}
