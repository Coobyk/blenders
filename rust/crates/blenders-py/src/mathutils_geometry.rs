/// Port of `mathutils_geometry.cc` —
/// `mathutils.geometry` (intersect/tesselate/
/// tessellate_polygon/volume_tetrahedron/
/// interpolate_bezier).
/// `geometry_intersect_ray_plane` (verbatim).
pub fn geometry_intersect_ray_plane(
    orig: [f32; 3],
    dir: [f32; 3],
    plane_co: [f32; 3],
    plane_no: [f32; 3],
) -> Option<[f32; 3]> {
    let d = plane_no[0] * dir[0] + plane_no[1] * dir[1] + plane_no[2] * dir[2];
    if d == 0.0 {
        return None;
    }
    let t = ((plane_co[0] - orig[0]) * plane_no[0]
        + (plane_co[1] - orig[1]) * plane_no[1]
        + (plane_co[2] - orig[2]) * plane_no[2])
        / d;
    Some([orig[0] + dir[0] * t, orig[1] + dir[1] * t, orig[2] + dir[2] * t])
}
/// `geometry_volume_tetrahedron` (verbatim).
pub fn geometry_volume_tetrahedron(
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    d: [f32; 3],
) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ad = [d[0] - a[0], d[1] - a[1], d[2] - a[2]];
    let cross = [
        ac[1] * ad[2] - ac[2] * ad[1],
        ac[2] * ad[0] - ac[0] * ad[2],
        ac[0] * ad[1] - ac[1] * ad[0],
    ];
    (ab[0] * cross[0] + ab[1] * cross[1] + ab[2] * cross[2]).abs() / 6.0
}
