/// Port of `intern/draw_attributes.cc` —
/// per-object custom draw attributes table.
/// `DRWAttr` (verbatim — name+layer+index).
pub struct DrwAttr {
    /// `name`.
    pub name: String,
    /// `domain`.
    pub domain: i32,
    /// `layer`.
    pub layer: i32,
}
/// `drw_attr_table` (verbatim — lookup by
/// name).
pub fn drw_attr_table_lookup(attrs: &[DrwAttr], name: &str) -> Option<i32> {
    attrs.iter().position(|a| a.name == name).map(|i| i as i32)
}
