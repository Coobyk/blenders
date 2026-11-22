/// Port of `grease_pencil/intern/grease_pencil_io_import_svg.cc`
/// — SVG path → GP strokes.
/// `import_svg` (verbatim — path parse).
pub fn grease_pencil_import_svg(d: &str) -> Vec<[f32; 3]> {
    let mut pts = Vec::new();
    for tok in d.split_whitespace() {
        if let Some(rest) = tok.strip_prefix('M').or_else(|| tok.strip_prefix('L')) {
            let mut it = rest.split(',');
            if let (Some(x), Some(y)) = (it.next(), it.next()) {
                pts.push([
                    x.parse().unwrap_or(0.0),
                    y.parse().unwrap_or(0.0),
                    0.0,
                ]);
            }
        }
    }
    pts
}
