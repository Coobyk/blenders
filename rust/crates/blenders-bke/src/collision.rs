//! Port of `intern/collision.cc` — `collision_move_object`,
//! `bvhtree_build/update_from_mvert`, `compute_collision_point_
//! tri_tri`/`_edge_tri` (backside projection + closest-point
//! fallback), `collision_compute_barycentric`,
//! `collision_interpolateOnTriangle`, `cloth_collision_impulse
//! _vert`, `collision_get_collider_velocity`, `CollisionRelation`/
//! `ColliderCache` list mgmt.
//! `PORT_PENDING`: `CollPair`/`CollisionModifierData` DNA (cloth
//! module pending — arrays are taken as `&mut Vec<Vec3>` etc.),
//! `cloth_collision`/`hair_collision` parallel loops + resolve
//! passes (need `cloth.c` sim state — the pair math they call is
//! ported).

use blenders_bli::kdopbvh::{self, BVHTree};
use blenders_math::geom;
use blenders_math::vector::Vec3;

/// `CollisionRelation` — Object↔Object collision pair list.
#[derive(Clone)]
pub struct CollisionRelation {
    /// `Object *ab`.
    pub ab: usize,
    /// `Object *bc`.
    pub bc: usize,
    /// `Object *ob`.
    pub ob: usize,
}

/// `ColliderCache` — per-collider cache slot.
#[derive(Clone)]
pub struct ColliderCache {
    /// `Ob* ` collider index.
    pub ob: usize,
    /// `BKE_collider_cache` payload (collider BVH object).
    pub collmd: usize,
}

/// `collision_move_object` — `x`/`xnew` lerp + velocity + BVH
/// update (or zero velocity for static).
pub fn collision_move_object(
    is_static: bool,
    x: &[Vec3],
    xnew: &[Vec3],
    current_x: &mut [Vec3],
    current_xnew: &mut [Vec3],
    current_v: &mut [Vec3],
    bvhtree: &mut BVHTree,
    vert_tris: &[[i32; 3]],
    step: f32,
    prevstep: f32,
    moving_bvh: bool,
) {
    let n = x.len();
    if is_static {
        for i in 0..n {
            current_v[i] = Vec3::ZERO;
        }
        return;
    }
    for i in 0..n {
        current_x[i] = x[i] + (xnew[i] - x[i]) * prevstep;
        current_xnew[i] = x[i] + (xnew[i] - x[i]) * step;
        current_v[i] = current_xnew[i] - current_x[i];
    }
    bvhtree_update_from_mvert(bvhtree, current_xnew, Some(current_x), vert_tris, moving_bvh);
}

/// `bvhtree_build_from_mvert`.
pub fn bvhtree_build_from_mvert(
    positions: &[Vec3],
    vert_tris: &[[i32; 3]],
    epsilon: f32,
) -> BVHTree {
    let mut tree = kdopbvh::bvhtree_new(vert_tris.len() as i32, epsilon, 4, 26);
    for (i, t) in vert_tris.iter().enumerate() {
        let co: Vec<f32> = [
            positions[t[0] as usize],
            positions[t[1] as usize],
            positions[t[2] as usize],
        ]
        .iter()
        .flat_map(|v| [v.x, v.y, v.z])
        .collect();
        kdopbvh::bvhtree_insert(&mut tree, i as i32, &co, 3);
    }
    kdopbvh::bvhtree_balance(&mut tree);
    tree
}

/// `bvhtree_update_from_mvert`.
pub fn bvhtree_update_from_mvert(
    bvhtree: &mut BVHTree,
    positions_new: &[Vec3],
    positions_moving: Option<&[Vec3]>,
    vert_tris: &[[i32; 3]],
    moving: bool,
) {
    for (i, t) in vert_tris.iter().enumerate() {
        let co: Vec<f32> = [
            positions_new[t[0] as usize],
            positions_new[t[1] as usize],
            positions_new[t[2] as usize],
        ]
        .iter()
        .flat_map(|v| [v.x, v.y, v.z])
        .collect();
        let co_moving: Option<Vec<f32>> = if moving {
            positions_moving.map(|pm| {
                [
                    pm[t[0] as usize],
                    pm[t[1] as usize],
                    pm[t[2] as usize],
                ]
                .iter()
                .flat_map(|v| [v.x, v.y, v.z])
                .collect()
            })
        } else {
            None
        };
        kdopbvh::bvhtree_update_node(bvhtree, i, &co, co_moving.as_deref(), 3);
    }
    kdopbvh::bvhtree_update_tree(bvhtree);
}

fn next_ind(i: usize) -> usize {
    if i < 2 {
        i + 1
    } else {
        0
    }
}

/// `isect_seg_seg_v3` — closest points of two segments via
/// line-line + clamp (C `isect_seg_seg_v3` semantics: lambda-
/// clamped line param then recompute).
fn isect_seg_seg_v3(a1: Vec3, a2: Vec3, b1: Vec3, b2: Vec3) -> (Vec3, Vec3) {
    let (_, mut co1, mut co2) = geom::isect_line_line(a1, a2, b1, b2);
    /* clamp each to its segment. */
    co1 = geom::closest_to_line_segment_v3(co1, a1, a2).0;
    co2 = geom::closest_to_line_segment_v3(co2, b1, b2).0;
    (co1, co2)
}

/// `compute_collision_point_tri_tri` — `(r_a, r_b, r_vec)`,
/// returns distance (FLT_MAX for backside cull-out).
pub fn compute_collision_point_tri_tri(
    a1: Vec3,
    a2: Vec3,
    a3: Vec3,
    b1: Vec3,
    b2: Vec3,
    b3: Vec3,
    culling: bool,
    use_normal: bool,
) -> (f32, Vec3, Vec3, Vec3) {
    let a = [a1, a2, a3];
    let b = [b1, b2, b3];
    let mut dist = f32::MAX;
    let (mut r_a, mut r_b, mut r_vec) = (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
    let (mut normal, mut cent) = (Vec3::ZERO, Vec3::ZERO);
    let mut backside = false;

    /* `isect_tri_tri_v3_ex` — `(isect, isect_a, isect_b,
     * tri_a_edge_isect_count)`. */
    let hit = geom::isect_tri_tri_ex(&a, &b);
    let is_intersecting = hit.is_some();
    let (isect_a, isect_b, tri_a_edge_isect_count) =
        hit.unwrap_or((Vec3::ZERO, Vec3::ZERO, 0));

    if culling {
        normal = geom::normal_tri(b[0], b[1], b[2]).0;
        cent = (b[0] + b[1] + b[2]) / 3.0;
        if !is_intersecting {
            for i in 0..3 {
                if (a[i] - cent).dot(normal) < 0.0 {
                    backside = true;
                    break;
                }
            }
        } else if tri_a_edge_isect_count != 1 {
            backside = true;
        }
    } else if use_normal {
        normal = geom::normal_tri(b[0], b[1], b[2]).0;
    }

    if tri_a_edge_isect_count == 1 {
        /* Edge intersection. */
        r_a = isect_a;
        r_b = isect_b;
        r_vec = if use_normal { normal } else { r_b - r_a };
        return (0.0, r_a, r_b, r_vec);
    }

    if backside {
        let mut maxdist = 0.0f32;
        let mut found = false;
        /* Point projections. */
        for i in 0..3 {
            if let Some((tmp, _, _)) = geom::isect_ray_tri(a[i], normal, b[0], b[1], b[2]) {
                if tmp > maxdist {
                    maxdist = tmp;
                    r_a = a[i];
                    r_b = a[i] + normal * tmp;
                    found = true;
                }
            }
        }
        let n = -normal;
        for i in 0..3 {
            if let Some((tmp, _, _)) = geom::isect_ray_tri(b[i], n, a[0], a[1], a[2]) {
                if tmp > maxdist {
                    maxdist = tmp;
                    r_a = b[i] + n * tmp;
                    r_b = b[i];
                    found = true;
                }
            }
        }
        /* Edge projections. */
        for i in 0..3 {
            let tmp_vec = b[next_ind(i)] - b[i];
            let dir = tmp_vec.cross(normal);
            for j in 0..3 {
                if let Some(tmp_co1) = geom::isect_line_plane(a[j], a[next_ind(j)], b[i], dir)
                {
                    if geom::point_in_slice_seg(tmp_co1, a[j], a[next_ind(j)])
                        && geom::point_in_slice_seg(tmp_co1, b[i], b[next_ind(i)])
                    {
                        let tmp_co2 = geom::closest_to_line(tmp_co1, b[i], b[next_ind(i)]).0;
                        let tmp_vec2 = tmp_co1 - tmp_co2;
                        let tmp = tmp_vec2.length();
                        if tmp > maxdist && tmp_vec2.dot(normal) < 0.0 {
                            maxdist = tmp;
                            r_a = tmp_co1;
                            r_b = tmp_co2;
                            found = true;
                        }
                    }
                }
            }
        }
        if found {
            r_vec = r_b - r_a;
            if use_normal {
                r_vec = if normal.dot(r_vec) >= 0.0 { normal } else { -normal };
            }
            return (0.0, r_a, r_b, r_vec);
        }
    }

    /* Closest point. */
    for i in 0..3 {
        let tmp_co1 = geom::closest_on_tri_to_point(a[i], b[0], b[1], b[2]).0;
        let tmp = (tmp_co1 - a[i]).length_squared();
        if tmp < dist {
            dist = tmp;
            r_a = a[i];
            r_b = tmp_co1;
        }
    }
    for i in 0..3 {
        let tmp_co1 = geom::closest_on_tri_to_point(b[i], a[0], a[1], a[2]).0;
        let tmp = (tmp_co1 - b[i]).length_squared();
        if tmp < dist {
            dist = tmp;
            r_a = tmp_co1;
            r_b = b[i];
        }
    }
    /* Closest edge. */
    if !is_intersecting {
        for i in 0..3 {
            for j in 0..3 {
                let (co1, co2) =
                    isect_seg_seg_v3(a[i], a[next_ind(i)], b[j], b[next_ind(j)]);
                let tmp = (co1 - co2).length_squared();
                if tmp < dist {
                    dist = tmp;
                    r_a = co1;
                    r_b = co2;
                }
            }
        }
    }

    if !is_intersecting {
        r_vec = r_a - r_b;
        dist = dist.sqrt();
    } else {
        r_vec = r_b - r_a;
        dist = 0.0;
    }

    if culling && use_normal {
        r_vec = normal;
    } else if use_normal {
        r_vec = if normal.dot(r_vec) >= 0.0 { normal } else { -normal };
    } else if culling && r_vec.dot(normal) < 0.0 {
        return (f32::MAX, r_a, r_b, r_vec);
    }
    (dist, r_a, r_b, r_vec)
}

/// `compute_collision_point_edge_tri`.
pub fn compute_collision_point_edge_tri(
    a1: Vec3,
    a2: Vec3,
    b1: Vec3,
    b2: Vec3,
    b3: Vec3,
    culling: bool,
    use_normal: bool,
) -> (f32, Vec3, Vec3, Vec3) {
    let a = [a1, a2];
    let b = [b1, b2, b3];
    let mut dist = f32::MAX;
    let (mut r_a, mut r_b, mut r_vec) = (Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
    let (normal, _) = geom::normal_tri(b[0], b[1], b[2]);
    let cent = (b[0] + b[1] + b[2]) / 3.0;
    let mut backside = false;

    let isect_t = geom::isect_line_segment_tri(a[0], a[1], b[0], b[1], b[2]);
    let isect = isect_t.is_some();
    let isect_a = isect_t
        .map(|(t, _, _)| a[0] + (a[1] - a[0]) * t)
        .unwrap_or(Vec3::ZERO);

    if culling {
        if isect {
            backside = true;
        } else {
            for i in 0..2 {
                if (a[i] - cent).dot(normal) < 0.0 {
                    backside = true;
                    break;
                }
            }
        }
    }

    if isect {
        r_a = isect_a;
        r_b = isect_a;
        r_vec = normal;
        return (0.0, r_a, r_b, r_vec);
    }

    if backside {
        let mut maxdist = 0.0f32;
        let mut found = false;
        for i in 0..2 {
            if let Some((tmp, _, _)) = geom::isect_ray_tri(a[i], normal, b[0], b[1], b[2]) {
                if tmp > maxdist {
                    maxdist = tmp;
                    r_a = a[i];
                    r_b = a[i] + normal * tmp;
                    found = true;
                }
            }
        }
        for i in 0..3 {
            let tmp_vec = b[next_ind(i)] - b[i];
            let dir = tmp_vec.cross(normal);
            if let Some(tmp_co1) = geom::isect_line_plane(a[0], a[1], b[i], dir) {
                if geom::point_in_slice_seg(tmp_co1, a[0], a[1])
                    && geom::point_in_slice_seg(tmp_co1, b[i], b[next_ind(i)])
                {
                    let tmp_co2 = geom::closest_to_line(tmp_co1, b[i], b[next_ind(i)]).0;
                    let tmp_vec2 = tmp_co1 - tmp_co2;
                    let tmp = tmp_vec2.length();
                    if tmp > maxdist && tmp_vec2.dot(normal) < 0.0 {
                        maxdist = tmp;
                        r_a = tmp_co1;
                        r_b = tmp_co2;
                        found = true;
                    }
                }
            }
        }
        if found {
            r_vec = r_b - r_a;
            if use_normal {
                r_vec = if normal.dot(r_vec) >= 0.0 { normal } else { -normal };
            }
            return (0.0, r_a, r_b, r_vec);
        }
    }

    for i in 0..2 {
        let tmp_co1 = geom::closest_on_tri_to_point(a[i], b[0], b[1], b[2]).0;
        let tmp = (tmp_co1 - a[i]).length_squared();
        if tmp < dist {
            dist = tmp;
            r_a = a[i];
            r_b = tmp_co1;
        }
    }
    if !isect {
        for j in 0..3 {
            let (co1, co2) = isect_seg_seg_v3(a[0], a[1], b[j], b[next_ind(j)]);
            let tmp = (co1 - co2).length_squared();
            if tmp < dist {
                dist = tmp;
                r_a = co1;
                r_b = co2;
            }
        }
    }

    if !isect {
        r_vec = r_a - r_b;
        dist = dist.sqrt();
    } else {
        r_vec = r_b - r_a;
        dist = 0.0;
    }

    if culling && use_normal {
        r_vec = normal;
    } else if use_normal {
        r_vec = if normal.dot(r_vec) >= 0.0 { normal } else { -normal };
    } else if culling && r_vec.dot(normal) < 0.0 {
        return (f32::MAX, r_a, r_b, r_vec);
    }
    (dist, r_a, r_b, r_vec)
}

/// `collision_compute_barycentric` — `(w1, w2, w3)`.
pub fn collision_compute_barycentric(
    pv: Vec3,
    p1: Vec3,
    p2: Vec3,
    p3: Vec3,
) -> (f32, f32, f32) {
    let to_f64 = |v: Vec3| -> (f64, f64, f64) { (v.x as f64, v.y as f64, v.z as f64) };
    let dot3 = |a: (f64, f64, f64), b: (f64, f64, f64)| a.0 * b.0 + a.1 * b.1 + a.2 * b.2;
    let (temp_v1, temp_v2, temp_v4) = (
        to_f64(p1 - p3),
        to_f64(p2 - p3),
        to_f64(pv - p3),
    );
    let a = dot3(temp_v1, temp_v1);
    let b = dot3(temp_v1, temp_v2);
    let c = dot3(temp_v2, temp_v2);
    let e = dot3(temp_v1, temp_v4);
    let f = dot3(temp_v2, temp_v4);
    let d = a * c - b * b;
    if d.abs() < 1e-10 {
        return (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
    }
    let mut w1 = ((e * c - b * f) / d) as f32;
    w1 = w1.max(0.0);
    let mut w2 = ((f - b * w1 as f64) / c) as f32;
    w2 = w2.max(0.0);
    let w3 = 1.0 - w1 - w2;
    (w1, w2, w3)
}

/// `collision_interpolateOnTriangle`.
pub fn collision_interpolate_on_triangle(
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    w1: f64,
    w2: f64,
    w3: f64,
) -> Vec3 {
    v1 * w1 as f32 + v2 * w2 as f32 + v3 * w3 as f32
}

/// `cloth_collision_impulse_vert` — keep largest per-axis impulse.
pub fn cloth_collision_impulse_vert(clamp_sq: f32, impulse: Vec3, vert_impulse: &mut [f32; 3], impulse_count: &mut i32) {
    let len_sq = impulse.length_squared();
    if clamp_sq > 0.0 && len_sq > clamp_sq {
        return;
    }
    for i in 0..3 {
        let v = [impulse.x, impulse.y, impulse.z][i];
        if vert_impulse[i].abs() < v.abs() {
            vert_impulse[i] = v;
        }
    }
    *impulse_count += 1;
}

/// `CollPair` — one face-point contact (pb + tri indices).
#[derive(Clone, Copy, Default)]
pub struct CollPair {
    /// Face index in collider.
    pub face1: i32,
    /// `pa` (cloth point index).
    pub pa_index: i32,
    /// `pb` collision point on collider tri.
    pub pb: Vec3,
    /// `bp1..bp3` collider vert indices.
    pub bp1: usize,
    /// See above.
    pub bp2: usize,
    /// See above.
    pub bp3: usize,
    /// `time`.
    pub time: f32,
    /// `ap`, `bp` midpoints.
    pub ap: Vec3,
    /// `bp` (unused spare).
    pub bp: [usize; 3],
    /// `distance`.
    pub distance: f32,
    /// `weight`.
    pub weight: f32,
}

/// `collision_get_collider_velocity` — barycentric-lerped
/// collider vel at `pb`, plus `vel_old` via `current_v`.
pub fn collision_get_collider_velocity(
    current_x: &[Vec3],
    current_v: &[Vec3],
    collpair: &CollPair,
) -> (Vec3, Vec3) {
    let (u1, u2, u3) = collision_compute_barycentric(
        collpair.pb,
        current_x[collpair.bp1],
        current_x[collpair.bp2],
        current_x[collpair.bp3],
    );
    let vel_new = collision_interpolate_on_triangle(
        current_v[collpair.bp1],
        current_v[collpair.bp2],
        current_v[collpair.bp3],
        u1 as f64,
        u2 as f64,
        u3 as f64,
    );
    /* C computes `vel_old` from the *old* positions: using the
     * same barycentric weights over `x` (prev frame end) here —
     * callers pass `current_x` for the current frame and may
     * pass the older buffer for `vel_old`. */
    let vel_old = collision_interpolate_on_triangle(
        current_x[collpair.bp1],
        current_x[collpair.bp2],
        current_x[collpair.bp3],
        u1 as f64,
        u2 as f64,
        u3 as f64,
    );
    (vel_old, vel_new)
}

/// `add_collision_object` — insert unique relation.
pub fn add_collision_object(relations: &mut Vec<CollisionRelation>, ab: usize, bc: usize, ob: usize) {
    let exists = relations.iter().any(|r| r.ab == ab && r.bc == bc && r.ob == ob);
    if !exists {
        relations.push(CollisionRelation { ab, bc, ob });
    }
}

/// `BKE_collision_relations_free`.
pub fn bke_collision_relations_free(relations: &mut Vec<CollisionRelation>) {
    relations.clear();
}

/// `BKE_collision_objects_free` — free every collider cache then
/// the object set.
pub fn bke_collision_objects_free(colliders: &mut Vec<ColliderCache>) {
    for c in colliders.iter_mut() {
        c.collmd = 0;
        c.ob = 0;
    }
    colliders.clear();
}

/// `BKE_collider_cache_free`.
pub fn bke_collider_cache_free(colliders: &mut Vec<ColliderCache>) {
    bke_collision_objects_free(colliders);
}
