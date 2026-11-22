/// Port of `io_stl/exporter/stl_export_writer.cc` —
/// `STLWriter` — emit ASCII/binary STL.
/// `STLWriter::write` (verbatim — binary:
/// 80B header + count + per-tri normal +
/// verts + attr; ASCII: `solid`/`facet`/
/// `vertex`/`endsolid` tokens).
pub struct STLWriter;
impl STLWriter {
    /// ASCII emit.
    pub fn write_ascii(
        tris: &[([f32; 3], [[f32; 3]; 3])],
        name: &str,
    ) -> String {
        let mut s = String::from("solid ");
        s.push_str(name);
        s.push('\n');
        for (n, v) in tris {
            s.push_str(&format!(
                "  facet normal {:e} {:e} {:e}\n",
                n[0], n[1], n[2]
            ));
            s.push_str("    outer loop\n");
            for vert in v {
                s.push_str(&format!(
                    "      vertex {:e} {:e} {:e}\n",
                    vert[0], vert[1], vert[2]
                ));
            }
            s.push_str("    endloop\n  endfacet\n");
        }
        s.push_str("endsolid ");
        s.push_str(name);
        s.push('\n');
        s
    }
    /// Binary emit.
    pub fn write_binary(tris: &[([f32; 3], [[f32; 3]; 3])]) -> Vec<u8> {
        let mut out = vec![0u8; 80];
        out.extend_from_slice(&(tris.len() as u32).to_le_bytes());
        for (n, v) in tris {
            for f in [n[0], n[1], n[2]] {
                out.extend_from_slice(&f.to_le_bytes());
            }
            for vert in v {
                for f in vert {
                    out.extend_from_slice(&f.to_le_bytes());
                }
            }
            out.extend_from_slice(&0u16.to_le_bytes());
        }
        out
    }
}
