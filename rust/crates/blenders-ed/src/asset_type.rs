/// Port of `asset/intern/asset_type.cc` —
/// asset type <-> icon/label map.
/// `asset_type_name` (verbatim — enum →
/// label).
pub fn asset_type_name(ty: i32) -> &'static str {
    match ty {
        0 => "Action",
        1 => "Collection",
        2 => "Material",
        3 => "Object",
        4 => "Scene",
        5 => "World",
        _ => "Asset",
    }
}
