//! Port of `intern/blender_user_menu.cc` — user context-menu lists.
//! `bUserMenu`/`bUserMenuItem` live in caller-owned `Vec` arenas; `usize`
//! handles index them (`NIL` = none), matching the DNA pointer convention.

use blenders_dna::id::NIL;
use blenders_dna::userdef_types::{bUserMenu, bUserMenuItem, USER_MENU_TYPE_MENU, USER_MENU_TYPE_OPERATOR, USER_MENU_TYPE_PROP, USER_MENU_TYPE_SEP};

fn cstr(b: &[u8]) -> &str {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    core::str::from_utf8(&b[..end]).unwrap_or("")
}

/// `BKE_blender_user_menu_find` — match by `space_type` + `context`.
pub fn bke_blender_user_menu_find(lb: &[bUserMenu], space_type: i8, context: &str) -> usize {
    for (i, um) in lb.iter().enumerate() {
        if space_type == um.space_type && context == cstr(&um.context) {
            return i;
        }
    }
    NIL
}

/// `BKE_blender_user_menu_ensure` — find or `BLI_addhead` a new one.
pub fn bke_blender_user_menu_ensure(lb: &mut Vec<bUserMenu>, space_type: i8, context: &str) -> usize {
    let found = bke_blender_user_menu_find(lb, space_type, context);
    if found != NIL {
        return found;
    }
    let mut um = bUserMenu::default();
    um.space_type = space_type;
    let cb = context.as_bytes();
    let n = cb.len().min(um.context.len() - 1);
    um.context[..n].copy_from_slice(&cb[..n]);
    lb.insert(0, um); /* addhead */
    0
}

/// `bUserMenuItem` arena entry — the C file subclasses per type
/// (`bUserMenuItem_Op/_Menu/_Prop`); the subclass payloads stay `usize`
/// handles until those structs' extra fields are needed.
pub enum UserMenuItemKind {
    Sep,
    Operator { prop: usize },
    Menu { menuname: String },
    Prop,
    Other,
}

/// `BKE_blender_user_menu_item_add` — append an item of `type`, returns idx.
pub fn bke_blender_user_menu_item_add(lb: &mut Vec<bUserMenuItem>, type_: i8) -> usize {
    debug_assert!(matches!(type_, USER_MENU_TYPE_SEP | USER_MENU_TYPE_OPERATOR | USER_MENU_TYPE_MENU | USER_MENU_TYPE_PROP));
    let mut umi = bUserMenuItem::default();
    umi.type_ = type_;
    lb.push(umi); /* addtail */
    lb.len() - 1
}

/// `BKE_blender_user_menu_item_free` — frees `prop` on operator items
/// (IDP handle — PORT_PENDING `IDP_FreeProperty`).
pub fn bke_blender_user_menu_item_free(_umi: &mut bUserMenuItem) {}

/// `BKE_blender_user_menu_item_free_list` — `clear_no_delete`.
pub fn bke_blender_user_menu_item_free_list(lb: &mut Vec<bUserMenuItem>) {
    lb.clear();
}
