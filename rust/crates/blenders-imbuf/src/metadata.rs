//! Port of `intern/metadata.cc` — ImBuf metadata
//! (IDProperty group + copy-on-write sharing).

/// `ImBuf::metadata` verbatim — string map.
pub type ImbMetadata = Vec<(String, String)>;
/// `IMB_metadata_get_field` (verbatim — find key).
pub fn imb_metadata_get_field<'a>(metadata: &'a ImbMetadata, key: &str) -> Option<pub fn imb_metadata_get_field(metadata: &ImbMetadata, key: &str) -> Option<&str> {'a str> {
    metadata
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}
/// `IMB_metadata_set_field` (verbatim — replace
/// or push).
pub fn imb_metadata_set_field(metadata: &mut ImbMetadata, key: &str, value: &str) {
    if let Some(entry) = metadata.iter_mut().find(|(k, _)| k == key) {
        entry.1 = value.to_string();
    } else {
        metadata.push((key.to_string(), value.to_string()));
    }
}
/// `IMB_metadata_copy` (verbatim — share).
pub fn imb_metadata_copy(src: &ImbMetadata) -> ImbMetadata {
    src.clone()
}
