/// Port of `intern/dna_parse.cc` — C
/// struct parser (tokenize `DNA_*.h` →
/// member list + sizes).
/// `DnaParse` (verbatim — struct/field
/// tables).
pub struct DnaParse {
    /// `structs` (name → fields).
    pub structs: Vec<DnaStruct>,
}
/// `DnaStruct` (verbatim).
pub struct DnaStruct {
    /// `name`.
    pub name: String,
    /// `fields` (type, name, array dims).
    pub fields: Vec<DnaField>,
}
/// `DnaField` (verbatim).
pub struct DnaField {
    /// `type` (struct or builtin).
    pub ty: String,
    /// `name`.
    pub name: String,
    /// `array_dims` (`[n][m]`).
    pub array_dims: Vec<i32>,
    /// `is_pointer`.
    pub is_pointer: bool,
}
