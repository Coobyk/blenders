//! Port of `gizmo/intern/wm_gizmo_target_props.cc` —
//! gizmo target-property defs + rna/func get/set.

/// `wmGizmoPropertyType` verbatim.
pub struct WmGizmoPropertyType {
    /// `idname`.
    pub idname: String,
    /// `data_type`.
    pub data_type: i32,
    /// `array_length`.
    pub array_length: i32,
    /// `index_in_type`.
    pub index_in_type: i32,
}

/// `wmGizmoProperty` verbatim.
pub struct WmGizmoProperty {
    /// `type` (index into defs).
    pub type_index: usize,
    /// `prop` (handle).
    pub prop: usize,
    /// `index` (-1 = scalar).
    pub index: i32,
    /// `custom_func` fns present.
    pub has_value_get_fn: bool,
    pub has_value_set_fn: bool,
}

/// `WM_gizmo_target_property_find` (verbatim —
/// findstringindex → indexed prop).
pub fn wm_gizmo_target_property_find<'a>(
    defs: &[WmGizmoPropertyType],
    props: &'a [WmGizmoProperty],
    idname: &str,
) -> Option<&'a WmGizmoProperty> {
    let index = defs.iter().position(|d| d.idname == idname)?;
    props.get(index)
}

/// `WM_gizmo_target_property_is_valid` (verbatim — rna
/// prop or get+set fns).
pub fn wm_gizmo_target_property_is_valid(gz_prop: &WmGizmoProperty) -> bool {
    gz_prop.prop != 0 || (gz_prop.has_value_get_fn && gz_prop.has_value_set_fn)
}

/// `WM_gizmo_target_property_is_valid_any` (verbatim).
pub fn wm_gizmo_target_property_is_valid_any(props: &[WmGizmoProperty]) -> bool {
    props.iter().any(wm_gizmo_target_property_is_valid)
}

/// `WM_gizmotype_target_property_find` (verbatim).
pub fn wm_gizmotype_target_property_find<'a>(
    defs: &'a [WmGizmoPropertyType],
    idname: &str,
) -> Option<&'a WmGizmoPropertyType> {
    defs.iter().find(|d| d.idname == idname)
}

/// `WM_gizmotype_target_property_def` (verbatim —
/// append with index_in_type).
pub fn wm_gizmotype_target_property_def(
    defs: &mut Vec<WmGizmoPropertyType>,
    idname: &str,
    data_type: i32,
    array_length: i32,
) {
    assert!(wm_gizmotype_target_property_find(defs, idname).is_none());
    let index_in_type = defs.len() as i32;
    defs.push(WmGizmoPropertyType {
        idname: idname.to_string(),
        data_type,
        array_length,
        index_in_type,
    });
}

/// `WM_gizmo_target_property_float_get` dispatch
/// (verbatim — custom fn, scalar, or indexed).
pub enum FloatGet {
    /// `value_get_fn(gz, prop, &mut f32)`.
    CustomFn,
    /// `RNA_property_float_get`.
    RnaScalar,
    /// `RNA_property_float_get_index(index)`.
    RnaIndex(i32),
}
pub fn wm_gizmo_target_property_float_get_dispatch(gz_prop: &WmGizmoProperty) -> FloatGet {
    if gz_prop.has_value_get_fn {
        FloatGet::CustomFn
    } else if gz_prop.index == -1 {
        FloatGet::RnaScalar
    } else {
        FloatGet::RnaIndex(gz_prop.index)
    }
}
