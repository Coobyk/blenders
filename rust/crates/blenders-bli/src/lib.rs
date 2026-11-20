//! Port of `source/blender/blenlib` (non-math modules).

pub mod hash_mm2a;
pub mod hash_md5;
pub mod hash_mm3;
#[test]
fn md5_vector() {
    let d = crate::hash_md5::hash_md5_buffer(b"abc");
    assert_eq!(crate::hash_md5::hash_md5_to_hexdigest(&d), "900150983cd24fb0d6963f7d28e17f72");
}
