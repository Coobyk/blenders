//! Port of `intern/pointcloud.cc` — `PointCloud` ID-type plumbing,
//! `PointCloudRuntime` caches, bounds, resize/copy-parameter helpers,
//! modifier-eval skeleton, and batch-cache callbacks.
//!
//! `AttributeStorage` stays a `usize` handle — attribute read/write
//! goes through injected accessors; blend read/write, animdata and
//! the depsgraph/modifier pipeline are injected or `PORT_PENDING`.

use blenders_dna::pointcloud_types::{PointCloud, PointCloudType};
use blenders_math::vector::Vec3;
use std::sync::Mutex;

/// `ATTR_POSITION`.
pub const ATTR_POSITION: &str = "position";

/// `Bounds<float3>`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl Bounds {
    /// `Bounds::pad` — grow by `radius` on all sides.
    pub fn pad(&mut self, radius: f32) {
        self.min -= Vec3::splat(radius);
        self.max += Vec3::splat(radius);
    }
}

/// `bounds::min_max`.
pub fn bounds_min_max(positions: &[Vec3]) -> Option<Bounds> {
    if positions.is_empty() {
        return None;
    }
    let mut b = Bounds {
        min: positions[0],
        max: positions[0],
    };
    for &p in &positions[1..] {
        b.min = b.min.min(p);
        b.max = b.max.max(p);
    }
    Some(b)
}

/// `bounds::min_max_with_radii`.
pub fn bounds_min_max_with_radii(positions: &[Vec3], radii: &[f32]) -> Option<Bounds> {
    if positions.is_empty() {
        return None;
    }
    let mut b = Bounds {
        min: positions[0] - Vec3::splat(radii.get(0).copied().unwrap_or(0.0)),
        max: positions[0] + Vec3::splat(radii.get(0).copied().unwrap_or(0.0)),
    };
    for (i, &p) in positions.iter().enumerate().skip(1) {
        let r = radii.get(i).copied().unwrap_or(0.0);
        b.min = b.min.min(p - Vec3::splat(r));
        b.max = b.max.max(p + Vec3::splat(r));
    }
    Some(b)
}

/// `SharedCache`-lite for bounds (see `mesh_runtime::SharedCache`).
pub struct OptCache<T: Clone> {
    data: Mutex<Option<T>>,
}

impl<T: Clone> Default for OptCache<T> {
    fn default() -> Self {
        Self { data: Mutex::new(None) }
    }
}

impl<T: Clone> OptCache<T> {
    pub fn ensure(&self, compute: impl FnOnce() -> Option<T>) -> Option<T> {
        let mut g = self.data.lock().unwrap();
        if g.is_none() {
            *g = compute();
        }
        g.clone()
    }
    pub fn tag_dirty(&self) {
        *self.data.lock().unwrap() = None;
    }
    pub fn is_cached(&self) -> bool {
        self.data.lock().unwrap().is_some()
    }
}

/// `bke::PointCloudRuntime`.
#[derive(Default)]
pub struct PointCloudRuntime {
    pub bounds_cache: OptCache<Bounds>,
    pub bounds_with_radius_cache: OptCache<Bounds>,
    /// `SharedBVHCache bvh_cache` handle.
    pub bvh_cache: Mutex<usize>,
    /// `bake::BakeMaterialsList` handle.
    pub bake_materials: Mutex<usize>,
}

impl PointCloudRuntime {
    /// `PointCloud::tag_positions_changed`.
    pub fn tag_positions_changed(&self) {
        self.bounds_cache.tag_dirty();
        self.bounds_with_radius_cache.tag_dirty();
        *self.bvh_cache.lock().unwrap() = 0;
    }
    /// `PointCloud::tag_radii_changed`.
    pub fn tag_radii_changed(&self) {
        self.bounds_with_radius_cache.tag_dirty();
    }
}

/// `PointCloud::bounds_min_max(use_radius)`.
///
/// `positions`/`radius` come from the attribute storage (`position`
/// / `radius` attrs, `radius` single = `Some(f32)` when uniform).
pub fn pointcloud_bounds_min_max(
    runtime: &PointCloudRuntime,
    totpoint: i32,
    positions: &[Vec3],
    use_radius: bool,
    radius_single: Option<f32>,
    radii: &[f32],
) -> Option<Bounds> {
    if totpoint == 0 {
        return None;
    }
    if use_radius {
        runtime.bounds_with_radius_cache.ensure(|| {
            if let Some(r) = radius_single {
                let mut b = bounds_min_max(positions)?;
                b.pad(r);
                return Some(b);
            }
            bounds_min_max_with_radii(positions, radii)
        })
    } else {
        runtime
            .bounds_cache
            .ensure(|| bounds_min_max(positions))
    }
}

/// `MAXMAT` (DNA_material_types.h).
pub const MAXMAT: i32 = 32767;

/// `PointCloud::material_index_max` — clamped `[0, MAXMAT]` max of
/// the `material_index` point attr.
pub fn pointcloud_material_index_max(totpoint: i32, material_indices: &[i32]) -> Option<i32> {
    if totpoint == 0 {
        return None;
    }
    material_indices
        .iter()
        .copied()
        .max()
        .map(|v| v.clamp(0, MAXMAT))
}

/// `BKE_pointcloud_attribute_required`.
pub fn bke_pointcloud_attribute_required(name: &str) -> bool {
    name == ATTR_POSITION
}

/// `pointcloud_copy_parameters` — `type`, `flag` and the `mat` ref
/// list (`totcol` follows `mat.len()`).
pub fn pointcloud_copy_parameters(src: &PointCloud, dst: &mut PointCloud) {
    dst.type_ = src.type_;
    dst.flag = src.flag;
    dst.mat = src.mat.clone();
    dst.totcol = src.totcol;
}

/// `pointcloud_resize` — set `totpoint`, ensure `position` exists
/// when growing from 0, resize the attribute storage and default-fill
/// the new range (both injected).
pub fn pointcloud_resize(
    pointcloud: &mut PointCloud,
    size: i32,
    ensure_position_attr: &mut dyn FnMut(&mut PointCloud),
    resize_domain: &mut dyn FnMut(&mut PointCloud, i32),
    fill_new_range: &mut dyn FnMut(&mut PointCloud, i32, i32),
) {
    assert!(size > 0);
    let old_totpoint = pointcloud.totpoint;
    if size == old_totpoint {
        return;
    }
    pointcloud.totpoint = size;
    if old_totpoint == 0 {
        /* If there were no points before, ensure the position
         * attribute exists. */
        ensure_position_attr(pointcloud);
    }
    resize_domain(pointcloud, size);
    if size > old_totpoint {
        /* Initialize new points. */
        fill_new_range(pointcloud, old_totpoint, size);
    }
}

/// `pointcloud_evaluate_modifiers` skeleton — modifier iteration
/// fully injected (`BKE_modifiers_get_virtual_modifierlist`,
/// `mti->modify_geometry_set`).
///
/// Returns the number of modifiers that ran `modify_geometry_set`.
pub fn pointcloud_evaluate_modifiers(
    use_render: bool,
    for_each_modifier: &mut dyn FnMut(
        /* md */ usize,
        /* required_mode */ i32,
        /* apply_flag */ i32,
    ),
) -> usize {
    /* Modifier evaluation modes. */
    const E_MODIFIER_MODE_RENDER: i32 = 1;
    const E_MODIFIER_MODE_REALTIME: i32 = 2;
    const MOD_APPLY_RENDER: i32 = 1;
    const MOD_APPLY_USECACHE: i32 = 2;
    let required_mode = if use_render {
        E_MODIFIER_MODE_RENDER
    } else {
        E_MODIFIER_MODE_REALTIME
    };
    let apply_flag = if use_render {
        MOD_APPLY_RENDER
    } else {
        MOD_APPLY_USECACHE
    };
    for_each_modifier(0, required_mode, apply_flag);
    0
}

/// `BKE_pointcloud_batch_cache_dirty_tag` — call the registered
/// draw callbacks when the caches exist (`draw` registers them).
pub fn bke_pointcloud_batch_cache_dirty_tag(
    pointcloud: &PointCloud,
    mode: i32,
    pointcloud_dirty_tag_cb: Option<&dyn Fn(usize, i32)>,
    gsplat_dirty_tag_cb: Option<&dyn Fn(usize, i32)>,
) {
    if pointcloud.pointcloud_batch_cache != 0 {
        if let Some(cb) = pointcloud_dirty_tag_cb {
            cb(pointcloud.pointcloud_batch_cache, mode);
        }
    }
    if pointcloud.gsplat_batch_cache != 0 {
        if let Some(cb) = gsplat_dirty_tag_cb {
            cb(pointcloud.gsplat_batch_cache, mode);
        }
    }
}

/// `BKE_pointcloud_batch_cache_free`.
pub fn bke_pointcloud_batch_cache_free(
    pointcloud: &PointCloud,
    pointcloud_free_cb: Option<&dyn Fn(usize)>,
    gsplat_free_cb: Option<&dyn Fn(usize)>,
) {
    if pointcloud.pointcloud_batch_cache != 0 {
        if let Some(cb) = pointcloud_free_cb {
            cb(pointcloud.pointcloud_batch_cache);
        }
    }
    if pointcloud.gsplat_batch_cache != 0 {
        if let Some(cb) = gsplat_free_cb {
            cb(pointcloud.gsplat_batch_cache);
        }
    }
}

/// `pointcloud_new_no_attributes`.
pub fn pointcloud_new_no_attributes(type_: PointCloudType, totpoint: i32) -> PointCloud {
    let mut pointcloud = PointCloud::default();
    pointcloud.type_ = type_;
    pointcloud.totpoint = totpoint;
    pointcloud
}
