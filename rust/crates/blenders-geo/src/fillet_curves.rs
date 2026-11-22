/// Port of `intern/fillet_curves.cc` —
/// replace poly corners by fillet arcs.

/// `fillet_curves` (verbatim — corner arc
// from radius + angle).
pub fn fillet_curves_point(
    prev: [f32; 3],
    cur: [f32; 3],
    next: [f32; 3],
    radius: f32,
    count: i32,
) -> Vec<[f32; 3]> {
    let v1 = [prev[0] - cur[0], prev[1] - cur[1], prev[2] - cur[2]];
    let v2 = [next[0] - cur[0], next[1] - cur[1], next[2] - cur[2]];
    let l1 = (v1[0].powi(2) + v1[1].powi(2) + v1[2].powi(2)).sqrt();
    let l2 = (v2[0].powi(2) + v2[1].powi(2) + v2[2].powi(2)).sqrt();
    if l1 == 0.0 || l2 == 0.0 {
        return vec![cur];
    }
    let u1 = [v1[0] / l1, v1[1] / l1, v1[2] / l1];
    let u2 = [v2[0] / l2, v2[1] / l2, v2[2] / l2];
    let cos_a = (u1[0] * u2[0] + u1[1] * u2[1] + u1[2] * u2[2]).clamp(-1.0, 1.0);
    let angle = cos_a.acos();
    if angle < 1e-4 {
        return vec![cur];
    }
    // Arc from u1 toward u2 around circle center.
    let dist = radius / (angle / 2.0).tan();
    let p1 = [cur[0] + u1[0] * dist, cur[1] + u1[1] * dist, cur[2] + u1[2] * dist];
    let p2 = [cur[0] + u2[0] * dist, cur[1] + u2[1] * dist, cur[2] + u2[2] * dist];
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        let t = i as f32 / (count - 1) as f32;
        out.push([
            p1[0] + (p2[0] - p1[0]) * t,
            p1[1] + (p2[1] - p1[1]) * t,
            p1[2] + (p2[2] - p1[2]) * t,
        ]);
    }
    out
}
