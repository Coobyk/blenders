/// Port of `mesh_extractors/extract_mesh.cc`
/// + `extract_mesh.hh` — shared
/// MeshRenderData + EditLoopData flag utils.
/// `VFLAG_*` vert/edge/face flag bits
/// (verbatim).
pub const VFLAG_VERT_UV_SELECT: u32 = 1 << 0;
pub const VFLAG_VERT_UV_PINNED: u32 = 1 << 1;
pub const VFLAG_EDGE_UV_SELECT: u32 = 1 << 2;
pub const VFLAG_FACE_UV_SELECT: u32 = 1 << 3;
pub const VFLAG_FACE_UV_ACTIVE: u32 = 1 << 4;
pub const VFLAG_FACE_ACTIVE: u32 = 1 << 5;
pub const VFLAG_FACE_SELECTED: u32 = 1 << 6;
pub const VFLAG_FACE_FREESTYLE: u32 = 1 << 7;
/// `EditLoopData` (verbatim — per-corner
/// edit flag).
#[derive(Default, Clone, Copy)]
pub struct EditLoopData {
    /// `v_flag`.
    pub v_flag: u32,
}
/// `MeshRenderData` (verbatim — mesh +
/// bmesh + loose-element index sets).
pub struct MeshRenderData {
    /// `vert_positions`.
    pub vert_positions: Vec<[f32; 3]>,
    /// `edges` (v1,v2).
    pub edges: Vec<(i32, i32)>,
    /// `faces` (offset+size into corner_vert).
    pub faces: Vec<(i32, i32)>,
    /// `corner_verts` (per-corner vert idx).
    pub corner_verts: Vec<i32>,
    /// `loop_tris` (corner idx triples).
    pub loop_tris: Vec<[i32; 3]>,
    /// `loose_edges` (index into `edges`).
    pub loose_edges: Vec<i32>,
    /// `loose_verts` (index into verts).
    pub loose_verts: Vec<i32>,
    /// `efa_act` (active face idx).
    pub efa_act: i32,
    /// `efa_act_uv`.
    pub efa_act_uv: i32,
}
/// `mesh_render_data_face_flag` (verbatim —
/// set face UV/active/select bits).
pub fn mesh_render_data_face_flag(
    mr: &MeshRenderData,
    efa: i32,
    selected: bool,
    hidden: bool,
    uv_select: bool,
    eattr: &mut EditLoopData,
) {
    if efa == mr.efa_act {
        eattr.v_flag |= VFLAG_FACE_ACTIVE;
    }
    if selected {
        eattr.v_flag |= VFLAG_FACE_SELECTED;
    }
    if efa == mr.efa_act_uv {
        eattr.v_flag |= VFLAG_FACE_UV_ACTIVE;
    }
    if !hidden && uv_select {
        eattr.v_flag |= VFLAG_FACE_UV_SELECT;
    }
}
