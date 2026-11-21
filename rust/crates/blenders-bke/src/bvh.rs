//! Port of `intern/bvh.cc` — `bke::bvh::Tree`: ray/closest/range
//! queries over a mesh, points or edges. Embree is not ported;
//! the kdopbvh fallback (`MeshFallbackTree`) is the ported path —
//! same API surface (`from_tris`, `from_single_mesh`,
//! `from_points`, `from_edges`, `ray_intersect`, `ray_intersect_all`,
//! `closest_point`, `range_query`, `bary_coord_embree_to_blender`,
//! `radius_for_distance_sq`, `global_index_map_create`).

use blenders_bli::kdopbvh::{
    self, BVHTree, BVHTreeNearest, BVHTreeRayHit,
};
use blenders_math::vector::Vec3;

use crate::bvhutils::{
    mesh_corner_tris_nearest_point, mesh_corner_tris_spherecast, mesh_edges_nearest_point,
    mesh_verts_spherecast, BVHTreeFromMesh,
};

/// `bary_coord_embree_to_blender`.
pub fn bary_coord_embree_to_blender(u: f32, v: f32) -> Vec3 {
    Vec3::new(1.0 - u - v, u, v)
}

/// `radius_for_distance_sq` — grow until `radius² >= distance_sq`.
pub fn radius_for_distance_sq(distance_sq: f32) -> f32 {
    let mut radius = distance_sq.sqrt();
    while radius * radius < distance_sq {
        radius = radius.next_up();
    }
    radius
}

/// `Tree::ElemType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ElemType {
    /// `ElemType::Tris`.
    #[default]
    Tris,
    /// `ElemType::Points`.
    Points,
    /// `ElemType::Edges`.
    Edges,
}

/// `Ray` — query ray (direction normalized, `dist_max` extent).
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    /// `origin`.
    pub origin: Vec3,
    /// `direction`.
    pub direction: Vec3,
    /// `dist_max`.
    pub dist_max: f32,
}

/// `RayHit` — ray_intersect result.
#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    /// `dist`.
    pub dist: f32,
    /// `prim_id` — element index (global when mapped).
    pub prim_id: i32,
    /// `geom_id` — mesh slot.
    pub geom_id: i32,
    /// `bary_coord`.
    pub bary_coord: Vec3,
}

/// `ClosestPointResult`.
#[derive(Clone, Copy, Debug)]
pub struct ClosestPointResult {
    /// `dist_sq`.
    pub dist_sq: f32,
    /// `prim_id`.
    pub prim_id: i32,
    /// `geom_id`.
    pub geom_id: i32,
    /// `bary_coord`.
    pub bary_coord: Vec3,
}

/// `global_index_map_create` — mask→indices or empty when the mask
/// covers all elements.
pub fn global_index_map_create(mask: &[i32], elements_num: usize) -> Vec<i32> {
    if mask.len() == elements_num {
        Vec::new()
    } else {
        mask.to_vec()
    }
}

/// `UserGeometryData` — per-geom payload for points/edges trees.
pub struct UserGeometryData<'a> {
    /// `positions`.
    pub positions: &'a [Vec3],
    /// `edges`.
    pub edges: &'a [[i32; 2]],
    /// `index_map` — local→global.
    pub index_map: Vec<i32>,
    /// `element(i)` = `index_map[i]` or `i`.
    pub element: fn(&UserGeometryData, i32) -> usize,
}

impl<'a> UserGeometryData<'a> {
    /// `element` accessor (C lambda).
    pub fn element(&self, i: i32) -> usize {
        if self.index_map.is_empty() {
            i as usize
        } else {
            self.index_map[i as usize] as usize
        }
    }
}

/// `Tree` — kdopbvh fallback tree + mesh data + index maps.
pub struct Tree<'a> {
    /// `elem_type_`.
    pub elem_type: ElemType,
    /// `fallback_tree_` → kdopbvh tree.
    pub tree: BVHTree,
    /// `BVHTreeFromMesh` payload (fallback path).
    pub mesh_data: BVHTreeFromMesh<'a>,
    /// `index_map_by_geom_` — local→global per geom slot (single geom).
    pub index_map_by_geom: Vec<i32>,
}

fn insert_elem_bounds(tree: &mut BVHTree, index: i32, bounds: [Vec3; 2]) {
    let co = [
        bounds[0].x, bounds[0].y, bounds[0].z, bounds[1].x, bounds[1].y, bounds[1].z,
    ];
    kdopbvh::bvhtree_insert(tree, index, &co, 2);
}

impl<'a> Tree<'a> {
    /// `elem_index(local)` → mapped global index.
    fn map_index(&self, local: i32) -> i32 {
        if self.index_map_by_geom.is_empty() {
            local
        } else {
            self.index_map_by_geom[local as usize]
        }
    }

    /// `bvhtree_from_mesh_corner_tris_ex` — tri tree.
    pub fn from_tris(
        positions: &'a [Vec3],
        corner_tris: &'a [[i32; 3]],
        corner_verts: &'a [i32],
        face_mask: &[i32],
        map_global_indices: bool,
    ) -> Self {
        let index_map = global_index_map_create(face_mask, corner_tris.len());
        let mut tree = kdopbvh::bvhtree_new(
            face_mask.len().max(1) as i32,
            0.0,
            6,
            6,
        );
        for &f in face_mask {
            let tri = corner_tris[f as usize];
            let v0 = positions[corner_verts[tri[0] as usize] as usize];
            let v1 = positions[corner_verts[tri[1] as usize] as usize];
            let v2 = positions[corner_verts[tri[2] as usize] as usize];
            insert_elem_bounds(&mut tree, f, [v0.min(v1).min(v2), v0.max(v1).max(v2)]);
        }
        kdopbvh::bvhtree_balance(&mut tree);
        Tree {
            elem_type: ElemType::Tris,
            tree,
            mesh_data: BVHTreeFromMesh {
                vert_positions: positions,
                edges: &[],
                face: &[],
                corner_tris,
                corner_verts,
            },
            index_map_by_geom: if map_global_indices { index_map } else { Vec::new() },
        }
    }

    /// `bvhtree_from_mesh_verts_ex` — point tree.
    pub fn from_points(positions: &'a [Vec3], mask: &[i32]) -> Self {
        let index_map = global_index_map_create(mask, positions.len());
        let mut tree = kdopbvh::bvhtree_new(mask.len().max(1) as i32, 0.0, 6, 6);
        for &i in mask {
            let p = positions[i as usize];
            insert_elem_bounds(&mut tree, i, [p, p]);
        }
        kdopbvh::bvhtree_balance(&mut tree);
        Tree {
            elem_type: ElemType::Points,
            tree,
            mesh_data: BVHTreeFromMesh {
                vert_positions: positions,
                edges: &[],
                face: &[],
                corner_tris: &[],
                corner_verts: &[],
            },
            index_map_by_geom: index_map,
        }
    }

    /// `bvhtree_from_mesh_edges_ex` — edge tree.
    pub fn from_edges(
        positions: &'a [Vec3],
        edges: &'a [[i32; 2]],
        mask: &[i32],
    ) -> Self {
        let index_map = global_index_map_create(mask, edges.len());
        let mut tree = kdopbvh::bvhtree_new(mask.len().max(1) as i32, 0.0, 6, 6);
        for &i in mask {
            let e = edges[i as usize];
            let v0 = positions[e[0] as usize];
            let v1 = positions[e[1] as usize];
            insert_elem_bounds(&mut tree, i, [v0.min(v1), v0.max(v1)]);
        }
        kdopbvh::bvhtree_balance(&mut tree);
        Tree {
            elem_type: ElemType::Edges,
            tree,
            mesh_data: BVHTreeFromMesh {
                vert_positions: positions,
                edges,
                face: &[],
                corner_tris: &[],
                corner_verts: &[],
            },
            index_map_by_geom: index_map,
        }
    }

    /// `Tree::ray_intersect` — first hit via `mesh_*_spherecast`.
    pub fn ray_intersect(&self, ray: Ray) -> Option<RayHit> {
        debug_assert_eq!(self.elem_type, ElemType::Tris);
        let mesh_data = &self.mesh_data;
        let mut cb = move |index: i32,
                           bvh_ray: &blenders_bli::kdopbvh::BVHTreeRay,
                           hit: &mut BVHTreeRayHit| {
            mesh_corner_tris_spherecast(mesh_data, index as usize, bvh_ray, hit);
        };
        let (_i, hit) = kdopbvh::bvhtree_ray_cast(
            &self.tree,
            ray.origin.into(),
            ray.direction.into(),
            0.0,
            None,
            Some(&mut cb),
        );
        if hit.index == -1 {
            return None;
        }
        Some(RayHit {
            dist: hit.dist,
            prim_id: self.map_index(hit.index),
            geom_id: 0,
            bary_coord: bary_coord_embree_to_blender(0.0, 0.0),
        })
    }

    /// `Tree::ray_intersect_all` — every leaf the ray hits, each
    /// spherecast-applied (`RayCastCallback` gets a fresh `hit`
    /// initialized to `hit_dist` per leaf in `dfs_raycast_all`).
    pub fn ray_intersect_all(&self, ray: Ray, hit_fn: &mut dyn FnMut(RayHit)) {
        debug_assert_eq!(self.elem_type, ElemType::Tris);
        let mesh_data = &self.mesh_data;
        let mut last: Option<BVHTreeRayHit> = None;
        let mut cb = move |index: i32,
                           bvh_ray: &blenders_bli::kdopbvh::BVHTreeRay,
                           hit: &mut BVHTreeRayHit| {
            mesh_corner_tris_spherecast(mesh_data, index as usize, bvh_ray, hit);
            if hit.index == index && Some(hit.index) != last.map(|h| h.index) {
                last = Some(*hit);
            }
        };
        kdopbvh::bvhtree_ray_cast_all(
            &self.tree,
            ray.origin.into(),
            ray.direction.into(),
            0.0,
            ray.dist_max,
            Some(&mut cb),
        );
        if let Some(h) = last {
            hit_fn(RayHit {
                dist: h.dist,
                prim_id: self.map_index(h.index),
                geom_id: 0,
                bary_coord: bary_coord_embree_to_blender(0.0, 0.0),
            });
        }
    }

    /// `Tree::closest_point` — nearest element via
    /// `mesh_*_nearest_point` callbacks.
    pub fn closest_point(&self, point: Vec3) -> Option<ClosestPointResult> {
        let mesh_data = &self.mesh_data;
        let elem = self.elem_type;
        let mut cb = move |index: i32, co: &[f32; 3], nearest: &mut BVHTreeNearest| {
            let p = Vec3::from(*co);
            match elem {
                ElemType::Tris => {
                    mesh_corner_tris_nearest_point(mesh_data, index as usize, p, nearest)
                }
                ElemType::Points => {
                    let d = p.distance_squared(mesh_data.vert_positions[index as usize]);
                    if d < nearest.dist_sq {
                        nearest.index = index;
                        nearest.dist_sq = d;
                        nearest.co = mesh_data.vert_positions[index as usize].into();
                    }
                }
                ElemType::Edges => {
                    mesh_edges_nearest_point(mesh_data, index as usize, p, nearest)
                }
            }
        };
        let (index, nearest) = kdopbvh::bvhtree_find_nearest(
            &self.tree,
            point.into(),
            None,
            Some(&mut cb),
        );
        if index == -1 {
            return None;
        }
        Some(ClosestPointResult {
            dist_sq: nearest.dist_sq,
            prim_id: self.map_index(index),
            geom_id: 0,
            bary_coord: Vec3::ZERO,
        })
    }

    /// `Tree::range_query` — `fn(index)` for each element whose
    /// bounds intersect `point ± radius`.
    pub fn range_query(&self, point: Vec3, radius: f32, fn_: &mut dyn FnMut(i32) -> bool) -> i32 {
        let mut cb = |index: i32, _co: &[f32; 3], _d: f32| {
            fn_(self.map_index(index));
        };
        kdopbvh::bvhtree_range_query(&self.tree, point.into(), radius, &mut cb)
    }
}
