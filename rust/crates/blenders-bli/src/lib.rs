//! Port of `source/blender/blenlib` (non-math modules).

pub mod bits;
pub mod hash_mm2a;
pub mod hash_md5;
pub mod hash_mm3;
#[cfg(test)]
mod tests {
    #[test]
    fn md5_vector() {
        let d = crate::hash_md5::hash_md5_buffer(b"abc");
        assert_eq!(crate::hash_md5::hash_md5_to_hexdigest(&d), "900150983cd24fb0d6963f7d28e17f72");
    }
}
pub mod endian;
pub mod ghash;
pub mod string;
pub mod string_utf8;
pub mod ghash_utils;
pub mod gsqueue;
pub mod hash_tables;
pub mod time;
pub mod dial_2d;
pub mod dynstr;
pub mod session_uid;
