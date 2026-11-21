//! Port of `intern/pbvh_pixels.cc` (+ `pbvh_pixels_rasterize.hh`) —
//! UV-space pixel encoding for PBVH sculpt/texture painting:
//! `TriRasterizer`, `calc_pixel_to_position_map`,
//! `extract_barycentric_pixels`, `build_border_triangles`,
//! `build_pixel_row_runs`, `foreach_uv_triangle`, `do_encode_pixels`,
//! node update masks and `apply_watertight_check`.
//!
//! `uv_islands::MeshData` (mesh + uv_map + island ids) is passed as
//! `PixelMeshData`; `UVIslandsMask`/`ImBuf`/image tiles are injected.

use blenders_math::{Mat3, Vec2, Vec3};

/// `uv_islands::MeshData` subset this file needs.
pub struct PixelMeshData<'a> {
    /// `vert_positions`.
    pub vert_positions: &'a [Vec3],
    /// `corner_verts`.
    pub corner_verts: &'a [i32],
    /// `corner_tris`.
    pub corner_tris: &'a [[i32; 3]],
    /// `uv_map` (corner-domain).
    pub uv_map: &'a [Vec2],
    /// `uv_island_ids` per tri.
    pub uv_island_ids: &'a [i32],
}

/// `uv_islands::UVPrimitive` — only `primitive_i` is used here.
#[derive(Clone, Copy)]
pub struct UVPrimitive {
    /// `primitive_i` — the mesh tri index.
    pub primitive_i: i32,
    /// `get_uv_vert(island, mesh, k)` result — index into
    /// `uv_verts`.
    pub uv_verts: [i32; 3],
}

/// `uv_islands::UVIsland` subset.
pub struct UVIsland<'a> {
    /// `num_original_primitives`.
    pub num_original_primitives: usize,
    /// `uv_primitives`.
    pub uv_primitives: &'a [UVPrimitive],
    /// `uv_verts[].uv` — island-local uvs.
    pub uv_verts: &'a [Vec2],
}

/// `UVIslandsMask::Tile` — per-tile island membership mask.
pub struct MaskTile<'a> {
    /// `mask_resolution`.
    pub mask_resolution: [i32; 2],
    /// `is_masked(uv_island_index, x, y)`.
    pub is_masked: &'a dyn Fn(i32, i32, i32) -> bool,
}

/// `TriRasterizer` — triangle edge-function rasterizer for a
/// triangle in normalized UV space over an `image_size` pixel grid.
pub struct TriRasterizer {
    edges: [[f32; 3]; 3],
    positive_side: [bool; 3],
    /// `bounds` — `(min, max)` pixel bounds, empty when `min > max`.
    pub bounds: ([i32; 2], [i32; 2]),
}

fn cross2(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

impl TriRasterizer {
    /// `TriRasterizer(uv0, uv1, uv2, image_size)`.
    pub fn new(uv0: Vec2, uv1: Vec2, uv2: Vec2, image_size: [i32; 2]) -> Self {
        let fs = Vec2::new(image_size[0] as f32, image_size[1] as f32);
        let p = [uv0 * fs, uv1 * fs, uv2 * fs];

        let bounds = if p.iter().all(|v| v.x.is_finite() && v.y.is_finite()) {
            let mut lo = p[0];
            let mut hi = p[0];
            for v in &p[1..] {
                lo = lo.min(*v);
                hi = hi.max(*v);
            }
            let clamp = |v: Vec2, f: fn(f32) -> f32| -> [i32; 2] {
                [
                    (f(v.x) as i32).clamp(0, image_size[0]),
                    (f(v.y) as i32).clamp(0, image_size[1]),
                ]
            };
            (clamp(lo, f32::floor), clamp(hi, f32::ceil))
        } else {
            ([1, 1], [0, 0])
        };

        let mut edges = [[0.0f32; 3]; 3];
        let mut positive_side = [false; 3];
        for i in 0..3 {
            let mut a = p[i];
            let mut b = p[(i + 1) % 3];
            let c = p[(i + 2) % 3];
            if b.x < a.x || (b.x == a.x && b.y < a.y) {
                std::mem::swap(&mut a, &mut b);
            }
            let d = b - a;
            edges[i] = [-d.y, d.x, cross2(a, d)];
            positive_side[i] = cross2(d, c - a) > 0.0;
        }

        Self {
            edges,
            positive_side,
            bounds,
        }
    }

    /// `inside(x, y)` — pixel-center edge-function test.
    pub fn inside(&self, x: i32, y: i32) -> bool {
        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
        (0..3).all(|i| {
            let v = self.edges[i][0] * px + self.edges[i][1] * py + self.edges[i][2];
            (v >= 0.0) == self.positive_side[i]
        })
    }
}

/// `calc_pixel_to_position_map` — affine map from pixel coords to
/// mesh positions via UV→position gradient. Returns `(delta_x,
/// delta_y, start_position)` packed as the C's `float3x3`.
pub fn calc_pixel_to_position_map(
    mesh_data: &PixelMeshData,
    tri: usize,
    uvs: &[Vec2; 3],
    inv_w: f32,
    inv_h: f32,
) -> [Vec3; 3] {
    let corner_tri = mesh_data.corner_tris[tri];
    let v0 = mesh_data.vert_positions[mesh_data.corner_verts[corner_tri[0] as usize] as usize];
    let v1 = mesh_data.vert_positions[mesh_data.corner_verts[corner_tri[1] as usize] as usize];
    let v2 = mesh_data.vert_positions[mesh_data.corner_verts[corner_tri[2] as usize] as usize];

    let e0 = v0 - v2;
    let e1 = v1 - v2;
    let a = uvs[0] - uvs[2];
    let b = uvs[1] - uvs[2];
    let det = a.x * b.y - a.y * b.x;
    if det == 0.0 {
        return [Vec3::ZERO, Vec3::ZERO, v2];
    }

    let dp_du = (b.y * e0 - a.y * e1) / det;
    let dp_dv = (a.x * e1 - b.x * e0) / det;

    let delta_x = dp_du * inv_w;
    let delta_y = dp_dv * inv_h;
    let start_position = dp_du * (0.5 * inv_w - uvs[2].x) + dp_dv * (0.5 * inv_h - uvs[2].y) + v2;

    [delta_x, delta_y, start_position]
}

/// `BuildPixelRow`.
#[derive(Debug, Clone, Copy)]
pub struct BuildPixelRow {
    /// `start_image_coordinate`.
    pub start_image_coordinate: [u16; 2],
    /// `num_pixels`.
    pub num_pixels: u16,
    /// `uv_primitive_index`.
    pub uv_primitive_index: u16,
}

/// `extract_barycentric_pixels` — for each rasterized row inside
/// `mask_tile`, emit one `BuildPixelRow`.
pub fn extract_barycentric_pixels(
    build_rows: &mut Vec<BuildPixelRow>,
    image_size: [i32; 2],
    mask_tile: &MaskTile,
    uv_island_index: i32,
    uv_primitive_index: u16,
    uvs: &[Vec2; 3],
) {
    let inv_w = 1.0 / image_size[0] as f32;
    let inv_h = 1.0 / image_size[1] as f32;

    let (mr_x, mr_y) = (mask_tile.mask_resolution[0], mask_tile.mask_resolution[1]);
    let mask_scale_x = mr_x as f32 * inv_w;
    let mask_scale_y = mr_y as f32 * inv_h;

    let rasterizer = TriRasterizer::new(uvs[0], uvs[1], uvs[2], image_size);
    let (min, max) = rasterizer.bounds;

    for y in min[1]..max[1] {
        let mut start_detected = false;
        let mut pixel_row = BuildPixelRow {
            start_image_coordinate: [0, 0],
            num_pixels: 0,
            uv_primitive_index,
        };
        let mut x = min[0];

        let fy = y as f32 + 0.5;
        let mask_y = ((fy * mask_scale_y) as i32).clamp(0, mr_y - 1);

        while x < max[0] {
            let fx = x as f32 + 0.5;
            let mask_x = ((fx * mask_scale_x) as i32).clamp(0, mr_x - 1);
            let is_masked = (mask_tile.is_masked)(uv_island_index, mask_x, mask_y);
            let is_inside = rasterizer.inside(x, y);

            if !start_detected && is_inside && is_masked {
                start_detected = true;
                pixel_row.start_image_coordinate = [x as u16, y as u16];
            } else if start_detected && (!is_inside || !is_masked) {
                break;
            }
            x += 1;
        }

        if !start_detected {
            continue;
        }
        pixel_row.num_pixels = (x - pixel_row.start_image_coordinate[0] as i32) as u16;
        build_rows.push(pixel_row);
    }
}

/// `BorderTriangle`.
#[derive(Clone, Copy)]
pub struct BorderTriangle {
    /// `uvs`.
    pub uvs: [Vec2; 3],
    /// `uv_island_index`.
    pub uv_island_index: i32,
}

/// `build_border_triangles` — group border (added) primitives per
/// source triangle. Returns `(offsets, data)` — `data[offsets[i]..
/// offsets[i+1]]` are tri `i`'s border tris (`GroupedSpan`).
pub fn build_border_triangles(
    _mesh_data: &PixelMeshData,
    geom_primitive_len: usize,
    uv_islands: &[UVIsland],
) -> (Vec<i32>, Vec<BorderTriangle>) {
    let mut counts = vec![0i32; geom_primitive_len];
    for island in uv_islands {
        for prim in &island.uv_primitives[island.num_original_primitives..] {
            counts[prim.primitive_i as usize] += 1;
        }
    }
    let mut offsets = vec![0i32; geom_primitive_len + 1];
    for i in 0..geom_primitive_len {
        offsets[i + 1] = offsets[i] + counts[i];
    }
    let total = *offsets.last().unwrap() as usize;
    let mut data = vec![
        BorderTriangle {
            uvs: [Vec2::ZERO; 3],
            uv_island_index: 0
        };
        total
    ];
    let mut current = offsets[..geom_primitive_len].to_vec();
    for (island_index, island) in uv_islands.iter().enumerate() {
        for prim in &island.uv_primitives[island.num_original_primitives..] {
            let mut uvs = [Vec2::ZERO; 3];
            for (k, slot) in uvs.iter_mut().enumerate() {
                *slot = island.uv_verts[prim.uv_verts[k] as usize];
            }
            let slot = &mut current[prim.primitive_i as usize];
            data[*slot as usize] = BorderTriangle {
                uvs,
                uv_island_index: island_index as i32,
            };
            *slot += 1;
        }
    }
    (offsets, data)
}

/// `PackedPixelRow` (stored in `UDIMTilePixels`).
#[derive(Debug, Clone, Copy)]
pub struct PackedPixelRow {
    /// `num_pixels`.
    pub num_pixels: u16,
    /// `uv_primitive_index`.
    pub uv_primitive_index: u16,
}

/// `UDIMTilePixels`.
#[derive(Default)]
pub struct UDIMTilePixels {
    /// `tile_number`.
    pub tile_number: i32,
    /// `pixel_rows`.
    pub pixel_rows: Vec<PackedPixelRow>,
    /// `pixel_row_run_starts`.
    pub pixel_row_run_starts: Vec<i32>,
    /// `pixel_row_run_start_coords`.
    pub pixel_row_run_start_coords: Vec<[u16; 2]>,
}

/// `build_pixel_row_runs` — sort rows by (y, x) and collapse
/// contiguous runs.
pub fn build_pixel_row_runs(build_rows: &mut Vec<BuildPixelRow>, tile_data: &mut UDIMTilePixels) {
    build_rows.sort_by(|a, b| {
        a.start_image_coordinate[1]
            .cmp(&b.start_image_coordinate[1])
            .then(a.start_image_coordinate[0].cmp(&b.start_image_coordinate[0]))
    });

    let n = build_rows.len();
    tile_data.pixel_rows.reserve(n);
    tile_data.pixel_row_run_starts.clear();
    tile_data.pixel_row_run_start_coords.clear();
    for (i, row) in build_rows.iter().enumerate() {
        if i == 0 {
            tile_data.pixel_row_run_starts.push(0);
            tile_data
                .pixel_row_run_start_coords
                .push(row.start_image_coordinate);
        } else {
            let prev = &build_rows[i - 1];
            let contiguous = row.start_image_coordinate[1] == prev.start_image_coordinate[1]
                && row.start_image_coordinate[0]
                    == prev.start_image_coordinate[0] + prev.num_pixels;
            if !contiguous {
                tile_data.pixel_row_run_starts.push(i as i32);
                tile_data
                    .pixel_row_run_start_coords
                    .push(row.start_image_coordinate);
            }
        }
        tile_data.pixel_rows.push(PackedPixelRow {
            num_pixels: row.num_pixels,
            uv_primitive_index: row.uv_primitive_index,
        });
    }
    tile_data.pixel_row_run_starts.push(n as i32);
}

/// `foreach_uv_triangle` — invoke `fn` on the original tri's uvs
/// then each of its border tris.
pub fn foreach_uv_triangle(
    mesh_data: &PixelMeshData,
    border_tris_offsets: &[i32],
    border_tris_data: &[BorderTriangle],
    tri: usize,
    mut f: impl FnMut([Vec2; 3], i32, bool),
) {
    let corner_tri = mesh_data.corner_tris[tri];
    let uvs = [
        mesh_data.uv_map[corner_tri[0] as usize],
        mesh_data.uv_map[corner_tri[1] as usize],
        mesh_data.uv_map[corner_tri[2] as usize],
    ];
    f(uvs, mesh_data.uv_island_ids[tri], true);

    let (lo, hi) = (
        border_tris_offsets[tri] as usize,
        border_tris_offsets[tri + 1] as usize,
    );
    for border_tri in &border_tris_data[lo..hi] {
        f(border_tri.uvs, border_tri.uv_island_index, false);
    }
}

/// `PixelNode` UV-primitive data (`uv_primitives`).
#[derive(Default)]
pub struct PixelNodeUVPrimitives {
    /// `tri_indices`.
    pub tri_indices: Vec<i32>,
    /// `pixel_to_position` — `(delta_x, delta_y, start)`.
    pub pixel_to_position: Vec<[Vec3; 3]>,
    /// `original_tris`.
    pub original_tris: Vec<bool>,
}

/// `PixelNode` — per-PBVH-node encoded pixel data.
#[derive(Default)]
pub struct PixelNode {
    /// `flags.rebuild`.
    pub rebuild: bool,
    /// `uv_primitives`.
    pub uv_primitives: PixelNodeUVPrimitives,
    /// `tiles`.
    pub tiles: Vec<UDIMTilePixels>,
    /// `flags.dirty`.
    pub dirty: bool,
}

impl PixelNode {
    /// `clear_data`.
    pub fn clear_data(&mut self) {
        self.uv_primitives = PixelNodeUVPrimitives::default();
        self.tiles.clear();
    }

    /// `find_tile_data` by tile number.
    pub fn find_tile_data(&mut self, tile_number: i32) -> Option<&mut UDIMTilePixels> {
        self.tiles.iter_mut().find(|t| t.tile_number == tile_number)
    }

    /// `collect_dirty_tiles`.
    pub fn collect_dirty_tiles(&self, r_dirty_tiles: &mut Vec<i32>) {
        if self.dirty {
            for t in &self.tiles {
                r_dirty_tiles.push(t.tile_number);
            }
        }
    }
}

/// `do_encode_pixels` — encode UV-rasterized pixels per image tile
/// into `pixel_node`. `image` exposes tiles via `tiles`; per-tile
/// `mask`/`acquire` hooks injected (`imbuf_size` gives the ImBuf
/// dims for a tile).
#[allow(clippy::too_many_arguments)]
pub fn do_encode_pixels<'m>(
    mesh_data: &PixelMeshData,
    border_tris_offsets: &[i32],
    border_tris_data: &[BorderTriangle],
    node_faces: &[i32],
    face_triangles: &dyn Fn(i32) -> Vec<i32>,
    tiles: &[[i32; 2]],
    tile_numbers: &[i32],
    mask_for_tile: &dyn Fn([i32; 2]) -> Option<MaskTile<'m>>,
    imbuf_size: &dyn Fn(i32) -> Option<[i32; 2]>,
    pixel_node: &mut PixelNode,
) {
    debug_assert!(
        pixel_node.rebuild
            || (pixel_node.uv_primitives.tri_indices.is_empty()
                && pixel_node.uv_primitives.pixel_to_position.is_empty()
                && pixel_node.tiles.is_empty())
    );

    let mut tri_indices: Vec<i32> = Vec::new();
    let mut pixel_to_position: Vec<[Vec3; 3]> = Vec::new();
    let mut original_tris: Vec<bool> = Vec::new();

    let multi_tile = tiles.len() > 1;
    let mut node_tiles: Vec<[i32; 2]> = Vec::new();
    if multi_tile {
        let mut tiles_min = [i32::MAX; 2];
        let mut tiles_max = [i32::MIN; 2];
        for t in tiles {
            for k in 0..2 {
                tiles_min[k] = tiles_min[k].min(t[k]);
                tiles_max[k] = tiles_max[k].max(t[k]);
            }
        }
        for &face in node_faces {
            for tri in face_triangles(face) {
                foreach_uv_triangle(
                    mesh_data,
                    border_tris_offsets,
                    border_tris_data,
                    tri as usize,
                    |uvs, _, _| {
                        let mut uv_min = Vec2::splat(f32::MAX);
                        let mut uv_max = Vec2::splat(f32::MIN);
                        for uv in uvs {
                            uv_min = uv_min.min(uv);
                            uv_max = uv_max.max(uv);
                        }
                        let tile_min = [
                            (uv_min.x.floor() as i32).clamp(tiles_min[0], tiles_max[0]),
                            (uv_min.y.floor() as i32).clamp(tiles_min[1], tiles_max[1]),
                        ];
                        let tile_max = [
                            (uv_max.x.floor() as i32).clamp(tiles_min[0], tiles_max[0]),
                            (uv_max.y.floor() as i32).clamp(tiles_min[1], tiles_max[1]),
                        ];
                        for ty in tile_min[1]..=tile_max[1] {
                            for tx in tile_min[0]..=tile_max[0] {
                                let t = [tx, ty];
                                if !node_tiles.contains(&t) {
                                    node_tiles.push(t);
                                }
                            }
                        }
                    },
                );
            }
        }
    }

    for (tile_i, &tile_offset_i) in tiles.iter().enumerate() {
        let tile_offset = Vec2::new(tile_offset_i[0] as f32, tile_offset_i[1] as f32);

        if multi_tile && !node_tiles.contains(&tile_offset_i) {
            continue;
        }

        let Some(mask_tile) = mask_for_tile(tile_offset_i) else {
            continue;
        };

        let tile_number = tile_numbers.get(tile_i).copied().unwrap_or(0);
        let Some(image_size) = imbuf_size(tile_number) else {
            continue;
        };
        let inv_w = 1.0 / image_size[0] as f32;
        let inv_h = 1.0 / image_size[1] as f32;

        let mut tile_data = UDIMTilePixels {
            tile_number,
            ..Default::default()
        };
        let mut build_rows: Vec<BuildPixelRow> = Vec::new();

        for &face in node_faces {
            for tri in face_triangles(face) {
                foreach_uv_triangle(
                    mesh_data,
                    border_tris_offsets,
                    border_tris_data,
                    tri as usize,
                    |tri_uvs, island_index, is_original| {
                        let uvs = [
                            tri_uvs[0] - tile_offset,
                            tri_uvs[1] - tile_offset,
                            tri_uvs[2] - tile_offset,
                        ];
                        let uv_prim_index = tri_indices.len() as u16;
                        let before = build_rows.len();
                        extract_barycentric_pixels(
                            &mut build_rows,
                            image_size,
                            &mask_tile,
                            island_index,
                            uv_prim_index,
                            &uvs,
                        );
                        if build_rows.len() == before {
                            return;
                        }
                        tri_indices.push(tri);
                        pixel_to_position.push(calc_pixel_to_position_map(
                            mesh_data, tri as usize, &uvs, inv_w, inv_h,
                        ));
                        original_tris.push(is_original);
                    },
                );
            }
        }

        if build_rows.is_empty() {
            continue;
        }

        build_pixel_row_runs(&mut build_rows, &mut tile_data);
        debug_assert_eq!(pixel_to_position.len(), tri_indices.len());
        debug_assert_eq!(original_tris.len(), tri_indices.len());

        pixel_node.tiles.push(tile_data);
    }

    pixel_node.uv_primitives.tri_indices = tri_indices;
    pixel_node.uv_primitives.pixel_to_position = pixel_to_position;
    pixel_node.uv_primitives.original_tris = original_tris;
}

/// `apply_watertight_check` — fill every encoded pixel to white in
/// the float+byte tile buffers (debug `USE_WATERTIGHT_CHECK` path,
/// ported for completeness).
pub fn apply_watertight_check(
    pixel_nodes: &mut [PixelNode],
    tile_number: i32,
    image_size: [i32; 2],
    data_float: Option<&mut [f32]>,
    data_byte: Option<&mut [u8]>,
) {
    let mut data_float = data_float;
    let mut data_byte = data_byte;
    for pixel_node in pixel_nodes {
        let Some(tile_node_data) = pixel_node.find_tile_data(tile_number) else {
            continue;
        };
        let num_runs = tile_node_data.pixel_row_run_starts.len().saturating_sub(1);
        for run_i in 0..num_runs {
            let mut run_x = tile_node_data.pixel_row_run_start_coords[run_i][0] as i32;
            let run_y = tile_node_data.pixel_row_run_start_coords[run_i][1] as i32;
            let run_begin = tile_node_data.pixel_row_run_starts[run_i] as usize;
            let run_end = tile_node_data.pixel_row_run_starts[run_i + 1] as usize;
            for row in &tile_node_data.pixel_rows[run_begin..run_end] {
                let mut pixel_offset = (run_y * image_size[0] + run_x) as usize;
                for _ in 0..row.num_pixels {
                    if let Some(f) = data_float.as_deref_mut() {
                        f[pixel_offset * 4..pixel_offset * 4 + 4].fill(1.0);
                    }
                    if let Some(b) = data_byte.as_deref_mut() {
                        b[pixel_offset * 4..pixel_offset * 4 + 4].fill(255);
                    }
                    pixel_offset += 1;
                }
                run_x += row.num_pixels as i32;
            }
        }
    }
}
