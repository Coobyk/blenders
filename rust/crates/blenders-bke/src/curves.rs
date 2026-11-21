//! Port of `intern/curves.cc` — `Curves` ID lifecycle, modifier
//! evaluation entry, surface-transforms math, edit hints.
//!
//! `BKE_id_*`/`Main` registration, blend read/write, depsgraph and
//! modifier machinery are injected; `Curves`/`CurvesGeometry` DNA
//! fields are handle-based.

use blenders_dna::id::NIL;
use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

/// `Curves` ID (subset — DNA `Curves` fields used by this file).
#[derive(Default)]
pub struct Curves {
    /// `ID id` — handle.
    pub id: usize,
    /// `CurvesGeometry geometry` — packed attribute storage handle.
    pub geometry: usize,
    pub point_num: i32,
    pub flag: i32,
    /// `Material **mat` — handle array.
    pub mat: Vec<usize>,
    pub totcol: i32,
    pub symmetry: i8,
    /// `char *surface_uv_map`.
    pub surface_uv_map: Option<String>,
    pub surface_collision_distance: f32,
    /// `Object *surface`.
    pub surface: usize,
    /// `int selection_domain` — `AttrDomain`.
    pub selection_domain: i32,
    /// `void *batch_cache` — non-null flag.
    pub has_batch_cache: bool,
}

/// `ATTR_POSITION`.
pub const ATTR_POSITION: &str = "position";

/// `curves_init_data`.
pub fn curves_init_data(curves: &mut Curves) {
    *curves = Curves::default();
}

/// `curves_copy_data` — shallow ID copy + geometry share
/// (`geometry.wrap()` copies share attribute data upstream).
pub fn curves_copy_data(src: &Curves, flag: i32) -> Curves {
    let _ = flag;
    Curves {
        id: NIL,
        geometry: src.geometry,
        point_num: src.point_num,
        flag: src.flag,
        mat: src.mat.clone(),
        totcol: src.totcol,
        symmetry: src.symmetry,
        surface_uv_map: src.surface_uv_map.clone(),
        surface_collision_distance: src.surface_collision_distance,
        surface: src.surface,
        selection_domain: src.selection_domain,
        has_batch_cache: false,
    }
}

/// `curves_free_data`.
pub fn curves_free_data(_curves: &mut Curves) {
    /* `BKE_animdata_free`, `BKE_material_free_eval` — caller-side. */
}

/// `BKE_curves_add`.
pub fn bke_curves_add(name: &str, new_id: &mut dyn FnMut(&str) -> usize) -> usize {
    new_id(name)
}

/// `BKE_curves_attribute_required`.
pub fn bke_curves_attribute_required(_curves: &Curves, name: &str) -> bool {
    name == ATTR_POSITION
}

/// `BKE_curves_material_remap` — delegates to
/// `BKE_material_attr_indices_remap` on the geometry attributes.
pub fn bke_curves_material_remap(
    curves_id: &Curves,
    remap: &[u32],
    attr_remap: &mut dyn FnMut(usize, &[u32]),
) {
    let _ = curves_id;
    attr_remap(curves_id.geometry, remap);
}

/// `BKE_curves_copy_for_eval` — `LIB_ID_COPY_LOCALIZE`.
pub fn bke_curves_copy_for_eval(curves_src: &Curves) -> Curves {
    curves_copy_data(curves_src, 0)
}

/// `ModifierEvalContext` — `{depsgraph, object, apply_flag}`.
pub struct ModifierEvalContext {
    pub depsgraph: usize,
    pub object: usize,
    pub apply_flag: i32,
}

/// `MOD_APPLY_*`.
pub const MOD_APPLY_USECACHE: i32 = 1;
pub const MOD_APPLY_RENDER: i32 = 2;
/// `eModifierMode_*`.
pub const E_MODIFIER_MODE_REALTIME: i32 = 4;
pub const E_MODIFIER_MODE_RENDER: i32 = 16;
pub const E_MODIFIER_MODE_EDITMODE: i32 = 1;
/// `OB_MODE_*`.
pub const OB_MODE_EDIT: i32 = 0x1;
pub const OB_MODE_SCULPT_CURVES: i32 = 0x100;
/// `DAG_EVAL_RENDER`.
pub const DAG_EVAL_RENDER: i32 = 2;

/// `curves_evaluate_modifiers` — iterate the (virtual) modifier
/// chain calling `modify_geometry_set`.
///
/// `virtual_modifierlist` = `BKE_modifiers_get_virtual_modifierlist`;
/// `mti` = `BKE_modifier_get_info(md->type)`;
/// `is_enabled` = `BKE_modifier_is_enabled`.
pub fn curves_evaluate_modifiers(
    use_render: bool,
    object_in_editmode: bool,
    modifiers: &[usize],
    modifier_type: &dyn Fn(usize) -> i32,
    is_enabled: &dyn Fn(usize) -> bool,
    modify_geometry_set: &mut dyn FnMut(usize),
    clear_errors: &mut dyn FnMut(),
) {
    /* Modifier evaluation modes. */
    let mut required_mode = if use_render {
        E_MODIFIER_MODE_RENDER
    } else {
        E_MODIFIER_MODE_REALTIME
    };
    if object_in_editmode {
        required_mode |= E_MODIFIER_MODE_EDITMODE;
    }
    let _apply_flag = if use_render {
        MOD_APPLY_RENDER
    } else {
        MOD_APPLY_USECACHE
    };

    clear_errors();

    /* Evaluate modifiers. */
    for &md in modifiers {
        let _mti = modifier_type(md);
        if !is_enabled(md) {
            continue;
        }
        /* `ScopedModifierTimer` — timing infra unported. */
        modify_geometry_set(md);
    }
}

/// `BKE_curves_data_update` — orchestration shell; depsgraph/mode
/// checks injected. `evaluate` performs the GeometrySet assembly
/// (`from_curves`, edit hints, `store_surface`, modifiers, assign).
pub fn bke_curves_data_update(
    free_derived_caches: &mut dyn FnMut(),
    evaluate: &mut dyn FnMut(),
) {
    /* Free any evaluated data and restore original data. */
    free_derived_caches();
    evaluate();
}

/// `BKE_curves_batch_cache_dirty_tag` /
/// `BKE_curves_batch_cache_free` — registered callbacks.
pub fn bke_curves_batch_cache_dirty_tag(
    curves: &Curves,
    mode: i32,
    dirty_tag_cb: &mut dyn FnMut(usize, i32),
) {
    if curves.has_batch_cache {
        dirty_tag_cb(curves.id, mode);
    }
}

/// `BKE_curves_batch_cache_free`.
pub fn bke_curves_batch_cache_free(curves: &Curves, free_cb: &mut dyn FnMut(usize)) {
    if curves.has_batch_cache {
        free_cb(curves.id);
    }
}

/// `curves_new_nomain(points_num, curves_num)` — new `Curves` with
/// resized geometry (`resize` = `CurvesGeometry::resize`).
pub fn curves_new_nomain(
    points_num: i32,
    curves_num: i32,
    geometry_resize: &mut dyn FnMut(i32, i32) -> usize,
) -> Curves {
    debug_assert!(points_num >= 0);
    debug_assert!(curves_num >= 0);
    Curves {
        geometry: geometry_resize(points_num, curves_num),
        point_num: points_num,
        ..Default::default()
    }
}

/// `curves_new_nomain_single` — one curve of `type` covering all
/// points.
pub fn curves_new_nomain_single(
    points_num: i32,
    curve_type: i8,
    geometry_resize: &mut dyn FnMut(i32, i32) -> usize,
    set_single_curve: &mut dyn FnMut(usize, i32, i8),
) -> Curves {
    let mut curves_id = curves_new_nomain(points_num, 1, geometry_resize);
    /* `offsets_for_write().last() = points_num;
     * fill_curve_types(type)`. */
    set_single_curve(curves_id.geometry, points_num, curve_type);
    curves_id.id = NIL;
    curves_id
}

/// `curves_copy_parameters`.
pub fn curves_copy_parameters(src: &Curves, dst: &mut Curves) {
    dst.flag = src.flag;
    dst.mat = src.mat.clone();
    dst.totcol = src.totcol;
    dst.symmetry = src.symmetry;
    dst.selection_domain = src.selection_domain;
    dst.surface = src.surface;
    dst.surface_uv_map = src.surface_uv_map.clone();
    dst.surface_collision_distance = src.surface_collision_distance;
}

/// `curves_store_surface_in_geometry_bundle` — adds
/// `surface_object`/`surface_uv_map_name` bundle entries when both
/// are present.
pub fn curves_store_surface_in_geometry_bundle(
    curves_id: &Curves,
    evaluated_surface: usize,
    bundle_add: &mut dyn FnMut(&str, usize, Option<String>),
) {
    if curves_id.surface == NIL {
        return;
    }
    let Some(uv_map) = curves_id.surface_uv_map.clone() else {
        return;
    };
    bundle_add("surface_object", evaluated_surface, None);
    bundle_add("surface_uv_map_name", NIL, Some(uv_map));
}

/// `CurvesSurfaceTransforms`.
#[derive(Clone)]
pub struct CurvesSurfaceTransforms {
    pub curves_to_world: Mat4,
    pub world_to_curves: Mat4,
    pub surface_to_world: Mat4,
    pub world_to_surface: Mat4,
    pub surface_to_curves: Mat4,
    pub curves_to_surface: Mat4,
    pub surface_to_curves_normal: Mat4,
}

impl CurvesSurfaceTransforms {
    /// `CurvesSurfaceTransforms(curves_ob, surface_ob)`.
    pub fn new(curves_ob_to_world: Mat4, surface_ob_to_world: Option<Mat4>) -> Self {
        let curves_to_world = curves_ob_to_world;
        let world_to_curves = curves_to_world.inverted().unwrap_or(Mat4::IDENTITY);

        let mut this = Self {
            curves_to_world,
            world_to_curves,
            surface_to_world: Mat4::IDENTITY,
            world_to_surface: Mat4::IDENTITY,
            surface_to_curves: Mat4::IDENTITY,
            curves_to_surface: Mat4::IDENTITY,
            surface_to_curves_normal: Mat4::IDENTITY,
        };
        if let Some(stw) = surface_ob_to_world {
            this.surface_to_world = stw;
            this.world_to_surface = stw.inverted().unwrap_or(Mat4::IDENTITY);
            this.surface_to_curves = this.world_to_curves * this.surface_to_world;
            this.curves_to_surface = this.world_to_surface * this.curves_to_world;
            this.surface_to_curves_normal = this
                .surface_to_curves
                .inverted()
                .map(|m| m.transposed())
                .unwrap_or(Mat4::IDENTITY);
        }
        this
    }
}

/// `CurvesEditHints` — sculpt-deformation hints kept through
/// modifier evaluation.
#[derive(Default)]
pub struct CurvesEditHints {
    /// `curves_id_orig` geometry point count.
    pub point_num: i32,
    /// Shared positions (implicit-sharing → `Vec` + shared flag).
    pub positions_data: Option<Vec<Vec3>>,
    /// `deform_mats` — `float4x4` per point.
    pub deform_mats: Option<Vec<Mat4>>,
}

impl CurvesEditHints {
    /// `is_valid` — optional arrays must match point count.
    pub fn is_valid(&self) -> bool {
        let point_num = self.point_num as usize;
        if let Some(p) = &self.positions_data {
            if p.len() != point_num {
                return false;
            }
        }
        if let Some(d) = &self.deform_mats {
            if d.len() != point_num {
                return false;
            }
        }
        true
    }

    /// `positions()`.
    pub fn positions(&self) -> Option<&[Vec3]> {
        self.positions_data.as_deref()
    }

    /// `positions_for_write()` — implicit sharing means copy-on-
    /// write; owned `Vec` here is always mutable.
    pub fn positions_for_write(&mut self) -> Option<&mut Vec<Vec3>> {
        self.positions_data.as_mut()
    }
}

/// `curves_normals_point_domain_calc` — field evaluator; the
/// `CurvesFieldContext`/`fn::Field` system is `PORT_PENDING`, so the
/// evaluator is injected (`evaluate` fills `normals`).
pub fn curves_normals_point_domain_calc(
    normals: &mut [Vec3],
    evaluate: &mut dyn FnMut(&mut [Vec3]),
) {
    evaluate(normals);
}
