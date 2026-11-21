//! Port of `intern/geometry_component_curves.cc` — `CurveComponent`
//! methods + curve normals/length field input.
//!
//! `CurveComponent` itself lives in `geometry_components`; this file
//! adds the cc's remaining methods plus the normals VArray and the
//! spline-length field input. `CurvesGeometry` attribute/evaluated
//! accessors are gathered into `CurvesAccess` (injected where the
//! attribute system is still `PORT_PENDING`).

use blenders_bli::index_mask::IndexMask;
use blenders_bli::offset_indices::OffsetIndices;
use blenders_dna::curves_types::{
    CurveType, CURVE_TYPE_BEZIER, CURVE_TYPE_CATMULL_ROM, CURVE_TYPE_NURBS, CURVE_TYPE_POLY,
    NORMAL_MODE_FREE, NORMAL_MODE_MINIMUM_TWIST, NORMAL_MODE_Z_UP,
};
use blenders_math::vector::Vec3;

use crate::attribute_legacy_convert::AttrDomain;
use crate::curve_poly::{calculate_normals_minimum, calculate_normals_z_up, calculate_tangents};
use crate::geometry_components::{CurveComponent, GeometryOwnershipType};

/// `adapt_domain` for Vec3 attribute data.
pub type AdaptDomainVec3<'a> = dyn FnMut(Vec<Vec3>, AttrDomain, AttrDomain) -> Option<Vec<Vec3>> + 'a;
/// `adapt_domain` for f32 attribute data.
pub type AdaptDomainF32<'a> = dyn FnMut(Vec<f32>, AttrDomain, AttrDomain) -> Option<Vec<f32>> + 'a;

/// `CurveComponent::copy` — `BKE_curves_copy_for_eval` injected.
pub fn curve_component_copy(
    c: &CurveComponent,
    copy_for_eval: &mut dyn FnMut(usize) -> usize,
) -> CurveComponent {
    let mut new_component = CurveComponent::new();
    if c.curves != blenders_dna::id::NIL {
        new_component.curves = copy_for_eval(c.curves);
        new_component.ownership = GeometryOwnershipType::Owned;
    }
    new_component
}

/// `owns_direct_data`.
pub fn curve_component_owns_direct_data(c: &CurveComponent) -> bool {
    c.ownership == GeometryOwnershipType::Owned
}

/// `ensure_owns_direct_data` — ReadOnly → `copy_for_eval`.
pub fn curve_component_ensure_owns_direct_data(
    c: &mut CurveComponent,
    copy_for_eval: &mut dyn FnMut(usize) -> usize,
) {
    if c.ownership != GeometryOwnershipType::Owned {
        if c.curves != blenders_dna::id::NIL {
            c.curves = copy_for_eval(c.curves);
        }
        c.ownership = GeometryOwnershipType::Owned;
    }
}

/// `CurveComponent::attributes` / `attributes_for_write` —
/// `AttributeAccessor` machinery not yet ported (`PORT_PENDING`:
/// `curves::get_attribute_accessor_functions`).

/// Inputs of `curve_normal_point_domain` for one `CurvesGeometry` —
/// replaces the C attribute lookups.
pub struct CurvesAccess<'a> {
    pub points_by_curve: OffsetIndices<'a>,
    pub evaluated_points_by_curve: OffsetIndices<'a>,
    pub types: &'a [i8],
    pub resolutions: &'a [i32],
    pub cyclic: &'a [bool],
    /// `custom_normal` point-domain VArray materialized
    /// (`lookup_or_default<float3>("custom_normal", Point, (0,0,1))`).
    pub custom_normals: &'a [Vec3],
    pub positions: &'a [Vec3],
    pub normal_modes: &'a [i8],
    pub evaluated_normals: &'a [Vec3],
    /// `bezier_evaluated_offsets_for_curve(i)` per curve.
    pub bezier_evaluated_offsets: &'a dyn Fn(i64) -> &'a [i32],
    pub points_num: usize,
    pub curves_num: usize,
}

/// `curve_normal_point_domain` — control-point-domain normals per
/// curve type (verbatim per-type dispatch).
pub fn curve_normal_point_domain(access: &CurvesAccess) -> Vec<Vec3> {
    let points_by_curve = access.points_by_curve;
    let evaluated_points_by_curve = access.evaluated_points_by_curve;
    let types = access.types;
    let resolutions = access.resolutions;
    let curves_cyclic = access.cyclic;
    let custom_normals = access.custom_normals;
    let positions = access.positions;
    let normal_modes = access.normal_modes;
    let evaluated_normals = access.evaluated_normals;

    let mut results = vec![Vec3::ZERO; access.points_num];

    /* `threading::parallel_for(curves.curves_range(), 128, …)` — serial. */
    let mut nurbs_tangents: Vec<Vec3> = Vec::new();

    for i_curve in 0..access.curves_num {
        let points = points_by_curve.group(i_curve);
        let evaluated_points = evaluated_points_by_curve.group(i_curve);
        let p_lo = points.start() as usize;
        let p_hi = points.one_after_last() as usize;
        let e_lo = evaluated_points.start() as usize;
        let e_hi = evaluated_points.one_after_last() as usize;

        match types[i_curve] as i32 {
            x if x == CURVE_TYPE_CATMULL_ROM as i32 => {
                let normals = &evaluated_normals[e_lo..e_hi];
                let resolution = resolutions[i_curve] as usize;
                for i in 0..points.size() as usize {
                    results[p_lo + i] = normals[resolution * i];
                }
            }
            x if x == CURVE_TYPE_POLY as i32 => {
                results[p_lo..p_hi].copy_from_slice(&evaluated_normals[e_lo..e_hi]);
            }
            x if x == CURVE_TYPE_BEZIER as i32 => {
                let normals = &evaluated_normals[e_lo..e_hi];
                results[p_lo] = normals[0];
                let offsets = (access.bezier_evaluated_offsets)(i_curve as i64);
                for i in 1..points.size() as usize {
                    results[p_lo + i] = normals[offsets[i] as usize];
                }
            }
            x if x == CURVE_TYPE_NURBS as i32 => {
                /* For NURBS curves there is no obvious correspondence
                 * between specific evaluated points and control
                 * points, so normals are determined by treating them
                 * as poly curves. */
                nurbs_tangents.clear();
                nurbs_tangents.resize(points.size() as usize, Vec3::ZERO);
                let cyclic = curves_cyclic[i_curve];
                let curve_positions = &positions[p_lo..p_hi];
                calculate_tangents(curve_positions, cyclic, &mut nurbs_tangents);
                match normal_modes[i_curve] {
                    NORMAL_MODE_Z_UP => {
                        calculate_normals_z_up(&nurbs_tangents, &mut results[p_lo..p_hi]);
                    }
                    NORMAL_MODE_MINIMUM_TWIST => {
                        calculate_normals_minimum(&nurbs_tangents, cyclic, &mut results[p_lo..p_hi]);
                    }
                    /* NORMAL_MODE_FREE → copy custom normals. */
                    _ => {
                        debug_assert_eq!(normal_modes[i_curve], NORMAL_MODE_FREE);
                        results[p_lo..p_hi].copy_from_slice(&custom_normals[p_lo..p_hi]);
                    }
                }
            }
            _ => {}
        }
    }
    results
}

/// `curve_normals_varray` — normals per `domain`.
///
/// `adapt_domain` is injected (`PORT_PENDING` attribute machinery);
/// `is_single_type`/`evaluated_normals` come via `access`.
pub fn curve_normals_varray(
    access: &CurvesAccess,
    domain: AttrDomain,
    is_single_type_poly: bool,
    adapt_domain: &mut AdaptDomainVec3<'_>,
) -> Option<Vec<Vec3>> {
    if is_single_type_poly {
        return adapt_domain(
            access.evaluated_normals.to_vec(),
            AttrDomain::Point,
            domain,
        );
    }

    let normals = curve_normal_point_domain(access);

    if domain == AttrDomain::Point {
        return Some(normals);
    }
    if domain == AttrDomain::Spline {
        return adapt_domain(normals, AttrDomain::Point, AttrDomain::Spline);
    }
    None
}

/// `construct_curve_length_gvarray` — per-curve evaluated lengths
/// (`from_func` → materialized Vec).
///
/// `evaluated_length_total_for_curve` is injected (evaluated-length
/// cache, `PORT_PENDING`).
pub fn construct_curve_length_varray(
    access: &CurvesAccess,
    cyclic: &[bool],
    domain: AttrDomain,
    evaluated_length_total_for_curve: &dyn Fn(i64, bool) -> f32,
    adapt_domain: &mut AdaptDomainF32<'_>,
) -> Option<Vec<f32>> {
    /* `curves.ensure_evaluated_lengths()` — caller-side. */
    let lengths: Vec<f32> = (0..access.curves_num as i64)
        .map(|i| evaluated_length_total_for_curve(i, cyclic[i as usize]))
        .collect();

    if domain == AttrDomain::Spline {
        return Some(lengths);
    }
    if domain == AttrDomain::Point {
        return adapt_domain(lengths, AttrDomain::Spline, AttrDomain::Point);
    }
    None
}

/// `CurveLengthFieldInput` — "Spline Length node" input on curves.
///
/// Field-system types (`FieldInput`, `GVArray`, `FieldHashDeep`) are
/// `PORT_PENDING`; the domain logic is ported.
pub struct CurveLengthFieldInput;

impl CurveLengthFieldInput {
    /// `get_varray_for_context`.
    pub fn get_varray_for_context(
        &self,
        access: &CurvesAccess,
        cyclic: &[bool],
        domain: AttrDomain,
        _mask: &IndexMask,
        evaluated_length_total_for_curve: &dyn Fn(i64, bool) -> f32,
        adapt_domain: &mut AdaptDomainF32<'_>,
    ) -> Option<Vec<f32>> {
        construct_curve_length_varray(
            access,
            cyclic,
            domain,
            evaluated_length_total_for_curve,
            adapt_domain,
        )
    }

    /// `hash_unique` — single constant id byte.
    pub fn hash_unique(&self) -> i8 {
        0
    }

    /// `preferred_domain`.
    pub fn preferred_domain(&self) -> Option<AttrDomain> {
        Some(AttrDomain::Spline)
    }
}
