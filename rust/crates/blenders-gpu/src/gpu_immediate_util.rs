/// Port of `intern/gpu_immediate_util.cc`
/// — immDraw helpers (rect/box/circle).
/// `immRectf` (verbatim — quad verts).
pub fn imm_rectf(pos: &mut Vec<[f32; 3]>, x1: f32, y1: f32, x2: f32, y2: f32) {
    pos.extend_from_slice(&[
        [x1, y1, 0.0],
        [x2, y1, 0.0],
        [x2, y2, 0.0],
        [x1, y1, 0.0],
        [x2, y2, 0.0],
        [x1, y2, 0.0],
    ]);
}
/// `imm_draw_circle` (verbatim — fan verts).
pub fn imm_draw_circle(cx: f32, cy: f32, r: f32, segments: i32) -> Vec<[f32; 3]> {
    let mut out = Vec::with_capacity(segments as usize + 1);
    for i in 0..=segments {
        let t = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        out.push([cx + r * t.cos(), cy + r * t.sin(), 0.0]);
    }
    out
}
