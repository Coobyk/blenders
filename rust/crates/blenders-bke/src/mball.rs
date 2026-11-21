//! Port of `intern/mball.cc` — `MetaElem` creation, display radius,
//! basis/group name logic, selection helpers, bounds/centers and
//! transform/translate.
//!
//! `mb->elems` / `mb->editelems` become `&[MetaElem]`/`Vec<MetaElem>`
//! slices; Object lookups for `is_basis`/`same_group`/`basis_find`
//! work on injected name/type queries. `SELECT` is `MB_SELECT`.

use blenders_bli::string_utils::string_split_name_number;
use blenders_dna::meta_types::{
    mb_type_size_squared, MetaElem, MB_BALL, MB_CUBE, MB_ELLIPSOID, MB_PLANE,
    MB_SCALE_RAD, MB_SELECT, MB_TUBE,
};
use blenders_math::matrix_c::{mat4_to_scale, mul_m4_v3};
use blenders_math::{Mat3, Mat4, Quaternion, Vec3};

/// `M_SQRT2`.
const M_SQRT2: f32 = std::f32::consts::SQRT_2;

/// `BKE_mball_element_add` — new element of `type` appended to `elems`.
pub fn bke_mball_element_add(elems: &mut Vec<MetaElem>, ty: i16) -> usize {
    let mut ml = MetaElem {
        quat: [Quaternion::IDENTITY.w, Quaternion::IDENTITY.x,
               Quaternion::IDENTITY.y, Quaternion::IDENTITY.z],
        rad: 2.0,
        s: 2.0,
        flag: MB_SCALE_RAD,
        ..Default::default()
    };
    match ty {
        MB_BALL => {
            ml.type_ = MB_BALL;
            ml.expx = 1.0;
            ml.expy = 1.0;
            ml.expz = 1.0;
        }
        MB_TUBE => {
            ml.type_ = MB_TUBE;
            ml.expx = 1.0;
            ml.expy = 1.0;
            ml.expz = 1.0;
        }
        MB_PLANE => {
            ml.type_ = MB_PLANE;
            ml.expx = 1.0;
            ml.expy = 1.0;
            ml.expz = 1.0;
        }
        MB_ELLIPSOID => {
            ml.type_ = MB_ELLIPSOID;
            ml.expx = 1.2;
            ml.expy = 0.8;
            ml.expz = 1.0;
        }
        MB_CUBE => {
            ml.type_ = MB_CUBE;
            ml.expx = 1.0;
            ml.expy = 1.0;
            ml.expz = 1.0;
        }
        _ => {}
    }
    elems.push(ml);
    elems.len() - 1
}

/// `BKE_mball_element_display_radius_calc_with_stiffness` —
/// `(radius, stiffness)` incl. cube corner offset.
pub fn bke_mball_element_display_radius_calc_with_stiffness(ml: &MetaElem) -> [f32; 2] {
    let mut radius_stiffness = [ml.rad, ml.s];
    if ml.type_ == MB_CUBE {
        let offset = ml.expx.min(ml.expy).min(ml.expz) * M_SQRT2;
        radius_stiffness[0] += offset;
        radius_stiffness[1] += offset;
    }
    radius_stiffness
}

/// `BKE_mball_element_display_radius_calc`.
pub fn bke_mball_element_display_radius_calc(ml: &MetaElem) -> f32 {
    let mut radius = ml.rad;
    if ml.type_ == MB_CUBE {
        let offset = ml.expx.min(ml.expy).min(ml.expz) * M_SQRT2;
        radius += offset;
    }
    radius
}

/// `BKE_mball_is_basis` — object name doesn't end in a digit.
pub fn bke_mball_is_basis(id_name: &str) -> bool {
    !id_name
        .chars()
        .last()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
}

/// `BKE_mball_is_same_group` — same letter and base name (number
/// after '.' ignored). `id_name` includes the 2-char ID prefix.
pub fn bke_mball_is_same_group(name1: &str, name2: &str) -> bool {
    if name1.chars().nth(2) != name2.chars().nth(2) {
        return false;
    }
    let (basis1, _) = string_split_name_number(&name1[2..], '.');
    let (basis2, _) = string_split_name_number(&name2[2..], '.');
    basis1 == basis2
}

/// `BKE_mball_is_basis_for`.
pub fn bke_mball_is_basis_for(name1: &str, name2: &str) -> bool {
    bke_mball_is_same_group(name1, name2) && bke_mball_is_basis(name1)
}

/// `BKE_mball_is_any_selected` — over `editelems`.
pub fn bke_mball_is_any_selected(editelems: &[MetaElem]) -> bool {
    editelems.iter().any(|ml| ml.flag & MB_SELECT != 0)
}

/// `BKE_mball_is_any_unselected`.
pub fn bke_mball_is_any_unselected(editelems: &[MetaElem]) -> bool {
    editelems.iter().any(|ml| ml.flag & MB_SELECT == 0)
}

/// `BKE_mball_minmax_ex` — flag-filtered bounds, scaled by `obmat`'s
/// scale; `flag == 0` selects all elements.
pub fn bke_mball_minmax_ex(
    elems: &[MetaElem],
    obmat: Option<Mat4>,
    flag: i16,
) -> Option<([f32; 3], [f32; 3])> {
    let scale = obmat.map(mat4_to_scale).unwrap_or(1.0);
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let mut changed = false;

    for ml in elems {
        if ml.flag & flag == flag {
            let scale_mb = ml.rad * 0.5 * scale;
            let centroid = match obmat {
                Some(m) => m.transform_point(Vec3::new(ml.x, ml.y, ml.z)),
                None => Vec3::new(ml.x, ml.y, ml.z),
            };
            for i in [-1.0f32, 1.0] {
                let vec = centroid + Vec3::splat(scale_mb * i);
                blenders_math::vector::minmax_v3(&mut min, &mut max, vec);
            }
            changed = true;
        }
    }
    changed.then_some((min.into(), max.into()))
}

/// `BKE_mball_minmax` — element center bounds.
pub fn bke_mball_minmax(elems: &[MetaElem]) -> Option<([f32; 3], [f32; 3])> {
    if elems.is_empty() {
        return None;
    }
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for ml in elems {
        blenders_math::vector::minmax_v3(&mut min, &mut max, Vec3::new(ml.x, ml.y, ml.z));
    }
    Some((min.into(), max.into()))
}

/// `BKE_mball_center_median`.
pub fn bke_mball_center_median(elems: &[MetaElem]) -> Option<[f32; 3]> {
    if elems.is_empty() {
        return None;
    }
    let mut cent = Vec3::ZERO;
    for ml in elems {
        cent += Vec3::new(ml.x, ml.y, ml.z);
    }
    Some((cent / elems.len() as f32).into())
}

/// `BKE_mball_center_bounds`.
pub fn bke_mball_center_bounds(elems: &[MetaElem]) -> Option<[f32; 3]> {
    let (min, max) = bke_mball_minmax(elems)?;
    Some(((Vec3::from(min) + Vec3::from(max)) / 2.0).into())
}

/// `BKE_mball_transform` — transform element positions + rotate
/// quats; `do_props` also scales `rad`/`exp*`.
pub fn bke_mball_transform(elems: &mut [MetaElem], mat: Mat4, do_props: bool) {
    let scale = mat4_to_scale(mat);
    let scale_sqrt = scale.sqrt();
    let quat = Mat3::from(mat).to_quat_scale().0;

    for ml in elems.iter_mut() {
        let mut v = Vec3::new(ml.x, ml.y, ml.z);
        mul_m4_v3(mat, &mut v);
        ml.x = v.x;
        ml.y = v.y;
        ml.z = v.z;
        let q = Quaternion::new(ml.quat[0], ml.quat[1], ml.quat[2], ml.quat[3]);
        let q = quat * q;
        ml.quat = [q.w, q.x, q.y, q.z];

        if do_props {
            ml.rad *= scale;
            /* hrmf, probably elems shouldn't be treating scale
             * differently - campbell */
            if !mb_type_size_squared(ml.type_) {
                ml.expx *= scale;
                ml.expy *= scale;
                ml.expz *= scale;
            } else {
                ml.expx *= scale_sqrt;
                ml.expy *= scale_sqrt;
                ml.expz *= scale_sqrt;
            }
        }
    }
}

/// `BKE_mball_translate`.
pub fn bke_mball_translate(elems: &mut [MetaElem], offset: [f32; 3]) {
    for ml in elems.iter_mut() {
        ml.x += offset[0];
        ml.y += offset[1];
        ml.z += offset[2];
    }
}

/// `BKE_mball_select_count` — over `editelems`.
pub fn bke_mball_select_count(editelems: &[MetaElem]) -> usize {
    editelems.iter().filter(|ml| ml.flag & MB_SELECT != 0).count()
}

/// `BKE_mball_select_all` — returns whether anything changed.
pub fn bke_mball_select_all(editelems: &mut [MetaElem]) -> bool {
    let mut changed = false;
    for ml in editelems.iter_mut() {
        if ml.flag & MB_SELECT == 0 {
            ml.flag |= MB_SELECT;
            changed = true;
        }
    }
    changed
}

/// `BKE_mball_deselect_all`.
pub fn bke_mball_deselect_all(editelems: &mut [MetaElem]) -> bool {
    let mut changed = false;
    for ml in editelems.iter_mut() {
        if ml.flag & MB_SELECT != 0 {
            ml.flag &= !MB_SELECT;
            changed = true;
        }
    }
    changed
}

/// `BKE_mball_select_swap`.
pub fn bke_mball_select_swap(editelems: &mut [MetaElem]) -> bool {
    for ml in editelems.iter_mut() {
        ml.flag ^= MB_SELECT;
    }
    !editelems.is_empty()
}
