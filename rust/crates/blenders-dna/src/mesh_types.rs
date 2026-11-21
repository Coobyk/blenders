//! Port of `DNA_mesh_types.h` — `Mesh` ID + mesh enums.
use crate::customdata_types::CustomData;
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `eMesh_TexSpaceFlag`.
pub const ME_TEXSPACE_FLAG_AUTO: i8 = 1 << 0;
pub const ME_TEXSPACE_FLAG_AUTO_EVALUATED: i8 = 1 << 1;

/// `eMesh_EditFlag`.
pub const ME_EDIT_MIRROR_VERTEX_GROUPS: i8 = 1 << 0;
pub const ME_EDIT_MIRROR_Y: i8 = 1 << 1;
pub const ME_EDIT_MIRROR_Z: i8 = 1 << 2;
pub const ME_EDIT_PAINT_FACE_SEL: i8 = 1 << 3;
pub const ME_EDIT_MIRROR_TOPO: i8 = 1 << 4;
pub const ME_EDIT_PAINT_VERT_SEL: i8 = 1 << 5;

/// `eMesh_Flag`.
pub const ME_FLAG_UV_SELECT_SYNC_VALID: u16 = 1 << 3;
pub const ME_AUTOSMOOTH_LEGACY: u16 = 1 << 5;
pub const ME_REMESH_REPROJECT_ATTRIBUTES: u16 = 1 << 8;
pub const ME_DS_EXPAND: u16 = 1 << 9;
pub const ME_SCULPT_DYNAMIC_TOPOLOGY: u16 = 1 << 10;
pub const ME_NO_OVERLAPPING_TOPOLOGY: u16 = 1 << 11;
pub const ME_REMESH_FIX_POLES: u16 = 1 << 13;
pub const ME_REMESH_REPROJECT_VOLUME: u16 = 1 << 14;

/// `eMesh_CDFlag` (DNA_DEPRECATED_ALLOW).
pub const ME_CDFLAG_VERT_BWEIGHT: i8 = 1 << 0;
pub const ME_CDFLAG_EDGE_BWEIGHT: i8 = 1 << 1;
pub const ME_CDFLAG_EDGE_CREASE: i8 = 1 << 2;
pub const ME_CDFLAG_VERT_CREASE: i8 = 1 << 3;

/// `eMesh_RemeshMode`.
pub const REMESH_VOXEL: i8 = 0;
pub const REMESH_QUAD: i8 = 1;

/// `MeshSubdivType`.
pub const ME_CC_SUBSURF: i32 = 0;
pub const ME_SIMPLE_SUBSURF: i32 = 1;

/// `eMeshSymmetryType`.
pub const ME_SYMMETRY_X: i8 = 1 << 0;
pub const ME_SYMMETRY_Y: i8 = 1 << 1;
pub const ME_SYMMETRY_Z: i8 = 1 << 2;
pub const ME_SYMMETRY_XYZ: i8 = ME_SYMMETRY_X | ME_SYMMETRY_Y | ME_SYMMETRY_Z;
pub const ME_SYMMETRY_RADIAL: i8 = 1 << 3;

/// `ME_USING_MIRROR_X_VERTEX_GROUPS`.
pub fn me_using_mirror_x_vertex_groups(me: &Mesh) -> bool {
    me.editflag & ME_EDIT_MIRROR_VERTEX_GROUPS != 0 && me.symmetry & ME_SYMMETRY_X != 0
}

/// `AttributeStorage` (DNA_attribute_types.h subset needed by Mesh).
#[derive(Clone, Default)]
pub struct AttributeStorage {
    /// `Attribute *dna_attributes` → Vec of raw attribute blobs (port pending).
    pub dna_attributes: Vec<u8>,
    pub dna_attributes_num: i32,
    /// `bke::AttributeStorageRuntime *runtime` — opaque.
    pub runtime: usize,
}

/// `Mesh` — the mesh data-block.
#[derive(Clone)]
pub struct Mesh {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `Key *key`.
    pub key: usize,
    /// `Material **mat` → arena indices.
    pub mat: Vec<usize>,
    pub verts_num: i32,
    pub edges_num: i32,
    pub faces_num: i32,
    pub corners_num: i32,
    /// `int *face_offset_indices` → Vec.
    pub face_offset_indices: Vec<i32>,
    pub attribute_storage: AttributeStorage,
    pub vert_data: CustomData,
    pub edge_data: CustomData,
    pub face_data: CustomData,
    pub corner_data: CustomData,
    /// `ListBaseT<bDeformGroup>`.
    pub vertex_group_names: ListBase,
    pub vertex_group_active_index: i32,
    pub attributes_active_index: i32,
    /// `MSelect *mselect` → Vec.
    pub mselect: Vec<crate::meshdata_types::MSelect>,
    pub totselect: i32,
    pub act_face: i32,
    /// `Mesh *texcomesh`.
    pub texcomesh: usize,
    pub texspace_location: [f32; 3],
    pub texspace_size: [f32; 3],
    pub texspace_flag: i8,
    pub editflag: i8,
    pub flag: u16,
    /// DNA_DEPRECATED `smoothresh_legacy`.
    pub smoothresh_legacy: f32,
    pub remesh_voxel_size: f32,
    pub remesh_voxel_adaptivity: f32,
    pub face_sets_color_seed: i32,
    pub face_sets_color_default: i32,
    /// `char *active_color_attribute`.
    pub active_color_attribute: Option<String>,
    /// `char *default_color_attribute`.
    pub default_color_attribute: Option<String>,
    /// `char *active_uv_map_attribute`.
    pub active_uv_map_attribute: Option<String>,
    /// `char *default_uv_map_attribute`.
    pub default_uv_map_attribute: Option<String>,
    /// `char *stencil_uv_map_attribute`.
    pub stencil_uv_map_attribute: Option<String>,
    /// `char *clone_uv_map_attribute`.
    pub clone_uv_map_attribute: Option<String>,
    pub symmetry: i8,
    pub remesh_mode: i8,
    pub totcol: i16,
    /// DNA_DEPRECATED `cd_flag`.
    pub cd_flag: i8,
    /// DNA_DEPRECATED `subdiv`.
    pub subdiv: i8,
    /// DNA_DEPRECATED `subdivr`.
    pub subdivr: i8,
    /// DNA_DEPRECATED `subsurftype`.
    pub subsurftype: i8,
    /// DNA_DEPRECATED `mpoly` → Vec.
    pub mpoly: Vec<crate::meshdata_types::MPoly>,
    /// DNA_DEPRECATED `mloop`.
    pub mloop: Vec<crate::meshdata_types::MLoop>,
    /// DNA_DEPRECATED `mvert`.
    pub mvert: Vec<crate::meshdata_types::MVert>,
    /// DNA_DEPRECATED `medge`.
    pub medge: Vec<crate::meshdata_types::MEdge>,
    /// DNA_DEPRECATED `dvert`.
    pub dvert: Vec<crate::meshdata_types::MDeformVert>,
    /// DNA_DEPRECATED `mtface`.
    pub mtface: Vec<crate::meshdata_types::MTFace>,
    /// DNA_DEPRECATED `tface`.
    pub tface: Vec<TFace>,
    /// DNA_DEPRECATED `mcol`.
    pub mcol: Vec<crate::meshdata_types::MCol>,
    /// DNA_DEPRECATED `mface`.
    pub mface: Vec<crate::meshdata_types::MFace>,
    /// DNA_DEPRECATED `fdata_legacy`.
    pub fdata_legacy: CustomData,
    /// DNA_DEPRECATED `totface_legacy`.
    pub totface_legacy: i32,
    pub radial_symmetry: [i8; 3],
    /// `bke::MeshRuntime *runtime` — opaque.
    pub runtime: usize,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            key: NIL,
            mat: Vec::new(),
            verts_num: 0,
            edges_num: 0,
            faces_num: 0,
            corners_num: 0,
            face_offset_indices: Vec::new(),
            attribute_storage: AttributeStorage::default(),
            vert_data: CustomData::default(),
            edge_data: CustomData::default(),
            face_data: CustomData::default(),
            corner_data: CustomData::default(),
            vertex_group_names: ListBase::new(),
            vertex_group_active_index: 0,
            attributes_active_index: -1,
            mselect: Vec::new(),
            totselect: 0,
            act_face: -1,
            texcomesh: NIL,
            texspace_location: [0.0; 3],
            texspace_size: [1.0; 3],
            texspace_flag: ME_TEXSPACE_FLAG_AUTO,
            editflag: ME_EDIT_MIRROR_VERTEX_GROUPS,
            flag: ME_REMESH_REPROJECT_VOLUME | ME_REMESH_REPROJECT_ATTRIBUTES,
            smoothresh_legacy: 0.0,
            remesh_voxel_size: 0.1,
            remesh_voxel_adaptivity: 0.0,
            face_sets_color_seed: 0,
            face_sets_color_default: 1,
            active_color_attribute: None,
            default_color_attribute: None,
            active_uv_map_attribute: None,
            default_uv_map_attribute: None,
            stencil_uv_map_attribute: None,
            clone_uv_map_attribute: None,
            symmetry: 0,
            remesh_mode: REMESH_VOXEL,
            totcol: 0,
            cd_flag: 0,
            subdiv: 0,
            subdivr: 0,
            subsurftype: 0,
            mpoly: Vec::new(),
            mloop: Vec::new(),
            mvert: Vec::new(),
            medge: Vec::new(),
            dvert: Vec::new(),
            mtface: Vec::new(),
            tface: Vec::new(),
            mcol: Vec::new(),
            mface: Vec::new(),
            fdata_legacy: CustomData::default(),
            totface_legacy: 0,
            radial_symmetry: [1, 1, 1],
            runtime: NIL,
        }
    }
}

/// `TFace` (DNA_DEPRECATED_ALLOW).
#[derive(Clone, Default)]
pub struct TFace {
    /// `void *tpage`.
    pub tpage: usize,
    pub uv: [[f32; 2]; 4],
    pub col: [u32; 4],
    pub flag: i8,
    pub transp: i8,
    pub mode: i16,
    pub tile: i16,
    pub unwrap: i16,
}

pub const MESH_MAX_VERTS: i64 = 2_000_000_000;
