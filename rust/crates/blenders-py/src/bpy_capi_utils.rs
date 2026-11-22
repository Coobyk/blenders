/// Port of `bpy_capi_utils.cc` — CAPI
/// helpers (PyC_*/PyObject utilities).
/// `PyC_String_From` / list utilities.
pub fn pyc_string_from(s: &str) -> String {
    s.to_string()
}
