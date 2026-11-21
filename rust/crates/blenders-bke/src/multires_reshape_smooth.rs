//! Port of `intern/multires_reshape_smooth.cc` (portable parts) —
//! `SurfacePoint`/`SurfaceGrid`/`Vertex`, `LinearGrid(s)` element
//! math, `foreach_toplevel_grid_coord`, grid-coord interpolation,
//! `add_grid_coord_to_vert`/`add_ptex_coord_to_vert`,
//! `evaluate_linear_delta_grids`, `propagate_linear_data_delta`,
//! `evaluate_base_surface_grids` (limit evaluation injected).
//! `PORT_PENDING`: `converter_init`, `reshape_subdiv_*` (OpenSubdiv
//! evaluator), `evaluate_higher_grid_positions`,
//! `multires_reshape_smooth_object_grids(_with_details)`.

use blenders_math::vector::Vec3;

/// `SurfacePoint`.
#[derive(Clone, Copy, Default)]
pub struct SurfacePoint {
    /// `P` limit position.
    pub p: Vec3,
    /// `tangent_matrix` (columns: u-tangent, v-tangent, normal).
    pub tangent_matrix: [Vec3; 3],
}

/// `SurfaceGrid`.
#[derive(Clone, Default)]
pub struct SurfaceGrid {
    /// `points`.
    pub points: Vec<SurfacePoint>,
}

/// `GridCoord` (grid index + uv within grid).
#[derive(Clone, Copy, Default)]
pub struct GridCoord {
    /// `grid_index`.
    pub grid_index: i32,
    /// `u`, `v`.
    pub u: f32,
    /// See `u`.
    pub v: f32,
}

/// `PTexCoord`.
#[derive(Clone, Copy, Default)]
pub struct PTexCoord {
    /// `ptex_face_index`.
    pub ptex_face_index: i32,
    /// `u`, `v`.
    pub u: f32,
    /// See `u`.
    pub v: f32,
}

/// `Vertex` — all grid coords a subdiv vertex maps to.
#[derive(Clone, Default)]
pub struct Vertex {
    /// `grid_coords`.
    pub grid_coords: Vec<GridCoord>,
}

/// `LinearGridElement` — a mask delta element.
#[derive(Clone, Copy, Default)]
pub struct LinearGridElement {
    /// `mask`.
    pub mask: f32,
}

/// `LinearGrids` — per-base-grid flat element storage.
#[derive(Clone, Default)]
pub struct LinearGrids {
    /// `num_grids`.
    pub num_grids: usize,
    /// `level`.
    pub level: i32,
    /// `grid_size`.
    pub grid_size: usize,
    /// `elements_storage` (grid_size² per grid).
    pub elements_storage: Vec<LinearGridElement>,
}

/// `linear_grids_allocate`.
pub fn linear_grids_allocate(num_grids: usize, level: i32) -> LinearGrids {
    let grid_size = crate::multires::grid_size_from_level(level) as usize;
    let grid_area = grid_size * grid_size;
    LinearGrids {
        num_grids,
        level,
        grid_size,
        elements_storage: vec![LinearGridElement::default(); num_grids * grid_area],
    }
}

/// `linear_grid_element_get`.
pub fn linear_grid_element_get<'a>(
    linear_grids: &'a mut [LinearGridElement],
    grid_size: usize,
    grid_coord: &GridCoord,
) -> &'a mut LinearGridElement {
    let grid_x = (grid_coord.u * (grid_size - 1) as f32).round() as usize;
    let grid_y = (grid_coord.v * (grid_size - 1) as f32).round() as usize;
    let grid_area = grid_size * grid_size;
    let idx = grid_coord.grid_index as usize * grid_area + grid_y * grid_size + grid_x;
    &mut linear_grids[idx]
}

/// `linear_grid_element_init`.
pub fn linear_grid_element_init(e: &mut LinearGridElement) {
    e.mask = 0.0;
}

/// `linear_grid_element_sub`.
pub fn linear_grid_element_sub(
    result: &mut LinearGridElement,
    a: &LinearGridElement,
    b: &LinearGridElement,
) {
    result.mask = a.mask - b.mask;
}

/// `linear_grid_element_interpolate`.
pub fn linear_grid_element_interpolate(
    result: &mut LinearGridElement,
    elements: &[LinearGridElement; 4],
    weights: &[f32; 4],
) {
    result.mask = elements[0].mask * weights[0]
        + elements[1].mask * weights[1]
        + elements[2].mask * weights[2]
        + elements[3].mask * weights[3];
}

/// `base_surface_grids` storage (per-grid point arrays).
#[derive(Clone, Default)]
pub struct BaseSurfaceGrids {
    /// `num_grids`.
    pub num_grids: usize,
    /// `grid_size`.
    pub grid_size: usize,
    /// `points` (grid_size² per grid).
    pub points: Vec<SurfacePoint>,
}

/// `base_surface_grids_allocate`.
pub fn base_surface_grids_allocate(num_grids: usize, grid_size: usize) -> BaseSurfaceGrids {
    BaseSurfaceGrids {
        num_grids,
        grid_size,
        points: vec![SurfacePoint::default(); num_grids * grid_size * grid_size],
    }
}

fn base_surface_index(grid_size: usize, coord: &GridCoord) -> usize {
    let grid_area = grid_size * grid_size;
    let grid_x = (coord.u * (grid_size - 1) as f32).round() as usize;
    let grid_y = (coord.v * (grid_size - 1) as f32).round() as usize;
    coord.grid_index as usize * grid_area + grid_y * grid_size + grid_x
}

/// `base_surface_grids_write`.
pub fn base_surface_grids_write(
    grids: &mut BaseSurfaceGrids,
    coord: &GridCoord,
    p: Vec3,
    tangent_matrix: [Vec3; 3],
) {
    let idx = base_surface_index(grids.grid_size, coord);
    grids.points[idx] = SurfacePoint {
        p,
        tangent_matrix,
    };
}

/// `base_surface_grids_read`.
pub fn base_surface_grids_read<'a>(grids: &'a BaseSurfaceGrids, coord: &GridCoord) -> &'a SurfacePoint {
    &grids.points[base_surface_index(grids.grid_size, coord)]
}

/// Geometry built during smoothing (`geometry` field of the context).
#[derive(Clone, Default)]
pub struct SmoothGeometry {
    /// `vertices`.
    pub vertices: Vec<Vertex>,
    /// `vert_sharpness`.
    pub vert_sharpness: Vec<f32>,
    /// `edges`.
    pub edges: Vec<[i32; 2]>,
    /// `edge_sharpness`.
    pub edge_sharpness: Vec<f32>,
    /// `corner_verts`.
    pub corner_verts: Vec<i32>,
    /// `corner_grid_indices`.
    pub corner_grid_indices: Vec<i32>,
    /// `face_offsets` (len faces+1).
    pub face_offsets: Vec<i32>,
}

impl SmoothGeometry {
    /// `get_face_grid_index`.
    pub fn get_face_grid_index(&self, face_start: usize) -> i32 {
        self.corner_grid_indices[face_start]
    }
}

/// `vert_grid_coord_with_grid_index`.
pub fn vert_grid_coord_with_grid_index(vert: &Vertex, grid_index: i32) -> Option<GridCoord> {
    vert.grid_coords
        .iter()
        .copied()
        .find(|c| c.grid_index == grid_index)
}

/// `grid_coords_from_face_verts` — one GridCoord per face corner.
pub fn grid_coords_from_face_verts(
    geometry: &SmoothGeometry,
    face: std::ops::Range<usize>,
) -> [Option<GridCoord>; 4] {
    let grid_index = geometry.get_face_grid_index(face.start);
    let mut result = [None; 4];
    for (i, corner_index) in face.enumerate() {
        let vert_index = geometry.corner_verts[corner_index] as usize;
        result[i] = vert_grid_coord_with_grid_index(&geometry.vertices[vert_index], grid_index);
    }
    result
}

fn lerp_f(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

/// `interpolate_grid_coord`.
pub fn interpolate_grid_coord(face_grid_coords: &[Option<GridCoord>; 4], u: f32, v: f32) -> GridCoord {
    let gc = |i: usize| face_grid_coords[i].unwrap();
    let u01 = lerp_f(u, gc(0).u, gc(1).u);
    let u32 = lerp_f(u, gc(3).u, gc(2).u);
    let v03 = lerp_f(v, gc(0).v, gc(3).v);
    let v12 = lerp_f(v, gc(1).v, gc(2).v);
    GridCoord {
        grid_index: gc(0).grid_index,
        u: lerp_f(v, u01, u32),
        v: lerp_f(u, v03, v12),
    }
}

/// `foreach_toplevel_grid_coord` — serial port: for each face, sample
/// the inner grid at `inner_grid_size = (1 << level_diff) + 1`.
#[allow(clippy::too_many_arguments)]
pub fn foreach_toplevel_grid_coord(
    geometry: &SmoothGeometry,
    faces_num: usize,
    top_level: i32,
    reshape_level: i32,
    mut callback: impl FnMut(PTexCoord, GridCoord),
) {
    let level_difference = top_level - reshape_level;
    let inner_grid_size = (1i32 << level_difference) + 1;
    let inv = 1.0 / (inner_grid_size - 1) as f32;
    for face_index in 0..faces_num {
        let start = geometry.face_offsets[face_index] as usize;
        let end = geometry.face_offsets[face_index + 1] as usize;
        let face_grid_coords = grid_coords_from_face_verts(geometry, start..end);
        for y in 0..inner_grid_size {
            let ptex_v = y as f32 * inv;
            for x in 0..inner_grid_size {
                let ptex_u = x as f32 * inv;
                let ptex = PTexCoord {
                    ptex_face_index: face_index as i32,
                    u: ptex_u,
                    v: ptex_v,
                };
                let grid_coord = interpolate_grid_coord(&face_grid_coords, ptex_u, ptex_v);
                callback(ptex, grid_coord);
            }
        }
    }
}

/// `get_reshape_level_resolution`.
pub fn get_reshape_level_resolution(reshape_level: i32) -> i32 {
    (1 << reshape_level) + 1
}

/// `MultiresSubdivideModeType` (verbatim subset).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MultiresSubdivideModeType {
    /// Catmull-Clark.
    CatmullClark,
    /// Simple.
    Simple,
    /// Linear.
    Linear,
}

/// `is_crease_supported`.
pub fn is_crease_supported(smoothing_type: MultiresSubdivideModeType) -> bool {
    !matches!(
        smoothing_type,
        MultiresSubdivideModeType::Linear | MultiresSubdivideModeType::Simple
    )
}

/// `get_effective_crease`.
pub fn get_effective_crease(
    smoothing_type: MultiresSubdivideModeType,
    cd_edge_crease: &[f32],
    base_edge_index: usize,
) -> f32 {
    if !is_crease_supported(smoothing_type) {
        return 1.0;
    }
    if cd_edge_crease.is_empty() {
        return 0.0;
    }
    cd_edge_crease[base_edge_index]
}

/// `get_effective_crease_float`.
pub fn get_effective_crease_float(smoothing_type: MultiresSubdivideModeType, crease: f32) -> f32 {
    if !is_crease_supported(smoothing_type) {
        return 1.0;
    }
    crease
}

/// `add_grid_coord_to_vert`.
pub fn add_grid_coord_to_vert(
    geometry: &mut SmoothGeometry,
    grid_coord: &GridCoord,
    coarse_vert_index: i32,
    subdiv_vert_index: usize,
    smoothing_type: MultiresSubdivideModeType,
    cd_vert_crease: &[f32],
) {
    geometry.vertices[subdiv_vert_index].grid_coords.push(*grid_coord);
    if coarse_vert_index == -1 || cd_vert_crease.is_empty() {
        return;
    }
    let mut crease = cd_vert_crease[coarse_vert_index as usize];
    if crease == 0.0 {
        return;
    }
    crease = get_effective_crease_float(smoothing_type, crease);
    geometry.vert_sharpness[subdiv_vert_index] =
        crate::subdiv_converter_mesh::crease_to_sharpness(crease);
}

/// `add_ptex_coord_to_vert` — `ptex_to_grid`/`grid_to_face_index` and
/// `base_faces` injected.
#[allow(clippy::too_many_arguments)]
pub fn add_ptex_coord_to_vert(
    geometry: &mut SmoothGeometry,
    coord: &PTexCoord,
    coarse_vert_index: i32,
    subdiv_vert_index: usize,
    smoothing_type: MultiresSubdivideModeType,
    cd_vert_crease: &[f32],
    ptex_to_grid: &dyn Fn(PTexCoord) -> GridCoord,
    base_faces: &[std::ops::Range<usize>], /* base face -> grid range */
    grid_to_face_index: &dyn Fn(i32) -> usize,
) {
    let grid_coord = ptex_to_grid(*coord);
    let face_index = grid_to_face_index(grid_coord.grid_index);
    let face = &base_faces[face_index];
    let num_corners = face.len() as i32;
    let start_grid_index = face.start as i32;
    let corner = grid_coord.grid_index - start_grid_index;

    if grid_coord.u == 0.0 && grid_coord.v == 0.0 {
        for current_corner in 0..num_corners {
            let mut corner_grid_coord = grid_coord;
            corner_grid_coord.grid_index = start_grid_index + current_corner;
            add_grid_coord_to_vert(
                geometry,
                &corner_grid_coord,
                coarse_vert_index,
                subdiv_vert_index,
                smoothing_type,
                cd_vert_crease,
            );
        }
        return;
    }

    add_grid_coord_to_vert(
        geometry,
        &grid_coord,
        coarse_vert_index,
        subdiv_vert_index,
        smoothing_type,
        cd_vert_crease,
    );

    if grid_coord.u == 0.0 {
        let prev_grid_coord = GridCoord {
            grid_index: start_grid_index + (corner + num_corners - 1).rem_euclid(num_corners),
            u: grid_coord.v,
            v: 0.0,
        };
        add_grid_coord_to_vert(
            geometry,
            &prev_grid_coord,
            coarse_vert_index,
            subdiv_vert_index,
            smoothing_type,
            cd_vert_crease,
        );
    }
    if grid_coord.v == 0.0 {
        let next_grid_coord = GridCoord {
            grid_index: start_grid_index + (corner + 1).rem_euclid(num_corners),
            u: 0.0,
            v: grid_coord.u,
        };
        add_grid_coord_to_vert(
            geometry,
            &next_grid_coord,
            coarse_vert_index,
            subdiv_vert_index,
            smoothing_type,
            cd_vert_crease,
        );
    }
}

/// `linear_grid_element_delta_interpolate` — bilinear interpolation of
/// (final − orig) mask over the reshape-level grid.
#[allow(clippy::too_many_arguments)]
pub fn linear_grid_element_delta_interpolate(
    reshape_level: i32,
    grid_coord: &GridCoord,
    orig_mask: &dyn Fn(&GridCoord) -> f32,
    final_mask: &dyn Fn(&GridCoord) -> f32,
    result: &mut LinearGridElement,
) {
    let reshape_level_grid_size = crate::multires::grid_size_from_level(reshape_level);
    let gs1 = (reshape_level_grid_size - 1) as f32;
    let gs1_inv = 1.0 / gs1;
    let x_f = grid_coord.u * gs1;
    let y_f = grid_coord.v * gs1;
    let x_i = x_f as i32;
    let y_i = y_f as i32;
    let x_n_i = if x_i == reshape_level_grid_size - 1 { x_i } else { x_i + 1 };
    let y_n_i = if y_i == reshape_level_grid_size - 1 { y_i } else { y_i + 1 };
    let corners = [[x_i, y_i], [x_n_i, y_i], [x_n_i, y_n_i], [x_i, y_n_i]];
    let mut corner_elements = [LinearGridElement::default(); 4];
    for (i, c) in corners.iter().enumerate() {
        let cc = GridCoord {
            grid_index: grid_coord.grid_index,
            u: c[0] as f32 * gs1_inv,
            v: c[1] as f32 * gs1_inv,
        };
        let orig = orig_mask(&cc);
        let fin = final_mask(&cc);
        corner_elements[i] = LinearGridElement { mask: fin - orig };
    }
    let u = x_f - x_i as f32;
    let v = y_f - y_i as f32;
    let weights = [(1.0 - u) * (1.0 - v), u * (1.0 - v), u * v, (1.0 - u) * v];
    linear_grid_element_interpolate(result, &corner_elements, &weights);
}

/// `evaluate_linear_delta_grids` — fill linear_delta_grids over the
/// top-level ptex grid. `orig_mask`/`final_mask` read grid elements.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_linear_delta_grids(
    geometry: &SmoothGeometry,
    faces_num: usize,
    num_grids: usize,
    top_level: i32,
    reshape_level: i32,
    orig_mask: &dyn Fn(&GridCoord) -> f32,
    final_mask: &dyn Fn(&GridCoord) -> f32,
) -> LinearGrids {
    let mut linear_delta_grids = linear_grids_allocate(num_grids, top_level);
    let grid_size = linear_delta_grids.grid_size;
    let mut elements = std::mem::take(&mut linear_delta_grids.elements_storage);
    foreach_toplevel_grid_coord(
        geometry,
        faces_num,
        top_level,
        reshape_level,
        |_ptex, grid_coord| {
            let el = linear_grid_element_get(&mut elements, grid_size, &grid_coord);
            linear_grid_element_delta_interpolate(
                reshape_level,
                &grid_coord,
                orig_mask,
                final_mask,
                el,
            );
        },
    );
    linear_delta_grids.elements_storage = elements;
    linear_delta_grids
}

/// `propagate_linear_data_delta` — add the interpolated linear delta
/// back into the final grid element mask (clamped 0..1).
pub fn propagate_linear_data_delta(
    linear_delta: &LinearGrids,
    grid_coord: &GridCoord,
    orig_mask: f32,
) -> f32 {
    let grid_size = linear_delta.grid_size;
    let grid_area = grid_size * grid_size;
    let grid_x = (grid_coord.u * (grid_size - 1) as f32).round() as usize;
    let grid_y = (grid_coord.v * (grid_size - 1) as f32).round() as usize;
    let el =
        &linear_delta.elements_storage[grid_coord.grid_index as usize * grid_area + grid_y * grid_size + grid_x];
    (orig_mask + el.mask).clamp(0.0, 1.0)
}

/// `evaluate_base_surface_grids` — evaluate limit positions/tangents
/// for every top-level grid coordinate. `evaluator` returns (P, dPdu,
/// dPdv) per ptex coord.
pub fn evaluate_base_surface_grids(
    geometry: &SmoothGeometry,
    faces_num: usize,
    num_grids: usize,
    grid_size: usize,
    top_level: i32,
    reshape_level: i32,
    evaluator: &dyn Fn(&PTexCoord, &GridCoord) -> (Vec3, [Vec3; 3]),
) -> BaseSurfaceGrids {
    let mut grids = base_surface_grids_allocate(num_grids, grid_size);
    foreach_toplevel_grid_coord(
        geometry,
        faces_num,
        top_level,
        reshape_level,
        |ptex, grid_coord| {
            let (p, tangent) = evaluator(&ptex, &grid_coord);
            base_surface_grids_write(&mut grids, &grid_coord, p, tangent);
        },
    );
    grids
}

/// `evaluate_final_original_point` — combine limit surface with the
/// interpolated delta. `final_limit`/`limit_tangent` from evaluator.
pub fn evaluate_final_original_point(
    final_limit: Vec3,
    orig_limit: Vec3,
    tangent_matrix: [Vec3; 3],
    delta: Vec3,
) -> Vec3 {
    /* P_final = P_limit_orig + T * delta */
    let d = tangent_matrix[0] * delta.x + tangent_matrix[1] * delta.y + tangent_matrix[2] * delta.z;
    let _ = final_limit;
    orig_limit + d
}
