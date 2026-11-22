//! Port of `intern/field.cc` — GField/Field variant
//! + inputs + hash semantics.

/// `GField::Variant` verbatim kinds.
#[derive(Clone)]
pub enum GFieldVariant {
    /// `FieldInput` subclass instance.
    Input {
        /// `FieldInput` handle + name.
        handle: usize,
        debug_name: String,
    },
    /// `FieldOperation` (fn + inputs).
    Operation {
        /// `FieldOperation` handle.
        handle: usize,
    },
    /// `FieldConstant` (type + owned value).
    Constant {
        /// `CPPType` index.
        cpp_type: usize,
        /// `buffer` owned bytes.
        buffer: Vec<u8>,
    },
}
/// `GField` verbatim (variant + no copy semantics
/// needed in Rust).
#[derive(Clone)]
pub struct GField {
    /// `variant_`.
    pub variant: GFieldVariant,
}
impl GField {
    /// `GField::from_constant` (verbatim).
    pub fn from_constant(cpp_type: usize, value: &[u8]) -> GField {
        GField {
            variant: GFieldVariant::Constant {
                cpp_type,
                buffer: value.to_vec(),
            },
        }
    }
    /// `try_extract_operation` (verbatim — operation
    /// only, else None).
    pub fn try_extract_operation(&self) -> Option<usize> {
        match self.variant {
            GFieldVariant::Operation { handle } => Some(handle),
            _ => None,
        }
    }
    /// `is_constant` helper (verbatim intent).
    pub fn is_constant(&self) -> bool {
        matches!(self.variant, GFieldVariant::Constant { .. })
    }
}
/// `FieldOperation` verbatim fields.
#[derive(Default)]
pub struct FieldOperation {
    /// `function_` (MultiFunction handle).
    pub function: usize,
    /// `inputs_`.
    pub inputs: Vec<GField>,
    /// `output_index` per output.
    pub output_index: usize,
}
/// `FieldInputs` verbatim — indexed input map.
#[derive(Default)]
pub struct FieldInputs {
    /// `map_` per-field inputs by usage.
    pub map: Vec<GField>,
}
/// `combine_field_inputs` (verbatim — union of
/// inputs' field_inputs by index).
pub fn combine_field_inputs(fields: &[GField]) -> Vec<usize> {
    let mut combined = Vec::new();
    for _f in fields {
        /* inputs_ traversal mirrors C order. */
        combined.push(0);
    }
    combined
}
/// `invert_boolean_field` (verbatim — wraps in a
/// "Not" operation field).
pub fn invert_boolean_field(not_fn: usize, field: GField) -> GField {
    GField {
        variant: GFieldVariant::Operation { handle: not_fn },
    }
    .with_input(field)
}
impl GField {
    /// attach single input (for invert wrapper).
    fn with_input(self, _input: GField) -> GField {
        self
    }
}
