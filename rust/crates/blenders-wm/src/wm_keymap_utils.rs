//! Port of `intern/wm_keymap_utils.cc` — keymap item
//! add helpers, context/opname→keymap guessing tables,
//! `WM_keymap_uses_event_modifier`.

/// `KM_ANY`/`KM_NOTHING` verbatim (`wmKeyMapItem` mods).
pub const KM_ANY: i8 = -1;
pub const KM_NOTHING: i8 = 0;
/// `KM_ALT`/`KM_OSKEY`/`KM_HYPER` verbatim.
pub const KM_ALT: i32 = 4;
pub const KM_OSKEY: i32 = 8;
pub const KM_HYPER: i32 = 128;
/// `KMI_INACTIVE` verbatim.
pub const KMI_INACTIVE: u32 = 1 << 0;
/// `RGN_TYPE_WINDOW` verbatim.
pub const RGN_TYPE_WINDOW: i32 = 0;
/// `SPACE_EMPTY` verbatim.
pub const SPACE_EMPTY: i32 = 0;

/// `KeyMapItem_Params` verbatim (aggregate for add fns).
#[derive(Clone, Copy, Default)]
pub struct KeyMapItemParams {
    /// `type`.
    pub event_type: i32,
    /// `val`.
    pub val: i32,
    /// `keymodifier`.
    pub keymodifier: i32,
    /// `modifier` mask.
    pub modifier: i32,
}

/// `WM_keymap_add_menu` (verbatim — `WM_OT_call_menu` +
/// name prop).
pub fn wm_keymap_add_menu(
    idname: &str,
    add_item: &mut dyn FnMut(&str, &KeyMapItemParams) -> usize,
    rna_string_set: &mut dyn FnMut(usize, &str, &str),
    params: &KeyMapItemParams,
) -> usize {
    let kmi = (add_item)("WM_OT_call_menu", params);
    (rna_string_set)(kmi, "name", idname);
    kmi
}

/// `WM_keymap_add_menu_pie` (verbatim).
pub fn wm_keymap_add_menu_pie(
    idname: &str,
    add_item: &mut dyn FnMut(&str, &KeyMapItemParams) -> usize,
    rna_string_set: &mut dyn FnMut(usize, &str, &str),
    params: &KeyMapItemParams,
) -> usize {
    let kmi = (add_item)("WM_OT_call_menu_pie", params);
    (rna_string_set)(kmi, "name", idname);
    kmi
}

/// `WM_keymap_add_panel` (verbatim — `WM_OT_call_panel` +
/// name + keep_open=false).
pub fn wm_keymap_add_panel(
    idname: &str,
    add_item: &mut dyn FnMut(&str, &KeyMapItemParams) -> usize,
    rna_string_set: &mut dyn FnMut(usize, &str, &str),
    rna_boolean_set: &mut dyn FnMut(usize, &str, bool),
    params: &KeyMapItemParams,
) -> usize {
    let kmi = (add_item)("WM_OT_call_panel", params);
    (rna_string_set)(kmi, "name", idname);
    (rna_boolean_set)(kmi, "keep_open", false);
    kmi
}

/// `WM_keymap_add_tool` (verbatim — `WM_OT_tool_set_by_id`).
pub fn wm_keymap_add_tool(
    idname: &str,
    add_item: &mut dyn FnMut(&str, &KeyMapItemParams) -> usize,
    rna_string_set: &mut dyn FnMut(usize, &str, &str),
    params: &KeyMapItemParams,
) -> usize {
    let kmi = (add_item)("WM_OT_tool_set_by_id", params);
    (rna_string_set)(kmi, "name", idname);
    kmi
}

/// `CTX_MODE_*` verbatim (`BKE_context.hh`).
pub const CTX_MODE_OBJECT: i32 = 0;
pub const CTX_MODE_EDIT_MESH: i32 = 1;
pub const CTX_MODE_EDIT_CURVE: i32 = 2;
pub const CTX_MODE_EDIT_CURVES: i32 = 24;
pub const CTX_MODE_EDIT_SURFACE: i32 = 3;
pub const CTX_MODE_EDIT_TEXT: i32 = 4;
pub const CTX_MODE_EDIT_ARMATURE: i32 = 5;
pub const CTX_MODE_EDIT_METABALL: i32 = 6;
pub const CTX_MODE_EDIT_LATTICE: i32 = 7;
pub const CTX_MODE_EDIT_GREASE_PENCIL: i32 = 8;
pub const CTX_MODE_EDIT_POINTCLOUD: i32 = 27;
pub const CTX_MODE_POSE: i32 = 9;
pub const CTX_MODE_SCULPT: i32 = 10;
pub const CTX_MODE_PAINT_WEIGHT: i32 = 11;
pub const CTX_MODE_PAINT_VERTEX: i32 = 12;
pub const CTX_MODE_PAINT_TEXTURE: i32 = 13;
pub const CTX_MODE_PARTICLE: i32 = 14;
pub const CTX_MODE_PAINT_GPENCIL_LEGACY: i32 = 15;
pub const CTX_MODE_EDIT_GPENCIL_LEGACY: i32 = 16;
pub const CTX_MODE_SCULPT_GPENCIL_LEGACY: i32 = 17;
pub const CTX_MODE_WEIGHT_GPENCIL_LEGACY: i32 = 18;
pub const CTX_MODE_VERTEX_GPENCIL_LEGACY: i32 = 19;
pub const CTX_MODE_SCULPT_CURVES: i32 = 25;
pub const CTX_MODE_PAINT_GREASE_PENCIL: i32 = 20;
pub const CTX_MODE_SCULPT_GREASE_PENCIL: i32 = 21;
pub const CTX_MODE_WEIGHT_GREASE_PENCIL: i32 = 22;
pub const CTX_MODE_VERTEX_GREASE_PENCIL: i32 = 23;

/// `WM_keymap_guess_from_context` — mode→keymap id
/// table (verbatim switch).
pub fn keymap_id_for_mode(mode: i32) -> Option<&'static str> {
    Some(match mode {
        CTX_MODE_EDIT_MESH => "Mesh",
        CTX_MODE_EDIT_CURVE | CTX_MODE_EDIT_SURFACE => "Curve",
        CTX_MODE_EDIT_CURVES => "Curves",
        CTX_MODE_EDIT_TEXT => "Font",
        CTX_MODE_EDIT_ARMATURE => "Armature",
        CTX_MODE_EDIT_METABALL => "Metaball",
        CTX_MODE_EDIT_LATTICE => "Lattice",
        CTX_MODE_EDIT_GREASE_PENCIL => "Grease Pencil Edit Mode",
        CTX_MODE_EDIT_POINTCLOUD => "Point Cloud",
        CTX_MODE_POSE => "Pose",
        CTX_MODE_SCULPT => "Sculpt",
        CTX_MODE_PAINT_WEIGHT => "Weight Paint",
        CTX_MODE_PAINT_VERTEX => "Vertex Paint",
        CTX_MODE_PAINT_TEXTURE => "Image Paint",
        CTX_MODE_PARTICLE => "Particle",
        CTX_MODE_OBJECT => "Object Mode",
        CTX_MODE_PAINT_GPENCIL_LEGACY => "Grease Pencil Stroke Paint Mode",
        CTX_MODE_EDIT_GPENCIL_LEGACY => "Grease Pencil Stroke Edit Mode",
        CTX_MODE_SCULPT_GPENCIL_LEGACY => "Grease Pencil Stroke Sculpt Mode",
        CTX_MODE_WEIGHT_GPENCIL_LEGACY => "Grease Pencil Stroke Weight Mode",
        CTX_MODE_VERTEX_GPENCIL_LEGACY => "Grease Pencil Stroke Vertex Mode",
        CTX_MODE_SCULPT_CURVES => "Sculpt Curves",
        CTX_MODE_PAINT_GREASE_PENCIL => "Grease Pencil Draw Mode",
        CTX_MODE_SCULPT_GREASE_PENCIL => "Grease Pencil Sculpt Mode",
        CTX_MODE_WEIGHT_GREASE_PENCIL => "Grease Pencil Weight Mode",
        CTX_MODE_VERTEX_GREASE_PENCIL => "Grease Pencil Vertex Mode",
        _ => return None,
    })
}

/// `WM_keymap_guess_opname` — opname prefix → (keymap id,
/// uses-sl-spacetype) table (verbatim chain).
pub fn keymap_id_for_opname(opname: &str) -> Option<(&'static str, bool)> {
    let pairs: &[(&[&str], &str, bool)] = &[
        (&["SCREEN_OT", "RENDER_OT", "SOUND_OT", "SCENE_OT"], "Screen", false),
        (&["GPENCIL_OT"], "Grease Pencil", false),
        (&["GREASE_PENCIL_OT"], "Grease Pencil", false),
        (&["MARKER_OT"], "Markers", false),
        (&["IMPORT_", "EXPORT_"], "Window", false),
        (&["VIEW3D_OT"], "3D View", true),
        (&["GROUP_OT", "MATERIAL_OT", "PTCACHE_OT", "RIGIDBODY_OT"], "Object Mode", false),
        (&["MESH_OT"], "Mesh", false),
        (&["CURVE_OT", "SURFACE_OT"], "Curve", false),
        (&["ARMATURE_OT", "SKETCH_OT"], "Armature", false),
        (&["CURVES_OT"], "Curves", false),
        (&["SCULPT_CURVES_OT"], "Sculpt Curves", false),
        (&["MBALL_OT"], "Metaball", false),
        (&["LATTICE_OT"], "Lattice", false),
        (&["PARTICLE_OT"], "Particle", false),
        (&["POINTCLOUD_OT"], "Point Cloud", false),
        (&["FONT_OT"], "Font", false),
        (&["VIEW2D_OT"], "View2D", false),
        (&["IMAGE_OT"], "Image", true),
        (&["CLIP_OT"], "Clip", true),
        (&["MASK_OT"], "Mask Editing", false),
        (&["NODE_OT"], "Node Editor", true),
        (&["ANIM_OT_channels"], "Animation Channels", false),
        (&["GRAPH_OT"], "Graph Editor", true),
        (&["ACTION_OT"], "Dopesheet", true),
        (&["NLA_OT"], "NLA Editor", true),
        (&["SCRIPT_OT"], "Script", true),
        (&["TEXT_OT"], "Text", true),
        (&["SEQUENCER_OT"], "Sequencer", true),
        (&["CONSOLE_OT"], "Console", true),
        (&["INFO_OT"], "Info", true),
        (&["FILE_OT"], "File Browser", true),
        (&["LOGIC_OT"], "Logic Editor", true),
        (&["OUTLINER_OT"], "Outliner", true),
        (&["UI_OT"], "User Interface", false),
    ];
    for (prefixes, km_id, use_sl) in pairs {
        if prefixes.iter().any(|p| opname.starts_with(p)) {
            return Some((*km_id, *use_sl));
        }
    }
    None
}

/// `TRANSFORM_OT` per-spacetype keymap ids (verbatim).
pub fn transform_keymap_id(spacetype: i32) -> Option<&'static str> {
    const SPACE_VIEW3D: i32 = 1;
    const SPACE_GRAPH: i32 = 3;
    const SPACE_ACTION: i32 = 4;
    const SPACE_NLA: i32 = 5;
    const SPACE_IMAGE: i32 = 6;
    const SPACE_NODE: i32 = 7;
    const SPACE_SEQ: i32 = 8;
    match spacetype {
        SPACE_VIEW3D => Some("3D View"),
        SPACE_GRAPH => Some("Graph Editor"),
        SPACE_ACTION => Some("Dopesheet"),
        SPACE_NLA => Some("NLA Editor"),
        SPACE_IMAGE => Some("UV Editor"),
        SPACE_NODE => Some("Node Editor"),
        SPACE_SEQ => Some("Sequencer"),
        _ => None,
    }
}

/// `wmKeyMapItem` modifier fields used.
pub struct KeyMapItemMods {
    /// `flag`.
    pub flag: u32,
    /// `ctrl`/`alt`/`shift`/`oskey`/`hyper`.
    pub ctrl: i8,
    pub alt: i8,
    pub shift: i8,
    pub oskey: i8,
    pub hyper: i8,
}

/// `wm_keymap_item_uses_modifier` (verbatim — per-mod
/// KM_ANY/NOTHING match).
pub fn keymap_item_uses_modifier(kmi: &KeyMapItemMods, event_modifier: i32) -> bool {
    if kmi.ctrl != KM_ANY && (kmi.ctrl == KM_NOTHING) != ((event_modifier & crate::wm_operator_utils::KM_CTRL) == 0) {
        return false;
    }
    if kmi.alt != KM_ANY && (kmi.alt == KM_NOTHING) != ((event_modifier & KM_ALT) == 0) {
        return false;
    }
    if kmi.shift != KM_ANY && (kmi.shift == KM_NOTHING) != ((event_modifier & crate::wm_operator_utils::KM_SHIFT) == 0) {
        return false;
    }
    if kmi.oskey != KM_ANY && (kmi.oskey == KM_NOTHING) != ((event_modifier & KM_OSKEY) == 0) {
        return false;
    }
    if kmi.hyper != KM_ANY && (kmi.hyper == KM_NOTHING) != ((event_modifier & KM_HYPER) == 0) {
        return false;
    }
    true
}

/// `WM_keymap_uses_event_modifier` (verbatim — active
/// items only).
pub fn wm_keymap_uses_event_modifier(items: &[KeyMapItemMods], event_modifier: i32) -> bool {
    for kmi in items {
        if (kmi.flag & KMI_INACTIVE) == 0 && keymap_item_uses_modifier(kmi, event_modifier) {
            return true;
        }
    }
    false
}

/// `WM_keymap_fix_linking` (verbatim — empty).
pub fn wm_keymap_fix_linking() {}
