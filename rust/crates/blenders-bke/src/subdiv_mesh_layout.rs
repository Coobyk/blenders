//! Port of `intern/subdiv_mesh_layout.cc` + `BKE_subdiv_mesh_layout.hh` —
//! computes the element layout of a Catmull-Clark subdivided mesh
//! (base verts → edge verts → inner face verts; base edges → inner
//! face edges; per-base-face quad faces + their corners), and fills
//! edge/corner/ptex index buffers for it. `parallel_for` → serial.

/// `PtexCoord` (BKE_subdiv_eval.hh).
#[derive(Clone, Copy, Default)]
pub struct PtexCoord {
    /// `ptex_face_index`.
    pub ptex_face_index: i32,
    /// `u`, `v`.
    pub u: f32,
    /// See `u`.
    pub v: f32,
}

/// `num_ptex_faces_per_face_get`.
pub fn num_ptex_faces_per_face_get(face_size: i32) -> i32 {
    if face_size == 4 {
        1
    } else {
        face_size
    }
}

/// `num_edges_per_ptex_face_get`.
pub fn num_edges_per_ptex_face_get(resolution: i32) -> i32 {
    2 * (resolution - 1) * resolution
}

/// `num_inner_edges_per_ptex_face_get`.
pub fn num_inner_edges_per_ptex_face_get(resolution: i32) -> i32 {
    if resolution < 2 {
        return 0;
    }
    (resolution - 2) * resolution + (resolution - 1) * (resolution - 1)
}

/// `num_faces_per_ptex_get`.
fn num_faces_per_ptex_get(resolution: i32) -> i32 {
    (resolution - 1) * (resolution - 1)
}

/// `ptex_face_resolution_get`.
fn ptex_face_resolution_get(face_size: i32, resolution: i32) -> i32 {
    if face_size == 4 {
        resolution
    } else {
        (resolution >> 1) + 1
    }
}

/// `MeshLayout`.
#[derive(Clone, Default)]
pub struct MeshLayout {
    /// `resolution`.
    pub resolution: i32,
    /// `verts_num`.
    pub verts_num: i32,
    /// `edges_num`.
    pub edges_num: i32,
    /// `faces_num`.
    pub faces_num: i32,
    /// `corners_num`.
    pub corners_num: i32,
    /// `edge_verts_start`.
    pub edge_verts_start: i32,
    /// `inner_verts_start`.
    pub inner_verts_start: i32,
    /// `inner_edges_start`.
    pub inner_edges_start: i32,
    /// `inner_vert_offsets`.
    pub inner_vert_offsets: Vec<i32>,
    /// `inner_edge_offsets`.
    pub inner_edge_offsets: Vec<i32>,
    /// `face_offsets`.
    pub face_offsets: Vec<i32>,
}

impl MeshLayout {
    /// `base_edge_verts` → `(start, len)`.
    pub fn base_edge_verts(&self, base_edge: i32) -> (i32, i32) {
        let n = self.resolution - 2;
        (self.edge_verts_start + base_edge * n, n)
    }
    /// `base_edge_edges` → `(start, len)`.
    pub fn base_edge_edges(&self, base_edge: i32) -> (i32, i32) {
        let n = self.resolution - 1;
        (base_edge * n, n)
    }
    /// `faces_by_base_face` range for `face` → `(start, len)`.
    pub fn faces_of_base_face(&self, face: usize) -> (i32, i32) {
        (
            self.face_offsets[face],
            self.face_offsets[face + 1] - self.face_offsets[face],
        )
    }
    /// `inner_verts_by_face` range → `(start, len)`.
    pub fn inner_verts_of_face(&self, face: usize) -> (i32, i32) {
        (
            self.inner_vert_offsets[face],
            self.inner_vert_offsets[face + 1] - self.inner_vert_offsets[face],
        )
    }
    /// `inner_edges_by_face` range → `(start, len)`.
    pub fn inner_edges_of_face(&self, face: usize) -> (i32, i32) {
        (
            self.inner_edge_offsets[face],
            self.inner_edge_offsets[face + 1] - self.inner_edge_offsets[face],
        )
    }
    /// `base_face_corners(face)` → `(start, len)` (4 per face).
    pub fn base_face_corners(&self, base_face: usize) -> (i32, i32) {
        let (s, n) = self.faces_of_base_face(base_face);
        (s * 4, n * 4)
    }
    /// `base_face_grain_size`.
    pub fn base_face_grain_size(&self) -> i32 {
        (4096 / (self.resolution * self.resolution)).max(1)
    }
}

/// `build_mesh_layout` — `coarse_faces` = face sizes (offsets form).
pub fn build_mesh_layout(
    verts_num: i32,
    edges_num: i32,
    face_sizes: &[i32],
    resolution: i32,
) -> MeshLayout {
    let resolution_2 = resolution - 2;
    let resolution_2_squared = resolution_2 * resolution_2;
    let no_quad_patch_resolution = (resolution >> 1) + 1;
    let num_irregular_vertices_per_patch =
        (no_quad_patch_resolution - 2) * (no_quad_patch_resolution - 1);
    let num_subdiv_vertices_per_coarse_edge = resolution - 2;
    let num_subdiv_edges_per_coarse_edge = resolution - 1;

    let mut layout = MeshLayout {
        resolution,
        edge_verts_start: verts_num,
        ..Default::default()
    };
    layout.inner_verts_start =
        layout.edge_verts_start + edges_num * num_subdiv_vertices_per_coarse_edge;
    layout.inner_edges_start = edges_num * num_subdiv_edges_per_coarse_edge;
    let faces_num = face_sizes.len();
    layout.inner_vert_offsets = vec![0; faces_num + 1];
    layout.inner_edge_offsets = vec![0; faces_num + 1];
    layout.face_offsets = vec![0; faces_num + 1];

    let mut vert_offset = 0;
    let mut edge_offset = 0;
    let mut face_offset = 0;
    for (face_index, &size) in face_sizes.iter().enumerate() {
        let num_ptex = num_ptex_faces_per_face_get(size);
        layout.inner_vert_offsets[face_index] = vert_offset;
        layout.inner_edge_offsets[face_index] = edge_offset;
        layout.face_offsets[face_index] = face_offset;
        if num_ptex == 1 {
            vert_offset += resolution_2_squared;
            edge_offset += num_edges_per_ptex_face_get(resolution - 2)
                + 4 * num_subdiv_vertices_per_coarse_edge;
            face_offset += num_faces_per_ptex_get(resolution);
        } else {
            vert_offset += 1 + num_ptex * num_irregular_vertices_per_patch;
            edge_offset += num_ptex
                * (num_inner_edges_per_ptex_face_get(no_quad_patch_resolution - 1)
                    + (no_quad_patch_resolution - 2)
                    + num_subdiv_vertices_per_coarse_edge);
            if no_quad_patch_resolution >= 3 {
                edge_offset += size;
            }
            face_offset += num_ptex * num_faces_per_ptex_get(no_quad_patch_resolution);
        }
    }
    *layout.inner_vert_offsets.last_mut().unwrap() = vert_offset;
    *layout.inner_edge_offsets.last_mut().unwrap() = edge_offset;
    *layout.face_offsets.last_mut().unwrap() = face_offset;
    layout.verts_num = layout.inner_verts_start + vert_offset;
    layout.edges_num = layout.inner_edges_start + edge_offset;
    layout.faces_num = face_offset;
    layout.corners_num = face_offset * 4;
    layout
}

/// `TraversalContext` — coarse mesh topology slices.
pub struct TraversalContext<'a> {
    /// `coarse_edges`.
    pub coarse_edges: &'a [[i32; 2]],
    /// `coarse_faces` — offsets.
    pub coarse_face_offsets: &'a [i32],
    /// `coarse_corner_verts`.
    pub coarse_corner_verts: &'a [i32],
    /// `coarse_corner_edges`.
    pub coarse_corner_edges: &'a [i32],
    /// `layout`.
    pub layout: &'a MeshLayout,
}

impl<'a> TraversalContext<'a> {
    fn face(&self, i: usize) -> (i32, i32) {
        (
            self.coarse_face_offsets[i],
            self.coarse_face_offsets[i + 1] - self.coarse_face_offsets[i],
        )
    }
    fn face_size(&self, i: usize) -> i32 {
        self.coarse_face_offsets[i + 1] - self.coarse_face_offsets[i]
    }
    fn corner(&self, face: usize, corner: usize) -> usize {
        (self.coarse_face_offsets[face] + corner as i32) as usize
    }
}

fn subdiv_foreach_inner_vertices_regular(
    ctx: &TraversalContext,
    ptex_face_index: i32,
    ptex_coords: &mut [PtexCoord],
) {
    let resolution = ctx.layout.resolution;
    let inv_resolution_1 = 1.0 / (resolution - 1) as f32;
    let mut subdiv_vert_index = 0usize;
    for y in 1..resolution - 1 {
        let v = y as f32 * inv_resolution_1;
        for x in 1..resolution - 1 {
            let u = x as f32 * inv_resolution_1;
            ptex_coords[subdiv_vert_index] = PtexCoord {
                ptex_face_index,
                u,
                v,
            };
            subdiv_vert_index += 1;
        }
    }
}

fn subdiv_foreach_inner_vertices_special(
    ctx: &TraversalContext,
    coarse_face_index: usize,
    mut ptex_face_index: i32,
    ptex_coords: &mut [PtexCoord],
) {
    let resolution = ctx.layout.resolution;
    let face_size = ctx.face_size(coarse_face_index);
    let ptex_face_resolution = ptex_face_resolution_get(face_size, resolution);
    let inv_ptex_face_resolution_1 = 1.0 / (ptex_face_resolution - 1) as f32;
    let mut subdiv_vert_index = 0usize;
    ptex_coords[subdiv_vert_index] = PtexCoord {
        ptex_face_index,
        u: 1.0,
        v: 1.0,
    };
    subdiv_vert_index += 1;
    for _corner in 0..face_size {
        for y in 1..ptex_face_resolution - 1 {
            let v = y as f32 * inv_ptex_face_resolution_1;
            for x in 1..ptex_face_resolution {
                let u = x as f32 * inv_ptex_face_resolution_1;
                ptex_coords[subdiv_vert_index] = PtexCoord {
                    ptex_face_index,
                    u,
                    v,
                };
                subdiv_vert_index += 1;
            }
        }
        ptex_face_index += 1;
    }
}

/// `face_corner_ptex_coord`.
pub fn face_corner_ptex_coord(face_size: i32, face_ptex_offset: i32, corner: i32) -> PtexCoord {
    if face_size == 4 {
        const QUAD_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        return PtexCoord {
            ptex_face_index: face_ptex_offset,
            u: QUAD_UVS[corner as usize][0],
            v: QUAD_UVS[corner as usize][1],
        };
    }
    PtexCoord {
        ptex_face_index: face_ptex_offset + corner,
        u: 0.0,
        v: 0.0,
    }
}

/// `fill_face_corner_edge_ptex_coords`.
pub fn fill_face_corner_edge_ptex_coords(
    layout: &MeshLayout,
    face_size: i32,
    face_ptex_offset: i32,
    corner: i32,
    flip: bool,
    ptex_coords: &mut [PtexCoord],
) {
    let resolution = layout.resolution;
    let num_subdiv_vertices_per_coarse_edge = resolution - 2;
    debug_assert_eq!(ptex_coords.len() as i32, num_subdiv_vertices_per_coarse_edge);
    if num_subdiv_vertices_per_coarse_edge <= 0 {
        return;
    }
    if face_size == 4 {
        let resolution_1 = resolution - 1;
        let inv_resolution_1 = 1.0 / resolution_1 as f32;
        for vert_index in 0..num_subdiv_vertices_per_coarse_edge {
            let mut fac = (vert_index + 1) as f32 * inv_resolution_1;
            if flip {
                fac = 1.0 - fac;
            }
            if corner >= 2 {
                fac = 1.0 - fac;
            }
            let (u, v);
            if (corner & 1) == 0 {
                u = fac;
                v = if corner == 2 { 1.0 } else { 0.0 };
            } else {
                u = if corner == 1 { 1.0 } else { 0.0 };
                v = fac;
            }
            ptex_coords[vert_index as usize] = PtexCoord {
                ptex_face_index: face_ptex_offset,
                u,
                v,
            };
        }
        return;
    }
    let num_vertices_per_ptex_edge = (resolution >> 1) + 1;
    let inv_ptex_resolution_1 = 1.0 / (num_vertices_per_ptex_edge - 1) as f32;
    let ptex_face_index = face_ptex_offset + corner;
    let next_corner = (corner + 1) % face_size;
    let next_ptex_face_index = face_ptex_offset + next_corner;
    let mut subdiv_vert_index: i32 = 0;
    let mut vert_delta: i32 = 1;
    if flip {
        subdiv_vert_index += num_subdiv_vertices_per_coarse_edge - 1;
        vert_delta = -1;
    }
    for vert_index in 1..num_vertices_per_ptex_edge {
        let u = vert_index as f32 * inv_ptex_resolution_1;
        ptex_coords[subdiv_vert_index as usize] = PtexCoord {
            ptex_face_index,
            u,
            v: 0.0,
        };
        subdiv_vert_index += vert_delta;
    }
    for vert_index in 1..num_vertices_per_ptex_edge - 1 {
        let v = 1.0 - vert_index as f32 * inv_ptex_resolution_1;
        ptex_coords[subdiv_vert_index as usize] = PtexCoord {
            ptex_face_index: next_ptex_face_index,
            u: 0.0,
            v,
        };
        subdiv_vert_index += vert_delta;
    }
}

/// `fill_inner_vert_ptex_coords`.
pub fn fill_inner_vert_ptex_coords(
    layout: &MeshLayout,
    ctx: &TraversalContext,
    face_ptex_offset: &[i32],
    vert_ptex_coords: &mut [PtexCoord],
) {
    for face in 0..ctx.coarse_face_offsets.len() - 1 {
        let (s, n) = layout.inner_verts_of_face(face);
        let face_coords =
            &mut vert_ptex_coords[s as usize..(s + n) as usize];
        if ctx.face_size(face) == 4 {
            subdiv_foreach_inner_vertices_regular(ctx, face_ptex_offset[face], face_coords);
        } else {
            subdiv_foreach_inner_vertices_special(ctx, face, face_ptex_offset[face], face_coords);
        }
    }
}

fn subdiv_foreach_edges_row(
    edges: &mut [[i32; 2]],
    start_subdiv_edge_index: usize,
    start_vert_index: i32,
    num_edges_per_row: i32,
) -> usize {
    let mut subdiv_edge_index = start_subdiv_edge_index;
    let mut vert_index = start_vert_index;
    for _ in 0..num_edges_per_row - 1 {
        edges[subdiv_edge_index] = [vert_index, vert_index + 1];
        vert_index += 1;
        subdiv_edge_index += 1;
    }
    subdiv_edge_index
}

fn subdiv_foreach_edges_column(
    edges: &mut [[i32; 2]],
    start_subdiv_edge_index: usize,
    start_vert_index: i32,
    num_edges_per_row: i32,
) -> usize {
    let mut subdiv_edge_index = start_subdiv_edge_index;
    let mut vert_index = start_vert_index;
    for _ in 0..num_edges_per_row {
        edges[subdiv_edge_index] = [vert_index, vert_index + num_edges_per_row];
        vert_index += 1;
        subdiv_edge_index += 1;
    }
    subdiv_edge_index
}

fn subdiv_foreach_edges_all_patches_regular(
    ctx: &TraversalContext,
    coarse_face_index: usize,
    edges: &mut [[i32; 2]],
) {
    let resolution = ctx.layout.resolution;
    let start_vert_index =
        ctx.layout.inner_verts_start + ctx.layout.inner_vert_offsets[coarse_face_index];
    let num_subdiv_vertices_per_coarse_edge = resolution - 2;
    let mut subdiv_edge_index = 0usize;
    subdiv_edge_index =
        subdiv_foreach_edges_row(edges, subdiv_edge_index, start_vert_index, resolution - 2);
    for row in 0..resolution - 3 {
        let start_row_vert_index = start_vert_index + row * (resolution - 2);
        subdiv_edge_index = subdiv_foreach_edges_column(
            edges,
            subdiv_edge_index,
            start_row_vert_index,
            resolution - 2,
        );
        subdiv_edge_index = subdiv_foreach_edges_row(
            edges,
            subdiv_edge_index,
            start_row_vert_index + resolution - 2,
            resolution - 2,
        );
    }
    let face_size = ctx.face_size(coarse_face_index);
    for corner in 0..face_size as usize {
        let c = ctx.corner(coarse_face_index, corner);
        let coarse_vert_index = ctx.coarse_corner_verts[c];
        let coarse_edge_index = ctx.coarse_corner_edges[c] as usize;
        let coarse_edge = ctx.coarse_edges[coarse_edge_index];
        let start_edge_vert = ctx.layout.edge_verts_start
            + coarse_edge_index as i32 * num_subdiv_vertices_per_coarse_edge;
        let flip = coarse_edge[1] == coarse_vert_index;
        let mut side_start_index = start_vert_index;
        let mut side_stride = 0;
        match corner {
            0 => side_stride = 1,
            1 => {
                side_start_index += resolution - 3;
                side_stride = resolution - 2;
            }
            2 => {
                side_start_index += num_subdiv_vertices_per_coarse_edge
                    * num_subdiv_vertices_per_coarse_edge
                    - 1;
                side_stride = -1;
            }
            3 => {
                side_start_index += num_subdiv_vertices_per_coarse_edge
                    * (num_subdiv_vertices_per_coarse_edge - 1);
                side_stride = -(resolution - 2);
            }
            _ => {}
        }
        for i in 0..resolution - 2 {
            let v1 = if flip {
                start_edge_vert + (resolution - i - 3)
            } else {
                start_edge_vert + i
            };
            let v2 = side_start_index + side_stride * i;
            edges[subdiv_edge_index] = [v1, v2];
            subdiv_edge_index += 1;
        }
    }
}

fn subdiv_foreach_edges_all_patches_special(
    ctx: &TraversalContext,
    coarse_face_index: usize,
    edges: &mut [[i32; 2]],
) {
    let resolution = ctx.layout.resolution;
    let face_size = ctx.face_size(coarse_face_index);
    let ptex_face_resolution = ptex_face_resolution_get(face_size, resolution);
    let ptex_face_inner_resolution = ptex_face_resolution - 2;
    let num_inner_vertices_per_ptex =
        (ptex_face_resolution - 1) * (ptex_face_resolution - 2);
    let num_subdiv_vertices_per_coarse_edge = resolution - 2;
    let center_vert_index =
        ctx.layout.inner_verts_start + ctx.layout.inner_vert_offsets[coarse_face_index];
    let start_vert_index = center_vert_index + 1;
    let mut subdiv_edge_index = 0usize;

    for corner in 0..face_size {
        let start_ptex_face_vert_index =
            start_vert_index + corner * num_inner_vertices_per_ptex;
        subdiv_edge_index = subdiv_foreach_edges_row(
            edges,
            subdiv_edge_index,
            start_ptex_face_vert_index,
            ptex_face_inner_resolution + 1,
        );
        for row in 0..ptex_face_inner_resolution - 1 {
            let start_row_vert_index =
                start_ptex_face_vert_index + row * (ptex_face_inner_resolution + 1);
            subdiv_edge_index = subdiv_foreach_edges_column(
                edges,
                subdiv_edge_index,
                start_row_vert_index,
                ptex_face_inner_resolution + 1,
            );
            subdiv_edge_index = subdiv_foreach_edges_row(
                edges,
                subdiv_edge_index,
                start_row_vert_index + ptex_face_inner_resolution + 1,
                ptex_face_inner_resolution + 1,
            );
        }
    }
    for corner in 0..face_size {
        let next_corner = (corner + 1) % face_size;
        let mut current_patch_vert_index = start_vert_index
            + corner * num_inner_vertices_per_ptex
            + ptex_face_inner_resolution;
        let mut next_path_vert_index = start_vert_index
            + next_corner * num_inner_vertices_per_ptex
            + num_inner_vertices_per_ptex
            - ptex_face_resolution
            + 1;
        for _row in 0..ptex_face_inner_resolution {
            edges[subdiv_edge_index] = [current_patch_vert_index, next_path_vert_index];
            subdiv_edge_index += 1;
            current_patch_vert_index += ptex_face_inner_resolution + 1;
            next_path_vert_index += 1;
        }
    }
    if ptex_face_resolution >= 3 {
        for corner in 0..face_size {
            let current_patch_end_vert_index = start_vert_index
                + corner * num_inner_vertices_per_ptex
                + num_inner_vertices_per_ptex
                - 1;
            edges[subdiv_edge_index] = [center_vert_index, current_patch_end_vert_index];
            subdiv_edge_index += 1;
        }
    }
    let mut prev_corner = face_size - 1;
    for corner in 0..face_size {
        let c = ctx.corner(coarse_face_index, corner as usize);
        let cp = ctx.corner(coarse_face_index, prev_corner as usize);
        let coarse_vert = ctx.coarse_corner_verts[c];
        let coarse_edge_i = ctx.coarse_corner_edges[c] as usize;
        let coarse_prev_edge = ctx.coarse_corner_edges[cp] as usize;
        {
            let coarse_edge = ctx.coarse_edges[coarse_edge_i];
            let start_edge_vert = ctx.layout.edge_verts_start
                + coarse_edge_i as i32 * num_subdiv_vertices_per_coarse_edge;
            let flip = coarse_edge[1] == coarse_vert;
            let side_start_index = if ptex_face_resolution >= 3 {
                start_vert_index + num_inner_vertices_per_ptex * corner
            } else {
                center_vert_index
            };
            for i in 0..ptex_face_resolution - 1 {
                let v1 = if flip {
                    start_edge_vert + (resolution - i - 3)
                } else {
                    start_edge_vert + i
                };
                edges[subdiv_edge_index] = [v1, side_start_index + i];
                subdiv_edge_index += 1;
            }
        }
        if ptex_face_resolution >= 3 {
            let coarse_edge = ctx.coarse_edges[coarse_prev_edge];
            let start_edge_vert = ctx.layout.edge_verts_start
                + coarse_prev_edge as i32 * num_subdiv_vertices_per_coarse_edge;
            let flip = coarse_edge[1] == coarse_vert;
            let side_start_index = start_vert_index + num_inner_vertices_per_ptex * corner;
            for i in 0..ptex_face_resolution - 2 {
                let v1 = if flip {
                    start_edge_vert + (resolution - i - 3)
                } else {
                    start_edge_vert + i
                };
                let v2 = side_start_index + (ptex_face_inner_resolution + 1) * i;
                edges[subdiv_edge_index] = [v1, v2];
                subdiv_edge_index += 1;
            }
        }
        prev_corner = corner;
    }
}

/// `fill_base_edge_edges` — `selection` = list of coarse edge indices.
pub fn fill_base_edge_edges(
    layout: &MeshLayout,
    base_edges: &[[i32; 2]],
    selection: &[i32],
    edges: &mut [[i32; 2]],
) {
    let resolution = layout.resolution;
    let num_subdiv_edges_per_coarse_edge = resolution - 1;
    debug_assert_eq!(
        edges.len() as i32,
        selection.len() as i32 * num_subdiv_edges_per_coarse_edge
    );
    for (pos, &cei) in selection.iter().enumerate() {
        let coarse_edge = base_edges[cei as usize];
        let (evs, _evn) = layout.base_edge_verts(cei);
        let dst = &mut edges
            [pos * num_subdiv_edges_per_coarse_edge as usize
                ..(pos + 1) * num_subdiv_edges_per_coarse_edge as usize];
        let mut last_vert_index = coarse_edge[0];
        let n = num_subdiv_edges_per_coarse_edge as usize;
        for i in 0..n - 1 {
            let v2 = evs + i as i32;
            dst[i] = [last_vert_index, v2];
            last_vert_index = v2;
        }
        dst[n - 1] = [last_vert_index, coarse_edge[1]];
    }
}

/// `fill_inner_edges`.
pub fn fill_inner_edges(layout: &MeshLayout, ctx: &TraversalContext, edges: &mut [[i32; 2]]) {
    for face in 0..ctx.coarse_face_offsets.len() - 1 {
        let (s, n) = layout.inner_edges_of_face(face);
        let face_edges = &mut edges[s as usize..(s + n) as usize];
        if ctx.face_size(face) == 4 {
            subdiv_foreach_edges_all_patches_regular(ctx, face, face_edges);
        } else {
            subdiv_foreach_edges_all_patches_special(ctx, face, face_edges);
        }
    }
}

fn rotate_indices(rot: i32, v: &mut [i32; 4]) {
    let values = *v;
    for i in 0..4 {
        v[i] = values[((i as i32 - rot + 4) % 4) as usize];
    }
}

/// `FaceCornerOutputs` — optional output slices (empty = skip).
#[derive(Default)]
pub struct FaceCornerOutputs<'a> {
    /// `corner_verts`.
    pub corner_verts: Option<&'a mut [i32]>,
    /// `corner_edges`.
    pub corner_edges: Option<&'a mut [i32]>,
    /// `corner_ptex_coords`.
    pub corner_ptex_coords: Option<&'a mut [PtexCoord]>,
    /// `face_base_corners` (per subdivided face).
    pub face_base_corners: Option<&'a mut [i32]>,
}

#[allow(clippy::too_many_arguments)]
fn subdiv_foreach_loops_of_face(
    out: &mut FaceCornerOutputs,
    subdiv_loop_start_index: usize,
    ptex_face_index: i32,
    coarse_corner_index: i32,
    rotation: i32,
    mut vv: [i32; 4],
    mut ee: [i32; 4],
    u: f32,
    v: f32,
    du: f32,
    dv: f32,
) {
    rotate_indices(rotation, &mut vv);
    rotate_indices(rotation, &mut ee);
    if let Some(s) = out.corner_verts.as_deref_mut() {
        for i in 0..4 {
            s[subdiv_loop_start_index + i] = vv[i];
        }
    }
    if let Some(s) = out.corner_edges.as_deref_mut() {
        for i in 0..4 {
            s[subdiv_loop_start_index + i] = ee[i];
        }
    }
    if let Some(s) = out.corner_ptex_coords.as_deref_mut() {
        let c = [
            (u, v),
            (u + du, v),
            (u + du, v + dv),
            (u, v + dv),
        ];
        for i in 0..4 {
            s[subdiv_loop_start_index + i] = PtexCoord {
                ptex_face_index,
                u: c[i].0,
                v: c[i].1,
            };
        }
    }
    if let Some(s) = out.face_base_corners.as_deref_mut() {
        s[subdiv_loop_start_index / 4] = coarse_corner_index;
    }
}

fn subdiv_foreach_loops_corner_index(u: f32, v: f32, du: f32, dv: f32) -> i32 {
    if u + du <= 0.5 && v + dv <= 0.5 {
        return 0;
    }
    if u >= 0.5 && v + dv <= 0.5 {
        return 1;
    }
    if u >= 0.5 && v >= 0.5 {
        return 2;
    }
    3
}

fn subdiv_foreach_loops_regular(
    ctx: &TraversalContext,
    coarse_face_index: usize,
    ptex_face_index: i32,
    out: &mut FaceCornerOutputs,
) {
    let resolution = ctx.layout.resolution;
    let face_size = ctx.face_size(coarse_face_index);
    let ptex_resolution = ptex_face_resolution_get(face_size, resolution);
    let ptex_inner_resolution = ptex_resolution - 2;
    let num_subdiv_edges_per_coarse_edge = resolution - 1;
    let num_subdiv_vertices_per_coarse_edge = resolution - 2;
    let inv_ptex_resolution_1 = 1.0 / (ptex_resolution - 1) as f32;
    let start_vert_index =
        ctx.layout.inner_verts_start + ctx.layout.inner_vert_offsets[coarse_face_index];
    let start_edge_index =
        ctx.layout.inner_edges_start + ctx.layout.inner_edge_offsets[coarse_face_index];
    let du = inv_ptex_resolution_1;
    let dv = inv_ptex_resolution_1;
    let mut subdiv_loop_index = 0usize;
    for y in 1..ptex_resolution - 2 {
        let v = y as f32 * inv_ptex_resolution_1;
        let inner_y = y - 1;
        for x in 1..ptex_resolution - 2 {
            let inner_x = x - 1;
            let u = x as f32 * inv_ptex_resolution_1;
            let v0 = start_vert_index + (inner_y * ptex_inner_resolution + inner_x);
            let v1 = v0 + 1;
            let v2 = v0 + ptex_inner_resolution + 1;
            let v3 = v0 + ptex_inner_resolution;
            let e0 = start_edge_index + (inner_y * (2 * ptex_inner_resolution - 1) + inner_x);
            let e1 = e0 + ptex_inner_resolution;
            let e2 = e0 + (2 * ptex_inner_resolution - 1);
            let e3 = e0 + ptex_inner_resolution - 1;
            subdiv_foreach_loops_of_face(
                out,
                subdiv_loop_index,
                ptex_face_index,
                subdiv_foreach_loops_corner_index(u, v, du, dv),
                0,
                [v0, v1, v2, v3],
                [e0, e1, e2, e3],
                u,
                v,
                du,
                dv,
            );
            subdiv_loop_index += 4;
        }
    }
    let mut prev_corner_index = face_size - 1;
    for corner in 0..face_size {
        let c = ctx.corner(coarse_face_index, corner as usize);
        let cp = ctx.corner(coarse_face_index, prev_corner_index as usize);
        let coarse_vert = ctx.coarse_corner_verts[c];
        let coarse_edge_i = ctx.coarse_corner_edges[c] as usize;
        let coarse_prev_vert = ctx.coarse_corner_verts[cp];
        let coarse_prev_edge = ctx.coarse_corner_edges[cp] as usize;
        let coarse_edge = ctx.coarse_edges[coarse_edge_i];
        let prev_coarse_edge = ctx.coarse_edges[coarse_prev_edge];
        let start_edge_vert = ctx.layout.edge_verts_start
            + coarse_edge_i as i32 * num_subdiv_vertices_per_coarse_edge;
        let flip = coarse_edge[1] == coarse_vert;
        let mut side_start_index = start_vert_index;
        let mut side_stride = 0;
        let mut v0 = coarse_vert;
        let (mut v3, mut e3);
        if coarse_prev_vert == prev_coarse_edge[0] {
            v3 = ctx.layout.edge_verts_start
                + coarse_prev_edge as i32 * num_subdiv_vertices_per_coarse_edge
                + num_subdiv_vertices_per_coarse_edge
                - 1;
            e3 = (coarse_prev_edge as i32 + 1) * num_subdiv_edges_per_coarse_edge - 1;
        } else {
            v3 = ctx.layout.edge_verts_start
                + coarse_prev_edge as i32 * num_subdiv_vertices_per_coarse_edge;
            e3 = coarse_prev_edge as i32 * num_subdiv_edges_per_coarse_edge;
        }
        let (mut e2_offset, mut e2_stride) = (0i32, 0i32);
        let (mut u, mut v, mut delta_u, mut delta_v) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        match corner {
            0 => {
                side_stride = 1;
                e2_offset = 0;
                e2_stride = 1;
                u = 0.0;
                v = 0.0;
                delta_u = du;
                delta_v = 0.0;
            }
            1 => {
                side_start_index += resolution - 3;
                side_stride = resolution - 2;
                e2_offset = 2 * num_subdiv_edges_per_coarse_edge - 4;
                e2_stride = 2 * num_subdiv_edges_per_coarse_edge - 3;
                u = 1.0 - du;
                v = 0.0;
                delta_u = 0.0;
                delta_v = dv;
            }
            2 => {
                side_start_index += num_subdiv_vertices_per_coarse_edge
                    * num_subdiv_vertices_per_coarse_edge
                    - 1;
                side_stride = -1;
                e2_offset = num_edges_per_ptex_face_get(resolution - 2) - 1;
                e2_stride = -1;
                u = 1.0 - du;
                v = 1.0 - dv;
                delta_u = -du;
                delta_v = 0.0;
            }
            3 => {
                side_start_index += num_subdiv_vertices_per_coarse_edge
                    * (num_subdiv_vertices_per_coarse_edge - 1);
                side_stride = -(resolution - 2);
                e2_offset = num_edges_per_ptex_face_get(resolution - 2)
                    - (2 * num_subdiv_edges_per_coarse_edge - 3);
                e2_stride = -(2 * num_subdiv_edges_per_coarse_edge - 3);
                u = 0.0;
                v = 1.0 - dv;
                delta_u = 0.0;
                delta_v = -dv;
            }
            _ => {}
        }
        for i in 0..resolution - 2 {
            let v1 = if flip {
                start_edge_vert + (resolution - i - 3)
            } else {
                start_edge_vert + i
            };
            let v2 = side_start_index + side_stride * i;
            let e0 = if flip {
                (coarse_edge_i as i32 + 1) * num_subdiv_edges_per_coarse_edge - i - 1
            } else {
                coarse_edge_i as i32 * num_subdiv_edges_per_coarse_edge + i
            };
            let e1 = start_edge_index
                + num_edges_per_ptex_face_get(resolution - 2)
                + corner * num_subdiv_vertices_per_coarse_edge
                + i;
            let e2 = if i == 0 {
                start_edge_index
                    + num_edges_per_ptex_face_get(resolution - 2)
                    + ((corner - 1 + face_size) % face_size) * num_subdiv_vertices_per_coarse_edge
                    + num_subdiv_vertices_per_coarse_edge
                    - 1
            } else {
                start_edge_index + e2_offset + e2_stride * (i - 1)
            };
            let loop_u = u + delta_u * i as f32;
            let loop_v = v + delta_v * i as f32;
            subdiv_foreach_loops_of_face(
                out,
                subdiv_loop_index,
                ptex_face_index,
                subdiv_foreach_loops_corner_index(loop_u, loop_v, du, dv),
                corner,
                [v0, v1, v2, v3],
                [e0, e1, e2, e3],
                loop_u,
                loop_v,
                du,
                dv,
            );
            v0 = v1;
            v3 = v2;
            e3 = e1;
            subdiv_loop_index += 4;
        }
        prev_corner_index = corner;
    }
}

fn subdiv_foreach_loops_special(
    ctx: &TraversalContext,
    coarse_face_index: usize,
    ptex_face_index: i32,
    out: &mut FaceCornerOutputs,
) {
    let resolution = ctx.layout.resolution;
    let face_size = ctx.face_size(coarse_face_index);
    let ptex_face_resolution = ptex_face_resolution_get(face_size, resolution);
    let ptex_face_inner_resolution = ptex_face_resolution - 2;
    let inv_ptex_resolution_1 = 1.0 / (ptex_face_resolution - 1) as f32;
    let num_inner_vertices_per_ptex =
        (ptex_face_resolution - 1) * (ptex_face_resolution - 2);
    let num_inner_edges_per_ptex_face =
        num_inner_edges_per_ptex_face_get(ptex_face_inner_resolution + 1);
    let num_subdiv_vertices_per_coarse_edge = resolution - 2;
    let num_subdiv_edges_per_coarse_edge = resolution - 1;
    let center_vert_index =
        ctx.layout.inner_verts_start + ctx.layout.inner_vert_offsets[coarse_face_index];
    let start_vert_index = center_vert_index + 1;
    let start_inner_vert_index = center_vert_index + 1;
    let start_edge_index =
        ctx.layout.inner_edges_start + ctx.layout.inner_edge_offsets[coarse_face_index];
    let du = inv_ptex_resolution_1;
    let dv = inv_ptex_resolution_1;
    let mut subdiv_loop_index = 0usize;
    for corner in 0..face_size {
        let corner_vert_index = start_vert_index + corner * num_inner_vertices_per_ptex;
        let corner_edge_index = start_edge_index + corner * num_inner_edges_per_ptex_face;
        for y in 1..ptex_face_inner_resolution {
            let v = y as f32 * inv_ptex_resolution_1;
            let inner_y = y - 1;
            for x in 1..ptex_face_inner_resolution + 1 {
                let inner_x = x - 1;
                let u = x as f32 * inv_ptex_resolution_1;
                let v0 = corner_vert_index
                    + (inner_y * (ptex_face_inner_resolution + 1) + inner_x);
                let v1 = v0 + 1;
                let v2 = v0 + ptex_face_inner_resolution + 2;
                let v3 = v0 + ptex_face_inner_resolution + 1;
                let e0 = corner_edge_index
                    + (inner_y * (2 * ptex_face_inner_resolution + 1) + inner_x);
                let e1 = e0 + ptex_face_inner_resolution + 1;
                let e2 = e0 + (2 * ptex_face_inner_resolution + 1);
                let e3 = e0 + ptex_face_inner_resolution;
                subdiv_foreach_loops_of_face(
                    out,
                    subdiv_loop_index,
                    ptex_face_index + corner,
                    corner,
                    0,
                    [v0, v1, v2, v3],
                    [e0, e1, e2, e3],
                    u,
                    v,
                    du,
                    dv,
                );
                subdiv_loop_index += 4;
            }
        }
    }
    for corner in 0..face_size {
        let next_corner = (corner + 1) % face_size;
        let corner_edge_index = start_edge_index + corner * num_inner_edges_per_ptex_face;
        let next_corner_edge_index =
            start_edge_index + next_corner * num_inner_edges_per_ptex_face;
        let mut current_patch_vert_index = start_inner_vert_index
            + corner * num_inner_vertices_per_ptex
            + ptex_face_inner_resolution;
        let mut next_path_vert_index = start_inner_vert_index
            + next_corner * num_inner_vertices_per_ptex
            + num_inner_vertices_per_ptex
            - ptex_face_resolution
            + 1;
        let mut v0 = current_patch_vert_index;
        let mut v1 = next_path_vert_index;
        current_patch_vert_index += ptex_face_inner_resolution + 1;
        next_path_vert_index += 1;
        let mut e0 = start_edge_index
            + face_size * num_inner_edges_per_ptex_face
            + corner * (ptex_face_resolution - 2);
        let mut e1 =
            next_corner_edge_index + num_inner_edges_per_ptex_face - ptex_face_resolution + 2;
        let mut e3 = corner_edge_index + 2 * ptex_face_resolution - 4;
        for row in 1..ptex_face_inner_resolution {
            let v2 = next_path_vert_index;
            let v3 = current_patch_vert_index;
            let e2 = e0 + 1;
            let u = row as f32 * du;
            let v = 1.0 - dv;
            subdiv_foreach_loops_of_face(
                out,
                subdiv_loop_index,
                ptex_face_index + next_corner,
                next_corner,
                3,
                [v0, v1, v2, v3],
                [e0, e1, e2, e3],
                u,
                v,
                du,
                dv,
            );
            current_patch_vert_index += ptex_face_inner_resolution + 1;
            next_path_vert_index += 1;
            v0 = v3;
            v1 = v2;
            e0 = e2;
            e1 += 1;
            e3 += 2 * ptex_face_resolution - 3;
            subdiv_loop_index += 4;
        }
    }
    if ptex_face_resolution >= 3 {
        let start_center_edge_index = start_edge_index
            + (num_inner_edges_per_ptex_face + ptex_face_inner_resolution) * face_size;
        let start_boundary_edge = start_edge_index
            + face_size * num_inner_edges_per_ptex_face
            + ptex_face_inner_resolution
            - 1;
        let mut prev_corner = face_size - 1;
        for corner in 0..face_size {
            let corner_edge_index = start_edge_index + corner * num_inner_edges_per_ptex_face;
            let current_patch_end_vert_index = start_vert_index
                + corner * num_inner_vertices_per_ptex
                + num_inner_vertices_per_ptex
                - 1;
            let prev_current_patch_end_vert_index = start_vert_index
                + prev_corner * num_inner_vertices_per_ptex
                + num_inner_vertices_per_ptex
                - 1;
            let v0 = center_vert_index;
            let v1 = prev_current_patch_end_vert_index;
            let v2 = current_patch_end_vert_index - 1;
            let v3 = current_patch_end_vert_index;
            let e0 = start_center_edge_index + prev_corner;
            let e1 = start_boundary_edge + prev_corner * ptex_face_inner_resolution;
            let e2 = corner_edge_index + num_inner_edges_per_ptex_face - 1;
            let e3 = start_center_edge_index + corner;
            let u = 1.0 - du;
            let v = 1.0 - dv;
            subdiv_foreach_loops_of_face(
                out,
                subdiv_loop_index,
                ptex_face_index + corner,
                corner,
                2,
                [v0, v1, v2, v3],
                [e0, e1, e2, e3],
                u,
                v,
                du,
                dv,
            );
            subdiv_loop_index += 4;
            prev_corner = corner;
        }
    }
    let mut prev_corner = face_size - 1;
    for corner in 0..face_size {
        let c = ctx.corner(coarse_face_index, corner as usize);
        let cp = ctx.corner(coarse_face_index, prev_corner as usize);
        let coarse_vert = ctx.coarse_corner_verts[c];
        let coarse_edge_i = ctx.coarse_corner_edges[c] as usize;
        let coarse_prev_vert = ctx.coarse_corner_verts[cp];
        let coarse_prev_edge = ctx.coarse_corner_edges[cp] as usize;
        let coarse_edge = ctx.coarse_edges[coarse_edge_i];
        let prev_coarse_edge = ctx.coarse_edges[coarse_prev_edge];
        let flip = coarse_edge[1] == coarse_vert;
        let start_edge_vert = ctx.layout.edge_verts_start
            + coarse_edge_i as i32 * num_subdiv_vertices_per_coarse_edge;
        let corner_vert_index = start_vert_index + corner * num_inner_vertices_per_ptex;
        let corner_edge_index = start_edge_index + corner * num_inner_edges_per_ptex_face;
        let mut v0 = coarse_vert;
        let (mut v3, mut e3);
        if coarse_prev_vert == prev_coarse_edge[0] {
            v3 = ctx.layout.edge_verts_start
                + coarse_prev_edge as i32 * num_subdiv_vertices_per_coarse_edge
                + num_subdiv_vertices_per_coarse_edge
                - 1;
            e3 = (coarse_prev_edge as i32 + 1) * num_subdiv_edges_per_coarse_edge - 1;
        } else {
            v3 = ctx.layout.edge_verts_start
                + coarse_prev_edge as i32 * num_subdiv_vertices_per_coarse_edge;
            e3 = coarse_prev_edge as i32 * num_subdiv_edges_per_coarse_edge;
        }
        for i in 0..=ptex_face_inner_resolution {
            let v1 = if flip {
                start_edge_vert + (resolution - i - 3)
            } else {
                start_edge_vert + i
            };
            let v2 = if ptex_face_inner_resolution >= 1 {
                corner_vert_index + i
            } else {
                center_vert_index
            };
            let e0 = if flip {
                (coarse_edge_i as i32 + 1) * num_subdiv_edges_per_coarse_edge - i - 1
            } else {
                coarse_edge_i as i32 * num_subdiv_edges_per_coarse_edge + i
            };
            let mut e1 = start_edge_index + corner * (2 * ptex_face_inner_resolution + 1);
            if ptex_face_resolution >= 3 {
                e1 += face_size
                    * (num_inner_edges_per_ptex_face + ptex_face_inner_resolution + 1)
                    + i;
            }
            let e2 = if i == 0 && ptex_face_resolution >= 3 {
                start_edge_index
                    + face_size
                        * (num_inner_edges_per_ptex_face + ptex_face_inner_resolution + 1)
                    + corner * (2 * ptex_face_inner_resolution + 1)
                    + ptex_face_inner_resolution
                    + 1
            } else if i == 0 && ptex_face_resolution < 3 {
                start_edge_index + prev_corner * (2 * ptex_face_inner_resolution + 1)
            } else {
                corner_edge_index + i - 1
            };
            let u = du * i as f32;
            let v = 0.0f32;
            subdiv_foreach_loops_of_face(
                out,
                subdiv_loop_index,
                ptex_face_index + corner,
                corner,
                0,
                [v0, v1, v2, v3],
                [e0, e1, e2, e3],
                u,
                v,
                du,
                dv,
            );
            v0 = v1;
            v3 = v2;
            e3 = e1;
            subdiv_loop_index += 4;
        }
        let flip_prev = prev_coarse_edge[1] == coarse_vert;
        v0 = corner_vert_index;
        if coarse_prev_vert == prev_coarse_edge[0] {
            v3 = ctx.layout.edge_verts_start
                + coarse_prev_edge as i32 * num_subdiv_vertices_per_coarse_edge
                + num_subdiv_vertices_per_coarse_edge
                - 1;
        } else {
            v3 = ctx.layout.edge_verts_start
                + coarse_prev_edge as i32 * num_subdiv_vertices_per_coarse_edge;
        }
        e3 = start_edge_index
            + face_size * (num_inner_edges_per_ptex_face + ptex_face_inner_resolution + 1)
            + corner * (2 * ptex_face_inner_resolution + 1)
            + ptex_face_inner_resolution
            + 1;
        for i in 0..=ptex_face_inner_resolution - 1 {
            let (v1, e0, e1);
            if i == ptex_face_inner_resolution - 1 {
                v1 = start_vert_index
                    + prev_corner * num_inner_vertices_per_ptex
                    + ptex_face_inner_resolution;
                e1 = start_edge_index
                    + face_size
                        * (num_inner_edges_per_ptex_face + ptex_face_inner_resolution + 1)
                    + prev_corner * (2 * ptex_face_inner_resolution + 1)
                    + ptex_face_inner_resolution;
                e0 = start_edge_index
                    + face_size * num_inner_edges_per_ptex_face
                    + prev_corner * ptex_face_inner_resolution;
            } else {
                v1 = v0 + ptex_face_inner_resolution + 1;
                e0 = corner_edge_index
                    + ptex_face_inner_resolution
                    + i * (2 * ptex_face_inner_resolution + 1);
                e1 = e3 + 1;
            }
            let v2 = if flip_prev { v3 - 1 } else { v3 + 1 };
            let e2 = if flip_prev {
                coarse_prev_edge as i32 * num_subdiv_edges_per_coarse_edge
                    + num_subdiv_edges_per_coarse_edge
                    - 2
                    - i
            } else {
                coarse_prev_edge as i32 * num_subdiv_edges_per_coarse_edge + 1 + i
            };
            let u = 0.0f32;
            let v = du * (i + 1) as f32;
            subdiv_foreach_loops_of_face(
                out,
                subdiv_loop_index,
                ptex_face_index + corner,
                corner,
                1,
                [v0, v1, v2, v3],
                [e0, e1, e2, e3],
                u,
                v,
                du,
                dv,
            );
            v0 = v1;
            v3 = v2;
            e3 = e1;
            subdiv_loop_index += 4;
        }
        prev_corner = corner;
    }
}

/// `fill_corner_verts_and_edges`.
pub fn fill_corner_verts_and_edges(
    layout: &MeshLayout,
    ctx: &TraversalContext,
    corner_verts: Option<&mut [i32]>,
    corner_edges: Option<&mut [i32]>,
) {
    let mut corner_verts = corner_verts;
    let mut corner_edges = corner_edges;
    for face in 0..ctx.coarse_face_offsets.len() - 1 {
        let (s, n) = layout.base_face_corners(face);
        let (s, n) = (s as usize, n as usize);
        let mut out = FaceCornerOutputs {
            corner_verts: corner_verts.as_deref_mut().map(|cv| &mut cv[s..s + n]),
            corner_edges: corner_edges.as_deref_mut().map(|ce| &mut ce[s..s + n]),
            corner_ptex_coords: None,
            face_base_corners: None,
        };
        if ctx.face_size(face) == 4 {
            subdiv_foreach_loops_regular(ctx, face, 0, &mut out);
        } else {
            subdiv_foreach_loops_special(ctx, face, 0, &mut out);
        }
    }
}

/// `fill_corner_ptex_coords`.
pub fn fill_corner_ptex_coords(
    layout: &MeshLayout,
    ctx: &TraversalContext,
    face_ptex_offset: &[i32],
    corner_ptex_coords: Option<&mut [PtexCoord]>,
    face_base_corners: Option<&mut [i32]>,
) {
    let mut corner_ptex_coords = corner_ptex_coords;
    let mut face_base_corners = face_base_corners;
    for face in 0..ctx.coarse_face_offsets.len() - 1 {
        let (cs, cn) = layout.base_face_corners(face);
        let (fs, fn_) = layout.faces_of_base_face(face);
        let mut out = FaceCornerOutputs {
            corner_verts: None,
            corner_edges: None,
            corner_ptex_coords: corner_ptex_coords
                .as_deref_mut()
                .map(|s| &mut s[cs as usize..(cs + cn) as usize]),
            face_base_corners: face_base_corners
                .as_deref_mut()
                .map(|s| &mut s[fs as usize..(fs + fn_) as usize]),
        };
        if ctx.face_size(face) == 4 {
            subdiv_foreach_loops_regular(ctx, face, face_ptex_offset[face], &mut out);
        } else {
            subdiv_foreach_loops_special(ctx, face, face_ptex_offset[face], &mut out);
        }
    }
}
