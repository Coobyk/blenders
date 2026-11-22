/// Port of `io_wavefront_obj/importer/obj_import_file_reader.cc` —
/// `OBJFileReader` — tokenize `v`/`vn`/`vt`/
/// `f`/`o`/`g`/`s`/`usemtl`/`mtllib`.
/// `OBJFaceCorner` — a face corner's
/// (v, vt, vn) 1-based indices.
#[derive(Clone, Copy, Default)]
pub struct OBJFaceCorner {
    pub vertex_index: i32,
    pub uv_vertex_index: i32,
    pub vertex_normal_index: i32,
}
/// `OBJMesh` — parsed mesh sections.
#[derive(Default)]
pub struct OBJMesh {
    /// Vertices.
    pub vertices: Vec<[f32; 3]>,
    /// UVs.
    pub uv_vertices: Vec<[f32; 2]>,
    /// Normals.
    pub vertex_normals: Vec<[f32; 3]>,
    /// Faces (per corner lists).
    pub faces: Vec<Vec<OBJFaceCorner>>,
    /// `usemtl` material index per face.
    pub face_material_index: Vec<i16>,
    /// `g` object name per face.
    pub face_object_name: Vec<i32>,
}
/// `OBJFileReader::read` (verbatim — stream
/// tokens; `v x y z`, `vt u v`, `vn x y z`,
/// `f v/vt/vn ...`, `usemtl name`, `mtllib`,
/// `o name`, `g name`, `s on/off`).
pub fn read(src: &str) -> OBJMesh {
    let mut m = OBJMesh::default();
    let mut cur_mtl = 0i16;
    let mut mtls: Vec<String> = Vec::new();
    let mut cur_obj = 0i32;
    let mut objs: Vec<String> = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        let mut it = t.split_whitespace();
        let kw = match it.next() {
            Some(k) => k,
            None => continue,
        };
        match kw {
            "v" => {
                let x: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                m.vertices.push([x, y, z]);
            }
            "vt" => {
                let u: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let v: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                m.uv_vertices.push([u, v]);
            }
            "vn" => {
                let x: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                m.vertex_normals.push([x, y, z]);
            }
            "f" => {
                let mut face = Vec::new();
                for tok in it {
                    let mut c = OBJFaceCorner::default();
                    let parts: Vec<&str> = tok.split('/').collect();
                    c.vertex_index = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                    if parts.len() > 1 && !parts[1].is_empty() {
                        c.uv_vertex_index = parts[1].parse().unwrap_or(0);
                    }
                    if parts.len() > 2 && !parts[2].is_empty() {
                        c.vertex_normal_index = parts[2].parse().unwrap_or(0);
                    }
                    face.push(c);
                }
                m.faces.push(face);
                m.face_material_index.push(cur_mtl);
                m.face_object_name.push(cur_obj);
            }
            "usemtl" => {
                let name = it.next().unwrap_or("").to_string();
                cur_mtl = mtls
                    .iter()
                    .position(|n| *n == name)
                    .map(|i| i as i16)
                    .unwrap_or_else(|| {
                        mtls.push(name);
                        (mtls.len() - 1) as i16
                    });
            }
            "o" | "g" => {
                let name = it.next().unwrap_or("").to_string();
                cur_obj = objs
                    .iter()
                    .position(|n| *n == name)
                    .map(|i| i as i32)
                    .unwrap_or_else(|| {
                        objs.push(name);
                        (objs.len() - 1) as i32
                    });
            }
            _ => {}
        }
    }
    m
}
