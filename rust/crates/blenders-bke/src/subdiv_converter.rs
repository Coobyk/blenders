//! Port of `intern/subdiv_converter.cc` — Settings → OpenSubdiv enum
//! mapping + `converter_free`. The `OpenSubdiv_Converter` struct itself
//! is CAPI-bound: held as a handle (PORT_PENDING the converter impls).

use crate::subdiv::{
    Settings, SUBDIV_FVAR_LINEAR_INTERPOLATION_ALL, SUBDIV_FVAR_LINEAR_INTERPOLATION_BOUNDARIES,
    SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_AND_JUNCTIONS,
    SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_JUNCTIONS_AND_CONCAVE,
    SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY, SUBDIV_FVAR_LINEAR_INTERPOLATION_NONE,
    SUBDIV_VTX_BOUNDARY_EDGE_AND_CORNER, SUBDIV_VTX_BOUNDARY_EDGE_ONLY, SUBDIV_VTX_BOUNDARY_NONE,
};

/// `OpenSubdiv_VtxBoundaryInterpolation` (opensubdiv_capi_type.hh).
pub const OSD_VTX_BOUNDARY_NONE: i32 = 0;
pub const OSD_VTX_BOUNDARY_EDGE_ONLY: i32 = 1;
pub const OSD_VTX_BOUNDARY_EDGE_AND_CORNER: i32 = 2;

/// `OpenSubdiv_FVarLinearInterpolation` (opensubdiv_capi_type.hh).
pub const OSD_FVAR_LINEAR_INTERPOLATION_NONE: i32 = 0;
pub const OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY: i32 = 1;
pub const OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS1: i32 = 2;
pub const OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS2: i32 = 3;
pub const OSD_FVAR_LINEAR_INTERPOLATION_BOUNDARIES: i32 = 4;
pub const OSD_FVAR_LINEAR_INTERPOLATION_ALL: i32 = 5;

/// `converter_free` — runs the optional `freeUserData` callback.
pub fn converter_free(free_user_data: &mut Option<Box<dyn FnMut() + Send>>) {
    if let Some(free) = free_user_data {
        free();
    }
}

/// `converter_vtx_boundary_interpolation_from_settings`.
pub fn converter_vtx_boundary_interpolation_from_settings(settings: &Settings) -> i32 {
    match settings.vtx_boundary_interpolation {
        SUBDIV_VTX_BOUNDARY_NONE => OSD_VTX_BOUNDARY_NONE,
        SUBDIV_VTX_BOUNDARY_EDGE_ONLY => OSD_VTX_BOUNDARY_EDGE_ONLY,
        SUBDIV_VTX_BOUNDARY_EDGE_AND_CORNER => OSD_VTX_BOUNDARY_EDGE_AND_CORNER,
        _ => OSD_VTX_BOUNDARY_EDGE_ONLY, /* assert_msg(0) fallback */
    }
}

/// `converter_fvar_linear_interpolation_from_settings`.
pub fn converter_fvar_linear_interpolation_from_settings(settings: &Settings) -> i32 {
    match settings.fvar_linear_interpolation {
        SUBDIV_FVAR_LINEAR_INTERPOLATION_NONE => OSD_FVAR_LINEAR_INTERPOLATION_NONE,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY => OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_ONLY,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_AND_JUNCTIONS => {
            OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS1
        }
        SUBDIV_FVAR_LINEAR_INTERPOLATION_CORNERS_JUNCTIONS_AND_CONCAVE => {
            OSD_FVAR_LINEAR_INTERPOLATION_CORNERS_PLUS2
        }
        SUBDIV_FVAR_LINEAR_INTERPOLATION_BOUNDARIES => OSD_FVAR_LINEAR_INTERPOLATION_BOUNDARIES,
        SUBDIV_FVAR_LINEAR_INTERPOLATION_ALL => OSD_FVAR_LINEAR_INTERPOLATION_ALL,
        _ => OSD_FVAR_LINEAR_INTERPOLATION_NONE, /* assert_msg(0) fallback */
    }
}
