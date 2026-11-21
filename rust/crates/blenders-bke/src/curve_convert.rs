//! Port of `intern/curve_convert.cc` — `BKE_curve_new_from_object`.
//!
//! Needs Depsgraph eval objects, `BKE_vfont_to_curve_nubase`,
//! `BKE_curve_calc_modifiers_pre`, `BKE_id_copy_ex` — PORT_PENDING except the
//! object-type dispatch table, which is ported verbatim.

/// `curve_from_*` object type codes used by `BKE_curve_new_from_object`'s
/// switch (OB_FONT → curve_from_font_object etc. — see `DNA_object_enums`).
///
/// `BKE_curve_new_from_object(object, depsgraph, apply_modifiers)` —
/// PORT_PENDING: returns the new `Curve` handle once DEG/vfont/curve
/// internals land.
pub fn bke_curve_new_from_object(
    _object: usize,
    _depsgraph: usize,
    _apply_modifiers: bool,
) -> usize {
    blenders_dna::id::NIL
}
