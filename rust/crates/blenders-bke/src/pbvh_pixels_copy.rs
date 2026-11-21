//! Port of `intern/pbvh_pixels_copy.cc` — seam-pixel copy for PBVH
//! texture painting: non-manifold UV edge extraction, `Rows`
//! pixel classification + source search, `CopyPixelCommand` delta
//! encoding, `CopyPixelTile`/`CopyPixelTiles` and `copy_pixels`
//! into float/byte image buffers.
//!
//! `uv_islands::MeshData`/`ImBuf`/PBVH access injected.

use std::collections::HashMap;

use blenders_math::{geom, Vec2};
use crate::pbvh_pixels::{PixelNode, UDIMTilePixels};

/// `SEAM_TILE_BITS`.
pub const SEAM_TILE_BITS: i32 = 6;
/// `SEAM_TILE_SIZE`.
pub const SEAM_TILE_SIZE: i32 = 1 << SEAM_TILE_BITS;

/// `rcti`.
#[derive(Clone, Copy)]
pub struct Rcti {
    /// `xmin`.
    pub xmin: i32,
    /// `xmax`.
    pub xmax: i32,
    /// `ymin`.
    pub ymin: i32,
    /// `ymax`.
    pub ymax: i32,
}

impl Rcti {
    /// `BLI_rcti_init`.
    pub fn new(xmin: i32, xmax: i32, ymin: i32, ymax: i32) -> Self {
        Self { xmin, xmax, ymin, ymax }
    }
    /// `BLI_rcti_init_minmax`.
    pub fn minmax() -> Self {
        Self::new(i32::MAX, i32::MIN, i32::MAX, i32::MIN)
    }
    /// `BLI_rcti_do_minmax_v`.
    pub fn do_minmax_v(&mut self, v: [i32; 2]) {
        self.xmin = self.xmin.min(v[0]);
        self.xmax = self.xmax.max(v[0]);
        self.ymin = self.ymin.min(v[1]);
        self.ymax = self.ymax.max(v[1]);
    }
    /// `BLI_rcti_size_x` + 1 (inclusive extent).
    pub fn size_x(&self) -> i32 {
        self.xmax - self.xmin
    }
    /// `BLI_rcti_size_y`.
    pub fn size_y(&self) -> i32 {
        self.ymax - self.ymin
    }
    /// `BLI_rcti_isect`.
    pub fn isect(&mut self, other: &Rcti) {
        self.xmin = self.xmin.max(other.xmin);
        self.xmax = self.xmax.min(other.xmax);
        self.ymin = self.ymin.max(other.ymin);
        self.ymax = self.ymax.min(other.ymax);
    }
}

/// `get_bounds`.
pub fn get_bounds(tile_edge: &TileEdge) -> Rcti {
    let mut bounds = Rcti::minmax();
    bounds.do_minmax_v([
        tile_edge.vertex_1.coordinate[0] as i32,
        tile_edge.vertex_1.coordinate[1] as i32,
    ]);
    bounds.do_minmax_v([
        tile_edge.vertex_2.coordinate[0] as i32,
        tile_edge.vertex_2.coordinate[1] as i32,
    ]);
    bounds
}

/// `add_margin`.
pub fn add_margin(bounds: &mut Rcti, margin: i32) {
    bounds.xmin -= margin;
    bounds.xmax += margin;
    bounds.ymin -= margin;
    bounds.ymax += margin;
}

/// `clamp` to `resolution` pixels.
pub fn clamp(bounds: &mut Rcti, resolution: [i32; 2]) {
    let clamping = Rcti::new(0, resolution[0] - 1, 0, resolution[1] - 1);
    bounds.isect(&clamping);
}

/// `Vertex<CoordSpace::UV>`.
#[derive(Clone, Copy)]
pub struct UVVertex {
    /// `coordinate`.
    pub coordinate: Vec2,
}
/// `Vertex<CoordSpace::Tile>`.
#[derive(Clone, Copy)]
pub struct TileVertex {
    /// `coordinate` — tile pixel coords (float).
    pub coordinate: Vec2,
}
/// `Edge<CoordSpace::UV>`.
#[derive(Clone, Copy)]
pub struct UVEdge {
    /// `vertex_1`.
    pub vertex_1: UVVertex,
    /// `vertex_2`.
    pub vertex_2: UVVertex,
}
/// `Edge<CoordSpace::Tile>`.
#[derive(Clone, Copy)]
pub struct TileEdge {
    /// `vertex_1`.
    pub vertex_1: TileVertex,
    /// `vertex_2`.
    pub vertex_2: TileVertex,
}

/// `convert_coord_space` UV → tile vertex.
pub fn convert_coord_space_vertex(
    uv_vertex: &UVVertex,
    tile_offset: [i32; 2],
    tile_resolution: [i32; 2],
) -> TileVertex {
    TileVertex {
        coordinate: Vec2::new(
            (uv_vertex.coordinate.x - tile_offset[0] as f32) * tile_resolution[0] as f32,
            (uv_vertex.coordinate.y - tile_offset[1] as f32) * tile_resolution[1] as f32,
        ),
    }
}

/// `convert_coord_space` UV → tile edge.
pub fn convert_coord_space_edge(
    uv_edge: &UVEdge,
    tile_offset: [i32; 2],
    tile_resolution: [i32; 2],
) -> TileEdge {
    TileEdge {
        vertex_1: convert_coord_space_vertex(&uv_edge.vertex_1, tile_offset, tile_resolution),
        vertex_2: convert_coord_space_vertex(&uv_edge.vertex_2, tile_offset, tile_resolution),
    }
}

/// `NonManifoldUVEdges` — all triangle edges on non-manifold mesh
/// edges, in UV space.
pub struct NonManifoldUVEdges {
    /// `edges`.
    pub edges: Vec<UVEdge>,
}

/// `uv_islands::MeshData` subset needed here.
pub struct CopyMeshData<'a> {
    /// `corner_tris`.
    pub corner_tris: &'a [[i32; 3]],
    /// `uv_map` (corner-domain).
    pub uv_map: &'a [Vec2],
    /// `corner_tri_get_real_edges(tri)` — mesh edge ids per tri.
    pub real_edges: &'a dyn Fn([i32; 3]) -> [i32; 3],
    /// `is_edge_manifold(edge_id)`.
    pub is_edge_manifold: &'a dyn Fn(i32) -> bool,
}

impl NonManifoldUVEdges {
    /// Construct from `uv_islands::MeshData`.
    pub fn new(mesh_data: &CopyMeshData) -> Self {
        let mut edges = Vec::new();
        for tri in mesh_data.corner_tris {
            let real_edges = (mesh_data.real_edges)(*tri);
            for j in 0..3 {
                let edge_id = real_edges[j];
                if edge_id == -1 || (mesh_data.is_edge_manifold)(edge_id) {
                    continue;
                }
                edges.push(UVEdge {
                    vertex_1: UVVertex {
                        coordinate: mesh_data.uv_map[tri[j] as usize],
                    },
                    vertex_2: UVVertex {
                        coordinate: mesh_data.uv_map[tri[(j + 1) % 3] as usize],
                    },
                });
            }
        }
        Self { edges }
    }

    /// `extract_tile_edges`.
    pub fn extract_tile_edges(
        &self,
        tile_offset: [i32; 2],
        tile_resolution: [i32; 2],
    ) -> Vec<TileEdge> {
        self.edges
            .iter()
            .map(|e| convert_coord_space_edge(e, tile_offset, tile_resolution))
            .collect()
    }
}

/// `PixelNodesTileData` — `UDIMTilePixels` for one tile from all
/// affected nodes.
pub fn pixel_nodes_tile_data<'a>(
    pixel_nodes: &'a mut [PixelNode],
    tile_number: i32,
) -> Vec<&'a mut UDIMTilePixels> {
    pixel_nodes
        .iter_mut()
        .filter_map(|n| n.find_tile_data(tile_number))
        .collect()
}

/// `CopyPixelCommand`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CopyPixelCommand {
    /// `destination`.
    pub destination: [i32; 2],
    /// `source_1`.
    pub source_1: [i32; 2],
    /// `source_2`.
    pub source_2: [i32; 2],
    /// `mix_factor`.
    pub mix_factor: f32,
}

/// `DeltaCopyPixelCommand`.
#[derive(Debug, Clone, Copy)]
pub struct DeltaCopyPixelCommand {
    /// `delta_source_1`.
    pub delta_source_1: [i8; 2],
    /// `delta_source_2`.
    pub delta_source_2: [i8; 2],
    /// `mix_factor`.
    pub mix_factor: u8,
}

/// `encode_delta`.
pub fn encode_delta(previous: &CopyPixelCommand, next: &CopyPixelCommand) -> DeltaCopyPixelCommand {
    DeltaCopyPixelCommand {
        delta_source_1: [
            (next.source_1[0] - previous.source_1[0]) as i8,
            (next.source_1[1] - previous.source_1[1]) as i8,
        ],
        delta_source_2: [
            (next.source_2[0] - next.source_1[0]) as i8,
            (next.source_2[1] - next.source_1[1]) as i8,
        ],
        mix_factor: (next.mix_factor * 255.0) as u8,
    }
}

/// `can_be_extended`.
pub fn can_be_extended(previous: &CopyPixelCommand, next: &CopyPixelCommand) -> bool {
    if previous.destination[0] != next.destination[0] - 1
        || previous.destination[1] != next.destination[1]
    {
        return false;
    }
    let ds = [
        (previous.source_1[0] - next.source_1[0]).abs(),
        (previous.source_1[1] - next.source_1[1]).abs(),
    ];
    ds[0].max(ds[1]) <= 127
}

/// `CopyPixelGroup`.
#[derive(Debug, Clone, Copy)]
pub struct CopyPixelGroup {
    /// `start_destination`.
    pub start_destination: [i32; 2],
    /// `start_source_1`.
    pub start_source_1: [i32; 2],
    /// `start_delta_index`.
    pub start_delta_index: i64,
    /// `num_deltas`.
    pub num_deltas: i32,
}

/// `CopyPixelTile`.
pub struct CopyPixelTile {
    /// `tile_number`.
    pub tile_number: i32,
    /// `groups`.
    pub groups: Vec<CopyPixelGroup>,
    /// `command_deltas`.
    pub command_deltas: Vec<DeltaCopyPixelCommand>,
    /// `seam_tile_to_groups` — seam tile index → group index range.
    pub seam_tile_to_groups: HashMap<i32, (i64, i64)>,
}

/// `seam_tile_index`.
pub fn seam_tile_index(source: [i32; 2], seam_tiles_x: i32) -> i32 {
    (source[0] >> SEAM_TILE_BITS) + (source[1] >> SEAM_TILE_BITS) * seam_tiles_x
}

/// `CopyPixelTiles`.
#[derive(Default)]
pub struct CopyPixelTiles {
    /// `tiles`.
    pub tiles: Vec<CopyPixelTile>,
}

impl CopyPixelTiles {
    /// `find_tile`.
    pub fn find_tile(&mut self, tile_number: i32) -> Option<&mut CopyPixelTile> {
        self.tiles.iter_mut().find(|t| t.tile_number == tile_number)
    }
    /// `clear`.
    pub fn clear(&mut self) {
        self.tiles.clear();
    }
}

/// `Rows::PixelType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelType {
    /// `Undecided`.
    Undecided,
    /// `Brush`.
    Brush,
    /// `SelectedForCloserExamination`.
    SelectedForCloserExamination,
    /// `CopyFromClosestEdge`.
    CopyFromClosestEdge,
}

/// `Rows::Pixel`.
#[derive(Clone, Copy)]
pub struct RowPixel {
    /// `distance`.
    pub distance: f32,
    /// `edge_index`.
    pub edge_index: i32,
    /// `type`.
    pub pixel_type: PixelType,
}

impl Default for RowPixel {
    fn default() -> Self {
        Self {
            distance: f32::MAX,
            edge_index: -1,
            pixel_type: PixelType::Undecided,
        }
    }
}

/// `Rows`.
pub struct Rows {
    /// `resolution`.
    pub resolution: [i32; 2],
    /// `margin`.
    pub margin: i32,
    /// `pixels`.
    pub pixels: Vec<RowPixel>,
}

impl Rows {
    /// `Rows(resolution, margin)`.
    pub fn new(resolution: [i32; 2], margin: i32) -> Self {
        Self {
            resolution,
            margin,
            pixels: vec![RowPixel::default(); (resolution[0] * resolution[1]).max(0) as usize],
        }
    }

    /// `coordinate_of`.
    pub fn coordinate_of(&self, index: i64) -> [i32; 2] {
        [
            (index % self.resolution[0] as i64) as i32,
            (index / self.resolution[0] as i64) as i32,
        ]
    }

    /// `mark_pixels_effected_by_brush`.
    pub fn mark_pixels_effected_by_brush(&mut self, nodes_tile_pixels: &[&mut UDIMTilePixels]) {
        for tile_pixels in nodes_tile_pixels.iter() {
            let num_runs = tile_pixels.pixel_row_run_starts.len().saturating_sub(1);
            for run_i in 0..num_runs {
                let mut x = tile_pixels.pixel_row_run_start_coords[run_i][0] as i32;
                let y = tile_pixels.pixel_row_run_start_coords[run_i][1] as i32;
                let run_begin = tile_pixels.pixel_row_run_starts[run_i] as usize;
                let run_end = tile_pixels.pixel_row_run_starts[run_i + 1] as usize;
                for row in &tile_pixels.pixel_rows[run_begin..run_end] {
                    for _ in 0..row.num_pixels {
                        let index = (y * self.resolution[0] + x) as usize;
                        if index < self.pixels.len() {
                            self.pixels[index].pixel_type = PixelType::Brush;
                            self.pixels[index].distance = 0.0;
                        }
                        x += 1;
                    }
                }
            }
        }
    }

    /// `find_second_source`.
    pub fn find_second_source(&self, destination: [i32; 2], first_source: [i32; 2]) -> [i32; 2] {
        let bounds = Rcti::new(
            (first_source[0] - 1).max(0),
            (first_source[0] + 1).min(self.resolution[0] - 1),
            (first_source[1] - 1).max(0),
            (first_source[1] + 1).min(self.resolution[1] - 1),
        );
        let mut found_source = first_source;
        let mut found_distance = f32::MAX;
        let dest = Vec2::new(destination[0] as f32, destination[1] as f32);
        for sy in bounds.ymin..=bounds.ymax {
            for sx in bounds.xmin..=bounds.xmax {
                if [sx, sy] == first_source {
                    continue;
                }
                let index = (sy * self.resolution[0] + sx) as usize;
                if self.pixels[index].pixel_type != PixelType::Brush {
                    continue;
                }
                let new_distance = Vec2::new(sx as f32, sy as f32).distance(dest);
                if new_distance < found_distance {
                    found_distance = new_distance;
                    found_source = [sx, sy];
                }
            }
        }
        found_source
    }

    /// `determine_mix_factor`.
    pub fn determine_mix_factor(
        &self,
        destination: [i32; 2],
        source_1: [i32; 2],
        source_2: [i32; 2],
        edge: &TileEdge,
    ) -> f32 {
        if source_1 == source_2 {
            return 0.0;
        }
        let (v1, v2) = (edge.vertex_1.coordinate, edge.vertex_2.coordinate);
        let d = |p: [i32; 2]| Vec2::new(p[0] as f32, p[1] as f32);
        let (_, destination_lambda) = geom::closest_to_line_v2(d(destination), v1, v2);
        let (_, source_1_lambda) = geom::closest_to_line_v2(d(source_1), v1, v2);
        let (_, source_2_lambda) = geom::closest_to_line_v2(d(source_2), v1, v2);
        ((destination_lambda - source_1_lambda) / (source_2_lambda - source_1_lambda))
            .clamp(0.0, 1.0)
    }

    /// `find_copy_source` for one pixel index.
    pub fn find_copy_source(
        &mut self,
        index: usize,
        r_command: &mut CopyPixelCommand,
        tile_edges: &[TileEdge],
    ) {
        debug_assert_eq!(
            self.pixels[index].pixel_type,
            PixelType::SelectedForCloserExamination
        );
        let destination = self.coordinate_of(index as i64);

        let mut bounds = Rcti::new(
            destination[0],
            destination[0],
            destination[1],
            destination[1],
        );
        add_margin(&mut bounds, self.margin);
        clamp(&mut bounds, self.resolution);

        let mut found_distance = f32::MAX;
        let mut found_source = [0, 0];
        let dest = Vec2::new(destination[0] as f32, destination[1] as f32);

        for sy in bounds.ymin..bounds.ymax {
            let row = (sy * self.resolution[0]) as usize;
            for sx in bounds.xmin..bounds.xmax {
                let source = self.pixels[row + sx as usize];
                if source.pixel_type != PixelType::Brush {
                    continue;
                }
                let new_distance = Vec2::new(sx as f32, sy as f32).distance(dest);
                if new_distance < found_distance {
                    found_source = [sx, sy];
                    found_distance = new_distance;
                }
            }
        }

        if found_distance == f32::MAX {
            return;
        }
        self.pixels[index].pixel_type = PixelType::CopyFromClosestEdge;
        self.pixels[index].distance = found_distance;
        r_command.destination = destination;
        r_command.source_1 = found_source;
        r_command.source_2 = self.find_second_source(destination, found_source);
        r_command.mix_factor = self.determine_mix_factor(
            destination,
            found_source,
            r_command.source_2,
            &tile_edges[self.pixels[index].edge_index as usize],
        );
    }

    /// `find_copy_source` over all selected pixels.
    pub fn find_copy_sources(
        &mut self,
        selected_pixels: &[i64],
        r_commands: &mut [CopyPixelCommand],
        tile_edges: &[TileEdge],
    ) {
        for (i, &p) in selected_pixels.iter().enumerate() {
            self.find_copy_source(p as usize, &mut r_commands[i], tile_edges);
        }
    }

    /// `filter_pixels_for_closer_examination`.
    pub fn filter_pixels_for_closer_examination(&mut self, tile_edges: &[TileEdge]) -> Vec<i64> {
        let mut selected_pixels = Vec::with_capacity(10000);
        for (tile_edge_index, tile_edge) in tile_edges.iter().enumerate() {
            let mut edge_bounds = get_bounds(tile_edge);
            add_margin(&mut edge_bounds, self.margin);
            clamp(&mut edge_bounds, self.resolution);

            for sy in edge_bounds.ymin..edge_bounds.ymax {
                for sx in edge_bounds.xmin..edge_bounds.xmax {
                    let index = (sy * self.resolution[0] + sx) as usize;
                    if index >= self.pixels.len() {
                        continue;
                    }
                    if self.pixels[index].pixel_type == PixelType::Brush {
                        continue;
                    }
                    let point = Vec2::new(sx as f32, sy as f32);
                    let (closest, _) = geom::closest_to_line_segment_v2(
                        point,
                        tile_edge.vertex_1.coordinate,
                        tile_edge.vertex_2.coordinate,
                    );
                    let distance_to_edge = closest.distance(point);
                    if distance_to_edge < self.margin as f32
                        && distance_to_edge < self.pixels[index].distance
                    {
                        if self.pixels[index].pixel_type
                            != PixelType::SelectedForCloserExamination
                        {
                            selected_pixels.push(index as i64);
                        }
                        self.pixels[index].pixel_type = PixelType::SelectedForCloserExamination;
                        self.pixels[index].distance = distance_to_edge;
                        self.pixels[index].edge_index = tile_edge_index as i32;
                    }
                }
            }
        }
        selected_pixels
    }

    /// `pack_into`.
    pub fn pack_into(
        &self,
        selected_pixels: &[i64],
        commands: &[CopyPixelCommand],
        copy_tile: &mut CopyPixelTile,
    ) {
        let mut last_group: Option<usize> = None;
        let mut last_command: Option<CopyPixelCommand> = None;
        let seam_tiles_x = (self.resolution[0] + SEAM_TILE_SIZE - 1) >> SEAM_TILE_BITS;
        let mut last_seam_tile = -1i32;

        for (i, &p) in selected_pixels.iter().enumerate() {
            if self.pixels[p as usize].pixel_type != PixelType::CopyFromClosestEdge {
                continue;
            }
            let command = commands[i];
            let seam_tile = seam_tile_index(command.source_1, seam_tiles_x);
            if last_command.is_none()
                || !can_be_extended(&last_command.unwrap(), &command)
                || seam_tile != last_seam_tile
            {
                let new_group = CopyPixelGroup {
                    start_destination: [command.destination[0] - 1, command.destination[1]],
                    start_source_1: command.source_1,
                    start_delta_index: copy_tile.command_deltas.len() as i64,
                    num_deltas: 0,
                };
                last_seam_tile = seam_tile;
                copy_tile.groups.push(new_group);
                last_group = Some(copy_tile.groups.len() - 1);
                last_command = Some(CopyPixelCommand {
                    destination: new_group.start_destination,
                    source_1: new_group.start_source_1,
                    source_2: [0, 0],
                    mix_factor: 0.0,
                });
            }

            let delta = encode_delta(&last_command.unwrap(), &command);
            copy_tile.command_deltas.push(delta);
            copy_tile.groups[last_group.unwrap()].num_deltas += 1;
            last_command = Some(command);
        }
    }
}

impl CopyPixelTile {
    /// `CopyPixelTile(tile_number)`.
    pub fn new(tile_number: i32) -> Self {
        Self {
            tile_number,
            groups: Vec::new(),
            command_deltas: Vec::new(),
            seam_tile_to_groups: HashMap::new(),
        }
    }

    /// `build_seam_tile_map`.
    pub fn build_seam_tile_map(&mut self, resolution: [i32; 2]) {
        let tiles_x = (resolution[0] + SEAM_TILE_SIZE - 1) >> SEAM_TILE_BITS;
        self.groups
            .sort_by_key(|g| seam_tile_index(g.start_source_1, tiles_x));

        self.seam_tile_to_groups.clear();
        let mut start = 0i64;
        while start < self.groups.len() as i64 {
            let tile = seam_tile_index(self.groups[start as usize].start_source_1, tiles_x);
            let mut end = start + 1;
            while end < self.groups.len() as i64
                && seam_tile_index(self.groups[end as usize].start_source_1, tiles_x) == tile
            {
                end += 1;
            }
            self.seam_tile_to_groups.insert(tile, (start, end));
            start = end;
        }
    }
}

/// `pixel_offset`.
fn pixel_offset(width: i32, coordinate: [i32; 2]) -> usize {
    ((coordinate[1] * width + coordinate[0]) * 4).max(0) as usize
}

fn rgba_uchar_to_float(b: &[u8]) -> [f32; 4] {
    [
        b[0] as f32 / 255.0,
        b[1] as f32 / 255.0,
        b[2] as f32 / 255.0,
        b[3] as f32 / 255.0,
    ]
}

fn rgba_float_to_uchar(v: &[f32; 4]) -> [u8; 4] {
    [
        (v[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (v[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (v[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (v[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

fn read_pixel_f(buffer: &[f32], width: i32, coordinate: [i32; 2]) -> [f32; 4] {
    let off = pixel_offset(width, coordinate);
    [buffer[off], buffer[off + 1], buffer[off + 2], buffer[off + 3]]
}

fn write_pixel_f(buffer: &mut [f32], width: i32, coordinate: [i32; 2], value: &[f32; 4]) {
    let off = pixel_offset(width, coordinate);
    buffer[off..off + 4].copy_from_slice(value);
}

fn read_pixel_b(buffer: &[u8], width: i32, coordinate: [i32; 2]) -> [f32; 4] {
    let off = pixel_offset(width, coordinate);
    rgba_uchar_to_float(&buffer[off..off + 4])
}

fn write_pixel_b(buffer: &mut [u8], width: i32, coordinate: [i32; 2], value: &[f32; 4]) {
    let off = pixel_offset(width, coordinate);
    buffer[off..off + 4].copy_from_slice(&rgba_float_to_uchar(value));
}

/// `pixels::copy_pixels` (float buffer variant).
pub fn copy_pixels_f32(
    buffer: &mut [f32],
    width: i32,
    groups: &[CopyPixelGroup],
    command_deltas: &[DeltaCopyPixelCommand],
) {
    for group in groups {
        let mut command = CopyPixelCommand {
            destination: group.start_destination,
            source_1: group.start_source_1,
            source_2: group.start_source_1,
            mix_factor: 0.0,
        };
        for delta in &command_deltas[group.start_delta_index as usize
            ..group.start_delta_index as usize + group.num_deltas as usize]
        {
            command.destination[0] += 1;
            command.source_1[0] += delta.delta_source_1[0] as i32;
            command.source_1[1] += delta.delta_source_1[1] as i32;
            command.source_2 = [
                command.source_1[0] + delta.delta_source_2[0] as i32,
                command.source_1[1] + delta.delta_source_2[1] as i32,
            ];
            command.mix_factor = delta.mix_factor as f32 / 255.0;

            let src_color_1 = read_pixel_f(buffer, width, command.source_1);
            let src_color_2 = read_pixel_f(buffer, width, command.source_2);
            let t = command.mix_factor;
            let dst_color = [
                src_color_2[0] * (1.0 - t) + src_color_1[0] * t,
                src_color_2[1] * (1.0 - t) + src_color_1[1] * t,
                src_color_2[2] * (1.0 - t) + src_color_1[2] * t,
                src_color_2[3] * (1.0 - t) + src_color_1[3] * t,
            ];
            write_pixel_f(buffer, width, command.destination, &dst_color);
        }
    }
}

/// `pixels::copy_pixels` (byte buffer variant).
pub fn copy_pixels_u8(
    buffer: &mut [u8],
    width: i32,
    groups: &[CopyPixelGroup],
    command_deltas: &[DeltaCopyPixelCommand],
) {
    for group in groups {
        let mut command = CopyPixelCommand {
            destination: group.start_destination,
            source_1: group.start_source_1,
            source_2: group.start_source_1,
            mix_factor: 0.0,
        };
        for delta in &command_deltas[group.start_delta_index as usize
            ..group.start_delta_index as usize + group.num_deltas as usize]
        {
            command.destination[0] += 1;
            command.source_1[0] += delta.delta_source_1[0] as i32;
            command.source_1[1] += delta.delta_source_1[1] as i32;
            command.source_2 = [
                command.source_1[0] + delta.delta_source_2[0] as i32,
                command.source_1[1] + delta.delta_source_2[1] as i32,
            ];
            command.mix_factor = delta.mix_factor as f32 / 255.0;

            let src_color_1 = read_pixel_b(buffer, width, command.source_1);
            let src_color_2 = read_pixel_b(buffer, width, command.source_2);
            let t = command.mix_factor;
            let dst_color = [
                src_color_2[0] * (1.0 - t) + src_color_1[0] * t,
                src_color_2[1] * (1.0 - t) + src_color_1[1] * t,
                src_color_2[2] * (1.0 - t) + src_color_1[2] * t,
                src_color_2[3] * (1.0 - t) + src_color_1[3] * t,
            ];
            write_pixel_b(buffer, width, command.destination, &dst_color);
        }
    }
}

/// `CopyPixelTile::copy_pixels` — dispatch to the buffer variant.
pub fn copy_pixel_tile_copy_pixels(
    tile: &CopyPixelTile,
    group_range: (usize, usize),
    buffer_f32: Option<&mut [f32]>,
    buffer_u8: Option<&mut [u8]>,
    width: i32,
) {
    let groups = &tile.groups[group_range.0..group_range.1];
    if let Some(buf) = buffer_f32 {
        copy_pixels_f32(buf, width, groups, &tile.command_deltas);
    } else if let Some(buf) = buffer_u8 {
        copy_pixels_u8(buf, width, groups, &tile.command_deltas);
    }
}

/// One image tile's geometry for `copy_update`.
#[derive(Clone, Copy)]
pub struct TileInfo {
    /// `tile_offset` (UDIM grid cell).
    pub tile_offset: [i32; 2],
    /// `tile_resolution` (pixels).
    pub tile_resolution: [i32; 2],
    /// `tile_number`.
    pub tile_number: i32,
}

/// `copy_update` — rebuild all `CopyPixelTile`s from pixel nodes +
/// non-manifold edges; `pixel_nodes` is the `PixelData.nodes` span.
pub fn copy_update(
    mesh_data: &CopyMeshData,
    pixel_nodes: &mut [PixelNode],
    tiles_copy_pixels: &mut CopyPixelTiles,
    tiles: &[TileInfo],
    seam_margin: i32,
) {
    tiles_copy_pixels.clear();
    let non_manifold_edges = NonManifoldUVEdges::new(mesh_data);
    if non_manifold_edges.edges.is_empty() {
        return;
    }

    for tile in tiles {
        let nodes_tile_pixels = pixel_nodes_tile_data(pixel_nodes, tile.tile_number);

        let tile_edges =
            non_manifold_edges.extract_tile_edges(tile.tile_offset, tile.tile_resolution);
        let mut copy_tile = CopyPixelTile::new(tile.tile_number);

        let mut rows = Rows::new(tile.tile_resolution, seam_margin);
        rows.mark_pixels_effected_by_brush(&nodes_tile_pixels);

        let selected_pixels = rows.filter_pixels_for_closer_examination(&tile_edges);
        let mut selected_commands = vec![CopyPixelCommand::default(); selected_pixels.len()];
        rows.find_copy_sources(&selected_pixels, &mut selected_commands, &tile_edges);
        rows.pack_into(&selected_pixels, &selected_commands, &mut copy_tile);
        copy_tile.build_seam_tile_map(tile.tile_resolution);

        tiles_copy_pixels.tiles.push(copy_tile);
    }
}

/// `copy_pixels` — apply seam copies to a tile buffer for all
/// modified seam tiles; `push_undo_tiles` receives
/// `(x_start, x_end, y)` spans.
pub fn copy_pixels(
    tiles_copy_pixels: &mut CopyPixelTiles,
    tile_number: i32,
    buffer_f32: Option<&mut [f32]>,
    buffer_u8: Option<&mut [u8]>,
    width: i32,
    seam_tiles_modified: &[bool],
    push_undo_tiles: &mut dyn FnMut(i32, i32, i32),
) {
    let Some(tile) = tiles_copy_pixels.find_tile(tile_number) else {
        return;
    };

    let mut buffer_f32 = buffer_f32;
    let mut buffer_u8 = buffer_u8;

    /* Collect sorted seam-tile entries. */
    let mut entries: Vec<(i32, (i64, i64))> = tile
        .seam_tile_to_groups
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    entries.sort_by_key(|(k, _)| *k);

    for (seam_tile, (gs, ge)) in entries {
        debug_assert!((seam_tile as usize) < seam_tiles_modified.len());
        if !seam_tiles_modified[seam_tile as usize] {
            continue;
        }
        for group in &tile.groups[gs as usize..ge as usize] {
            push_undo_tiles(
                group.start_destination[0] + 1,
                group.start_destination[0] + group.num_deltas,
                group.start_destination[1],
            );
        }
        copy_pixel_tile_copy_pixels(
            tile,
            (gs as usize, ge as usize),
            buffer_f32.as_deref_mut(),
            buffer_u8.as_deref_mut(),
            width,
        );
    }
}
