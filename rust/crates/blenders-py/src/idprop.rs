/// Port of `generic/idprop.cc` — IDProperty
/// (typed custom property container).
/// `eIDPropertyType` (verbatim).
pub const IDP_STRING: i32 = 0;
pub const IDP_INT: i32 = 1;
pub const IDP_FLOAT: i32 = 2;
pub const IDP_ARRAY: i32 = 3;
pub const IDP_GROUP: i32 = 4;
pub const IDP_ID: i32 = 5;
pub const IDP_DOUBLE: i32 = 6;
pub const IDP_IDPARRAY: i32 = 7;
/// `IDProperty` (verbatim — name+type+data).
pub struct IdProperty {
    /// `name`.
    pub name: String,
    /// `type`.
    pub prop_type: i32,
    /// `flag`.
    pub flag: i32,
}
