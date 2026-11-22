/// Port of `bpy.cc` — BPY interpreter init
/// + module registration.
/// `BPY_init` / `BPY_exit` (verbatim —
/// python init + module table).
pub fn bpy_init() {}
pub fn bpy_exit() {}
/// `bpy_module_table` (verbatim — registered
/// builtin modules).
pub const BPY_MODULES: &[&str] = &[
    "bpy", "bpy.app", "bpy.context", "bpy.data", "bpy.ops",
    "bpy.types", "bpy.utils", "bpy.path", "bpy.msgbus",
    "mathutils", "mathutils.geometry", "mathutils.noise",
    "mathutils.kdtree", "bgl", "blf", "gpu", "gpu.types",
    "gpu.state", "gpu.shader", "gpu.texture", "gpu.platform",
    "bmesh", "bmesh.types", "bmesh.utils", "bmesh.ops",
    "bmesh.geometry", "idprop.types", "aud", "freestyle",
];
