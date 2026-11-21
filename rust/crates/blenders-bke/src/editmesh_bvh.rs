//! Port of `intern/editmesh_bvh.cc` — `BMBVHTree`: a `BVHTree` over
//! edit-mesh loop-tris plus the ray-cast / closest-vert / closest-face
//! / overlap queries that ride on it.
//!
//! `BMLoop*`/`BMVert*`/`BMFace*` become index handles; vertex coords
//! come from an injected `cos_of` (or `cos_cage`); tris carry their
//! face index like `ltri[0]->f`.

use blenders_bli::kdopbvh::{
    self, BVHTree, BVHTreeNearest, BVHTreeOverlap, BVHTreeRayHit,
};
use blenders_math::geom;
use blenders_math::{Vec2, Vec3};
use blenders_math::vector;

/// `BMBVH_*` flags (BKE_editmesh_bvh.hh).
pub const BMBVH_RETURN_ORIG: i32 = 1 << 0;
/// Restrict to selected geometry (overrides RESPECT_HIDDEN).
pub const BMBVH_RESPECT_SELECT: i32 = 1 << 1;
/// Restrict to unhidden geometry.
pub const BMBVH_RESPECT_HIDDEN: i32 = 1 << 2;

/// `BMBVHTree`.
pub struct BMBVHTree {
    /// `looptris` — `[v0, v1, v2]` vert handles per tri.
    pub looptris: Vec<[usize; 3]>,
    /// `ltri[0]->f` per tri.
    pub face_of_tri: Vec<usize>,
    /// `cos_cage` — optional deform coords indexed by vert handle.
    pub cos_cage: Option<Vec<[f32; 3]>>,
    /// `flag`.
    pub flag: i32,
    /// `tree` — the BVH over tri bounds.
    pub tree: BVHTree,
    /// Which looptris are in the tree (`index` used in hits maps back
    /// to this list's positions — C uses the full span index because
    /// it inserts the span index; we insert positions into
    /// `tree_index_to_tri` since tests may skip tris).
    pub tree_index_to_tri: Vec<usize>,
}

/// `bmbvh_tri_from_face` — tri coords from loop verts or cage.
fn bmbvh_tri_from_face(
    ltri: [usize; 3],
    cos_cage: Option<&Vec<[f32; 3]>>,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> [[f32; 3]; 3] {
    match cos_cage {
        None => [cos_of(ltri[0]), cos_of(ltri[1]), cos_of(ltri[2])],
        Some(cage) => [cage[ltri[0]], cage[ltri[1]], cage[ltri[2]]],
    }
}

/// `BKE_bmbvh_new_ex` — build the tree over filtered tris.
///
/// `test_fn(face) -> bool` filters faces (the C caches the result per
/// face since tris of a face are contiguous); `cos_of` resolves vert
/// handles to coordinates.
pub fn bke_bmbvh_new_ex(
    looptris: &[[usize; 3]],
    face_of_tri: &[usize],
    flag: i32,
    cos_cage: Option<Vec<[f32; 3]>>,
    mut test_fn: Option<&mut dyn FnMut(usize) -> bool>,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> BMBVHTree {
    /* could become argument */
    let epsilon = f32::EPSILON * 2.0;

    let mut passing: Vec<usize> = Vec::with_capacity(looptris.len());
    match &mut test_fn {
        Some(test_fn) => {
            let mut f_test_prev = usize::MAX;
            let mut test_fn_ret = false;
            for (i, _) in looptris.iter().enumerate() {
                let f_test = face_of_tri[i];
                if f_test != f_test_prev {
                    test_fn_ret = test_fn(f_test);
                    f_test_prev = f_test;
                }
                if test_fn_ret {
                    passing.push(i);
                }
            }
        }
        None => passing.extend(0..looptris.len()),
    }

    let mut tree = kdopbvh::bvhtree_new(passing.len() as i32, epsilon, 8, 8);
    let mut tree_index_to_tri = Vec::with_capacity(passing.len());
    for &i in &passing {
        let cos = bmbvh_tri_from_face(looptris[i], cos_cage.as_ref(), cos_of);
        let flat: Vec<f32> = cos.iter().flatten().copied().collect();
        kdopbvh::bvhtree_insert(&mut tree, i as i32, &flat, 3);
        tree_index_to_tri.push(i);
    }
    kdopbvh::bvhtree_balance(&mut tree);

    BMBVHTree {
        looptris: looptris.to_vec(),
        face_of_tri: face_of_tri.to_vec(),
        cos_cage,
        flag,
        tree,
        tree_index_to_tri,
    }
}

/// `bm_face_is_select` / `bm_face_is_not_hidden` wrappers —
/// `BKE_bmbvh_new`'s flag dispatch (`flag_select(face)` and
/// `face_hidden(face)` injected).
pub fn bke_bmbvh_new(
    looptris: &[[usize; 3]],
    face_of_tri: &[usize],
    flag: i32,
    cos_cage: Option<Vec<[f32; 3]>>,
    face_selected: &dyn Fn(usize) -> bool,
    face_hidden: &dyn Fn(usize) -> bool,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> BMBVHTree {
    let mut test: Box<dyn FnMut(usize) -> bool> = if flag & BMBVH_RESPECT_SELECT != 0 {
        Box::new(move |f| face_selected(f))
    } else if flag & BMBVH_RESPECT_HIDDEN != 0 {
        Box::new(move |f| !face_hidden(f))
    } else {
        Box::new(|_| true)
    };
    let flag = flag & !(BMBVH_RESPECT_SELECT | BMBVH_RESPECT_HIDDEN);
    let has_filter = flag & BMBVH_RESPECT_SELECT != 0 || flag & BMBVH_RESPECT_HIDDEN != 0;
    let _ = has_filter;
    bke_bmbvh_new_ex(
        looptris,
        face_of_tri,
        flag,
        cos_cage,
        Some(&mut *test),
        cos_of,
    )
}

/// `BKE_bmbvh_tree_get`.
pub fn bke_bmbvh_tree_get(bmtree: &BMBVHTree) -> &BVHTree {
    &bmtree.tree
}

/// `bmbvh_ray_cast_handle_hit` — returns `(face, hitout, cagehit)`.
fn bmbvh_ray_cast_handle_hit(
    bmtree: &BMBVHTree,
    hit: &BVHTreeRayHit,
    uv: [f32; 2],
    want_hitout: bool,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> (usize, Option<[f32; 3]>, Option<[f32; 3]>) {
    let hitout = if want_hitout {
        if bmtree.flag & BMBVH_RETURN_ORIG != 0 {
            let ltri = bmtree.looptris[hit.index as usize];
            let co = [
                cos_of(ltri[0]),
                cos_of(ltri[1]),
                cos_of(ltri[2]),
            ];
            Some(vector::interp_v3_uv(
                Vec3::from(co[0]),
                Vec3::from(co[1]),
                Vec3::from(co[2]),
                Vec2::new(uv[0], uv[1]),
            )
            .into())
        } else {
            Some(hit.co)
        }
    } else {
        None
    };
    let cagehit = if want_hitout { Some(hit.co) } else { None };
    (bmtree.face_of_tri[hit.index as usize], hitout, cagehit)
}

/// `bmbvh_ray_cast_cb` — per-tri ray test.
fn bmbvh_ray_cast_cb(
    bmtree: &BMBVHTree,
    index: usize,
    ray_origin: [f32; 3],
    ray_direction: [f32; 3],
    ray_radius: f32,
    hit: &mut BVHTreeRayHit,
    uv_out: &mut [f32; 2],
    cos_of: &dyn Fn(usize) -> [f32; 3],
) {
    let ltri = bmtree.looptris[index];
    let tri_cos = bmbvh_tri_from_face(ltri, bmtree.cos_cage.as_ref(), cos_of);

    let result = if ray_radius > 0.0 {
        geom::isect_ray_tri_epsilon(
            Vec3::from(ray_origin),
            Vec3::from(ray_direction),
            Vec3::from(tri_cos[0]),
            Vec3::from(tri_cos[1]),
            Vec3::from(tri_cos[2]),
            ray_radius,
        )
    } else {
        geom::isect_ray_tri(
            Vec3::from(ray_origin),
            Vec3::from(ray_direction),
            Vec3::from(tri_cos[0]),
            Vec3::from(tri_cos[1]),
            Vec3::from(tri_cos[2]),
        )
    };

    if let Some((dist, u, v)) = result {
        if dist < hit.dist {
            hit.index = index as i32;
            hit.dist = dist;
            *uv_out = [u, v];
        }
    }
}

/// `BKE_bmbvh_ray_cast` — returns `(face, hitout, cagehit)` on hit.
pub fn bke_bmbvh_ray_cast(
    bmtree: &BMBVHTree,
    co: [f32; 3],
    dir: [f32; 3],
    radius: f32,
    r_dist: f32,
    want_hitout: bool,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> Option<(usize, f32, Option<[f32; 3]>, Option<[f32; 3]>)> {
    let mut uv = [0.0f32; 2];
    let mut hit = BVHTreeRayHit {
        index: -1,
        dist: r_dist,
        co: [0.0; 3],
        no: [0.0; 3],
    };

    kdopbvh::bvhtree_ray_cast(
        &bmtree.tree,
        co,
        dir,
        radius,
        Some(hit),
        Some(&mut |index: i32, ray: &kdopbvh::BVHTreeRay, h: &mut BVHTreeRayHit| {
            bmbvh_ray_cast_cb(
                bmtree,
                index as usize,
                ray.origin,
                ray.direction,
                ray.radius,
                h,
                &mut uv,
                cos_of,
            );
        }),
    )
    .1
    .tap(|h| hit = *h);

    if hit.index != -1 && hit.dist != r_dist {
        let (face, hitout, cagehit) =
            bmbvh_ray_cast_handle_hit(bmtree, &hit, uv, want_hitout, cos_of);
        return Some((face, hit.dist, hitout, cagehit));
    }
    None
}

/// `BKE_bmbvh_ray_cast_filter` — like ray_cast but skips tris whose
/// face fails `filter_cb`.
#[allow(clippy::too_many_arguments)]
pub fn bke_bmbvh_ray_cast_filter(
    bmtree: &BMBVHTree,
    co: [f32; 3],
    dir: [f32; 3],
    radius: f32,
    r_dist: f32,
    want_hitout: bool,
    filter_cb: &dyn Fn(usize) -> bool,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> Option<(usize, f32, Option<[f32; 3]>, Option<[f32; 3]>)> {
    let mut uv = [0.0f32; 2];
    let mut hit = BVHTreeRayHit {
        index: -1,
        dist: r_dist,
        co: [0.0; 3],
        no: [0.0; 3],
    };

    kdopbvh::bvhtree_ray_cast(
        &bmtree.tree,
        co,
        dir,
        radius,
        Some(hit),
        Some(&mut |index: i32, ray: &kdopbvh::BVHTreeRay, h: &mut BVHTreeRayHit| {
            if filter_cb(bmtree.face_of_tri[index as usize]) {
                bmbvh_ray_cast_cb(
                    bmtree,
                    index as usize,
                    ray.origin,
                    ray.direction,
                    ray.radius,
                    h,
                    &mut uv,
                    cos_of,
                );
            }
        }),
    )
    .1
    .tap(|h| hit = *h);

    if hit.index != -1 && hit.dist != r_dist {
        let (face, hitout, cagehit) =
            bmbvh_ray_cast_handle_hit(bmtree, &hit, uv, want_hitout, cos_of);
        return Some((face, hit.dist, hitout, cagehit));
    }
    None
}

/// `BKE_bmbvh_find_vert_closest` — `(vert_handle, co, dist_sq)`.
pub fn bke_bmbvh_find_vert_closest(
    bmtree: &BMBVHTree,
    co: [f32; 3],
    dist_max: f32,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> Option<(usize, [f32; 3], f32)> {
    let dist_max_sq = dist_max * dist_max;
    let mut index_tri = 0usize;
    let mut hit = BVHTreeNearest {
        index: -1,
        dist_sq: dist_max_sq,
        co: [0.0; 3],
        no: [0.0; 3],
    };

    let (_, nearest) = kdopbvh::bvhtree_find_nearest(
        &bmtree.tree,
        co,
        Some(hit),
        Some(&mut |index: i32, co_q: &[f32; 3], h: &mut BVHTreeNearest| {
            let ltri = bmtree.looptris[index as usize];
            let tri_cos = bmbvh_tri_from_face(ltri, bmtree.cos_cage.as_ref(), cos_of);
            for (i, tri_co) in tri_cos.iter().enumerate() {
                let dist_sq = Vec3::from(*co_q).distance_squared(Vec3::from(*tri_co));
                if dist_sq < h.dist_sq && dist_sq < dist_max_sq {
                    h.co = *tri_co;
                    h.dist_sq = dist_sq;
                    h.index = index;
                    index_tri = i;
                }
            }
        }),
    );
    hit = nearest;

    if hit.index != -1 {
        let ltri = bmtree.looptris[hit.index as usize];
        return Some((ltri[index_tri], hit.co, hit.dist_sq));
    }
    None
}

/// `BKE_bmbvh_find_face_closest` — `(face, dist_sq)`.
pub fn bke_bmbvh_find_face_closest(
    bmtree: &BMBVHTree,
    co: [f32; 3],
    dist_max: f32,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> Option<(usize, f32)> {
    let dist_max_sq = dist_max * dist_max;
    let hit = BVHTreeNearest {
        index: -1,
        dist_sq: dist_max_sq,
        co: [0.0; 3],
        no: [0.0; 3],
    };

    let (_, nearest) = kdopbvh::bvhtree_find_nearest(
        &bmtree.tree,
        co,
        Some(hit),
        Some(&mut |index: i32, co_q: &[f32; 3], h: &mut BVHTreeNearest| {
            let ltri = bmtree.looptris[index as usize];
            let tri_cos = bmbvh_tri_from_face(ltri, bmtree.cos_cage.as_ref(), cos_of);
            let (co_close, _) = geom::closest_on_tri_to_point(
                Vec3::from(*co_q),
                Vec3::from(tri_cos[0]),
                Vec3::from(tri_cos[1]),
                Vec3::from(tri_cos[2]),
            );
            let dist_sq = Vec3::from(*co_q).distance_squared(co_close);
            if dist_sq < h.dist_sq && dist_sq < dist_max_sq {
                h.dist_sq = dist_sq;
                h.index = index;
            }
        }),
    );

    if nearest.index != -1 {
        return Some((bmtree.face_of_tri[nearest.index as usize], nearest.dist_sq));
    }
    None
}

/// `bmbvh_overlap_cb` — tri/tri test; same-tree pairs sharing a face
/// or 2+ verts are skipped, single shared verts need a non-point
/// intersection.
fn bmbvh_overlap_cb(
    bmtree_a: &BMBVHTree,
    bmtree_b: &BMBVHTree,
    index_a: i32,
    index_b: i32,
    epsilon: f32,
    cos_of_a: &dyn Fn(usize) -> [f32; 3],
    cos_of_b: &dyn Fn(usize) -> [f32; 3],
) -> bool {
    let ltri_a = bmtree_a.looptris[index_a as usize];
    let ltri_b = bmtree_b.looptris[index_b as usize];
    let tri_a_co = bmbvh_tri_from_face(ltri_a, bmtree_a.cos_cage.as_ref(), cos_of_a);
    let tri_b_co = bmbvh_tri_from_face(ltri_b, bmtree_b.cos_cage.as_ref(), cos_of_b);

    let mut verts_shared = 0;

    if std::ptr::eq(&bmtree_a.looptris, &bmtree_b.looptris) {
        if bmtree_a.face_of_tri[index_a as usize] == bmtree_b.face_of_tri[index_b as usize] {
            return false;
        }
        verts_shared = tri_a_co.iter().filter(|a| tri_b_co.contains(a)).count();
        /* if 2 points are shared, bail out */
        if verts_shared >= 2 {
            return false;
        }
    }

    match geom::isect_tri_tri(
        &[
            Vec3::from(tri_a_co[0]),
            Vec3::from(tri_a_co[1]),
            Vec3::from(tri_a_co[2]),
        ],
        &[
            Vec3::from(tri_b_co[0]),
            Vec3::from(tri_b_co[1]),
            Vec3::from(tri_b_co[2]),
        ],
    ) {
        None => false,
        Some((p0, p1)) => {
            /* if we share a vertex, check the intersection isn't a
             * 'point' */
            verts_shared == 0 || p0.distance_squared(p1) > epsilon
        }
    }
}

/// `BKE_bmbvh_overlap` — overlaps between two trees.
#[allow(clippy::too_many_arguments)]
pub fn bke_bmbvh_overlap(
    bmtree_a: &BMBVHTree,
    bmtree_b: &BMBVHTree,
    cos_of_a: &dyn Fn(usize) -> [f32; 3],
    cos_of_b: &dyn Fn(usize) -> [f32; 3],
) -> Vec<BVHTreeOverlap> {
    let epsilon = kdopbvh::bvhtree_get_epsilon(&bmtree_a.tree)
        .max(kdopbvh::bvhtree_get_epsilon(&bmtree_b.tree));
    kdopbvh::bvhtree_overlap(
        &bmtree_a.tree,
        &bmtree_b.tree,
        Some(&mut |index_a, index_b, _thread| {
            bmbvh_overlap_cb(bmtree_a, bmtree_b, index_a, index_b, epsilon, cos_of_a, cos_of_b)
        }),
    )
}

/// `BKE_bmbvh_overlap_self` — `index_a < index_b` pairs only.
pub fn bke_bmbvh_overlap_self(
    bmtree: &BMBVHTree,
    cos_of: &dyn Fn(usize) -> [f32; 3],
) -> Vec<BVHTreeOverlap> {
    let epsilon = kdopbvh::bvhtree_get_epsilon(&bmtree.tree);
    kdopbvh::bvhtree_overlap_self(&bmtree.tree, Some(&mut |index_a, index_b, _thread| {
        index_a < index_b
            && bmbvh_overlap_cb(bmtree, bmtree, index_a, index_b, epsilon, cos_of, cos_of)
    }))
}

/// `tap` helper.
trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&mut Self)) -> Self {
        let mut s = self;
        f(&mut s);
        s
    }
}
impl<T> Tap for T {}
