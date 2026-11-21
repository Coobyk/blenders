//! Port of `intern/particle_system.cc` pure parts —
//! `particles_are_dynamic`, `psys_get_current_display_
//! percentage`, `tot_particles`, `psys_reset`,
//! `psys_get_child_number`/`_tot_child`,
//! `psys_get_pointcache_start_end`, `psys_get_target_
//! system`, `psys_count_keyed_targets`, `PSYS_RESET_*`.
//!
//! `PORT_PENDING`: `realloc_particles`, `psys_calc_dmcache`,
//! `init_particle`/`reset_particle`, the SPH solver
//! (`sph_*`), collision response (`nr_*`, `collision_*`),
//! integrators (`basic_integrate`/`sph_integrate`), keyed
//! target setup (`set_keyed_keys`), bvh updates, threading.

use blenders_dna::particle_types::*;
use blenders_math::base::{max_ii, min_ii};

/// `PSYS_RESET_*` verbatim.
pub const PSYS_RESET_ALL: i32 = 1;
/// `PSYS_RESET_DEPSGRAPH`.
pub const PSYS_RESET_DEPSGRAPH: i32 = 2;
/// `PSYS_RESET_CACHE_MISS`.
pub const PSYS_RESET_CACHE_MISS: i32 = 3;

/// `particles_are_dynamic` (verbatim — returns flag or
/// phystype-membership as i32 truth like C).
pub fn particles_are_dynamic(
    pointcache_baked: bool,
    part_type: i16,
    psys_flag: i32,
    part_phystype: i16,
) -> bool {
    if pointcache_baked {
        return false;
    }
    if part_type == PART_HAIR {
        return psys_flag & PSYS_HAIR_DYNAMICS != 0;
    }
    part_phystype == PART_PHYS_NEWTON
        || part_phystype == PART_PHYS_BOIDS
        || part_phystype == PART_PHYS_FLUID
}

/// `psys_get_current_display_percentage` (verbatim).
pub fn psys_get_current_display_percentage(
    use_render_params: bool,
    dynamic: bool,
    child_percent: i32,
    childtype: i16,
    pointcache_baking: bool,
    disp: i16,
) -> f32 {
    if (use_render_params && !dynamic)
        || (child_percent != 0 && childtype != 0)
        || pointcache_baking
    {
        return 1.0;
    }
    disp as f32 / 100.0
}

/// `tot_particles` (verbatim — `pid_totpoint` is
/// `Some(tot)` when the pointcache is external).
pub fn tot_particles(
    pid_totpoint: Option<i32>,
    distr: i16,
    from: i16,
    grid_res: i32,
    totunexist: i32,
    totpart: i32,
) -> i32 {
    if let Some(tot) = pid_totpoint {
        return tot;
    }
    if distr == PART_DISTR_GRID && from != PART_FROM_VERT {
        return grid_res * grid_res * grid_res - totunexist;
    }
    totpart - totunexist
}

/// `psys_get_child_number` (verbatim — injected
/// `get_render_child_particle_number`).
pub fn psys_get_child_number(
    use_render_params: bool,
    childtype: i16,
    child_render_percent: i32,
    child_percent: i32,
    render_child_particle_number: &dyn Fn(i32, bool) -> i32,
) -> i32 {
    if childtype == 0 {
        return 0;
    }
    let child_num = if use_render_params {
        child_render_percent
    } else {
        child_percent
    };
    render_child_particle_number(child_num, use_render_params)
}

/// `psys_get_tot_child` (verbatim).
pub fn psys_get_tot_child(totpart: i32, child_number: i32) -> i32 {
    totpart * child_number
}

/// `psys_get_pointcache_start_end` (verbatim).
pub fn psys_get_pointcache_start_end(
    sta: f32,
    end: f32,
    lifetime: f32,
    pefra: i32,
    efra: i32,
) -> (i32, i32) {
    (
        max_ii(1, sta as i32),
        min_ii((end + lifetime + 1.0) as i32, max_ii(pefra, efra)),
    )
}

/// `psys_get_target_system` — resolves `pt->psys - 1`
/// inside a particle-system list; returns index + sets
/// `PTARGET_VALID` on the flag (verbatim).
pub fn psys_get_target_system(
    ob_is_target: bool,
    pt_psys: i32,
    ob_list_len: usize,
    pt_ob_list_len: usize,
    pt_flag: &mut ParticletargetFlag,
) -> Option<usize> {
    let len = if ob_is_target { ob_list_len } else { pt_ob_list_len };
    let idx = (pt_psys - 1) as usize;
    let found = pt_psys >= 1 && idx < len;
    if found {
        *pt_flag |= PTARGET_VALID;
        Some(idx)
    } else {
        *pt_flag &= !PTARGET_VALID;
        None
    }
}

/// `psys_count_keyed_targets` (verbatim — targets passed as
/// `(has_valid_totpart, duration)` tuples).
pub fn psys_count_keyed_targets(
    targets: &[(bool, f32)],
    psys_flag: i32,
    keyed_loops: i32,
) -> i32 {
    let mut totkeyed = 0i32;
    let mut keys_valid = 1i32;
    for &(valid, duration) in targets {
        if valid {
            totkeyed += keys_valid;
            if psys_flag & PSYS_KEYED_TIMING != 0 && duration != 0.0 {
                totkeyed += 1;
            }
        } else {
            keys_valid = 0;
        }
    }
    totkeyed * if psys_flag & PSYS_KEYED_TIMING != 0 { 1 } else { keyed_loops }
}
