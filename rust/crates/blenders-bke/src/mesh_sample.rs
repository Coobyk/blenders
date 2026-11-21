//! Port of `intern/mesh_sample.cc` — `mesh_surface_sample`:
//! point/corner/face attribute sampling on corner-tris via
//! barycentric coords, spherical/projected surface sampling, and the
//! `NearestCornerFromPositionFn`/`BaryWeightSampleFn` field fns.
//!
//! `VArray`/`GVArray` → `&[T]` slices; `bvh::Tree` ray-cast injected;
//! `attribute_math::mix3` = barycentric lerp.

use blenders_bli::index_mask::IndexMask;
use blenders_bli::rand::RandomNumberGenerator;
use blenders_math::geom;
use blenders_math::vector::{self, Vec2, Vec3};
use std::f32::consts::PI;

/// `attribute_math::mix3` — barycentric blend `w0*a + w1*b + w2*c`.
#[inline]
fn mix3<T>(w: Vec3, a: T, b: T, c: T) -> T
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>,
{
    a * w.x + b * w.y + c * w.z
}

/// `sample_corner_attribute_with_bary_coords` (inline in
/// BKE_mesh_sample.hh) — corner-domain bary lookup.
#[inline]
pub fn sample_corner_attribute_with_bary_coords<T: Copy>(
    bary_coords: Vec3,
    tri: &[i32; 3],
    src: &[T],
) -> T
where
    T: std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>,
{
    mix3(
        bary_coords,
        src[tri[0] as usize],
        src[tri[1] as usize],
        src[tri[2] as usize],
    )
}

/// `sample_point_attribute<T>` — point-domain attr on tris.
pub fn sample_point_attribute<T: Copy>(
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    tri_indices: &[i32],
    bary_coords: &[Vec3],
    src: &[T],
    mask: &IndexMask,
    dst: &mut [T],
) where
    T: std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>,
{
    mask.iter().for_each(|i| {
        let i = i as usize;
        let tri = &corner_tris[tri_indices[i] as usize];
        dst[i] = mix3(
            bary_coords[i],
            src[corner_verts[tri[0] as usize] as usize],
            src[corner_verts[tri[1] as usize] as usize],
            src[corner_verts[tri[2] as usize] as usize],
        );
    });
}

/// `sample_point_normals` — normalized bary interp.
pub fn sample_point_normals(
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    tri_indices: &[i32],
    bary_coords: &[Vec3],
    src: &[Vec3],
    mask: &IndexMask,
    dst: &mut [Vec3],
) {
    mask.iter().for_each(|i| {
        let i = i as usize;
        let tri = &corner_tris[tri_indices[i] as usize];
        let value = mix3(
            bary_coords[i],
            src[corner_verts[tri[0] as usize] as usize],
            src[corner_verts[tri[1] as usize] as usize],
            src[corner_verts[tri[2] as usize] as usize],
        );
        dst[i] = value.normalized();
    });
}

/// `sample_corner_attribute<T>` — corner-domain (loop) attr.
pub fn sample_corner_attribute<T: Copy>(
    corner_tris: &[[i32; 3]],
    tri_indices: &[i32],
    bary_coords: &[Vec3],
    src: &[T],
    mask: &IndexMask,
    dst: &mut [T],
) where
    T: std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>,
{
    mask.iter().for_each(|i| {
        let i = i as usize;
        let tri = &corner_tris[tri_indices[i] as usize];
        dst[i] = sample_corner_attribute_with_bary_coords(bary_coords[i], tri, src);
    });
}

/// `sample_corner_normals`.
pub fn sample_corner_normals(
    corner_tris: &[[i32; 3]],
    tri_indices: &[i32],
    bary_coords: &[Vec3],
    src: &[Vec3],
    mask: &IndexMask,
    dst: &mut [Vec3],
) {
    mask.iter().for_each(|i| {
        let i = i as usize;
        let tri = &corner_tris[tri_indices[i] as usize];
        let value = sample_corner_attribute_with_bary_coords(bary_coords[i], tri, src);
        dst[i] = value.normalized();
    });
}

/// `sample_face_attribute<T>` — face attr via `tri_faces`.
pub fn sample_face_attribute<T: Copy>(
    tri_faces: &[i32],
    tri_indices: &[i32],
    src: &[T],
    mask: &IndexMask,
    dst: &mut [T],
) {
    mask.iter().for_each(|i| {
        let i = i as usize;
        let tri_index = tri_indices[i] as usize;
        let face_index = tri_faces[tri_index] as usize;
        dst[i] = src[face_index];
    });
}

/// `sample_nearest_corner` — nearest tri corner index
/// (-1 when `check_indices` and `tri_indices[i] == -1`).
pub fn sample_nearest_corner(
    vert_positions: &[Vec3],
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    tri_indices: &[i32],
    sample_positions: &[Vec3],
    mask: &IndexMask,
    nearest_corner: &mut [i32],
    check_indices: bool,
) {
    mask.iter().for_each(|i| {
        let i = i as usize;
        if check_indices && tri_indices[i] == -1 {
            nearest_corner[i] = -1;
            return;
        }
        let tri = &corner_tris[tri_indices[i] as usize];
        let distances = [
            (sample_positions[i] - vert_positions[corner_verts[tri[0] as usize] as usize])
                .length_squared(),
            (sample_positions[i] - vert_positions[corner_verts[tri[1] as usize] as usize])
                .length_squared(),
            (sample_positions[i] - vert_positions[corner_verts[tri[2] as usize] as usize])
                .length_squared(),
        ];
        let index = distances
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap();
        nearest_corner[i] = tri[index];
    });
}

/// `sample_surface_points_spherical` — random barycentric or disc
/// sampling per tri (small/large tri strategies).
#[allow(clippy::too_many_arguments)]
pub fn sample_surface_points_spherical(
    rng: &mut RandomNumberGenerator,
    vert_positions: &[Vec3],
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    tris_to_sample: &[i32],
    sample_pos: Vec3,
    sample_radius: f32,
    approximate_density: f32,
    r_bary_coords: &mut Vec<Vec3>,
    r_tri_indices: &mut Vec<i32>,
    r_positions: &mut Vec<Vec3>,
) -> usize {
    let sample_radius_sq = sample_radius * sample_radius;
    let sample_plane_area = PI * sample_radius_sq;
    /* Used for switching between two triangle sampling strategies. */
    let area_threshold = sample_plane_area;

    let old_num = r_bary_coords.len();

    for &tri_index in tris_to_sample {
        let tri = &corner_tris[tri_index as usize];

        let v0 = vert_positions[corner_verts[tri[0] as usize] as usize];
        let v1 = vert_positions[corner_verts[tri[1] as usize] as usize];
        let v2 = vert_positions[corner_verts[tri[2] as usize] as usize];

        let corner_tri_area = geom::area_tri(v0, v1, v2);

        if corner_tri_area < area_threshold {
            /* The triangle is small compared to the sample radius.
             * Sample by generating random barycentric coordinates. */
            let amount = rng.round_probabilistic(approximate_density * corner_tri_area);
            for _ in 0..amount {
                let bary = rng.get_barycentric_coordinates();
                let bary_coord = Vec3::from(bary);
                let point_pos = mix3(bary_coord, v0, v1, v2);
                let dist_to_sample_sq = (point_pos - sample_pos).length_squared();
                if dist_to_sample_sq > sample_radius_sq {
                    continue;
                }
                r_bary_coords.push(bary_coord);
                r_tri_indices.push(tri_index);
                r_positions.push(point_pos);
            }
        } else {
            /* The triangle is large compared to the sample radius.
             * Sample by generating random points on the triangle
             * plane within the sample radius. */
            let (normal, _d) = geom::normal_tri(v0, v1, v2);

            let sample_pos_proj = vector::project_point_to_plane(sample_pos, normal, v0);

            let proj_distance_sq = (sample_pos_proj - sample_pos).length_squared();
            let sample_radius_factor_sq =
                1.0 - (proj_distance_sq / sample_radius_sq).min(1.0);
            let radius_proj_sq = sample_radius_sq * sample_radius_factor_sq;
            let radius_proj = radius_proj_sq.sqrt();
            let circle_area = PI * radius_proj_sq;

            let amount = rng.round_probabilistic(approximate_density * circle_area);

            let axis_1 = (v1 - v0).normalized() * radius_proj;
            let axis_2 =
                axis_1.cross(axis_1.cross(v2 - v0)).normalized() * radius_proj;

            for _ in 0..amount {
                let r = rng.get_float().sqrt();
                let angle = rng.get_float() * 2.0 * PI;
                let x = r * angle.cos();
                let y = r * angle.sin();
                let point_pos = sample_pos_proj + axis_1 * x + axis_2 * y;
                if !geom::isect_point_tri_prism(point_pos, v0, v1, v2) {
                    /* Sampled point is not in the triangle. */
                    continue;
                }

                let bary_coord = geom::interp_weights_tri(v0, v1, v2, point_pos);

                r_bary_coords.push(bary_coord);
                r_tri_indices.push(tri_index);
                r_positions.push(point_pos);
            }
        }
    }
    r_bary_coords.len() - old_num
}

/// `bvh::RayHit` — raycast result needed by `sample_surface_points_projected`.
pub struct BvhRayHit {
    pub index: i32,
    pub normal: Vec3,
    /// Hit distance along the ray (`position(ray)` = start + dir * t).
    pub t: f32,
}

/// `sample_surface_points_projected` — ray-cast sampling around a
/// 2D region position; `region_position_to_ray` and the BVH
/// `ray_intersect` are injected.
#[allow(clippy::too_many_arguments)]
pub fn sample_surface_points_projected(
    rng: &mut RandomNumberGenerator,
    vert_positions: &[Vec3],
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    sample_pos_re: Vec2,
    sample_radius_re: f32,
    region_position_to_ray: &dyn Fn(Vec2) -> (Vec3, Vec3),
    ray_intersect: &dyn Fn(Vec3, Vec3) -> Option<BvhRayHit>,
    front_face_only: bool,
    tries_num: i32,
    max_points: i32,
    r_bary_coords: &mut Vec<Vec3>,
    r_tri_indices: &mut Vec<i32>,
    r_positions: &mut Vec<Vec3>,
) -> usize {
    let mut point_count = 0;
    for _ in 0..tries_num {
        if point_count == max_points as usize {
            break;
        }

        let r = sample_radius_re * rng.get_float().sqrt();
        let angle = rng.get_float() * 2.0 * PI;
        let pos_re = sample_pos_re + Vec2::new(angle.cos(), angle.sin()) * r;
        let (ray_start, ray_end) = region_position_to_ray(pos_re);
        let ray_direction = (ray_end - ray_start).normalized();

        let Some(ray_hit) = ray_intersect(ray_start, ray_direction) else {
            continue;
        };

        if front_face_only {
            let normal = ray_hit.normal.normalized();
            if ray_direction.dot(normal) >= 0.0 {
                continue;
            }
        }

        let tri_index = ray_hit.index;
        let pos = ray_start + ray_direction * ray_hit.t;

        let bary_coords = compute_bary_coord_in_triangle(
            vert_positions,
            corner_verts,
            &corner_tris[tri_index as usize],
            pos,
        );

        r_positions.push(pos);
        r_bary_coords.push(bary_coords);
        r_tri_indices.push(tri_index);
        point_count += 1;
    }
    point_count
}

/// `compute_bary_coord_in_triangle`.
pub fn compute_bary_coord_in_triangle(
    vert_positions: &[Vec3],
    corner_verts: &[i32],
    tri: &[i32; 3],
    position: Vec3,
) -> Vec3 {
    let v0 = vert_positions[corner_verts[tri[0] as usize] as usize];
    let v1 = vert_positions[corner_verts[tri[1] as usize] as usize];
    let v2 = vert_positions[corner_verts[tri[2] as usize] as usize];
    geom::interp_weights_tri(v0, v1, v2, position)
}

/// `NearestCornerFromPositionFn` — the field-function wrapper:
/// inputs `(Position, Triangle Index)` → `Nearest Corner` via
/// `sample_nearest_corner<check_indices=true>`.
pub fn nearest_corner_from_position_call(
    vert_positions: &[Vec3],
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    triangle_indices: &[i32],
    sample_positions: &[Vec3],
    mask: &IndexMask,
    nearest_corner: &mut [i32],
) {
    sample_nearest_corner(
        vert_positions,
        corner_verts,
        corner_tris,
        triangle_indices,
        sample_positions,
        mask,
        nearest_corner,
        true,
    );
}
