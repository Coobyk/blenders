//! Ports of `intern/subdiv_converter.cc`, `intern/subdiv_displacement.cc`,
//! `intern/subdiv_topology.cc` plus the `subdiv::Settings` shell
//! (`BKE_subdiv.hh`). OpenSubdiv itself is a foreign dependency — the
//! OpenSubdiv enum constants are mirrored as plain ints.

/// `eSubdivVtxBoundaryInterpolation` (BKE_subdiv.hh).
pub const SUBDIV_VTX_BOUNDARY_NONE: i32 = 0;
pub const SUBDIV_VTX_BOUNDARY_EDGE_ONLY: i32 = 1;
pub const SUBDIV_VTX_BOUNDARY_EDGE_AND_CORNER: i32 = 2;

/// `eSubdivFVarLinearInterpolation` (BKE_subdiv.hh).
pub const SUBDIV_FVAR_LINEAR_INTERPOLATION_NONE: i32 = 0;
pub const SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY: i32 = 1;
pub const SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_AND_JUNCTIONS: i32 = 2;
pub const SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_JUNCTIONS_AND_CONCAVE: i32 = 3;
pub const SUBDIV_FVAR_LINEAR_INTERPOLATION_BOUNDARIES: i32 = 4;
pub const SUBDIV_FVAR_LINEAR_INTERPOLATION_ALL: i32 = 5;

/// `OpenSubdiv_TopologyRefinerSettings_VtxBoundaryInterpolation` (OSD).
pub const OSD_VTX_BOUNDARY_NONE: i32 = 0;
pub const OSD_VTX_BOUNDARY_EDGE_ONLY: i32 = 1;
pub const OSD_VTX_BOUNDARY_EDGE_AND_CORNER: i32 = 2;

/// `OpenSubdiv_FVarLinearInterpolation` (OSD).
pub const OSD_FVAR_LINEAR_INTERPOLATION_NONE: i32 = 0;
pub const OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY: i32 = 1;
pub const OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS1: i32 = 2;
pub const OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS2: i32 = 3;
pub const OSD_FVAR_LINEAR_INTERPOLATION_BOUNDARIES: i32 = 4;
pub const OSD_FVAR_LINEAR_INTERPOLATION_ALL: i32 = 5;

/// `bke::subdiv::Settings` (BKE_subdiv.hh).
#[derive(Clone)]
pub struct Settings {
    pub is_simple: bool,
    pub is_adaptive: bool,
    pub level: i32,
    pub use_creases: bool,
    pub vtx_boundary_interpolation: i32,
    pub fvar_linear_interpolation: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            is_simple: false,
            is_adaptive: false,
            level: 0,
            use_creases: true,
            vtx_boundary_interpolation: SUBDIV_VTX_BOUNDARY_EDGE_ONLY,
            fvar_linear_interpolation: SUBDIV_FVAR_LINEAR_INTERPOLATION_ALL,
        }
    }
}

/// `bke::subdiv::ToMeshSettings` (BKE_subdiv.hh).
#[derive(Clone)]
pub struct ToMeshSettings {
    pub resolution: i32,
    pub use_optimal_display: bool,
}

impl Default for ToMeshSettings {
    fn default() -> Self {
        Self { resolution: 0, use_optimal_display: false }
    }
}

/// `Subdiv` — the runtime subdiv object; OpenSubdiv evaluators are `usize`
/// handles (PORT_PENDING the evaluator itself).
#[derive(Default)]
pub struct Subdiv {
    pub displacement_evaluator: usize,
    pub topology_refiner: usize,
    pub evaluator: usize,
}

// --- intern/subdiv_converter.cc ------------------------------------------

/// `converter_free` — `freeUserData(converter)`; allocator handle.
pub fn converter_free(_converter: usize) {
    // `converter->freeUserData` callback — PORT_PENDING (OpenSubdiv).
}

/// `converter_vtx_boundary_interpolation_from_settings`.
pub fn converter_vtx_boundary_interpolation_from_settings(settings: &Settings) -> i32 {
    match settings.vtx_boundary_interpolation {
        SUBDIV_VTX_BOUNDARY_NONE => OSD_VTX_BOUNDARY_NONE,
        SUBDIV_VTX_BOUNDARY_EDGE_ONLY => OSD_VTX_BOUNDARY_EDGE_ONLY,
        SUBDIV_VTX_BOUNDARY_EDGE_AND_CORNER => OSD_VTX_BOUNDARY_EDGE_AND_CORNER,
        _ => {
            debug_assert!(false, "Unknown vtx boundary interpolation");
            OSD_VTX_BOUNDARY_EDGE_ONLY
        }
    }
}

/// `converter_fvar_linear_from_settings`.
pub fn converter_fvar_linear_from_settings(settings: &Settings) -> i32 {
    match settings.fvar_linear_interpolation {
        SUBDIV_FVAR_LINEAR_INTERPOLATION_NONE => OSD_FVAR_LINEAR_INTERPOLATION_NONE,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY => OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_AND_JUNCTIONS => OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS1,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_JUNCTIONS_AND_CONCAVE => OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS2,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_BOUNDARIES => OSD_FVAR_LINEAR_INTERPOLATION_BOUNDARIES,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_ALL => OSD_FVAR_LINEAR_INTERPOLATION_ALL,
        _ => {
            debug_assert!(false, "Unknown fvar linear interpolation");
            OSD_FVAR_LINEAR_INTERPOLATION_NONE
        }
    }
}

// --- intern/subdiv_displacement.cc ---------------------------------------

/// `displacement_detach` — frees `subdiv->displacement_evaluator`.
pub fn displacement_detach(subdiv: &mut Subdiv) {
    if subdiv.displacement_evaluator == 0 {
        return;
    }
    // `evaluator->free(evaluator)` + MEM_delete — PORT_PENDING (OpenSubdiv).
    subdiv.displacement_evaluator = 0;
}

// --- intern/subdiv_topology.cc -------------------------------------------

/// `topology_num_fvar_layers_get` — 0 without OpenSubdiv (the `!WITH_OPENSUBDIV`
/// branch, which matches our state until the refiner is ported).
pub fn topology_num_fvar_layers_get(_subdiv: &Subdiv) -> i32 {
    0
}
