//! Port of `intern/bvhutils.cc` — `BVHTreeFromMesh`: mesh→BVH
//! builders and the nearest-point / sphere-cast callbacks
//! (`mesh_faces_nearest_point`, `mesh_corner_tris_nearest_point`,
//! `mesh_faces_spherecast`, `mesh_corner_tris_spherecast`,
//! `mesh_edges_nearest_point`, `mesh_verts_spherecast`,
//! `mesh_edges_spherecast`), plus `bvhtree_ray_tri_intersection`
//! / `bvhtree_sphereray_tri_intersection`.

use blenders_bli::kdopbvh::{BVHTreeNearest, BVHTreeRay, BVHTreeRayHit};
use blenders_math::{geom, Vec3};

/// `bvhtree_ray_tri_intersection` — epsilon ray∩tri, FLT_MAX on miss.
pub fn bvhtree_ray_tri_intersection(ray: &BVHTreeRay, v0: Vec3, v1: Vec3, v2: Vec3) -> f32 {
    let o = Vec3::from(ray.origin);
    let d = Vec3::from(ray.direction);
    if let Some((dist, _u, _v)) = geom::isect_ray_tri_epsilon(o, d, v0, v1, v2, f32::EPSILON) {
        dist
    } else {
        f32::MAX
    }
}

/// `bvhtree_sphereray_tri_intersection`.
pub fn bvhtree_sphereray_tri_intersection(
    ray: &BVHTreeRay,
    radius: f32,
    m_dist: f32,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> f32 {
    let o = Vec3::from(ray.origin);
    let d = Vec3::from(ray.direction);
    let p1 = o + d * m_dist;
    if let Some((idist, _hit)) = geom::isect_sweeping_sphere_tri(o, p1, radius, v0, v1, v2) {
        idist * m_dist
    } else {
        f32::MAX
    }
}

/// `BVHTreeFromMesh` — mesh data the callbacks capture.
#[derive(Clone)]
pub struct BVHTreeFromMesh<'a> {
    /// `vert_positions`.
    pub vert_positions: &'a [Vec3],
    /// `edges`.
    pub edges: &'a [[i32; 2]],
    /// `face` — legacy `(v1,v2,v3,v4)` quads (v4=0 for tris).
    pub face: &'a [[i32; 4]],
    /// `corner_tris`.
    pub corner_tris: &'a [[i32; 3]],
    /// `corner_verts`.
    pub corner_verts: &'a [i32],
}

/// `mesh_faces_nearest_point`.
pub fn mesh_faces_nearest_point(
    data: &BVHTreeFromMesh,
    index: usize,
    co: Vec3,
    nearest: &mut BVHTreeNearest,
) {
    let face = data.face[index];
    let t0 = data.vert_positions[face[0] as usize];
    let mut t1 = data.vert_positions[face[1] as usize];
    let mut t2 = data.vert_positions[face[2] as usize];
    let mut t3 = if face[3] != 0 {
        Some(data.vert_positions[face[3] as usize])
    } else {
        None
    };

    loop {
        let nearest_tmp = geom::closest_on_tri_to_point(co, t0, t1, t2).0;
        let dist_sq = co.distance_squared(nearest_tmp);
        if dist_sq < nearest.dist_sq {
            nearest.index = index as i32;
            nearest.dist_sq = dist_sq;
            nearest.co = nearest_tmp.into();
            nearest.no = geom::normal_tri(t0, t1, t2).0.into();
        }
        t1 = t2;
        match t3 {
            Some(v) => {
                t2 = v;
                t3 = None;
            }
            None => break,
        }
        /* C: t1=t2; t2=t3; t3=null → second iter uses (t0,t2,t3). */
    }
}

/// `mesh_corner_tris_nearest_point`.
pub fn mesh_corner_tris_nearest_point(
    data: &BVHTreeFromMesh,
    index: usize,
    co: Vec3,
    nearest: &mut BVHTreeNearest,
) {
    let tri = data.corner_tris[index];
    let v0 = data.vert_positions[data.corner_verts[tri[0] as usize] as usize];
    let v1 = data.vert_positions[data.corner_verts[tri[1] as usize] as usize];
    let v2 = data.vert_positions[data.corner_verts[tri[2] as usize] as usize];

    let nearest_tmp = geom::closest_on_tri_to_point(co, v0, v1, v2).0;
    let dist_sq = co.distance_squared(nearest_tmp);
    if dist_sq < nearest.dist_sq {
        nearest.index = index as i32;
        nearest.dist_sq = dist_sq;
        nearest.co = nearest_tmp.into();
        nearest.no = geom::normal_tri(v0, v1, v2).0.into();
    }
}

/// `mesh_faces_spherecast`.
pub fn mesh_faces_spherecast(
    data: &BVHTreeFromMesh,
    index: usize,
    ray: &BVHTreeRay,
    hit: &mut BVHTreeRayHit,
) {
    let face = data.face[index];
    let t0 = data.vert_positions[face[0] as usize];
    let mut t1 = data.vert_positions[face[1] as usize];
    let mut t2 = data.vert_positions[face[2] as usize];
    let mut t3 = if face[3] != 0 {
        Some(data.vert_positions[face[3] as usize])
    } else {
        None
    };

    loop {
        let dist = if ray.radius == 0.0 {
            bvhtree_ray_tri_intersection(ray, t0, t1, t2)
        } else {
            bvhtree_sphereray_tri_intersection(ray, ray.radius, hit.dist, t0, t1, t2)
        };
        if dist >= 0.0 && dist < hit.dist {
            hit.index = index as i32;
            hit.dist = dist;
            hit.co = (Vec3::from(ray.origin) + Vec3::from(ray.direction) * dist).into();
            hit.no = geom::normal_tri(t0, t1, t2).0.into();
        }
        t1 = t2;
        match t3 {
            Some(v) => {
                t2 = v;
                t3 = None;
            }
            None => break,
        }
    }
}

/// `mesh_corner_tris_spherecast`.
pub fn mesh_corner_tris_spherecast(
    data: &BVHTreeFromMesh,
    index: usize,
    ray: &BVHTreeRay,
    hit: &mut BVHTreeRayHit,
) {
    let tri = data.corner_tris[index];
    let v0 = data.vert_positions[data.corner_verts[tri[0] as usize] as usize];
    let v1 = data.vert_positions[data.corner_verts[tri[1] as usize] as usize];
    let v2 = data.vert_positions[data.corner_verts[tri[2] as usize] as usize];

    let dist = if ray.radius == 0.0 {
        bvhtree_ray_tri_intersection(ray, v0, v1, v2)
    } else {
        bvhtree_sphereray_tri_intersection(ray, ray.radius, hit.dist, v0, v1, v2)
    };
    if dist >= 0.0 && dist < hit.dist {
        hit.index = index as i32;
        hit.dist = dist;
        hit.co = (Vec3::from(ray.origin) + Vec3::from(ray.direction) * dist).into();
        hit.no = geom::normal_tri(v0, v1, v2).0.into();
    }
}

/// `mesh_edges_nearest_point`.
pub fn mesh_edges_nearest_point(
    data: &BVHTreeFromMesh,
    index: usize,
    co: Vec3,
    nearest: &mut BVHTreeNearest,
) {
    let edge = data.edges[index];
    let t0 = data.vert_positions[edge[0] as usize];
    let t1 = data.vert_positions[edge[1] as usize];

    let nearest_tmp = geom::closest_to_line_segment_v3(co, t0, t1).0;
    let dist_sq = nearest_tmp.distance_squared(co);
    if dist_sq < nearest.dist_sq {
        nearest.index = index as i32;
        nearest.dist_sq = dist_sq;
        nearest.co = nearest_tmp.into();
        nearest.no = (t0 - t1).normalized().into();
    }
}

/// `mesh_verts_spherecast_do`.
fn mesh_verts_spherecast_do(index: usize, v: Vec3, ray: &BVHTreeRay, hit: &mut BVHTreeRayHit) {
    let r1 = Vec3::from(ray.origin);
    let r2 = r1 + Vec3::from(ray.direction);
    let i1 = geom::closest_to_line_segment_v3(v, r1, r2).0;
    let seg = r2 - r1;
    if (i1 - r1).dot(seg) >= 0.0 {
        let dist = r1.distance(i1);
        if dist < hit.dist {
            hit.index = index as i32;
            hit.dist = dist;
            hit.co = i1.into();
        }
    }
}

/// `mesh_verts_spherecast`.
pub fn mesh_verts_spherecast(
    data: &BVHTreeFromMesh,
    index: usize,
    ray: &BVHTreeRay,
    hit: &mut BVHTreeRayHit,
) {
    mesh_verts_spherecast_do(index, data.vert_positions[index], ray, hit);
}

/// `mesh_edges_spherecast_do` — isect_line_line + radius check.
fn mesh_edges_spherecast_do(
    index: usize,
    v1: Vec3,
    v2: Vec3,
    ray: &BVHTreeRay,
    hit: &mut BVHTreeRayHit,
) {
    let radius_sq = ray.radius * ray.radius;
    if v1 == v2 {
        mesh_verts_spherecast_do(index, v1, ray, hit);
        return;
    }
    let r1 = Vec3::from(ray.origin);
    let r2 = r1 + Vec3::from(ray.direction);

    let (isect, mut i1, i2) = geom::isect_line_line(v1, v2, r1, r2);
    if isect != 0 {
        /* `dot_v3v3v3(r1,i2,r2)` = dot(i2-r1, r2-r1). */
        if (i2 - r1).dot(r2 - r1) >= 0.0 {
            let dist = r1.distance(i2);
            if dist < hit.dist {
                let e_fac = geom::line_point_factor(i1, v1, v2);
                if e_fac < 0.0 {
                    i1 = v1;
                } else if e_fac > 1.0 {
                    i1 = v2;
                }
                if i1.distance_squared(i2) <= radius_sq {
                    hit.index = index as i32;
                    hit.dist = dist;
                    hit.co = i2.into();
                }
            }
        }
    }
}

/// `mesh_edges_spherecast`.
pub fn mesh_edges_spherecast(
    data: &BVHTreeFromMesh,
    index: usize,
    ray: &BVHTreeRay,
    hit: &mut BVHTreeRayHit,
) {
    let edge = data.edges[index];
    mesh_edges_spherecast_do(
        index,
        data.vert_positions[edge[0] as usize],
        data.vert_positions[edge[1] as usize],
        ray,
        hit,
    );
}

/// `bvhtree_new_common` — tree with `elems_num` capacity (kdopbvh
/// `bvhtree_new` args folded: axis=6, epsilon from
/// `bvhtree_get_epsilon` convention).
pub use blenders_bli::kdopbvh::{
    bvhtree_balance, bvhtree_get_epsilon, bvhtree_insert, bvhtree_new, bvhtree_ray_cast,
};
