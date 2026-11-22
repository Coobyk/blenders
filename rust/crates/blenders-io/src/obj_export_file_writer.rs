/// Port of `io_wavefront_obj/exporter/obj_export_file_writer.cc` —
/// `OBJWriter` — emit `v`/`vn`/`vt`/`f`/`usemtl`.
/// `OBJWriter::write` (verbatim —
/// per-object: `o name`, v/vn/vt lists,
/// `usemtl` + `s off` + `f` indices
/// (1-based)).
pub struct OBJWriter;
impl OBJWriter {
    /// Emit OBJ text for a mesh.
    pub fn write(
        verts: &[[f32; 3]],
        uvs: &[[f32; 2]],
        normals: &[[f32; 3]],
        faces: &[Vec<(i32, i32, i32)>], // (v, vt, vn) 0-based
        object_name: &str,
        material_name: Option<&str>,
    ) -> String {
        let mut s = String::new();
        s.push_str(&format!("o {}\n", object_name));
        for v in verts {
            s.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
        }
        for uv in uvs {
            s.push_str(&format!("vt {:.6} {:.6}\n", uv[0], uv[1]));
        }
        for n in normals {
            s.push_str(&format!("vn {:.4} {:.4} {:.4}\n", n[0], n[1], n[2]));
        }
        if let Some(m) = material_name {
            s.push_str(&format!("usemtl {}\n", m));
        }
        s.push_str("s off\n");
        for f in faces {
            s.push('f');
            for (v, vt, vn) in f {
                s.push_str(&format!(" {}/{}/{}", v + 1, vt + 1, vn + 1));
            }
            s.push('\n');
        }
        s
    }
}
