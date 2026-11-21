//! Port of `intern/subdiv_converter_mesh.cc` — mesh → OpenSubdiv
//! converter storage + manifold (non-loose) index remapping.
//!
//! `OpenSubdiv_Converter` C struct → `ConverterOutput`; Mesh
//! accessors (`loose_verts`/`loose_edges`/`uv_map_names`/
//! `vert_to_corner_map`) and `calc_uv_verts` are injected.

use blenders_bli::index_mask::IndexMask;
use blenders_bli::index_range::IndexRange;
use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::vector::Vec2;

use crate::multires_reshape::OPENSUBDIV_SHARPNESS_INFINITE;
use crate::subdiv::Settings;

/// `OpenSubdiv_SchemeType`.
pub const OSD_SCHEME_BILINEAR: i32 = 0;
pub const OSD_SCHEME_CATMARK: i32 = 1;

/// `crease_to_sharpness` (`BKE_subdiv.hh` inline) — crease ∈ [0,1]
/// scaled to sharpness range.
pub fn crease_to_sharpness(crease: f32) -> f32 {
    crease * OPENSUBDIV_SHARPNESS_INFINITE
}

/// `OpenSubdiv_Converter::UVLayer`.
#[derive(Clone)]
pub struct UvLayer {
    pub verts_num: i32,
    /// `const int *face_vary_indices` — owned here.
    pub face_vary_indices: Vec<i32>,
}

/// `ConverterStorage` — all the manifold-space arrays the converter
/// exposes by pointer upstream; owned `Vec`s here.
#[derive(Default)]
pub struct ConverterStorage {
    pub settings: Settings,
    /// `Span<int2>` manifold edges.
    pub edges: Vec<[i32; 2]>,
    /// `OffsetIndices<int> faces` — offset array.
    pub face_offsets: Vec<i32>,
    /// `Span<int>` manifold corner→vert indices.
    pub corner_verts: Vec<i32>,
    /// Sharpness of every manifold edge/vertex, empty when none sharp.
    pub edge_sharpness: Vec<f32>,
    pub vert_sharpness: Vec<f32>,
    /// Face-varying topology for each UV map.
    pub uv_layers: Vec<UvLayer>,
    /// coarse→manifold vertex map (empty = identity).
    pub manifold_verts: Vec<i32>,
    /// manifold→coarse maps.
    pub manifold_vert_to_orig: Vec<i32>,
    pub manifold_edge_to_orig: Vec<i32>,
    /// Number of non-loose elements.
    pub num_manifold_vertices: i32,
    pub num_manifold_edges: i32,
}

/// `get_scheme_type` — `BUGGY_SIMPLE_SCHEME_WORKAROUND` is on:
/// always Catmark.
pub fn get_scheme_type(_settings: &Settings) -> i32 {
    OSD_SCHEME_CATMARK
}

/// `original_to_manifold` — empty map is the identity.
pub fn original_to_manifold(forward_map: &[i32], original_index: usize) -> i32 {
    if forward_map.is_empty() {
        original_index as i32
    } else {
        forward_map[original_index]
    }
}

/// `manifold_to_original` — empty map is the identity.
pub fn manifold_to_original(reverse_map: &[i32], manifold_index: usize) -> i32 {
    if reverse_map.is_empty() {
        manifold_index as i32
    } else {
        reverse_map[manifold_index]
    }
}

/// `unused_is_trailing_range` — loose elements form a contiguous run
/// at the end → no remapping needed.
fn unused_is_trailing_range(unused_mask: &IndexMask, num_elements: usize) -> bool {
    if unused_mask.is_empty() {
        return true;
    }
    match unused_mask.to_range() {
        Some(range) => range.end == num_elements as i64,
        None => false,
    }
}

/// `initialize_manifold_index_map` — returns manifold element count
/// and optional forward/reverse maps.
fn initialize_manifold_index_map(
    unused_mask: &IndexMask,
    num_elements: usize,
    r_index_forward: Option<&mut Vec<i32>>,
    r_index_reverse: Option<&mut Vec<i32>>,
) -> i32 {
    let num_manifold_elements = num_elements as i64 - unused_mask.size();
    if unused_is_trailing_range(unused_mask, num_elements) {
        return num_manifold_elements as i32;
    }
    let universe = IndexMask::from_range(0, num_elements as i64);
    let used_mask = unused_mask.complement(&universe);
    if let Some(fwd) = r_index_forward {
        fwd.clear();
        fwd.resize(num_elements, 0);
        let mut map64 = vec![0i64; num_elements];
        used_mask.build_reverse_map(&mut map64);
        for (i, v) in map64.iter().enumerate() {
            fwd[i] = *v as i32;
        }
    }
    if let Some(rev) = r_index_reverse {
        rev.clear();
        rev.extend(used_mask.to_indices().iter().map(|&v| v as i32));
    }
    num_manifold_elements as i32
}

/// `initialize_manifold_edges`.
fn initialize_manifold_edges(storage: &mut ConverterStorage, mesh_edges: &[[i32; 2]]) {
    if storage.manifold_edge_to_orig.is_empty() && storage.manifold_verts.is_empty() {
        storage.edges = mesh_edges[..storage.num_manifold_edges as usize].to_vec();
        return;
    }
    let mut edges_manifold = vec![[0i32; 2]; storage.num_manifold_edges as usize];
    for (manifold_edge_index, e) in edges_manifold.iter_mut().enumerate() {
        let edge_index = manifold_to_original(&storage.manifold_edge_to_orig, manifold_edge_index);
        let edge = mesh_edges[edge_index as usize];
        *e = [
            original_to_manifold(&storage.manifold_verts, edge[0] as usize),
            original_to_manifold(&storage.manifold_verts, edge[1] as usize),
        ];
    }
    storage.edges = edges_manifold;
}

/// `initialize_manifold_indices` — build both maps from the loose
/// element masks.
pub fn initialize_manifold_indices(
    storage: &mut ConverterStorage,
    loose_verts: &IndexMask,
    loose_edges: &IndexMask,
    verts_num: usize,
    edges_num: usize,
    mesh_edges: &[[i32; 2]],
    mesh_corner_verts: &[i32],
) {
    /* verts */
    {
        let mut fwd = Vec::new();
        let mut rev = Vec::new();
        storage.num_manifold_vertices = initialize_manifold_index_map(
            loose_verts, verts_num, Some(&mut fwd), Some(&mut rev));
        storage.manifold_verts = fwd;
        storage.manifold_vert_to_orig = rev;
    }
    if storage.manifold_verts.is_empty() {
        storage.corner_verts = mesh_corner_verts.to_vec();
    } else {
        /* `array_utils::gather(manifold_verts, mesh_corner_verts, …)` —
         * corner_verts stores ORIGINAL→ wait: `gather(map, src, dst)`
         * = dst[i] = src[map[i]] — maps corner→manifold vertex. */
        storage.corner_verts = mesh_corner_verts
            .iter()
            .map(|&v| storage.manifold_verts[v as usize])
            .collect();
    }

    /* edges */
    {
        let mut rev = Vec::new();
        storage.num_manifold_edges = initialize_manifold_index_map(
            loose_edges, edges_num, None, Some(&mut rev));
        storage.manifold_edge_to_orig = rev;
    }
    initialize_manifold_edges(storage, mesh_edges);
}

/// `gather_manifold_sharpness` — crease → sharpness in manifold order.
fn gather_manifold_sharpness(
    creases: &[f32],
    manifold_to_orig: &[i32],
    sharpness: &mut [f32],
) {
    if manifold_to_orig.is_empty() {
        sharpness.copy_from_slice(&creases[..sharpness.len()]);
    } else {
        for (i, s) in sharpness.iter_mut().enumerate() {
            *s = creases[manifold_to_orig[i] as usize];
        }
    }
    for s in sharpness.iter_mut() {
        *s = crease_to_sharpness(*s);
    }
}

/// `initialize_edge_sharpness`.
fn initialize_edge_sharpness(storage: &mut ConverterStorage, cd_edge_crease: &[f32]) {
    /* `BUGGY_SIMPLE_SCHEME_WORKAROUND`: emulate bilinear by making
     * every edge infinitely sharp. */
    if storage.settings.is_simple {
        storage.edge_sharpness = vec![
            OPENSUBDIV_SHARPNESS_INFINITE;
            storage.num_manifold_edges as usize
        ];
        return;
    }
    if cd_edge_crease.is_empty() {
        return;
    }
    storage.edge_sharpness = vec![0.0; storage.num_manifold_edges as usize];
    let rev = storage.manifold_edge_to_orig.clone();
    gather_manifold_sharpness(cd_edge_crease, &rev, &mut storage.edge_sharpness);
}

/// `initialize_vert_sharpness` — crease plus infinite sharpness on
/// loose-edge-adjacent verts.
fn initialize_vert_sharpness(
    storage: &mut ConverterStorage,
    loose_edges: &IndexMask,
    verts_num: usize,
    mesh_edges: &[[i32; 2]],
    cd_vertex_crease: &[f32],
) {
    if storage.settings.is_simple {
        storage.vert_sharpness = vec![
            OPENSUBDIV_SHARPNESS_INFINITE;
            storage.num_manifold_vertices as usize
        ];
        return;
    }
    if cd_vertex_crease.is_empty() && loose_edges.is_empty() {
        return;
    }

    let mut sharpness = vec![0.0f32; storage.num_manifold_vertices as usize];
    if !cd_vertex_crease.is_empty() {
        gather_manifold_sharpness(cd_vertex_crease, &storage.manifold_vert_to_orig, &mut sharpness);
    }

    if loose_edges.is_empty() {
        storage.vert_sharpness = sharpness;
        return;
    }

    /* Vertices adjacent to a loose edge are infinitely sharp. */
    let mut infinite_sharp_verts = vec![false; verts_num];
    for edge_index in loose_edges.iter() {
        let edge = mesh_edges[edge_index as usize];
        infinite_sharp_verts[edge[0] as usize] = true;
        infinite_sharp_verts[edge[1] as usize] = true;
    }
    for manifold_vertex_index in 0..sharpness.len() {
        let vertex_index =
            manifold_to_original(&storage.manifold_vert_to_orig, manifold_vertex_index);
        if infinite_sharp_verts[vertex_index as usize] {
            sharpness[manifold_vertex_index] = OPENSUBDIV_SHARPNESS_INFINITE;
        }
    }
    storage.vert_sharpness = sharpness;
}

/// `initialize_uv_layers` — per-UV-map face-varying corner indices.
///
/// `calc_uv_verts(faces, corner_verts, vert_to_corner_groups,
/// uv_map)` → (uvs_num, corner→uv indices) — injected
/// (`bke::mesh::calc_uv_verts`, `PORT_PENDING`).
pub fn initialize_uv_layers(
    storage: &mut ConverterStorage,
    uv_map_names: &[String],
    corners_num: usize,
    calc_uv_verts: &dyn Fn(&str) -> (i32, Vec<i32>),
) {
    storage.uv_layers.clear();
    if uv_map_names.is_empty() {
        return;
    }
    /* Group corners with the original vertex indices rather than the
     * manifold indices to use the cached topology map, which only
     * affects the order of the UV vertices. */
    let _ = corners_num;
    for name in uv_map_names {
        let (uvs_num, corner_uv_indices) = calc_uv_verts(name);
        storage.uv_layers.push(UvLayer {
            verts_num: uvs_num,
            face_vary_indices: corner_uv_indices,
        });
    }
}

/// `OpenSubdiv_Converter` — the public output of
/// `converter_init_for_mesh`.
pub struct ConverterOutput {
    pub storage: ConverterStorage,
    pub verts_num: i32,
    pub scheme_type: i32,
    pub vtx_boundary_interpolation: i32,
    pub fvar_linear_interpolation: i32,
}

/// `converter_init_for_mesh` — full manifold/sharpness/UV init.
#[allow(clippy::too_many_arguments)]
pub fn converter_init_for_mesh(
    settings: &Settings,
    loose_verts: &IndexMask,
    loose_edges: &IndexMask,
    verts_num: usize,
    edges_num: usize,
    mesh_edges: &[[i32; 2]],
    mesh_corner_verts: &[i32],
    face_offsets: Vec<i32>,
    cd_vertex_crease: &[f32],
    cd_edge_crease: &[f32],
    uv_map_names: &[String],
    corners_num: usize,
    calc_uv_verts: &dyn Fn(&str) -> (i32, Vec<i32>),
    vtx_boundary_fn: &dyn Fn(&Settings) -> i32,
    fvar_linear_fn: &dyn Fn(&Settings) -> i32,
) -> ConverterOutput {
    let mut storage = ConverterStorage {
        settings: settings.clone(),
        face_offsets,
        ..Default::default()
    };
    initialize_manifold_indices(
        &mut storage,
        loose_verts,
        loose_edges,
        verts_num,
        edges_num,
        mesh_edges,
        mesh_corner_verts,
    );
    if settings.use_creases {
        initialize_edge_sharpness(&mut storage, cd_edge_crease);
        initialize_vert_sharpness(
            &mut storage,
            loose_edges,
            verts_num,
            mesh_edges,
            cd_vertex_crease,
        );
    }
    initialize_uv_layers(&mut storage, uv_map_names, corners_num, calc_uv_verts);

    let out = ConverterOutput {
        verts_num: storage.num_manifold_vertices,
        scheme_type: get_scheme_type(settings),
        vtx_boundary_interpolation: vtx_boundary_fn(settings),
        fvar_linear_interpolation: fvar_linear_fn(settings),
        storage,
    };
    out
}

/// `GroupedSpan<int>` → Vec2 helper for uv fvar consumers.
pub fn uv_layer_span(l: &UvLayer) -> &[i32] {
    &l.face_vary_indices
}
