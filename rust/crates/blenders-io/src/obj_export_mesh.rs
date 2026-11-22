/// Port of `wavefront_obj/exporter/obj_export_mesh.cc`
/// — mesh → OBJ face/vertex emission.
/// `write_mesh` (verbatim — gather faces
/// then per-vertex index map).
pub fn obj_export_mesh(
    positions: &[[f32; 3]],
    corner_verts: &[i32],
    poly_offsets: &[(i32, i32)],
) -> (Vec<String>, Vec<String>) {
    let mut verts = Vec::new();
    let mut faces = Vec::new();
    for p in positions {
        verts.push(format!("v {} {} {}", p[0], p[1], p[2]));
    }
    for &(s, e) in poly_offsets {
        let mut f = String::from("f");
        for i in s..e {
            f.push_str(&format!(" {}", corner_verts[i as usize] + 1));
        }
        faces.push(f);
    }
    (verts, faces)
}
