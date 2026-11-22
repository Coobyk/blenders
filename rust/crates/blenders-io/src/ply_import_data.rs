/// Port of `ply/importer/ply_import_data.cc`
/// — per-element PLY property decode.
/// `ply_read_element` (verbatim — typed
/// scalar read).
pub fn ply_read_scalar(buf: &[u8], ty: &str) -> f32 {
    match ty {
        "float" | "float32" => f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
        "uchar" | "uint8" => buf[0] as f32,
        "char" | "int8" => buf[0] as i8 as f32,
        _ => 0.0,
    }
}
