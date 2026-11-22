//! Port of `intern/wm_toolsystem.cc` — `bToolKey`/
//! `bToolRef` lookup/ensure/unlink, mode-from-spacetype,
//! default-tool table, refresh-screen dispatch.

/// `bToolKey` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BToolKey {
    /// `space_type`.
    pub space_type: i32,
    /// `mode`.
    pub mode: i32,
}

/// `bToolRef` fields used.
pub struct BToolRef {
    /// `tkey`.
    pub tkey: BToolKey,
    /// `idname`.
    pub idname: String,
}

/// `SI_MODE_*` verbatim (`DNA_space_enums.h`).
pub const SI_MODE_VIEW: i32 = 0;
pub const SI_MODE_PAINT: i32 = 1;
pub const SI_MODE_MASK: i32 = 2;
pub const SI_MODE_UV: i32 = 3;
/// `SPACE_VIEW3D`/`SPACE_IMAGE`/`SPACE_NODE`/`SPACE_SEQ`.
pub const SPACE_VIEW3D: i32 = 1;
pub const SPACE_GRAPH: i32 = 3;
pub const SPACE_NLA: i32 = 5;
pub const SPACE_IMAGE: i32 = 6;
pub const SPACE_NODE: i32 = 7;
pub const SPACE_SEQ: i32 = 8;
/// `OB_MODE_OBJECT` sentinel for mode=-1 handling.
pub const OB_MODE_OBJECT: i32 = 0;

/// `WM_toolsystem_ref_find` (verbatim — tkey match in
/// workspace tools list).
pub fn wm_toolsystem_ref_find(trefs: &[BToolRef], tkey: &BToolKey) -> Option<usize> {
    trefs.iter().position(|t| t.tkey == *tkey)
}

/// `WM_toolsystem_ref_ensure` (verbatim — find-or-create;
/// returns true when created).
pub fn wm_toolsystem_ref_ensure(trefs: &mut Vec<BToolRef>, tkey: &BToolKey) -> (usize, bool) {
    if let Some(i) = wm_toolsystem_ref_find(trefs, tkey) {
        return (i, false);
    }
    trefs.push(BToolRef {
        tkey: *tkey,
        idname: String::new(),
    });
    (trefs.len() - 1, true)
}

/// `WM_toolsystem_mode_from_spacetype` (verbatim —
/// per-space mode source).
pub fn wm_toolsystem_mode_from_spacetype(
    space_type: i32,
    view3d_mode: i32,
    image_mode: i32,
    seq_view: i32,
    obact_present: bool,
) -> i32 {
    match space_type {
        SPACE_VIEW3D => {
            if obact_present {
                view3d_mode
            } else {
                crate::wm_keymap_utils::CTX_MODE_OBJECT
            }
        }
        SPACE_IMAGE => image_mode,
        SPACE_NODE => 0,
        SPACE_SEQ => seq_view,
        _ => -1,
    }
}

/// `WM_toolsystem_key_from_context` (verbatim — fills
/// tkey when mode != -1).
pub fn wm_toolsystem_key_from_context(space_type: i32, mode: i32) -> Option<BToolKey> {
    if space_type == crate::wm_keymap_utils::SPACE_EMPTY && mode == -1 {
        return None;
    }
    if mode != -1 {
        return Some(BToolKey { space_type, mode });
    }
    None
}

/// `toolsystem_default_tool` (verbatim table).
pub fn toolsystem_default_tool(tkey: &BToolKey) -> &'static str {
    use crate::wm_keymap_utils::*;
    match tkey.space_type {
        SPACE_VIEW3D => match tkey.mode {
            CTX_MODE_SCULPT
            | CTX_MODE_PAINT_VERTEX
            | CTX_MODE_PAINT_WEIGHT
            | CTX_MODE_PAINT_TEXTURE
            | CTX_MODE_PAINT_GPENCIL_LEGACY
            | CTX_MODE_PAINT_GREASE_PENCIL
            | CTX_MODE_SCULPT_GPENCIL_LEGACY
            | CTX_MODE_SCULPT_GREASE_PENCIL
            | CTX_MODE_WEIGHT_GPENCIL_LEGACY
            | CTX_MODE_WEIGHT_GREASE_PENCIL
            | CTX_MODE_VERTEX_GPENCIL_LEGACY
            | CTX_MODE_VERTEX_GREASE_PENCIL
            | CTX_MODE_SCULPT_CURVES => "builtin.brush",
            CTX_MODE_PARTICLE => "builtin_brush.Comb",
            CTX_MODE_EDIT_TEXT => "builtin.select_text",
            CTX_MODE_EDIT_GREASE_PENCIL => "builtin.select_lasso",
            _ => "builtin.select_box",
        },
        SPACE_IMAGE => match tkey.mode {
            SI_MODE_PAINT => "builtin.brush",
            SI_MODE_VIEW => "builtin.sample",
            SI_MODE_MASK => "builtin.select_box",
            _ => "builtin.select_box",
        },
        SPACE_NODE | SPACE_SEQ => "builtin.select_box",
        _ => "builtin.select_box",
    }
}
