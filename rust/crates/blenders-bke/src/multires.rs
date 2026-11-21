//! Port of `intern/multires.cc` — multires modifier helpers:
//! grid-size tables, mdisp hidden downsample, `multires_get_level`
//! / `set_tot_level`, `get_levels_from_disps`, grid copy/downsample,
//! `del_levels`/`del_higher`, `old_mdisps_bilinear`, uniform scale +
//! smat application, `mdisp_corners`, `topology_changed`,
//! `ensure_external_read`.
//!
//! Modifier/Depsgraph/CustomData plumbing is injected (`MultiresIO`
//! trait); array math is fully ported.

/// `multires_grid_tot` — `totdisp` per level (side²·corners).
pub const MULTIRES_GRID_TOT: [i32; 14] = [
    0, 4, 9, 25, 81, 289, 1089, 4225, 16641, 66049, 263169, 1050625, 4198401, 16785409,
];
/// `multires_side_tot` — grid side per level (`(1<<(l-1))+1`).
pub const MULTIRES_SIDE_TOT: [i32; 14] =
    [0, 2, 3, 5, 9, 17, 33, 65, 129, 257, 513, 1025, 2049, 4097];

/// `CCG_grid_size`.
pub fn ccg_grid_size(level: i32) -> i32 {
    MULTIRES_SIDE_TOT[level as usize]
}
/// `CCG_grid_factor` — subsample factor between two levels.
pub fn ccg_grid_factor(new_level: i32, old_level: i32) -> i32 {
    1 << (old_level - new_level).max(0)
}
/// `bke::subdiv::grid_size_from_level` alias.
pub fn grid_size_from_level(level: i32) -> i32 {
    ccg_grid_size(level)
}

/// `MultiresModifiedFlags`.
pub const MULTIRES_COORDS_MODIFIED: i32 = 1 << 0;
/// `MULTIRES_HIDDEN_MODIFIED`.
pub const MULTIRES_HIDDEN_MODIFIED: i32 = 1 << 1;

/// `MDisps` — one corner's displacement grid.
#[derive(Debug, Clone, Default)]
pub struct MDisps {
    /// `totdisp`.
    pub totdisp: i32,
    /// `disps` — `totdisp` xyz displacement vectors.
    pub disps: Vec<[f32; 3]>,
    /// `hidden` — `totdisp`-bit bitmap.
    pub hidden: Vec<bool>,
    /// `level` — mirror of the grid side's level.
    pub level: i32,
}

/// `GridPaintMask`.
#[derive(Debug, Clone, Default)]
pub struct GridPaintMask {
    /// `level`.
    pub level: i32,
    /// `data` — `grid_size(level)²` floats.
    pub data: Vec<f32>,
}

/// `BKE_paint_grid_paint_mask` — read `gpm.data` at `(x, y)` of
/// `level`'s grid (C interpolates between levels).
pub fn paint_grid_paint_mask(gpm: &GridPaintMask, level: i32, x: i32, y: i32) -> f32 {
    if gpm.level == level {
        let g = ccg_grid_size(level);
        return gpm.data[(y * g + x) as usize];
    }
    /* Subsampled read. */
    let factor = ccg_grid_factor(level, gpm.level);
    let g = ccg_grid_size(gpm.level);
    gpm.data[(y * factor * g + x * factor) as usize]
}

/// `multires_mdisps_downsample_hidden`.
pub fn multires_mdisps_downsample_hidden(
    old_hidden: &[bool],
    old_level: i32,
    new_level: i32,
) -> Vec<bool> {
    let new_gridsize = ccg_grid_size(new_level);
    let old_gridsize = ccg_grid_size(old_level);
    debug_assert!(new_level <= old_level);
    let factor = ccg_grid_factor(new_level, old_level);
    let mut new_hidden = vec![false; (new_gridsize * new_gridsize) as usize];
    for y in 0..new_gridsize {
        for x in 0..new_gridsize {
            new_hidden[(y * new_gridsize + x) as usize] =
                old_hidden[(factor * y * old_gridsize + x * factor) as usize];
        }
    }
    new_hidden
}

/// `MultiresModifierData` subset.
#[derive(Debug, Clone, Default)]
pub struct MultiresModifierData {
    /// `totlvl`.
    pub totlvl: i8,
    /// `lvl`.
    pub lvl: i8,
    /// `sculptlvl`.
    pub sculptlvl: i8,
    /// `renderlvl`.
    pub renderlvl: i8,
}

/// `OB_MODE_SCULPT`.
pub const OB_MODE_SCULPT: i32 = 1 << 4;

/// `multires_get_level` — `render_subsurf_level` callback resolves
/// scene simplify (`get_render_subsurf_level(r, lvl, for_render)`).
pub fn multires_get_level(
    mmd: &MultiresModifierData,
    render: bool,
    ignore_simplify: bool,
    object_mode_sculpt: bool,
    render_subsurf_level: &dyn Fn(i8, bool) -> i8,
) -> i8 {
    if render {
        return render_subsurf_level(mmd.renderlvl, true);
    }
    if object_mode_sculpt {
        return mmd.sculptlvl;
    }
    if ignore_simplify {
        return mmd.lvl;
    }
    render_subsurf_level(mmd.lvl, false)
}

/// `multires_set_tot_level`.
pub fn multires_set_tot_level(mmd: &mut MultiresModifierData, lvl: i8, object_mode_sculpt: bool) {
    mmd.totlvl = lvl;
    if !object_mode_sculpt {
        mmd.lvl = mmd.lvl.max(lvl).clamp(0, mmd.totlvl);
    }
    mmd.sculptlvl = mmd.sculptlvl.max(lvl).clamp(0, mmd.totlvl);
    mmd.renderlvl = mmd.renderlvl.max(lvl).clamp(0, mmd.totlvl);
}

/// `get_levels_from_disps` — detect `totlvl` from mdisp grid sizes.
pub fn get_levels_from_disps(faces: &dyn Fn() -> Vec<Vec<i32>>, mdisps: &[MDisps]) -> i32 {
    let mut totlvl = 0i32;
    for face in faces() {
        for corner in face {
            let md = &mdisps[corner as usize];
            if md.totdisp == 0 {
                continue;
            }
            loop {
                let side = (1 << (totlvl - 1).max(0)) + 1;
                let lvl_totdisp = side * side;
                if md.totdisp == lvl_totdisp {
                    break;
                }
                if md.totdisp < lvl_totdisp {
                    totlvl -= 1;
                } else {
                    totlvl += 1;
                }
            }
            return totlvl;
        }
    }
    totlvl
}

/// `multiresModifier_set_levels_from_disps`.
pub fn multires_modifier_set_levels_from_disps(
    mmd: &mut MultiresModifierData,
    faces: &dyn Fn() -> Vec<Vec<i32>>,
    mdisps: Option<&[MDisps]>,
) {
    if let Some(mdisps) = mdisps {
        mmd.totlvl = get_levels_from_disps(faces, mdisps) as i8;
        mmd.lvl = mmd.lvl.min(mmd.totlvl);
        mmd.sculptlvl = mmd.sculptlvl.min(mmd.totlvl);
        mmd.renderlvl = mmd.renderlvl.min(mmd.totlvl);
    }
}

/// `multires_set_tot_mdisps`.
pub fn multires_set_tot_mdisps(mdisps: &mut [MDisps], lvl: i32) {
    for md in mdisps.iter_mut() {
        md.totdisp = MULTIRES_GRID_TOT[lvl as usize];
    }
}

/// `multires_copy_grid` — resample `gridB` (sizeB side) into
/// `gridA` (sizeA side), skipping or interpolating.
pub fn multires_copy_grid(
    grid_a: &mut [[f32; 3]],
    grid_b: &[[f32; 3]],
    size_a: i32,
    size_b: i32,
) {
    if size_a > size_b {
        let skip = (size_a - 1) / (size_b - 1);
        let mut j = 0;
        for y in 0..size_b {
            for x in 0..size_b {
                grid_a[(y * skip * size_a + x * skip) as usize] = grid_b[j as usize];
                j += 1;
            }
        }
    } else {
        let skip = (size_b - 1) / (size_a - 1);
        let mut j = 0;
        for y in 0..size_a {
            for x in 0..size_a {
                grid_a[j as usize] = grid_b[(y * skip * size_b + x * skip) as usize];
                j += 1;
            }
        }
    }
}

/// `multires_grid_paint_mask_downsample`.
pub fn multires_grid_paint_mask_downsample(gpm: &mut GridPaintMask, level: i32) {
    if level < gpm.level {
        let gridsize = ccg_grid_size(level);
        let mut data = vec![0.0f32; (gridsize * gridsize) as usize];
        for y in 0..gridsize {
            for x in 0..gridsize {
                data[(y * gridsize + x) as usize] = paint_grid_paint_mask(gpm, level, x, y);
            }
        }
        gpm.data = data;
        gpm.level = level;
    }
}

/// `multires_del_higher` — downsample all corners' disps/hidden/
/// paint-mask grids to `lvl`. `mdisps`/`gpms` are per-corner.
pub fn multires_del_higher(
    mmd: &mut MultiresModifierData,
    mdisps: &mut [MDisps],
    mut gpms: Option<&mut [GridPaintMask]>,
    lvl: i32,
    object_mode_sculpt: bool,
    corners: &dyn Fn() -> Vec<i32>,
) {
    let levels = mmd.totlvl as i32 - lvl;

    if levels > 0 {
        if lvl > 0 {
            let nsize = MULTIRES_SIDE_TOT[lvl as usize];
            let hsize = MULTIRES_SIDE_TOT[mmd.totlvl as usize];
            for corner in corners() {
                let mdisp = &mut mdisps[corner as usize];
                let totdisp = MULTIRES_GRID_TOT[lvl as usize];
                let mut disps = vec![[0.0f32; 3]; totdisp as usize];
                if !mdisp.disps.is_empty() {
                    let hdisps = std::mem::take(&mut mdisp.disps);
                    multires_copy_grid(&mut disps, &hdisps, nsize, hsize);
                    if !mdisp.hidden.is_empty() {
                        mdisp.hidden = multires_mdisps_downsample_hidden(
                            &mdisp.hidden,
                            mmd.totlvl as i32,
                            lvl,
                        );
                    }
                }
                mdisp.disps = disps;
                mdisp.totdisp = totdisp;
                if let Some(gpms) = gpms.as_deref_mut() {
                    multires_grid_paint_mask_downsample(&mut gpms[corner as usize], lvl);
                }
            }
        }
        /* lvl == 0 → `multires_customdata_delete` (caller's
         * CustomData hook). */
    }

    multires_set_tot_level(mmd, lvl as i8, object_mode_sculpt);
}

/// `MultiresSubdivideModeType`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MultiresSubdivideMode {
    /// `MultiresSubdivideModeType::CatmullClark`.
    CatmullClark,
    /// `MultiresSubdivideModeType::Simple`.
    Simple,
    /// `MultiresSubdivideModeType::Linear`.
    Linear,
}

/// `multiresModifier_sync_levels_ex`.
pub fn multires_modifier_sync_levels_ex(
    mmd_src: &MultiresModifierData,
    mmd_dst: &mut MultiresModifierData,
    subdivide_to_level: &mut dyn FnMut(&mut MultiresModifierData, i8, MultiresSubdivideMode),
    del_higher: &mut dyn FnMut(&mut MultiresModifierData, i8),
) {
    if mmd_src.totlvl == mmd_dst.totlvl {
        return;
    }
    if mmd_src.totlvl > mmd_dst.totlvl {
        subdivide_to_level(mmd_dst, mmd_src.totlvl, MultiresSubdivideMode::CatmullClark);
    } else {
        del_higher(mmd_dst, mmd_src.totlvl);
    }
}

/// `old_mdisps_bilinear` — bilinear displacement sample.
pub fn old_mdisps_bilinear(disps: &[[f32; 3]], st: i32, mut u: f32, mut v: f32) -> [f32; 3] {
    if disps.is_empty() || u.is_nan() || v.is_nan() {
        return [0.0; 3];
    }
    let st_max = st - 1;
    u = u.clamp(0.0, st_max as f32);
    v = v.clamp(0.0, st_max as f32);

    let x = u.floor() as i32;
    let y = v.floor() as i32;
    let x2 = (x + 1).min(st_max);
    let y2 = (y + 1).min(st_max);

    let urat = u - x as f32;
    let vrat = v - y as f32;
    let uopp = 1.0 - urat;

    let d0 = disps[(y * st + x) as usize].map(|c| c * uopp);
    let d1 = disps[(y * st + x2) as usize].map(|c| c * urat);
    let d2 = disps[(y2 * st + x) as usize].map(|c| c * uopp);
    let d3 = disps[(y2 * st + x2) as usize].map(|c| c * urat);

    let e0 = [
        (d0[0] + d1[0]) * (1.0 - vrat),
        (d0[1] + d1[1]) * (1.0 - vrat),
        (d0[2] + d1[2]) * (1.0 - vrat),
    ];
    let e1 = [
        (d2[0] + d3[0]) * vrat,
        (d2[1] + d3[1]) * vrat,
        (d2[2] + d3[2]) * vrat,
    ];
    [e0[0] + e1[0], e0[1] + e1[1], e0[2] + e1[2]]
}

/// `multires_apply_uniform_scale`.
pub fn multires_apply_uniform_scale(mdisps: &mut [MDisps], scale: f32) {
    for grid in mdisps.iter_mut() {
        for d in grid.disps.iter_mut() {
            d[0] *= scale;
            d[1] *= scale;
            d[2] *= scale;
        }
    }
}

/// `multires_apply_smat` — uniform-scale branch (non-uniform takes
/// the same path in the C).
pub fn multires_apply_smat(mdisps: &mut [MDisps], scale: f32) {
    multires_apply_uniform_scale(mdisps, scale);
}

/// `multires_mdisp_corners`.
pub fn multires_mdisp_corners(s: &MDisps) -> i32 {
    let mut lvl = 13i32;
    while lvl > 0 {
        let side = (1 << (lvl - 1)) + 1;
        if s.totdisp % (side * side) == 0 {
            return s.totdisp / (side * side);
        }
        lvl -= 1;
    }
    0
}

/// `multires_topology_changed` — fill missing grids so every corner
/// has `totdisp` matching the first non-empty grid.
pub fn multires_topology_changed(mdisps: &mut [MDisps]) {
    let mut grid = 0;
    for md in mdisps.iter() {
        if md.totdisp != 0 {
            grid = md.totdisp;
            break;
        }
    }
    for md in mdisps.iter_mut() {
        if (md.totdisp == 0 || md.disps.is_empty()) && grid != 0 {
            md.totdisp = grid;
            md.disps = vec![[0.0; 3]; grid as usize];
        }
    }
}

/// `multires_ensure_external_read` — ensure each corner's mdisp has
/// `grid_area` disps at `top_level` (allocation part; the
/// `CustomData_external_*` calls are the caller's).
pub fn multires_ensure_external_read(mdisps: &mut [MDisps], top_level: i32) {
    let grid_size = grid_size_from_level(top_level);
    let grid_area = grid_size * grid_size;
    for md in mdisps.iter_mut() {
        if md.disps.len() != grid_area as usize {
            md.disps = vec![[0.0; 3]; grid_area as usize];
            md.totdisp = grid_area;
        }
    }
}

/// `multires_stitch_grids` — injected (`BKE_subdiv_ccg_average_stitch_faces`).
pub fn multires_stitch_grids(average_stitch: &mut dyn FnMut()) {
    average_stitch();
}
