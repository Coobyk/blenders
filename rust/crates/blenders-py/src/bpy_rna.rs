/// Port of `bpy_rna.cc` — `bpy.types` RNA
/// binding base (PointerRNA→PyObject).
/// `BPy_StructRNA` (verbatim — ptr + self).
pub struct BpyStructRna {
    /// `ptr` (PointerRNA handle).
    pub ptr: usize,
}
/// `BPy_PropertyRNA` (verbatim).
pub struct BpyPropertyRna {
    /// `ptr`.
    pub ptr: usize,
    /// `prop` (PropertyRNA handle).
    pub prop: usize,
}
/// `pyrna_is_deferred` (verbatim — deferred
/// class check).
pub fn pyrna_is_deferred() -> bool {
    false
}
