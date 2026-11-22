//! Port of `intern/bundle_item.cc` — bundle
//! item value variant (buffer/value/image/single).

/// `BundleItemValue` verbatim kinds.
pub enum BundleItemValueKind {
    /// `Int`/`Float`/`Bool`/`Vector`/`Color`/`Rotation`.
    Int(i32),
    Float(f32),
    Bool(bool),
    Vector([f32; 3]),
    Color([f32; 4]),
    /// `Buffer`/`Image`/`Menu`/`Socket` handles.
    Buffer(usize),
    Image(usize),
    Menu(String),
    Socket(usize),
}
/// `BundleItem` verbatim.
pub struct BundleItem {
    /// `key`.
    pub key: String,
    /// `value`.
    pub value: BundleItemValueKind,
}
/// `bundle_item_type_name` (verbatim — for UI).
pub fn bundle_item_type_name(v: &BundleItemValueKind) -> &'static str {
    match v {
        BundleItemValueKind::Int(_) => "Int",
        BundleItemValueKind::Float(_) => "Float",
        BundleItemValueKind::Bool(_) => "Bool",
        BundleItemValueKind::Vector(_) => "Vector",
        BundleItemValueKind::Color(_) => "Color",
        BundleItemValueKind::Buffer(_) => "Buffer",
        BundleItemValueKind::Image(_) => "Image",
        BundleItemValueKind::Menu(_) => "Menu",
        BundleItemValueKind::Socket(_) => "Socket",
    }
}
