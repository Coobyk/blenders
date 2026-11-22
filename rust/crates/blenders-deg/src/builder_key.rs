//! Port of `intern/builder/deg_builder_key.cc` — key
/// `identifier()` strings.

/// `TimeSourceKey::identifier` (verbatim).
pub fn time_source_key_identifier() -> &'static str {
    "TimeSourceKey"
}

/// `ComponentKey::identifier` (verbatim).
pub fn component_key_identifier(
    id_name: &str,
    node_type_as_string: &str,
    name: &str,
) -> String {
    let mut result = String::from("ComponentKey(");
    result += id_name;
    result += ", ";
    result += node_type_as_string;
    if !name.is_empty() {
        result += ", '";
        result += name;
        result += "'";
    }
    result += ")";
    result
}

/// `OperationKey::identifier` (verbatim).
pub fn operation_key_identifier(
    component_type_as_string: &str,
    component_name: &str,
    opcode_as_string: &str,
    name: &str,
) -> String {
    let mut result = String::from("OperationKey(");
    result += "type: ";
    result += component_type_as_string;
    result += ", component name: '";
    result += component_name;
    result += "'";
    result += ", operation code: ";
    result += opcode_as_string;
    if !name.is_empty() {
        result += ", '";
        result += name;
        result += "'";
    }
    result += ")";
    result
}

/// `RNAPathKey` — resolved path pointer/prop.
/// `PORT_PENDING`: `RNA_path_resolve_full` — RNA module.
pub struct RnaPathKey {
    /// `id` arena index.
    pub id: usize,
    /// `ptr` resolved pointer (arena index).
    pub ptr: usize,
    /// `prop` resolved property (arena index).
    pub prop: usize,
    /// `source`.
    pub source: i32,
}

/// `RNAPathKey::identifier` (verbatim; `prop_name`
/// is `RNA_property_identifier`).
pub fn rna_path_key_identifier(id_name: &str, prop_name: &str) -> String {
    format!("RnaPathKey(id: {}, prop: '{}')", id_name, prop_name)
}
