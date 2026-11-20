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
pub mod heap;
pub mod heap_simple;
pub mod linklist;
pub mod listbase;
pub mod rand;
pub mod rct;
pub mod stack;
pub mod memarena;
pub mod timecode;
pub mod uuid;
pub mod array_utils;
pub mod fnmatch;
pub mod mempool;
pub mod ordered_edge;
pub mod index_range;
pub mod string_ref;
pub mod ustring;
pub mod lasso_2d;
pub mod quadric;
pub mod jitter_2d;
