//! Port of `intern/particle.cc` pure parts — particle
//! settings curve initialization (clump/rough/twist).
//!
//! `PORT_PENDING`: full particle simulation
//! (`psys_get_particle_on_path` state eval, keyed/boid
//! physics, effector forces, `do_guides`, hair matrices),
//! `get_particle_uv`/`BKE_particle_co_hair`/`_uv_on_emitter`
//! (need Mesh/`MTFace` legacy data), batch cache, blend IO.
//!
//! `PART_*`/`PSYS_*`/`PARS_*`/`PARTICLE*` enums live
//! verbatim in `blenders_dna::particle_types` — re-exported
//! for parity with the C include surface.

pub use blenders_dna::particle_types::*;

use crate::colortools::{curvemapping_add, curvemapping_init};
use blenders_dna::color_types::CurveMapping;

fn flat_linear_curve() -> CurveMapping {
    let mut cumap = curvemapping_add(1, 0.0, 0.0, 1.0, 1.0);
    cumap.cm[0].curve[0].x = 0.0;
    cumap.cm[0].curve[0].y = 1.0;
    cumap.cm[0].curve[1].x = 1.0;
    cumap.cm[0].curve[1].y = 1.0;
    curvemapping_init(&mut cumap);
    cumap
}

/// `BKE_particlesettings_clump_curve_init` (verbatim —
/// caller assigns the handle).
pub fn particlesettings_clump_curve_init() -> CurveMapping {
    flat_linear_curve()
}
/// `BKE_particlesettings_rough_curve_init` (verbatim).
pub fn particlesettings_rough_curve_init() -> CurveMapping {
    flat_linear_curve()
}
/// `BKE_particlesettings_twist_curve_init` (verbatim).
pub fn particlesettings_twist_curve_init() -> CurveMapping {
    flat_linear_curve()
}
