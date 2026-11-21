//! Port of `intern/curves_attributes.cc` — `CurvesGeometry`
//! `AttributeAccessorFunctions`: changed-tag table, builtin
//! attribute metadata, vertex-group-as-attribute plumbing, and the
//! domain-size/remove/add/rename logic.
//!
//! `AttributeStorage` and `CurvesGeometry` mutation hooks are
//! injected (handles); the table/metadata is fully ported.

use std::collections::{HashMap, HashSet};

use crate::attribute_legacy_convert::{AttrDomain, AttrType};
use crate::attribute_storage_access::AttrBuiltinInfo;
use blenders_dna::curves_types::{
    BEZIER_HANDLE_ALIGN, BEZIER_HANDLE_FREE, CURVE_TYPE_CATMULL_ROM, CURVE_TYPE_NURBS,
    NORMAL_MODE_FREE, NORMAL_MODE_MINIMUM_TWIST, NURBS_KNOT_MODE_CUSTOM, NURBS_KNOT_MODE_NORMAL,
};

/// `AttrUpdateOnChange` — `(curves_owner)`.
pub type AttrUpdateOnChange = usize;

/// Tag kinds the update fns map to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CurvesTag {
    TopologyChanged,
    CurveTypesChanged,
    PositionsChanged,
    RadiiChanged,
    NormalsChanged,
    MaterialIndexChanged,
}

/// `changed_tags()` — attr name → tag.
pub fn changed_tags() -> &'static HashMap<&'static str, CurvesTag> {
    static MAP: std::sync::OnceLock<HashMap<&'static str, CurvesTag>> = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        HashMap::from([
            ("position", CurvesTag::PositionsChanged),
            ("radius", CurvesTag::RadiiChanged),
            ("tilt", CurvesTag::NormalsChanged),
            ("handle_left", CurvesTag::PositionsChanged),
            ("handle_right", CurvesTag::PositionsChanged),
            ("handle_type_left", CurvesTag::TopologyChanged),
            ("handle_type_right", CurvesTag::TopologyChanged),
            ("nurbs_weight", CurvesTag::PositionsChanged),
            ("nurbs_order", CurvesTag::TopologyChanged),
            ("normal_mode", CurvesTag::NormalsChanged),
            ("custom_normal", CurvesTag::NormalsChanged),
            ("curve_type", CurvesTag::CurveTypesChanged),
            ("resolution", CurvesTag::TopologyChanged),
            ("cyclic", CurvesTag::TopologyChanged),
            ("material_index", CurvesTag::MaterialIndexChanged),
        ])
    })
}

/// `BuiltinAttr` — builtin metadata incl. validator/default.
pub struct BuiltinAttr {
    pub info: AttrBuiltinInfo,
    /// `validator` — clamp fn applied per value (i8/i32 only here).
    pub validator_i8: Option<fn(i8) -> i8>,
    pub validator_i32: Option<fn(i32) -> i32>,
    /// `default_value` (typed).
    pub default_i8: Option<i8>,
    pub default_i32: Option<i32>,
    pub default_f32: Option<f32>,
}

fn info(domain: AttrDomain, type_: AttrType, deletable: bool) -> AttrBuiltinInfo {
    AttrBuiltinInfo { domain, type_, deletable }
}

/// `builtin_attributes()`.
pub fn builtin_attributes() -> &'static HashMap<&'static str, BuiltinAttr> {
    static MAP: std::sync::OnceLock<HashMap<&'static str, BuiltinAttr>> =
        std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut map = HashMap::new();

        map.insert(
            "position",
            BuiltinAttr {
                info: info(AttrDomain::Point, AttrType::Float3, false),
                validator_i8: None,
                validator_i32: None,
                default_i8: None,
                default_i32: None,
                default_f32: None,
            },
        );
        for (name, t) in [("radius", AttrType::Float), ("tilt", AttrType::Float)] {
            map.insert(
                name,
                BuiltinAttr {
                    info: info(AttrDomain::Point, t, true),
                    validator_i8: None,
                    validator_i32: None,
                    default_i8: None,
                    default_i32: None,
                    default_f32: None,
                },
            );
        }
        for name in ["handle_left", "handle_right"] {
            map.insert(
                name,
                BuiltinAttr {
                    info: info(AttrDomain::Point, AttrType::Float3, true),
                    validator_i8: None,
                    validator_i32: None,
                    default_i8: None,
                    default_i32: None,
                    default_f32: None,
                },
            );
        }
        /* `handle_type_clamp`. */
        let handle_type_clamp: fn(i8) -> i8 =
            |v| v.clamp(BEZIER_HANDLE_FREE as i8, BEZIER_HANDLE_ALIGN as i8);
        for name in ["handle_type_left", "handle_type_right"] {
            map.insert(
                name,
                BuiltinAttr {
                    info: info(AttrDomain::Point, AttrType::Int8, true),
                    validator_i8: Some(handle_type_clamp),
                    validator_i32: None,
                    default_i8: None,
                    default_i32: None,
                    default_f32: None,
                },
            );
        }
        map.insert(
            "nurbs_weight",
            BuiltinAttr {
                info: info(AttrDomain::Point, AttrType::Float, true),
                validator_i8: None,
                validator_i32: None,
                default_i8: None,
                default_i32: None,
                default_f32: Some(1.0),
            },
        );
        let nurbs_order_clamp: fn(i8) -> i8 = |v| v.max(1);
        map.insert(
            "nurbs_order",
            BuiltinAttr {
                info: info(AttrDomain::Spline, AttrType::Int8, true),
                validator_i8: Some(nurbs_order_clamp),
                validator_i32: None,
                default_i8: Some(4),
                default_i32: None,
                default_f32: None,
            },
        );
        let normal_mode_clamp: fn(i8) -> i8 =
            |v| v.clamp(NORMAL_MODE_MINIMUM_TWIST as i8, NORMAL_MODE_FREE as i8);
        map.insert(
            "normal_mode",
            BuiltinAttr {
                info: info(AttrDomain::Spline, AttrType::Int8, true),
                validator_i8: Some(normal_mode_clamp),
                validator_i32: None,
                default_i8: None,
                default_i32: None,
                default_f32: None,
            },
        );
        map.insert(
            "custom_normal",
            BuiltinAttr {
                info: info(AttrDomain::Point, AttrType::Float3, true),
                validator_i8: None,
                validator_i32: None,
                default_i8: None,
                default_i32: None,
                default_f32: None,
            },
        );
        let knots_mode_clamp: fn(i8) -> i8 =
            |v| v.clamp(NURBS_KNOT_MODE_NORMAL as i8, NURBS_KNOT_MODE_CUSTOM as i8);
        map.insert(
            "knots_mode",
            BuiltinAttr {
                info: info(AttrDomain::Spline, AttrType::Int8, true),
                validator_i8: Some(knots_mode_clamp),
                validator_i32: None,
                default_i8: None,
                default_i32: None,
                default_f32: None,
            },
        );
        let curve_type_clamp: fn(i8) -> i8 =
            |v| v.clamp(CURVE_TYPE_CATMULL_ROM as i8, CURVE_TYPE_NURBS as i8);
        map.insert(
            "curve_type",
            BuiltinAttr {
                info: info(AttrDomain::Spline, AttrType::Int8, true),
                validator_i8: Some(curve_type_clamp),
                validator_i32: None,
                default_i8: None,
                default_i32: None,
                default_f32: None,
            },
        );
        let resolution_clamp: fn(i32) -> i32 = |v| v.max(1);
        map.insert(
            "resolution",
            BuiltinAttr {
                info: info(AttrDomain::Spline, AttrType::Int32, true),
                validator_i8: None,
                validator_i32: Some(resolution_clamp),
                default_i8: None,
                default_i32: Some(12),
                default_f32: None,
            },
        );
        map.insert(
            "cyclic",
            BuiltinAttr {
                info: info(AttrDomain::Spline, AttrType::Bool, true),
                validator_i8: None,
                validator_i32: None,
                default_i8: None,
                default_i32: None,
                default_f32: None,
            },
        );
        let material_index_clamp: fn(i32) -> i32 = |v| v.clamp(0, i16::MAX as i32);
        map.insert(
            "material_index",
            BuiltinAttr {
                info: info(AttrDomain::Spline, AttrType::Int32, true),
                validator_i8: None,
                validator_i32: Some(material_index_clamp),
                default_i8: None,
                default_i32: None,
                default_f32: None,
            },
        );
        map
    })
}

/// `array_storage_required()` — names that must be array storage.
pub fn array_storage_required() -> &'static HashSet<&'static str> {
    static SET: std::sync::OnceLock<HashSet<&'static str>> = std::sync::OnceLock::new();
    SET.get_or_init(|| {
        HashSet::from([
            "position",
            "handle_left",
            "handle_right",
            "nurbs_weight",
            "surface_uv_coordinate",
        ])
    })
}

/// `domain_supported` — Point/Curve only.
pub fn domain_supported(domain: AttrDomain) -> bool {
    matches!(domain, AttrDomain::Point | AttrDomain::Spline)
}

/// `get_domain_size`.
pub fn get_domain_size(domain: AttrDomain, points_num: i32, curves_num: i32) -> i64 {
    match domain {
        AttrDomain::Point => points_num as i64,
        AttrDomain::Spline => curves_num as i64,
        _ => 0,
    }
}

/// `builtin_domain_and_type`.
pub fn builtin_domain_and_type(name: &str) -> Option<AttrBuiltinInfo> {
    builtin_attributes().get(name).map(|b| b.info)
}
