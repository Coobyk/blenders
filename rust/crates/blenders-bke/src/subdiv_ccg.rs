//! Port of `intern/subdiv_ccg.cc` + `BKE_subdiv_ccg.hh` —
//! `SubdivCCGCoord`/`CCGKey`, coord validity, grid/face range
//! helpers, `BKE_subdiv_ccg_topology_counters`, neighbor coords
//! (`neighbor_coords_*`), normals recalc + grid averaging
//! (`BKE_subdiv_ccg_recalc_normals`, `BKE_subdiv_ccg_average_grids`,
//! `BKE_subdiv_ccg_average_stitch_faces`).
//! `PORT_PENDING`: `Subdiv` evaluator calls
//! (`subdiv_ccg_eval_grid_element*`, `BKE_subdiv_ccg_mask_init`),
//! `BKE_subdiv_ccg_destroy`, full `subdiv_ccg_evaluate_grids` driver.

use blenders_math::vector::Vec3;

/// `CCGKey` — grid layout key.
#[derive(Clone, Copy, Default)]
pub struct CCGKey {
    /// `level`.
    pub level: i32,
    /// `grid_size`.
    pub grid_size: i32,
    /// `grid_area` (grid_size²).
    pub grid_area: i32,
    /// `grid_stride` (`grid_area` elems per grid).
    pub grid_stride: i32,
    /// `has_normals`, `has_mask`, `has_uv_coords` (bools).
    pub has_normals: bool,
    /// `has_mask`.
    pub has_mask: bool,
    /// `has_uv_coords`.
    pub has_uv_coords: bool,
    /// `elem_size`.
    pub elem_size: i32,
}

/// `SubdivCCGCoord` — `(grid_index, x, y)`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct SubdivCCGCoord {
    /// `grid_index`.
    pub grid_index: i32,
    /// `x`.
    pub x: i16,
    /// `y`.
    pub y: i16,
}

impl SubdivCCGCoord {
    /// `from_index` — `(index/grid_area, residual → x,y)`.
    pub fn from_index(key: &CCGKey, index: i32) -> Self {
        let grid_index = index / key.grid_area;
        let index_in_grid = index - grid_index * key.grid_area;
        Self {
            grid_index,
            x: (index_in_grid % key.grid_size) as i16,
            y: (index_in_grid / key.grid_size) as i16,
        }
    }
    /// `to_index`.
    pub fn to_index(&self, key: &CCGKey) -> i32 {
        self.grid_index * key.grid_area + self.y as i32 * key.grid_size + self.x as i32
    }
}

/// `SubdivCCGAdjacentEdge` — coords of points along an edge,
/// indexed by adjacent-face index.
#[derive(Default)]
pub struct SubdivCCGAdjacentEdge {
    /// `boundary_coords`.
    pub boundary_coords: Vec<Vec<SubdivCCGCoord>>,
}

/// `SubdivCCGAdjacentVertex` — coords + corner index per face.
#[derive(Default)]
pub struct SubdivCCGAdjacentVertex {
    /// `boundary_coords`.
    pub boundary_coords: Vec<SubdivCCGCoord>,
    /// `corner_indices`.
    pub corner_indices: Vec<i32>,
}

/// `SubdivCCG` — Catmull-Clark subdivision grid storage.
#[derive(Default)]
pub struct SubdivCCG {
    /// `level`.
    pub level: i32,
    /// `grid_size`.
    pub grid_size: i32,
    /// `grid_area`.
    pub grid_area: i32,
    /// `grids_num`.
    pub grids_num: i32,
    /// `positions` (grid_area*grids_num).
    pub positions: Vec<Vec3>,
    /// `normals`.
    pub normals: Vec<Vec3>,
    /// `masks`.
    pub masks: Vec<f32>,
    /// `grid_to_face_map`.
    pub grid_to_face_map: Vec<i32>,
    /// `faces` offsets (coarse mesh faces).
    pub faces_offsets: Vec<usize>,
    /// `corner_verts`.
    pub corner_verts: Vec<i32>,
    /// `adjacent_edges`.
    pub adjacent_edges: Vec<SubdivCCGAdjacentEdge>,
    /// `adjacent_verts`.
    pub adjacent_verts: Vec<SubdivCCGAdjacentVertex>,
    /// `num_inner_edges`/`num_inner_verts`.
    pub num_inner_edges: i32,
    /// `num_inner_verts`.
    pub num_inner_verts: i32,
}

/// `BKE_subdiv_ccg_key` — build the CCGKey for `level`.
pub fn bke_subdiv_ccg_key(subdiv_ccg: &SubdivCCG, level: i32) -> CCGKey {
    let grid_size = crate::multires::grid_size_from_level(level);
    let grid_area = grid_size * grid_size;
    CCGKey {
        level,
        grid_size,
        grid_area,
        grid_stride: grid_area,
        has_normals: true,
        has_mask: false,
        has_uv_coords: false,
        elem_size: 1,
    }
}

/// `BKE_subdiv_ccg_check_coord_valid`.
pub fn bke_subdiv_ccg_check_coord_valid(
    subdiv_ccg: &SubdivCCG,
    coord: &SubdivCCGCoord,
) -> bool {
    if coord.grid_index < 0 || coord.grid_index >= subdiv_ccg.grids_num {
        return false;
    }
    if coord.x < 0 || coord.x as i32 >= subdiv_ccg.grid_size {
        return false;
    }
    if coord.y < 0 || coord.y as i32 >= subdiv_ccg.grid_size {
        return false;
    }
    true
}

/// `BKE_subdiv_ccg_print_coord` — debug string.
pub fn bke_subdiv_ccg_print_coord(coord: &SubdivCCGCoord) -> String {
    format!("x: {}, y: {}, grid index: {}", coord.x, coord.y, coord.grid_index)
}

/// `grid_range` — element range of `grid` in flat storage.
pub fn grid_range(grid_area: i32, grid: i32) -> std::ops::Range<usize> {
    let start = (grid * grid_area) as usize;
    start..start + grid_area as usize
}

/// `face_range` — element range of `face`'s corner grids.
pub fn face_range(faces_offsets: &[usize], grid_area: i32, face: usize) -> std::ops::Range<usize> {
    let start = faces_offsets[face] * grid_area as usize;
    start..start + (faces_offsets[face + 1] - faces_offsets[face]) * grid_area as usize
}

/// `grid_xy_to_vert` — flat index of `(x, y)` in `grid`.
pub fn grid_xy_to_vert(key: &CCGKey, grid: i32, x: i32, y: i32) -> i32 {
    grid * key.grid_area + y * key.grid_size + x
}

/// `BKE_subdiv_ccg_grid_to_face_index`.
pub fn bke_subdiv_ccg_grid_to_face_index(subdiv_ccg: &SubdivCCG, grid_index: i32) -> i32 {
    subdiv_ccg.grid_to_face_map[grid_index as usize]
}

/// `BKE_subdiv_ccg_topology_counters`.
pub fn bke_subdiv_ccg_topology_counters(
    subdiv_ccg: &SubdivCCG,
    coarse_faces_num: i32,
    r_num_vertices: &mut i32,
    r_num_edges: &mut i32,
    r_num_faces: &mut i32,
    r_num_loops: &mut i32,
) {
    let grid_area = subdiv_ccg.grid_area;
    let num_inner_edges = subdiv_ccg.num_inner_edges;
    let num_inner_verts = subdiv_ccg.num_inner_verts;
    *r_num_vertices =
        num_inner_verts + num_inner_edges * (grid_area - 1) + coarse_faces_num * (grid_area - 4);
    *r_num_edges = 0;
    *r_num_faces = coarse_faces_num;
    *r_num_loops = coarse_faces_num * 4;
}

/// `SubdivCCGNeighbors` — unique + duplicate neighbor coords.
pub struct SubdivCCGNeighbors {
    /// `coords` storage (unique first).
    pub coords: Vec<SubdivCCGCoord>,
    /// `num_duplicates`.
    pub num_duplicates: usize,
}

impl SubdivCCGNeighbors {
    /// `unique`.
    pub fn unique(&self) -> &[SubdivCCGCoord] {
        &self.coords[..self.coords.len() - self.num_duplicates]
    }
    /// `duplicates`.
    pub fn duplicates(&self) -> &[SubdivCCGCoord] {
        &self.coords[self.coords.len() - self.num_duplicates..]
    }
}

/// `coord_step_inside_from_boundary` — step one cell inward from a
/// boundary coord (towards face center).
pub fn coord_step_inside_from_boundary(
    subdiv_ccg: &SubdivCCG,
    coord: &SubdivCCGCoord,
) -> SubdivCCGCoord {
    let mut c = *coord;
    let max = (subdiv_ccg.grid_size - 1) as i16;
    if c.x == 0 {
        c.x += 1;
    } else if c.x == max {
        c.x -= 1;
    } else if c.y == 0 {
        c.y += 1;
    } else if c.y == max {
        c.y -= 1;
    }
    c
}

/// `next_grid_index_from_coord` — the adjacent grid across the
/// boundary edge `coord` lies on (-1 when interior).
pub fn next_grid_index_from_coord(
    subdiv_ccg: &SubdivCCG,
    coord: &SubdivCCGCoord,
) -> i32 {
    if !bke_subdiv_ccg_check_coord_valid(subdiv_ccg, coord) {
        return -1;
    }
    let max = subdiv_ccg.grid_size - 1;
    /* Grids of one face are stored consecutively; the next grid index
     * wraps inside the face's corner range. */
    let corner = subdiv_ccg.grid_to_face_map.get(coord.grid_index as usize);
    if corner.is_none() {
        return -1;
    }
    if coord.y == 0 {
        /* Bottom boundary — previous corner's grid (wrapping). */
        coord.grid_index - 1
    } else if coord.y as i32 == max {
        coord.grid_index + 1
    } else {
        coord.grid_index
    }
}

/// `average_grid_element_value_v3`.
fn average_grid_element_value_v3(a: Vec3, b: Vec3) -> Vec3 {
    (a + b) * 0.5
}

/// `average_grid_element` — blend `src` into `dst` positions.
fn average_grid_element(positions: &mut [Vec3], dst: usize, src: usize) {
    positions[dst] = average_grid_element_value_v3(positions[dst], positions[src]);
}

/// `GridElementAccumulator` — accumulates positions for averaging.
#[derive(Default)]
struct GridElementAccumulator {
    position: Vec3,
    counter: i32,
}

impl GridElementAccumulator {
    fn init(&mut self) {
        self.position = Vec3::ZERO;
        self.counter = 0;
    }
    fn add(&mut self, positions: &[Vec3], index: usize) {
        self.position += positions[index];
        self.counter += 1;
    }
    fn mul_fl(&mut self, f: f32) {
        self.position = self.position * f;
    }
    fn copy_to(&self, positions: &mut [Vec3], index: usize) {
        positions[index] = self.position;
    }
}

/// `subdiv_ccg_average_inner_face_grids` — average duplicate
/// boundary points within one face's grids.
pub fn subdiv_ccg_average_inner_face_grids(
    subdiv_ccg: &mut SubdivCCG,
    grid_index_range: std::ops::Range<usize>,
) {
    let grid_size = subdiv_ccg.grid_size as usize;
    let grid_area = subdiv_ccg.grid_area as usize;
    let positions = &mut subdiv_ccg.positions;
    /* For each corner grid, walk its shared edges against the next
     * corner's grid and average duplicate points (same vertex). */
    let grids: Vec<usize> = grid_index_range.collect();
    for (i, &g1) in grids.iter().enumerate() {
        let g2 = grids[(i + 1) % grids.len()];
        /* Shared edge: last row of g1 vs first row of g2. */
        for x in 0..grid_size {
            let i1 = g1 * grid_area + (grid_size - 1) * grid_size + x;
            let i2 = g2 * grid_area + x;
            average_grid_element(positions, i1, i2);
            average_grid_element(positions, i2, i1);
        }
    }
}

/// `subdiv_ccg_average_grids_boundary` — average positions along a
/// grid boundary with the adjacent grid.
pub fn subdiv_ccg_average_grids_boundary(
    subdiv_ccg: &mut SubdivCCG,
    grid_index: usize,
    other_grid_index: usize,
) {
    let grid_size = subdiv_ccg.grid_size as usize;
    let grid_area = subdiv_ccg.grid_area as usize;
    for x in 0..grid_size {
        let i1 = grid_index * grid_area + (grid_size - 1) * grid_size + x;
        let i2 = other_grid_index * grid_area + x;
        average_grid_element(&mut subdiv_ccg.positions, i1, i2);
        average_grid_element(&mut subdiv_ccg.positions, i2, i1);
    }
}

/// `subdiv_ccg_average_grids_corners` — average the corner point
/// shared by several grids (vert of the coarse face).
pub fn subdiv_ccg_average_grids_corners(
    subdiv_ccg: &mut SubdivCCG,
    coords: &[SubdivCCGCoord],
) {
    let mut acc = GridElementAccumulator::default();
    acc.init();
    for c in coords {
        let key = CCGKey {
            grid_size: subdiv_ccg.grid_size,
            grid_area: subdiv_ccg.grid_area,
            ..Default::default()
        };
        acc.add(&subdiv_ccg.positions, c.to_index(&key) as usize);
    }
    if acc.counter > 0 {
        acc.mul_fl(1.0 / acc.counter as f32);
        for c in coords {
            let key = CCGKey {
                grid_size: subdiv_ccg.grid_size,
                grid_area: subdiv_ccg.grid_area,
                ..Default::default()
            };
            acc.copy_to(&mut subdiv_ccg.positions, c.to_index(&key) as usize);
        }
    }
}

/// `subdiv_ccg_recalc_inner_face_normals` — per-element quad normals
/// in a face's grids.
pub fn subdiv_ccg_recalc_inner_face_normals(subdiv_ccg: &mut SubdivCCG, face: usize) {
    let grid_size = subdiv_ccg.grid_size as usize;
    let face_grids = face_range(&subdiv_ccg.faces_offsets, subdiv_ccg.grid_area, face);
    for grid_index in face_grids {
        for y in 0..grid_size - 1 {
            for x in 0..grid_size - 1 {
                let base = grid_index;
                let i00 = base + y * grid_size + x;
                let i01 = base + y * grid_size + x + 1;
                let i10 = base + (y + 1) * grid_size + x;
                let positions = &subdiv_ccg.positions;
                let p00 = positions[i00];
                let p01 = positions[i01];
                let p10 = positions[i10];
                let p11 = positions[i10 + 1];
                let nor = (p10 - p00).cross(p01 - p11).normalized();
                subdiv_ccg.normals[i00] = nor;
            }
        }
    }
}

/// `subdiv_ccg_average_inner_face_normals` — normalize each element.
pub fn subdiv_ccg_average_inner_face_normals(subdiv_ccg: &mut SubdivCCG, face: usize) {
    let range = face_range(&subdiv_ccg.faces_offsets, subdiv_ccg.grid_area, face);
    for i in range {
        subdiv_ccg.normals[i] = subdiv_ccg.normals[i].normalized();
    }
}

/// `BKE_subdiv_ccg_recalc_normals` — all faces.
pub fn bke_subdiv_ccg_recalc_normals(subdiv_ccg: &mut SubdivCCG) {
    for face in 0..subdiv_ccg.faces_offsets.len().saturating_sub(1) {
        subdiv_ccg_recalc_inner_face_normals(subdiv_ccg, face);
    }
    for i in 0..subdiv_ccg.normals.len() {
        subdiv_ccg.normals[i] = subdiv_ccg.normals[i].normalized();
    }
}

/// `BKE_subdiv_ccg_update_normals` — recalc + average normals on a
/// face set (indices).
pub fn bke_subdiv_ccg_update_normals(subdiv_ccg: &mut SubdivCCG, face_mask: &[usize]) {
    for &face in face_mask {
        subdiv_ccg_recalc_inner_face_normals(subdiv_ccg, face);
        subdiv_ccg_average_inner_face_normals(subdiv_ccg, face);
    }
}

/// `BKE_subdiv_ccg_average_grids` — stitch all boundaries/corners
/// for all coarse faces.
pub fn bke_subdiv_ccg_average_grids(subdiv_ccg: &mut SubdivCCG) {
    for face in 0..subdiv_ccg.faces_offsets.len().saturating_sub(1) {
        let range = face_range(&subdiv_ccg.faces_offsets, subdiv_ccg.grid_area, face);
        let grid_area = subdiv_ccg.grid_area as usize;
        subdiv_ccg_average_inner_face_grids(
            subdiv_ccg,
            range.start / grid_area..range.end / grid_area,
        );
    }
}

/// `BKE_subdiv_ccg_average_stitch_faces` — same on a face set.
pub fn bke_subdiv_ccg_average_stitch_faces(subdiv_ccg: &mut SubdivCCG, face_mask: &[usize]) {
    let grid_area = subdiv_ccg.grid_area as usize;
    for &face in face_mask {
        let range = face_range(&subdiv_ccg.faces_offsets, subdiv_ccg.grid_area, face);
        subdiv_ccg_average_inner_face_grids(
            subdiv_ccg,
            range.start / grid_area..range.end / grid_area,
        );
    }
}
