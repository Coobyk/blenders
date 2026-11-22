/// Port of `wavefront_obj/importer/obj_import_file_reader.cc`
/// — OBJ line tokenizer + dispatch.
/// `OBJKeywords` (verbatim — line kind enum).
#[derive(Clone, Copy, PartialEq)]
pub enum ObjLineKind {
    Vertex,
    Normal,
    Uv,
    Face,
    Object,
    Group,
    Usemtl,
    Mtllib,
    Smoothing,
    Other,
}
/// `read_obj_line` (verbatim — first-token
/// dispatch).
pub fn obj_line_kind(line: &str) -> ObjLineKind {
    match line.split_whitespace().next().unwrap_or("") {
        "v" => ObjLineKind::Vertex,
        "vn" => ObjLineKind::Normal,
        "vt" => ObjLineKind::Uv,
        "f" => ObjLineKind::Face,
        "o" => ObjLineKind::Object,
        "g" => ObjLineKind::Group,
        "usemtl" => ObjLineKind::Usemtl,
        "mtllib" => ObjLineKind::Mtllib,
        "s" => ObjLineKind::Smoothing,
        _ => ObjLineKind::Other,
    }
}
/// `parse_vertex` (verbatim — "v x y z [w]").
pub fn obj_parse_vertex(line: &str) -> Option<[f32; 3]> {
    let mut it = line.split_whitespace().skip(1);
    let x: f32 = it.next()?.parse().ok()?;
    let y: f32 = it.next()?.parse().ok()?;
    let z: f32 = it.next()?.parse().ok()?;
    Some([x, y, z])
}
/// `parse_face_index` (verbatim — "v/vt/vn"
/// three 1-based indices).
pub fn obj_parse_face_index(tok: &str) -> (i32, i32, i32) {
    let mut it = tok.split('/');
    let v: i32 = it.next().unwrap_or("0").parse().unwrap_or(0);
    let vt: i32 = it.next().and_then(|s| if s.is_empty() { None } else { s.parse().ok() }).unwrap_or(0);
    let vn: i32 = it.next().and_then(|s| if s.is_empty() { None } else { s.parse().ok() }).unwrap_or(0);
    (v, vt, vn)
}
