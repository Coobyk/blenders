//! Port of `intern/annotations_geom.cc` — stroke fill triangulation + UV
//! updates (`BKE_annotation_stroke_geometry_update`).

use blenders_bli::polyfill_2d::polyfill_calc_pub;
use blenders_dna::gpencil_legacy_types::{bGPDspoint, bGPDstroke, bGPDtriangle};
use blenders_math::vector::{Vec2, Vec3};

/// `stroke_2d_flat` — project stroke points onto the plane through pt0
/// spanned by `locx = pt1 - pt0` and `normal x locx`; returns the 2D coords
/// and the signed area direction (`r_direction` in C — derived implicitly by
/// the caller via `polyfill_calc_pub`'s sign).
fn stroke_2d_flat(points: &[bGPDspoint]) -> (Vec<[f32; 2]>, f32) {
    debug_assert!(points.len() >= 2);
    let totpoints = points.len();
    let pt0 = &points[0];
    let pt1 = &points[1];
    let pt3 = &points[(totpoints as f32 * 0.75) as usize];
    let p0 = Vec3::new(pt0.x, pt0.y, pt0.z);
    let p1 = Vec3::new(pt1.x, pt1.y, pt1.z);
    let p3 = Vec3::new(pt3.x, pt3.y, pt3.z);
    let locx = p1 - p0;
    /* `pt3` gets scaled to 1/1000th when only 2 points exist. */
    let v3 = if totpoints == 2 { p3 * 0.001 } else { p3 };
    let loc3 = v3 - p0;
    let normal = locx.cross(loc3);
    let locy = normal.cross(locx);
    let locx = locx.normalized();
    let locy = locy.normalized();
    let mut points2d = vec![[0.0f32; 2]; totpoints];
    let pt_last = &points[totpoints - 1];
    let tmp = Vec3::new(pt_last.x, pt_last.y, pt_last.z) - p0;
    points2d[totpoints - 1][0] = tmp.dot(locx);
    points2d[totpoints - 1][1] = tmp.dot(locy);
    /* `cross` — winding sum for the direction (C only accumulates it for
     * polyfill's sign arg). */
    let mut cross = 0.0f32;
    let mut co_prev = points2d[totpoints - 1];
    for i in 0..totpoints - 1 {
        let pt = &points[i];
        let loc = Vec3::new(pt.x, pt.y, pt.z) - p0;
        points2d[i][0] = loc.dot(locx);
        points2d[i][1] = loc.dot(locy);
        let co_curr = points2d[i];
        cross += (co_curr[0] - co_prev[0]) * (co_curr[1] + co_prev[1]);
        co_prev = co_curr;
    }
    (points2d, cross)
}

/// `gpencil_calc_stroke_fill_uv` — map 2D fill coords to stroke UV space.
fn gpencil_calc_stroke_fill_uv(points2d: &[[f32; 2]], gps: &bGPDstroke, minv: [f32; 2], maxv: [f32; 2]) -> Vec<Vec2> {
    let s = gps.uv_rotation.sin();
    let c = gps.uv_rotation.cos();
    let center = Vec2::new(0.5, 0.5);
    let d = [maxv[0] - minv[0], maxv[1] - minv[1]];
    let mut r_uv = Vec::with_capacity(gps.totpoints as usize);
    for i in 0..gps.totpoints as usize {
        let mut uv = Vec2::new(
            (points2d[i][0] - minv[0]) / d[0],
            (points2d[i][1] - minv[1]) / d[1],
        );
        uv.x += gps.uv_translation[0];
        uv.y += gps.uv_translation[1];
        uv -= center;
        let x = uv.x * c - uv.y * s;
        let y = uv.x * s + uv.y * c;
        uv = Vec2::new(x, y) + center;
        if gps.uv_scale != 0.0 {
            uv *= 1.0 / gps.uv_scale;
        }
        r_uv.push(uv);
    }
    r_uv
}

/// `stroke_fill_triangulate`.
pub fn stroke_fill_triangulate(gps: &mut bGPDstroke) {
    debug_assert!(gps.totpoints >= 3);
    gps.tot_triangles = gps.totpoints - 2;
    let (points2d, cross) = stroke_2d_flat(&gps.points);
    let direction = if cross > 0.0 { 1 } else { -1 };
    let tmp_triangles = polyfill_calc_pub(&points2d, direction);
    let uv = gpencil_calc_stroke_fill_uv(&points2d, gps, [-1.0, -1.0], [1.0, 1.0]);
    gps.tot_triangles = tmp_triangles.len() as i32;
    if gps.tot_triangles > 0 {
        gps.triangles = tmp_triangles
            .into_iter()
            .map(|verts| bGPDtriangle { verts })
            .collect();
        for (i, u) in uv.iter().enumerate() {
            gps.points[i].uv_fill = [u.x, u.y];
        }
    } else {
        gps.triangles = Vec::new();
    }
}

/// `stroke_uv_update` — cumulative length into `uv_fac`.
fn stroke_uv_update(gps: &mut bGPDstroke) {
    if gps.totpoints == 0 {
        return;
    }
    let mut totlen = 0.0f32;
    gps.points[0].uv_fac = totlen;
    for i in 1..gps.totpoints as usize {
        let a = Vec3::new(gps.points[i - 1].x, gps.points[i - 1].y, gps.points[i - 1].z);
        let b = Vec3::new(gps.points[i].x, gps.points[i].y, gps.points[i].z);
        totlen += (b - a).length();
        gps.points[i].uv_fac = totlen;
    }
}

/// `BKE_annotation_stroke_geometry_update`.
pub fn bke_annotation_stroke_geometry_update(gps: &mut bGPDstroke) {
    if gps.totpoints > 2 {
        stroke_fill_triangulate(gps);
    } else {
        gps.tot_triangles = 0;
        gps.triangles = Vec::new();
    }
    stroke_uv_update(gps);
}
