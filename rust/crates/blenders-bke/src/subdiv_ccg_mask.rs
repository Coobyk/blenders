//! Port of `intern/subdiv_ccg_mask.cc` — `BKE_subdiv_ccg_mask_init_from_paint`
//! evaluator over `CD_GRID_PAINT_MASK` data.

use blenders_dna::meshdata_types::GridPaintMask;
use blenders_math::vector::Vec2;

/// `ptex_face_uv_to_grid_uv` (subdiv_inline.hh) — `(1 - v, 1 - u)`.
pub fn ptex_face_uv_to_grid_uv(ptex_uv: Vec2) -> Vec2 {
    Vec2::new(1.0 - ptex_uv.y, 1.0 - ptex_uv.x)
}

/// `grid_size_from_level`.
pub fn grid_size_from_level(level: i32) -> i32 {
    (1 << (level - 1)) + 1
}

/// `rotate_quad_to_corner(quad_u, quad_v, &corner_u, &corner_v)` — quadrant
/// index + folded corner-local uv (the second `rotate_quad_to_corner`
/// overload applied to the result).
pub fn rotate_quad_to_corner(quad_u: f32, quad_v: f32) -> (i32, Vec2) {
    let quad = Vec2::new(quad_u, quad_v);
    let corner = if quad_u <= 0.5 && quad_v <= 0.5 {
        0
    } else if quad_u > 0.5 && quad_v <= 0.5 {
        1
    } else if quad_u > 0.5 && quad_v > 0.5 {
        2
    } else {
        3
    };
    let corner_uv = match corner {
        0 => 2.0 * quad,
        1 => Vec2::new(2.0 * quad.y, 2.0 * (1.0 - quad.x)),
        2 => Vec2::new(2.0 * (1.0 - quad.x), 2.0 * (1.0 - quad.y)),
        _ => Vec2::new(2.0 * (1.0 - quad.y), 2.0 * quad.x),
    };
    (corner, corner_uv)
}

/// `PolyCornerIndex`.
#[derive(Clone, Copy)]
pub struct PolyCornerIndex {
    pub face_index: i32,
    pub corner: i32,
}

/// `GridPaintMaskData` — the evaluator's `user_data`.
pub struct GridPaintMaskData {
    /// `OffsetIndices<int> faces` — caller-provided (face corner ranges).
    pub faces: Vec<std::ops::Range<i64>>,
    /// `const GridPaintMask *grid_paint_mask` — the CustomData layer
    /// (`CD_GRID_PAINT_MASK`), one entry per corner-grid.
    pub grid_paint_mask: Vec<GridPaintMask>,
    /// `PolyCornerIndex *ptex_face_corner` — ptex-face → (face, corner).
    pub ptex_face_corner: Vec<PolyCornerIndex>,
}

/// `mask_get_grid_and_coord` — ptex-face → grid element + rotated uv.
/// Returns `(grid_index, grid_uv, corner)`.
fn mask_get_grid_and_coord(data: &GridPaintMaskData, ptex_face_index: usize, u: f32, v: f32) -> (usize, Vec2) {
    let poly_corner = data.ptex_face_corner[ptex_face_index];
    let face = data.faces[poly_corner.face_index as usize].clone();
    let start_grid_index = face.start + poly_corner.corner as i64;
    let (grid_u, grid_v) = if face.end - face.start == 4 {
        let (_corner, corner_uv) = rotate_quad_to_corner(u, v);
        let g = ptex_face_uv_to_grid_uv(corner_uv);
        (g.x, g.y)
    } else {
        let g = ptex_face_uv_to_grid_uv(Vec2::new(u, v));
        (g.x, g.y)
    };
    (start_grid_index as usize, Vec2::new(grid_u, grid_v))
}

/// `read_mask_grid`.
fn read_mask_grid(mask_grid: &GridPaintMask, grid_u: f32, grid_v: f32) -> f32 {
    if mask_grid.data.is_empty() {
        return 0.0;
    }
    let grid_size = grid_size_from_level(mask_grid.level as i32) as usize;
    let x = (grid_u * (grid_size - 1) as f32).round() as usize;
    let y = (grid_v * (grid_size - 1) as f32).round() as usize;
    mask_grid.data[y * grid_size + x]
}

/// `eval_mask` — the evaluator callback.
pub fn eval_mask(data: &GridPaintMaskData, ptex_face_index: usize, u: f32, v: f32) -> f32 {
    let (grid_index, grid_uv) = mask_get_grid_and_coord(data, ptex_face_index, u, v);
    read_mask_grid(&data.grid_paint_mask[grid_index], grid_uv.x, grid_uv.y)
}

/// `count_num_ptex_faces` — quads count once, other polys once per corner.
fn count_num_ptex_faces(faces: &[std::ops::Range<i64>]) -> usize {
    let mut num_ptex_faces = 0;
    for face in faces {
        let size = face.end - face.start;
        num_ptex_faces += if size == 4 { 1 } else { size as usize };
    }
    num_ptex_faces
}

/// `mask_data_init_mapping` — build `ptex_face_corner`.
fn mask_data_init_mapping(faces: &[std::ops::Range<i64>]) -> Vec<PolyCornerIndex> {
    let mut ptex_face_corner = Vec::with_capacity(count_num_ptex_faces(faces));
    for (face_index, face) in faces.iter().enumerate() {
        let size = face.end - face.start;
        if size == 4 {
            ptex_face_corner.push(PolyCornerIndex { face_index: face_index as i32, corner: 0 });
        } else {
            for corner in 0..size {
                ptex_face_corner.push(PolyCornerIndex { face_index: face_index as i32, corner: corner as i32 });
            }
        }
    }
    ptex_face_corner
}

/// `BKE_subdiv_ccg_mask_init_from_paint` — false when no
/// `CD_GRID_PAINT_MASK` layer; `grid_paint_mask` is the resolved layer
/// data (CustomData lookup stays with the caller — PORT_PENDING there).
pub fn bke_subdiv_ccg_mask_init_from_paint(
    faces: Vec<std::ops::Range<i64>>,
    grid_paint_mask: Option<Vec<GridPaintMask>>,
) -> Option<GridPaintMaskData> {
    let grid_paint_mask = grid_paint_mask?;
    let ptex_face_corner = mask_data_init_mapping(&faces);
    Some(GridPaintMaskData { faces, grid_paint_mask, ptex_face_corner })
}
