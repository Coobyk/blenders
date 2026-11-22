//! Port of `intern/util.cc` — misc helpers
//! (buffer dimensions, rect conversions).

/// `imb_check_py_socket_writing`/`imb_addrectImBuf`
//! shells (verbatim names).
pub fn imb_check_dimensions(x: i32, y: i32) -> bool {
    x > 0 && y > 0
}
/// `imb_rect_from_float`/`imb_float_from_rect` are
//! in `conversion.rs`.
