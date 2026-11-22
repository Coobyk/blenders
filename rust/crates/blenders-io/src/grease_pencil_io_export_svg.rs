/// Port of `grease_pencil/intern/grease_pencil_io_export_svg.cc`
/// — GP → SVG export.
/// `export_svg` (verbatim — path elements).
pub fn grease_pencil_export_svg(strokes: &[Vec<[f32; 3]>]) -> String {
    let mut s = String::from("<svg>\n");
    for stroke in strokes {
        s.push_str("<path d=\"");
        for (i, p) in stroke.iter().enumerate() {
            if i == 0 {
                s.push_str(&format!("M {} {}", p[0], p[1]));
            } else {
                s.push_str(&format!(" L {} {}", p[0], p[1]));
            }
        }
        s.push_str("\"/>\n");
    }
    s.push_str("</svg>\n");
    s
}
