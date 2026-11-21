//! Port of `bmesh_opdefines.cc` — operator signature table.
//! `exec`/`init` are None until each op is ported.
use crate::operators::*;

/* BMO_OPTYPE_FLAG_* */
pub const BMO_OPTYPE_FLAG_UNTAN_MULTIRES: i32 = 1 << 0;
pub const BMO_OPTYPE_FLAG_RESERVED: i32 = 1 << 1;
pub const BMO_OPTYPE_FLAG_NORMALS_CALC: i32 = 1 << 2;
pub const BMO_OPTYPE_FLAG_SELECT_FLUSH: i32 = 1 << 3;
pub const BMO_OPTYPE_FLAG_SELECT_VALIDATE: i32 = 1 << 4;
pub const BMO_OPTYPE_FLAG_NOP: i32 = 0;

/// `smooth_vert`.
pub const OPDEF_SMOOTH_VERT: BMOpDefine = BMOpDefine {
    name: "smooth_vert",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("factor", BMOSlotType::Flt),
        ("mirror_clip_x", BMOSlotType::Bool),
        ("mirror_clip_y", BMOSlotType::Bool),
        ("mirror_clip_z", BMOSlotType::Bool),
        ("clip_dist", BMOSlotType::Flt),
        ("use_axis_x", BMOSlotType::Bool),
        ("use_axis_y", BMOSlotType::Bool),
        ("use_axis_z", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_smooth_vert_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `smooth_laplacian_vert`.
pub const OPDEF_SMOOTH_LAPLACIAN_VERT: BMOpDefine = BMOpDefine {
    name: "smooth_laplacian_vert",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("lambda_factor", BMOSlotType::Flt),
        ("lambda_border", BMOSlotType::Flt),
        ("use_x", BMOSlotType::Bool),
        ("use_y", BMOSlotType::Bool),
        ("use_z", BMOSlotType::Bool),
        ("preserve_volume", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_smooth_laplacian::bmo_smooth_laplacian_vert_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `recalc_face_normals`.
pub const OPDEF_RECALC_FACE_NORMALS: BMOpDefine = BMOpDefine {
    name: "recalc_face_normals",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_normals::bmo_recalc_face_normals_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `relax_edge_loops`.
pub const OPDEF_RELAX_EDGE_LOOPS: BMOpDefine = BMOpDefine {
    name: "relax_edge_loops",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("interpolation", BMOSlotType::Int),
        ("iterations", BMOSlotType::Int),
        ("even_spacing", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_relax::bmo_relax_edge_loops_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `planar_faces`.
pub const OPDEF_PLANAR_FACES: BMOpDefine = BMOpDefine {
    name: "planar_faces",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("iterations", BMOSlotType::Int),
        ("factor", BMOSlotType::Flt)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_flatten::bmo_planar_faces_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `region_extend`.
pub const OPDEF_REGION_EXTEND: BMOpDefine = BMOpDefine {
    name: "region_extend",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("use_contract", BMOSlotType::Bool),
        ("use_faces", BMOSlotType::Bool),
        ("use_face_step", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_region_extend_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `rotate_edges`.
pub const OPDEF_ROTATE_EDGES: BMOpDefine = BMOpDefine {
    name: "rotate_edges",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("use_ccw", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_rotate_edges::bmo_rotate_edges_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `reverse_faces`.
pub const OPDEF_REVERSE_FACES: BMOpDefine = BMOpDefine {
    name: "reverse_faces",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("flip_multires", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_reverse_faces_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `flip_quad_tessellation`.
pub const OPDEF_FLIP_QUAD_TESSELLATION: BMOpDefine = BMOpDefine {
    name: "flip_quad_tessellation",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_flip_quad_tessellation_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `bisect_edges`.
pub const OPDEF_BISECT_EDGES: BMOpDefine = BMOpDefine {
    name: "bisect_edges",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("cuts", BMOSlotType::Int),
        ("edge_percents", BMOSlotType::Map)
    ],
    slot_types_out: &[
        ("geom_split.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `mirror`.
pub const OPDEF_MIRROR: BMOpDefine = BMOpDefine {
    name: "mirror",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("matrix", BMOSlotType::Mat),
        ("merge_dist", BMOSlotType::Flt),
        ("axis", BMOSlotType::Int),
        ("mirror_u", BMOSlotType::Bool),
        ("mirror_v", BMOSlotType::Bool),
        ("mirror_udim", BMOSlotType::Bool),
        ("use_shapekey", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_mirror::bmo_mirror_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `find_doubles`.
pub const OPDEF_FIND_DOUBLES: BMOpDefine = BMOpDefine {
    name: "find_doubles",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("keep_verts", BMOSlotType::ElemBuf),
        ("use_connected", BMOSlotType::Bool),
        ("dist", BMOSlotType::Flt)
    ],
    slot_types_out: &[
        ("targetmap.out", BMOSlotType::Map)
    ],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_find_doubles_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `remove_doubles`.
pub const OPDEF_REMOVE_DOUBLES: BMOpDefine = BMOpDefine {
    name: "remove_doubles",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("use_connected", BMOSlotType::Bool),
        ("dist", BMOSlotType::Flt)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_remove_doubles_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `circularize`.
pub const OPDEF_CIRCULARIZE: BMOpDefine = BMOpDefine {
    name: "circularize",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("factor", BMOSlotType::Flt),
        ("custom_radius", BMOSlotType::Flt),
        ("angle", BMOSlotType::Flt),
        ("fit_method", BMOSlotType::Int),
        ("flatten", BMOSlotType::Flt),
        ("regular", BMOSlotType::Bool),
        ("lock_x", BMOSlotType::Bool),
        ("lock_y", BMOSlotType::Bool),
        ("lock_z", BMOSlotType::Bool),
        ("mirror_x", BMOSlotType::Bool),
        ("mirror_y", BMOSlotType::Bool),
        ("mirror_z", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_circularize::bmo_circularize_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `flatten`.
pub const OPDEF_FLATTEN: BMOpDefine = BMOpDefine {
    name: "flatten",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("factor", BMOSlotType::Flt),
        ("method", BMOSlotType::Int),
        ("view_normal", BMOSlotType::Vec),
        ("lock_x", BMOSlotType::Bool),
        ("lock_y", BMOSlotType::Bool),
        ("lock_z", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_flatten::bmo_flatten_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `collapse`.
pub const OPDEF_COLLAPSE: BMOpDefine = BMOpDefine {
    name: "collapse",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_collapse_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `pointmerge_facedata`.
pub const OPDEF_POINTMERGE_FACEDATA: BMOpDefine = BMOpDefine {
    name: "pointmerge_facedata",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("vert_target", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_pointmerge_facedata_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `average_vert_facedata`.
pub const OPDEF_AVERAGE_VERT_FACEDATA: BMOpDefine = BMOpDefine {
    name: "average_vert_facedata",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_average_vert_facedata_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `pointmerge`.
pub const OPDEF_POINTMERGE: BMOpDefine = BMOpDefine {
    name: "pointmerge",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("merge_co", BMOSlotType::Vec),
        ("vert_target", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_pointmerge_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `collapse_uvs`.
pub const OPDEF_COLLAPSE_UVS: BMOpDefine = BMOpDefine {
    name: "collapse_uvs",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_collapse_uvs_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `weld_verts`.
pub const OPDEF_WELD_VERTS: BMOpDefine = BMOpDefine {
    name: "weld_verts",
    slot_types_in: &[
        ("targetmap", BMOSlotType::Map),
        ("use_centroid", BMOSlotType::Bool),
        ("average_vert_data", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_removedoubles::bmo_weld_verts_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_vert`.
pub const OPDEF_CREATE_VERT: BMOpDefine = BMOpDefine {
    name: "create_vert",
    slot_types_in: &[
        ("co", BMOSlotType::Vec)
    ],
    slot_types_out: &[
        ("vert.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_create_vert_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `curve_edge_loops_between_selected`.
pub const OPDEF_CURVE_EDGE_LOOPS_BETWEEN_SELECTED: BMOpDefine = BMOpDefine {
    name: "curve_edge_loops_between_selected",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("factor", BMOSlotType::Flt),
        ("clamp_elevation", BMOSlotType::Int),
        ("extend_loop", BMOSlotType::Bool),
        ("regular", BMOSlotType::Bool),
        ("lock_x", BMOSlotType::Bool),
        ("lock_y", BMOSlotType::Bool),
        ("lock_z", BMOSlotType::Bool),
        ("interpolation", BMOSlotType::Int)
    ],
    slot_types_out: &[],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `join_triangles`.
pub const OPDEF_JOIN_TRIANGLES: BMOpDefine = BMOpDefine {
    name: "join_triangles",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("cmp_seam", BMOSlotType::Bool),
        ("cmp_sharp", BMOSlotType::Bool),
        ("cmp_uvs", BMOSlotType::Bool),
        ("cmp_vcols", BMOSlotType::Bool),
        ("cmp_materials", BMOSlotType::Bool),
        ("angle_face_threshold", BMOSlotType::Flt),
        ("angle_shape_threshold", BMOSlotType::Flt),
        ("topology_influence", BMOSlotType::Flt),
        ("deselect_joined", BMOSlotType::Bool),
        ("merge_limit", BMOSlotType::Int),
        ("neighbor_debug", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `contextual_create`.
pub const OPDEF_CONTEXTUAL_CREATE: BMOpDefine = BMOpDefine {
    name: "contextual_create",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("mat_nr", BMOSlotType::Int),
        ("use_smooth", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf),
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_create::bmo_contextual_create_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `bridge_loops`.
pub const OPDEF_BRIDGE_LOOPS: BMOpDefine = BMOpDefine {
    name: "bridge_loops",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("use_pairs", BMOSlotType::Bool),
        ("use_cyclic", BMOSlotType::Bool),
        ("use_merge", BMOSlotType::Bool),
        ("merge_factor", BMOSlotType::Flt),
        ("twist_offset", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf),
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `grid_fill`.
pub const OPDEF_GRID_FILL: BMOpDefine = BMOpDefine {
    name: "grid_fill",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("mat_nr", BMOSlotType::Int),
        ("use_smooth", BMOSlotType::Bool),
        ("use_interp_simple", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `holes_fill`.
pub const OPDEF_HOLES_FILL: BMOpDefine = BMOpDefine {
    name: "holes_fill",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("sides", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_fill_holes::bmo_holes_fill_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `face_attribute_fill`.
pub const OPDEF_FACE_ATTRIBUTE_FILL: BMOpDefine = BMOpDefine {
    name: "face_attribute_fill",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("use_normals", BMOSlotType::Bool),
        ("use_data", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("faces_fail.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_fill_attribute::bmo_face_attribute_fill_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `edgeloop_fill`.
pub const OPDEF_EDGELOOP_FILL: BMOpDefine = BMOpDefine {
    name: "edgeloop_fill",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("mat_nr", BMOSlotType::Int),
        ("use_smooth", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_fill_edgeloop::bmo_edgeloop_fill_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `edgenet_fill`.
pub const OPDEF_EDGENET_FILL: BMOpDefine = BMOpDefine {
    name: "edgenet_fill",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("mat_nr", BMOSlotType::Int),
        ("use_smooth", BMOSlotType::Bool),
        ("sides", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_edgenet::bmo_edgenet_fill_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `edgenet_prepare`.
pub const OPDEF_EDGENET_PREPARE: BMOpDefine = BMOpDefine {
    name: "edgenet_prepare",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_edgenet::bmo_edgenet_prepare_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `rotate`.
pub const OPDEF_ROTATE: BMOpDefine = BMOpDefine {
    name: "rotate",
    slot_types_in: &[
        ("cent", BMOSlotType::Vec),
        ("matrix", BMOSlotType::Mat),
        ("verts", BMOSlotType::ElemBuf),
        ("space", BMOSlotType::Mat),
        ("use_shapekey", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_rotate_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `translate`.
pub const OPDEF_TRANSLATE: BMOpDefine = BMOpDefine {
    name: "translate",
    slot_types_in: &[
        ("vec", BMOSlotType::Vec),
        ("space", BMOSlotType::Mat),
        ("verts", BMOSlotType::ElemBuf),
        ("use_shapekey", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_translate_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `scale`.
pub const OPDEF_SCALE: BMOpDefine = BMOpDefine {
    name: "scale",
    slot_types_in: &[
        ("vec", BMOSlotType::Vec),
        ("space", BMOSlotType::Mat),
        ("verts", BMOSlotType::ElemBuf),
        ("use_shapekey", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_scale_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `fit_edge_loops_to_annotation`.
pub const OPDEF_FIT_EDGE_LOOPS_TO_ANNOTATION: BMOpDefine = BMOpDefine {
    name: "fit_edge_loops_to_annotation",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("factor", BMOSlotType::Flt),
        ("method", BMOSlotType::Int),
        ("strokes", BMOSlotType::Ptr),
        ("lock_x", BMOSlotType::Bool),
        ("lock_y", BMOSlotType::Bool),
        ("lock_z", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `transform`.
pub const OPDEF_TRANSFORM: BMOpDefine = BMOpDefine {
    name: "transform",
    slot_types_in: &[
        ("matrix", BMOSlotType::Mat),
        ("space", BMOSlotType::Mat),
        ("verts", BMOSlotType::ElemBuf),
        ("use_shapekey", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_utils::bmo_transform_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `object_load_bmesh`.
pub const OPDEF_OBJECT_LOAD_BMESH: BMOpDefine = BMOpDefine {
    name: "object_load_bmesh",
    slot_types_in: &[
        ("scene", BMOSlotType::Ptr),
        ("object", BMOSlotType::Ptr)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_mesh_convert::bmo_object_load_bmesh_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `bmesh_to_mesh`.
pub const OPDEF_BMESH_TO_MESH: BMOpDefine = BMOpDefine {
    name: "bmesh_to_mesh",
    slot_types_in: &[
        ("mesh", BMOSlotType::Ptr),
        ("object", BMOSlotType::Ptr)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_mesh_convert::bmo_bmesh_to_mesh_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `mesh_to_bmesh`.
pub const OPDEF_MESH_TO_BMESH: BMOpDefine = BMOpDefine {
    name: "mesh_to_bmesh",
    slot_types_in: &[
        ("mesh", BMOSlotType::Ptr),
        ("object", BMOSlotType::Ptr),
        ("use_shapekey", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_mesh_convert::bmo_mesh_to_bmesh_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `extrude_discrete_faces`.
pub const OPDEF_EXTRUDE_DISCRETE_FACES: BMOpDefine = BMOpDefine {
    name: "extrude_discrete_faces",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("use_normal_flip", BMOSlotType::Bool),
        ("use_select_history", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_extrude::bmo_extrude_discrete_faces_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `extrude_edge_only`.
pub const OPDEF_EXTRUDE_EDGE_ONLY: BMOpDefine = BMOpDefine {
    name: "extrude_edge_only",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("use_normal_flip", BMOSlotType::Bool),
        ("use_select_history", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_extrude::bmo_extrude_edge_only_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `extrude_vert_indiv`.
pub const OPDEF_EXTRUDE_VERT_INDIV: BMOpDefine = BMOpDefine {
    name: "extrude_vert_indiv",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("use_select_history", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf),
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_extrude::bmo_extrude_vert_indiv_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `connect_verts`.
pub const OPDEF_CONNECT_VERTS: BMOpDefine = BMOpDefine {
    name: "connect_verts",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("faces_exclude", BMOSlotType::ElemBuf),
        ("check_degenerate", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_connect::bmo_connect_verts_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `connect_verts_concave`.
pub const OPDEF_CONNECT_VERTS_CONCAVE: BMOpDefine = BMOpDefine {
    name: "connect_verts_concave",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf),
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_connect_ex::bmo_connect_verts_concave_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `connect_verts_nonplanar`.
pub const OPDEF_CONNECT_VERTS_NONPLANAR: BMOpDefine = BMOpDefine {
    name: "connect_verts_nonplanar",
    slot_types_in: &[
        ("angle_limit", BMOSlotType::Flt),
        ("faces", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf),
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_connect_ex::bmo_connect_verts_nonplanar_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `connect_vert_pair`.
pub const OPDEF_CONNECT_VERT_PAIR: BMOpDefine = BMOpDefine {
    name: "connect_vert_pair",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("verts_exclude", BMOSlotType::ElemBuf),
        ("faces_exclude", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_connect_pair::bmo_connect_vert_pair_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `extrude_face_region`.
pub const OPDEF_EXTRUDE_FACE_REGION: BMOpDefine = BMOpDefine {
    name: "extrude_face_region",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("edges_exclude", BMOSlotType::Map),
        ("use_keep_orig", BMOSlotType::Bool),
        ("use_normal_flip", BMOSlotType::Bool),
        ("use_normal_from_adjacent", BMOSlotType::Bool),
        ("use_dissolve_ortho_edges", BMOSlotType::Bool),
        ("use_select_history", BMOSlotType::Bool),
        ("skip_input_flip", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_extrude::bmo_extrude_face_region_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `dissolve_verts`.
pub const OPDEF_DISSOLVE_VERTS: BMOpDefine = BMOpDefine {
    name: "dissolve_verts",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("use_face_split", BMOSlotType::Bool),
        ("use_boundary_tear", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_dissolve::bmo_dissolve_verts_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `dissolve_edges`.
pub const OPDEF_DISSOLVE_EDGES: BMOpDefine = BMOpDefine {
    name: "dissolve_edges",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("use_verts", BMOSlotType::Bool),
        ("use_face_split", BMOSlotType::Bool),
        ("angle_threshold", BMOSlotType::Flt),
        ("use_preserve_quads", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("region.out", BMOSlotType::ElemBuf)
    ],
    init: Some(crate::bmo::bmo_dissolve::bmo_dissolve_edges_init), exec: Some(crate::bmo::bmo_dissolve::bmo_dissolve_edges_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `dissolve_faces`.
pub const OPDEF_DISSOLVE_FACES: BMOpDefine = BMOpDefine {
    name: "dissolve_faces",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("use_verts", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("region.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_dissolve::bmo_dissolve_faces_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `dissolve_limit`.
pub const OPDEF_DISSOLVE_LIMIT: BMOpDefine = BMOpDefine {
    name: "dissolve_limit",
    slot_types_in: &[
        ("angle_limit", BMOSlotType::Flt),
        ("use_dissolve_boundaries", BMOSlotType::Bool),
        ("verts", BMOSlotType::ElemBuf),
        ("edges", BMOSlotType::ElemBuf),
        ("delimit", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("region.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_dissolve::bmo_dissolve_limit_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `dissolve_degenerate`.
pub const OPDEF_DISSOLVE_DEGENERATE: BMOpDefine = BMOpDefine {
    name: "dissolve_degenerate",
    slot_types_in: &[
        ("dist", BMOSlotType::Flt),
        ("edges", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_dissolve::bmo_dissolve_degenerate_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `triangulate`.
pub const OPDEF_TRIANGULATE: BMOpDefine = BMOpDefine {
    name: "triangulate",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("quad_method", BMOSlotType::Int),
        ("ngon_method", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf),
        ("faces.out", BMOSlotType::ElemBuf),
        ("face_map.out", BMOSlotType::Map),
        ("face_map_double.out", BMOSlotType::Map)
    ],
    init: None, exec: Some(crate::bmo::bmo_triangulate::bmo_triangulate_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `unsubdivide`.
pub const OPDEF_UNSUBDIVIDE: BMOpDefine = BMOpDefine {
    name: "unsubdivide",
    slot_types_in: &[
        ("verts", BMOSlotType::ElemBuf),
        ("iterations", BMOSlotType::Int)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_unsubdivide::bmo_unsubdivide_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `subdivide_edges`.
pub const OPDEF_SUBDIVIDE_EDGES: BMOpDefine = BMOpDefine {
    name: "subdivide_edges",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("smooth", BMOSlotType::Flt),
        ("smooth_falloff", BMOSlotType::Int),
        ("fractal", BMOSlotType::Flt),
        ("along_normal", BMOSlotType::Flt),
        ("cuts", BMOSlotType::Int),
        ("seed", BMOSlotType::Int),
        ("custom_patterns", BMOSlotType::Map),
        ("edge_percents", BMOSlotType::Map),
        ("quad_corner_type", BMOSlotType::Int),
        ("use_grid_fill", BMOSlotType::Bool),
        ("use_single_edge", BMOSlotType::Bool),
        ("use_only_quads", BMOSlotType::Bool),
        ("use_sphere", BMOSlotType::Bool),
        ("use_smooth_even", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom_inner.out", BMOSlotType::ElemBuf),
        ("geom_split.out", BMOSlotType::ElemBuf),
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `subdivide_edgering`.
pub const OPDEF_SUBDIVIDE_EDGERING: BMOpDefine = BMOpDefine {
    name: "subdivide_edgering",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("interp_mode", BMOSlotType::Int),
        ("smooth", BMOSlotType::Flt),
        ("cuts", BMOSlotType::Int),
        ("profile_shape", BMOSlotType::Int),
        ("profile_shape_factor", BMOSlotType::Flt)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `bisect_plane`.
pub const OPDEF_BISECT_PLANE: BMOpDefine = BMOpDefine {
    name: "bisect_plane",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("dist", BMOSlotType::Flt),
        ("plane_co", BMOSlotType::Vec),
        ("plane_no", BMOSlotType::Vec),
        ("use_snap_center", BMOSlotType::Bool),
        ("clear_outer", BMOSlotType::Bool),
        ("clear_inner", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom_cut.out", BMOSlotType::ElemBuf),
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_bisect_plane::bmo_bisect_plane_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `delete`.
pub const OPDEF_DELETE: BMOpDefine = BMOpDefine {
    name: "delete",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("context", BMOSlotType::Int)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_dupe::bmo_delete_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `duplicate`.
pub const OPDEF_DUPLICATE: BMOpDefine = BMOpDefine {
    name: "duplicate",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("dest", BMOSlotType::Ptr),
        ("use_select_history", BMOSlotType::Bool),
        ("use_edge_flip_from_face", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom_orig.out", BMOSlotType::ElemBuf),
        ("geom.out", BMOSlotType::ElemBuf),
        ("vert_map.out", BMOSlotType::Map),
        ("edge_map.out", BMOSlotType::Map),
        ("face_map.out", BMOSlotType::Map),
        ("boundary_map.out", BMOSlotType::Map),
        ("isovert_map.out", BMOSlotType::Map)
    ],
    init: None, exec: Some(crate::bmo::bmo_dupe::bmo_duplicate_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `split`.
pub const OPDEF_SPLIT: BMOpDefine = BMOpDefine {
    name: "split",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("dest", BMOSlotType::Ptr),
        ("use_only_faces", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf),
        ("boundary_map.out", BMOSlotType::Map),
        ("isovert_map.out", BMOSlotType::Map)
    ],
    init: None, exec: Some(crate::bmo::bmo_dupe::bmo_split_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `spin`.
pub const OPDEF_SPIN: BMOpDefine = BMOpDefine {
    name: "spin",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("cent", BMOSlotType::Vec),
        ("axis", BMOSlotType::Vec),
        ("dvec", BMOSlotType::Vec),
        ("angle", BMOSlotType::Flt),
        ("space", BMOSlotType::Mat),
        ("steps", BMOSlotType::Int),
        ("use_merge", BMOSlotType::Bool),
        ("use_normal_flip", BMOSlotType::Bool),
        ("use_duplicate", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom_last.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_dupe::bmo_spin_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `rotate_uvs`.
pub const OPDEF_ROTATE_UVS: BMOpDefine = BMOpDefine {
    name: "rotate_uvs",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("use_ccw", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `reverse_uvs`.
pub const OPDEF_REVERSE_UVS: BMOpDefine = BMOpDefine {
    name: "reverse_uvs",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf)
    ],
    slot_types_out: &[],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `rotate_colors`.
pub const OPDEF_ROTATE_COLORS: BMOpDefine = BMOpDefine {
    name: "rotate_colors",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("use_ccw", BMOSlotType::Bool),
        ("color_index", BMOSlotType::Int)
    ],
    slot_types_out: &[],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `reverse_colors`.
pub const OPDEF_REVERSE_COLORS: BMOpDefine = BMOpDefine {
    name: "reverse_colors",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("color_index", BMOSlotType::Int)
    ],
    slot_types_out: &[],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `split_edges`.
pub const OPDEF_SPLIT_EDGES: BMOpDefine = BMOpDefine {
    name: "split_edges",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("verts", BMOSlotType::ElemBuf),
        ("use_verts", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_split_edges::bmo_split_edges_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_grid`.
pub const OPDEF_CREATE_GRID: BMOpDefine = BMOpDefine {
    name: "create_grid",
    slot_types_in: &[
        ("x_segments", BMOSlotType::Int),
        ("y_segments", BMOSlotType::Int),
        ("size", BMOSlotType::Flt),
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_grid_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_quadsphere`.
pub const OPDEF_CREATE_QUADSPHERE: BMOpDefine = BMOpDefine {
    name: "create_quadsphere",
    slot_types_in: &[
        ("segments", BMOSlotType::Int),
        ("method", BMOSlotType::Int),
        ("radius", BMOSlotType::Flt),
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_quadsphere_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_uvsphere`.
pub const OPDEF_CREATE_UVSPHERE: BMOpDefine = BMOpDefine {
    name: "create_uvsphere",
    slot_types_in: &[
        ("u_segments", BMOSlotType::Int),
        ("v_segments", BMOSlotType::Int),
        ("radius", BMOSlotType::Flt),
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_uvsphere_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_icosphere`.
pub const OPDEF_CREATE_ICOSPHERE: BMOpDefine = BMOpDefine {
    name: "create_icosphere",
    slot_types_in: &[
        ("subdivisions", BMOSlotType::Int),
        ("radius", BMOSlotType::Flt),
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_icosphere_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_monkey`.
pub const OPDEF_CREATE_MONKEY: BMOpDefine = BMOpDefine {
    name: "create_monkey",
    slot_types_in: &[
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_monkey_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_cone`.
pub const OPDEF_CREATE_CONE: BMOpDefine = BMOpDefine {
    name: "create_cone",
    slot_types_in: &[
        ("cap_ends", BMOSlotType::Bool),
        ("cap_tris", BMOSlotType::Bool),
        ("segments", BMOSlotType::Int),
        ("radius1", BMOSlotType::Flt),
        ("radius2", BMOSlotType::Flt),
        ("depth", BMOSlotType::Flt),
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_cone_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_circle`.
pub const OPDEF_CREATE_CIRCLE: BMOpDefine = BMOpDefine {
    name: "create_circle",
    slot_types_in: &[
        ("cap_ends", BMOSlotType::Bool),
        ("cap_tris", BMOSlotType::Bool),
        ("segments", BMOSlotType::Int),
        ("radius", BMOSlotType::Flt),
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_circle_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `create_cube`.
pub const OPDEF_CREATE_CUBE: BMOpDefine = BMOpDefine {
    name: "create_cube",
    slot_types_in: &[
        ("size", BMOSlotType::Flt),
        ("matrix", BMOSlotType::Mat),
        ("calc_uvs", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_primitive::bmo_create_cube_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `bevel`.
pub const OPDEF_BEVEL: BMOpDefine = BMOpDefine {
    name: "bevel",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("offset", BMOSlotType::Flt),
        ("offset_type", BMOSlotType::Int),
        ("profile_type", BMOSlotType::Int),
        ("segments", BMOSlotType::Int),
        ("profile", BMOSlotType::Flt),
        ("affect", BMOSlotType::Int),
        ("clamp_overlap", BMOSlotType::Bool),
        ("material", BMOSlotType::Int),
        ("loop_slide", BMOSlotType::Bool),
        ("mark_seam", BMOSlotType::Bool),
        ("mark_sharp", BMOSlotType::Bool),
        ("harden_normals", BMOSlotType::Bool),
        ("face_strength_mode", BMOSlotType::Int),
        ("miter_outer", BMOSlotType::Int),
        ("miter_inner", BMOSlotType::Int),
        ("spread", BMOSlotType::Flt),
        ("custom_profile", BMOSlotType::Ptr),
        ("vmesh_method", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf),
        ("edges.out", BMOSlotType::ElemBuf),
        ("verts.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_bevel::bmo_bevel_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `beautify_fill`.
pub const OPDEF_BEAUTIFY_FILL: BMOpDefine = BMOpDefine {
    name: "beautify_fill",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("edges", BMOSlotType::ElemBuf),
        ("use_restrict_tag", BMOSlotType::Bool),
        ("method", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_beautify::bmo_beautify_fill_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `triangle_fill`.
pub const OPDEF_TRIANGLE_FILL: BMOpDefine = BMOpDefine {
    name: "triangle_fill",
    slot_types_in: &[
        ("use_beauty", BMOSlotType::Bool),
        ("use_dissolve", BMOSlotType::Bool),
        ("edges", BMOSlotType::ElemBuf),
        ("normal", BMOSlotType::Vec)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `solidify`.
pub const OPDEF_SOLIDIFY: BMOpDefine = BMOpDefine {
    name: "solidify",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("thickness", BMOSlotType::Flt)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_extrude::bmo_solidify_face_region_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `inset_individual`.
pub const OPDEF_INSET_INDIVIDUAL: BMOpDefine = BMOpDefine {
    name: "inset_individual",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("thickness", BMOSlotType::Flt),
        ("depth", BMOSlotType::Flt),
        ("use_even_offset", BMOSlotType::Bool),
        ("use_interpolate", BMOSlotType::Bool),
        ("use_relative_offset", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `inset_region`.
pub const OPDEF_INSET_REGION: BMOpDefine = BMOpDefine {
    name: "inset_region",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("faces_exclude", BMOSlotType::ElemBuf),
        ("use_boundary", BMOSlotType::Bool),
        ("use_even_offset", BMOSlotType::Bool),
        ("use_interpolate", BMOSlotType::Bool),
        ("use_relative_offset", BMOSlotType::Bool),
        ("use_edge_rail", BMOSlotType::Bool),
        ("thickness", BMOSlotType::Flt),
        ("depth", BMOSlotType::Flt),
        ("use_outset", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `offset_edgeloops`.
pub const OPDEF_OFFSET_EDGELOOPS: BMOpDefine = BMOpDefine {
    name: "offset_edgeloops",
    slot_types_in: &[
        ("edges", BMOSlotType::ElemBuf),
        ("use_cap_endpoint", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("edges.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_offset_edgeloops::bmo_offset_edgeloops_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `wireframe`.
pub const OPDEF_WIREFRAME: BMOpDefine = BMOpDefine {
    name: "wireframe",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("thickness", BMOSlotType::Flt),
        ("offset", BMOSlotType::Flt),
        ("use_replace", BMOSlotType::Bool),
        ("use_boundary", BMOSlotType::Bool),
        ("use_even_offset", BMOSlotType::Bool),
        ("use_crease", BMOSlotType::Bool),
        ("crease_weight", BMOSlotType::Flt),
        ("use_relative_offset", BMOSlotType::Bool),
        ("material_offset", BMOSlotType::Int)
    ],
    slot_types_out: &[
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_wireframe::bmo_wireframe_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `poke`.
pub const OPDEF_POKE: BMOpDefine = BMOpDefine {
    name: "poke",
    slot_types_in: &[
        ("faces", BMOSlotType::ElemBuf),
        ("offset", BMOSlotType::Flt),
        ("center_mode", BMOSlotType::Int),
        ("use_relative_offset", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("verts.out", BMOSlotType::ElemBuf),
        ("faces.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_poke::bmo_poke_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `convex_hull`.
pub const OPDEF_CONVEX_HULL: BMOpDefine = BMOpDefine {
    name: "convex_hull",
    slot_types_in: &[
        ("input", BMOSlotType::ElemBuf),
        ("use_existing_faces", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf),
        ("geom_interior.out", BMOSlotType::ElemBuf),
        ("geom_unused.out", BMOSlotType::ElemBuf),
        ("geom_holes.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: None,
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `space_edge_loops_evenly`.
pub const OPDEF_SPACE_EDGE_LOOPS_EVENLY: BMOpDefine = BMOpDefine {
    name: "space_edge_loops_evenly",
    slot_types_in: &[
        ("geom", BMOSlotType::ElemBuf),
        ("interpolation", BMOSlotType::Int),
        ("factor", BMOSlotType::Flt),
        ("lock_x", BMOSlotType::Bool),
        ("lock_y", BMOSlotType::Bool),
        ("lock_z", BMOSlotType::Bool)
    ],
    slot_types_out: &[],
    init: None, exec: Some(crate::bmo::bmo_space::bmo_space_edge_loops_evenly_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `symmetrize`.
pub const OPDEF_SYMMETRIZE: BMOpDefine = BMOpDefine {
    name: "symmetrize",
    slot_types_in: &[
        ("input", BMOSlotType::ElemBuf),
        ("direction", BMOSlotType::Int),
        ("dist", BMOSlotType::Flt),
        ("use_shapekey", BMOSlotType::Bool)
    ],
    slot_types_out: &[
        ("geom.out", BMOSlotType::ElemBuf)
    ],
    init: None, exec: Some(crate::bmo::bmo_mirror::bmo_symmetrize_exec),
    type_flag: BMO_OPTYPE_FLAG_NOP,
};

/// `bmo_opdefines[]` — all defs in C order.
pub static BMO_OPDEFINES: &[&BMOpDefine] = &[
    &OPDEF_SMOOTH_VERT,
    &OPDEF_SMOOTH_LAPLACIAN_VERT,
    &OPDEF_RECALC_FACE_NORMALS,
    &OPDEF_RELAX_EDGE_LOOPS,
    &OPDEF_PLANAR_FACES,
    &OPDEF_REGION_EXTEND,
    &OPDEF_ROTATE_EDGES,
    &OPDEF_REVERSE_FACES,
    &OPDEF_FLIP_QUAD_TESSELLATION,
    &OPDEF_BISECT_EDGES,
    &OPDEF_MIRROR,
    &OPDEF_FIND_DOUBLES,
    &OPDEF_REMOVE_DOUBLES,
    &OPDEF_CIRCULARIZE,
    &OPDEF_FLATTEN,
    &OPDEF_COLLAPSE,
    &OPDEF_POINTMERGE_FACEDATA,
    &OPDEF_AVERAGE_VERT_FACEDATA,
    &OPDEF_POINTMERGE,
    &OPDEF_COLLAPSE_UVS,
    &OPDEF_WELD_VERTS,
    &OPDEF_CREATE_VERT,
    &OPDEF_CURVE_EDGE_LOOPS_BETWEEN_SELECTED,
    &OPDEF_JOIN_TRIANGLES,
    &OPDEF_CONTEXTUAL_CREATE,
    &OPDEF_BRIDGE_LOOPS,
    &OPDEF_GRID_FILL,
    &OPDEF_HOLES_FILL,
    &OPDEF_FACE_ATTRIBUTE_FILL,
    &OPDEF_EDGELOOP_FILL,
    &OPDEF_EDGENET_FILL,
    &OPDEF_EDGENET_PREPARE,
    &OPDEF_ROTATE,
    &OPDEF_TRANSLATE,
    &OPDEF_SCALE,
    &OPDEF_FIT_EDGE_LOOPS_TO_ANNOTATION,
    &OPDEF_TRANSFORM,
    &OPDEF_OBJECT_LOAD_BMESH,
    &OPDEF_BMESH_TO_MESH,
    &OPDEF_MESH_TO_BMESH,
    &OPDEF_EXTRUDE_DISCRETE_FACES,
    &OPDEF_EXTRUDE_EDGE_ONLY,
    &OPDEF_EXTRUDE_VERT_INDIV,
    &OPDEF_CONNECT_VERTS,
    &OPDEF_CONNECT_VERTS_CONCAVE,
    &OPDEF_CONNECT_VERTS_NONPLANAR,
    &OPDEF_CONNECT_VERT_PAIR,
    &OPDEF_EXTRUDE_FACE_REGION,
    &OPDEF_DISSOLVE_VERTS,
    &OPDEF_DISSOLVE_EDGES,
    &OPDEF_DISSOLVE_FACES,
    &OPDEF_DISSOLVE_LIMIT,
    &OPDEF_DISSOLVE_DEGENERATE,
    &OPDEF_TRIANGULATE,
    &OPDEF_UNSUBDIVIDE,
    &OPDEF_SUBDIVIDE_EDGES,
    &OPDEF_SUBDIVIDE_EDGERING,
    &OPDEF_BISECT_PLANE,
    &OPDEF_DELETE,
    &OPDEF_DUPLICATE,
    &OPDEF_SPLIT,
    &OPDEF_SPIN,
    &OPDEF_ROTATE_UVS,
    &OPDEF_REVERSE_UVS,
    &OPDEF_ROTATE_COLORS,
    &OPDEF_REVERSE_COLORS,
    &OPDEF_SPLIT_EDGES,
    &OPDEF_CREATE_GRID,
    &OPDEF_CREATE_QUADSPHERE,
    &OPDEF_CREATE_UVSPHERE,
    &OPDEF_CREATE_ICOSPHERE,
    &OPDEF_CREATE_MONKEY,
    &OPDEF_CREATE_CONE,
    &OPDEF_CREATE_CIRCLE,
    &OPDEF_CREATE_CUBE,
    &OPDEF_BEVEL,
    &OPDEF_BEAUTIFY_FILL,
    &OPDEF_TRIANGLE_FILL,
    &OPDEF_SOLIDIFY,
    &OPDEF_INSET_INDIVIDUAL,
    &OPDEF_INSET_REGION,
    &OPDEF_OFFSET_EDGELOOPS,
    &OPDEF_WIREFRAME,
    &OPDEF_POKE,
    &OPDEF_CONVEX_HULL,
    &OPDEF_SPACE_EDGE_LOOPS_EVENLY,
    &OPDEF_SYMMETRIZE,
];

/// `BMO_opcode_from_opname` — index into BMO_OPDEFINES or -1.
pub fn bmo_opcode_from_opname(opname: &str) -> i32 {
    BMO_OPDEFINES.iter().position(|d| d.name == opname).map(|i| i as i32).unwrap_or(-1)
}
