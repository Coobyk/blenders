//! Port of `intern/volume_render.cc` — dense-voxel extraction,
//! wireframe/selection-surface mesh building, and
//! `BKE_volume_density_scale`.
//!
//! OpenVDB internals (`copyToDense`, `evalLeafBoundingBox`, tree node
//! iteration, `Transform::indexToWorld`) are injected via closures/
//! handles — everything else is ported faithfully.

use blenders_math::matrix::Mat4;
use blenders_math::vector::{Vec3, Vec4};

use blenders_dna::volume_types::{
    VOLUME_SPACE_OBJECT, VOLUME_WIREFRAME_BOUNDS, VOLUME_WIREFRAME_COARSE, VOLUME_WIREFRAME_NONE,
    VOLUME_WIREFRAME_POINTS,
};

/// `openvdb::CoordBBox` — integer min/max (inclusive) index-space box.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CoordBBox {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl CoordBBox {
    pub fn dim(&self) -> [i32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
    pub fn volume(&self) -> i64 {
        let d = self.dim();
        d[0] as i64 * d[1] as i64 * d[2] as i64
    }
    pub fn center(&self) -> [f64; 3] {
        [
            (self.min[0] as f64 + self.max[0] as f64) * 0.5,
            (self.min[1] as f64 + self.max[1] as f64) * 0.5,
            (self.min[2] as f64 + self.max[2] as f64) * 0.5,
        ]
    }
    /// `getCornerPoints` — lexicographic corner ordering.
    pub fn corner_points(&self) -> [[i32; 3]; 8] {
        let (mn, mx) = (self.min, self.max);
        [
            [mn[0], mn[1], mn[2]],
            [mx[0], mn[1], mn[2]],
            [mn[0], mx[1], mn[2]],
            [mx[0], mx[1], mn[2]],
            [mn[0], mn[1], mx[2]],
            [mx[0], mn[1], mx[2]],
            [mn[0], mx[1], mx[2]],
            [mx[0], mx[1], mx[2]],
        ]
    }
}

/// `DenseFloatVolumeGrid`.
pub struct DenseFloatVolumeGrid {
    /// Dense `channels`-interleaved voxel data.
    pub voxels: Vec<f32>,
    pub channels: i32,
    pub resolution: [i32; 3],
    pub texture_to_object: Mat4,
}

impl Default for DenseFloatVolumeGrid {
    fn default() -> Self {
        Self {
            voxels: Vec::new(),
            channels: 0,
            resolution: [0; 3],
            texture_to_object: Mat4::IDENTITY,
        }
    }
}

/// `create_texture_to_object_matrix` — index→object transform times
/// a texture-space→index scale/offset matrix.
///
/// `index_to_object` = grid transform (`openvdb::Mat4d`), injected.
pub fn create_texture_to_object_matrix(
    index_to_object: &Mat4,
    bbox: &CoordBBox,
) -> Mat4 {
    let loc = Vec3::new(
        bbox.min[0] as f32 - 0.5,
        bbox.min[1] as f32 - 0.5,
        bbox.min[2] as f32 - 0.5,
    );
    let dim = bbox.dim();
    let size = Vec3::new(dim[0] as f32, dim[1] as f32, dim[2] as f32);
    /* `size_to_mat4` + translation. */
    let mut texture_to_index = Mat4::IDENTITY;
    texture_to_index[0][0] = size.x;
    texture_to_index[1][1] = size.y;
    texture_to_index[2][2] = size.z;
    texture_to_index[3] = Vec4::new(loc.x, loc.y, loc.z, 1.0);
    *index_to_object * texture_to_index
}

/// `BKE_volume_grid_dense_floats` — bounds checks + channel count
/// (extraction via the injected `copy_to_dense`; returns `None` on
/// empty bbox / over-large indices as the C does).
pub fn bke_volume_grid_dense_floats(
    eval_active_voxel_bbox: &dyn Fn() -> Option<CoordBBox>,
    channels: i32,
    index_to_object: &Mat4,
    copy_to_dense: &dyn Fn(&CoordBBox, &mut [f32]),
) -> Option<DenseFloatVolumeGrid> {
    let bbox = eval_active_voxel_bbox()?;
    if bbox.volume() <= 0 {
        return None;
    }
    let mut max_bbox_index: i64 = 0;
    for i in 0..3 {
        max_bbox_index = max_bbox_index
            .max((bbox.min[i] as i64).abs())
            .max((bbox.max[i] as i64).abs());
    }
    if max_bbox_index > (1 << 30) {
        /* There is an integer overflow when trying to extract dense
         * voxels when the indices are very large. */
        return None;
    }

    let resolution = bbox.dim();
    let num_voxels = bbox.volume();
    let mut voxels = vec![0.0f32; channels as usize * num_voxels as usize];
    copy_to_dense(&bbox, &mut voxels);

    Some(DenseFloatVolumeGrid {
        voxels,
        channels,
        resolution,
        texture_to_object: create_texture_to_object_matrix(index_to_object, &bbox),
    })
}

/* Wireframe */

const BOX_EDGES: [[i32; 2]; 12] = [
    [0, 1], [0, 2], [0, 4], [1, 3], [1, 5], [2, 3], [2, 6], [3, 7],
    [4, 5], [4, 6], [5, 7], [6, 7],
];

const BOX_TRIS: [[i32; 3]; 12] = [
    [0, 1, 4], [4, 1, 5], [0, 2, 1], [1, 2, 3], [1, 3, 5], [5, 3, 7],
    [6, 4, 5], [7, 5, 6], [2, 0, 4], [2, 4, 6], [3, 7, 2], [6, 2, 7],
];

/// `boxes_to_center_points` — `transform` = index→world.
fn boxes_to_center_points(
    boxes: &[CoordBBox],
    index_to_world: &dyn Fn([f64; 3]) -> Vec3,
    r_verts: &mut [Vec3],
) {
    debug_assert_eq!(boxes.len(), r_verts.len());
    for (i, box_) in boxes.iter().enumerate() {
        r_verts[i] = index_to_world(box_.center());
    }
}

/// `boxes_to_corner_points`.
fn boxes_to_corner_points(
    boxes: &[CoordBBox],
    index_to_world: &dyn Fn([f64; 3]) -> Vec3,
    r_verts: &mut [Vec3],
) {
    debug_assert_eq!(boxes.len() * 8, r_verts.len());
    for (i, box_) in boxes.iter().enumerate() {
        /* The ordering of the corner points is lexicographic. */
        for (j, corner_i) in box_.corner_points().iter().enumerate() {
            r_verts[8 * i + j] =
                index_to_world([corner_i[0] as f64, corner_i[1] as f64, corner_i[2] as f64]);
        }
    }
}

/// `boxes_to_edge_mesh`.
pub fn boxes_to_edge_mesh(
    boxes: &[CoordBBox],
    index_to_world: &dyn Fn([f64; 3]) -> Vec3,
    r_verts: &mut Vec<Vec3>,
    r_edges: &mut Vec<[i32; 2]>,
) {
    /* TODO: Deduplicate edges, hide flat edges? */
    let mut vert_offset = r_verts.len() as i32;
    let vert_amount = 8 * boxes.len();
    let vert_start = r_verts.len();
    r_verts.resize(r_verts.len() + vert_amount, Vec3::splat(0.0));
    boxes_to_corner_points(boxes, index_to_world, &mut r_verts[vert_start..]);

    r_edges.reserve(12 * boxes.len());
    for _ in boxes {
        for j in 0..12 {
            r_edges.push([
                vert_offset + BOX_EDGES[j][0],
                vert_offset + BOX_EDGES[j][1],
            ]);
        }
        vert_offset += 8;
    }
}

/// `boxes_to_cube_mesh`.
pub fn boxes_to_cube_mesh(
    boxes: &[CoordBBox],
    index_to_world: &dyn Fn([f64; 3]) -> Vec3,
    r_verts: &mut Vec<Vec3>,
    r_tris: &mut Vec<[i32; 3]>,
) {
    let mut vert_offset = r_verts.len() as i32;
    let vert_amount = 8 * boxes.len();
    let vert_start = r_verts.len();
    r_verts.resize(r_verts.len() + vert_amount, Vec3::splat(0.0));
    boxes_to_corner_points(boxes, index_to_world, &mut r_verts[vert_start..]);

    r_tris.reserve(12 * boxes.len());
    for _ in boxes {
        for j in 0..12 {
            r_tris.push([
                vert_offset + BOX_TRIS[j][0],
                vert_offset + BOX_TRIS[j][1],
                vert_offset + BOX_TRIS[j][2],
            ]);
        }
        vert_offset += 8;
    }
}

/// `BKE_volume_grid_wireframe` — dispatch on `wireframe_type`;
/// `get_bounding_boxes`/`leaf_bbox`/`index_to_world` inject the
/// OpenVDB side.
pub fn bke_volume_grid_wireframe(
    wireframe_type: i32,
    wireframe_detail: i32,
    leaf_bbox: &dyn Fn() -> Option<CoordBBox>,
    get_bounding_boxes: &dyn Fn(bool) -> Vec<CoordBBox>,
    index_to_world: &dyn Fn([f64; 3]) -> Vec3,
) -> (Vec<Vec3>, Vec<[i32; 2]>) {
    let mut verts = Vec::new();
    let mut edges = Vec::new();
    if wireframe_type == VOLUME_WIREFRAME_NONE {
        return (verts, edges);
    }

    if wireframe_type == VOLUME_WIREFRAME_BOUNDS {
        /* Bounding box. */
        if let Some(box_) = leaf_bbox() {
            boxes_to_edge_mesh(&[box_], index_to_world, &mut verts, &mut edges);
        }
    } else {
        let boxes = get_bounding_boxes(wireframe_detail == VOLUME_WIREFRAME_COARSE);
        if wireframe_type == VOLUME_WIREFRAME_POINTS {
            verts.resize(boxes.len(), Vec3::splat(0.0));
            boxes_to_center_points(&boxes, index_to_world, &mut verts);
        } else {
            boxes_to_edge_mesh(&boxes, index_to_world, &mut verts, &mut edges);
        }
    }
    (verts, edges)
}

/// `grow_triangles` — lengthen all connected edges by `factor` to
/// reduce outline artifacts.
pub fn grow_triangles(verts: &mut [Vec3], tris: &[[i32; 3]], factor: f32) {
    /* Compute the offset for every vertex based on the connected
     * edges. This formula simply tries increases the length of all
     * edges. */
    let mut offsets = vec![Vec3::splat(0.0); verts.len()];
    let mut weights = vec![0.0f32; verts.len()];
    for tri in tris {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        offsets[a] += (verts[a] * 2.0 - verts[b] - verts[c]) * factor;
        offsets[b] += (verts[b] * 2.0 - verts[a] - verts[c]) * factor;
        offsets[c] += (verts[c] * 2.0 - verts[a] - verts[b]) * factor;
        weights[a] += 1.0;
        weights[b] += 1.0;
        weights[c] += 1.0;
    }
    /* Apply the computed offsets. */
    for i in 0..verts.len() {
        if weights[i] > 0.0 {
            verts[i] += offsets[i] / weights[i];
        }
    }
}

/// `BKE_volume_grid_selection_surface` — coarse boxes → cube mesh →
/// `grow_triangles(0.01)`.
pub fn bke_volume_grid_selection_surface(
    get_bounding_boxes: &dyn Fn(bool) -> Vec<CoordBBox>,
    index_to_world: &dyn Fn([f64; 3]) -> Vec3,
) -> (Vec<Vec3>, Vec<[i32; 3]>) {
    let boxes = get_bounding_boxes(true);
    let mut verts = Vec::new();
    let mut tris = Vec::new();
    boxes_to_cube_mesh(&boxes, index_to_world, &mut verts, &mut tris);
    /* By slightly scaling the individual boxes up, we can avoid some
     * artifacts when drawing the selection outline. */
    const OFFSET_FACTOR: f32 = 0.01;
    grow_triangles(&mut verts, &tris, OFFSET_FACTOR);
    (verts, tris)
}

/* Render */

/// `BKE_volume_density_scale` — inverse of object-space unit scale
/// (`VOLUME_SPACE_WORLD` → 1).
pub fn bke_volume_density_scale(volume_space: i32, matrix: &Mat4) -> f32 {
    if volume_space == VOLUME_SPACE_OBJECT {
        let unit = Vec3::splat(1.0).normalized();
        let scaled = matrix.transform_direction(unit);
        1.0 / scaled.length()
    } else {
        1.0
    }
}
