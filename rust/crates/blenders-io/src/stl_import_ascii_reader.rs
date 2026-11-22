/// Port of `io_stl/importer/stl_import_ascii_reader.cc` —
/// `STLASCIIReader` — parse ASCII STL
/// `solid`/`facet`/`vertex`/`end` tokens.
/// `STLASCIIReader::read` (verbatim —
/// line-tokenized parser: `solid` start,
/// per `facet`→`outer loop`→3 `vertex`
///→`endloop`→`endfacet`, `endsolid`).
pub struct STLASCIIReader<'a> {
    /// `m_file` — input text.
    src: &'a str,
    /// `m_pos` — read offset.
    pos: usize,
}
impl<'a> STLASCIIReader<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }
    /// `read` → tris `[(normal, v0, v1, v2)]`.
    pub fn read(&mut self) -> Vec<([f32; 3], [[f32; 3]; 3])> {
        let mut tris = Vec::new();
        let lines = self.src.lines();
        let mut facet_normal = [0.0f32; 3];
        let mut verts: Vec<[f32; 3]> = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.starts_with("facet") {
                // `facet normal nx ny nz`
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    facet_normal = [
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(0.0),
                        parts[4].parse().unwrap_or(0.0),
                    ];
                }
                verts.clear();
            } else if line.starts_with("vertex") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    verts.push([
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(0.0),
                    ]);
                }
            } else if line.starts_with("endfacet") {
                if verts.len() == 3 {
                    tris.push((facet_normal, [verts[0], verts[1], verts[2]]));
                }
                verts.clear();
            }
            self.pos += line.len() + 1;
        }
        tris
    }
}
/// `is_stl_ascii` — check the file's first
/// non-whitespace token is `solid`.
pub fn is_stl_ascii(src: &str) -> bool {
    src.trim_start().starts_with("solid")
}
