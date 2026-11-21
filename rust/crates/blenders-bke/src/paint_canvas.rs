//! Port of `intern/paint_canvas.cc` — `BKE_paint_canvas_image_get` +
//! `bke::paint::canvas::get_active_slot`.
//!
//! `Object`/`Material`/`TexPaintSlot`/`Image` are DNA structs; `BKE_object_material_get`
//! is caller-supplied (object API PORT_PENDING).

use blenders_dna::id::NIL;
use blenders_dna::image_types::ImageUser;

/// `TexPaintSlot`-sourced image + user (the pair the C returns).
#[derive(Clone)]
pub struct CanvasImageData {
    /// `Image *ima` handle.
    pub ima: usize,
    /// `ImageUser` of the slot.
    pub image_user: ImageUser,
}

/// `PAINT_CANVAS_SOURCE_*` (BKE_paint.hh).
pub const PAINT_CANVAS_SOURCE_MATERIAL: i32 = 0;
pub const PAINT_CANVAS_SOURCE_IMAGE: i32 = 1;
pub const PAINT_CANVAS_SOURCE_SCENE_COLOR_ATTRIBUTE: i32 = 2;
pub const PAINT_CANVAS_SOURCE_OBJECT_COLOR_ATTRIBUTE: i32 = 3;

/// `bke::paint::canvas::get_active_slot` — active TexPaintSlot index of the
/// object's active material (`texpaintslot[paint_active_slot]` bounds-checked).
/// `mat_slot` = `(texpaintslot_len, paint_active_slot)` from the material.
pub fn get_active_slot(mat_slot: Option<(usize, i32)>) -> Option<i32> {
    let (tot_slots, paint_active_slot) = mat_slot?;
    if paint_active_slot < 0 || paint_active_slot >= tot_slots as i32 {
        return None;
    }
    Some(paint_active_slot)
}

/// `BKE_paint_canvas_image_get` — the mode switch over canvas sources.
/// The `settings.mode` dispatch is verbatim; slot/image resolution is by
/// handle (PORT_PENDING image layer).
pub fn bke_paint_canvas_image_get(
    mode: i32,
    slot_image: Option<CanvasImageData>,
    settings_image: Option<CanvasImageData>,
) -> Option<CanvasImageData> {
    match mode {
        PAINT_CANVAS_SOURCE_MATERIAL => slot_image.filter(|d| d.ima != NIL),
        PAINT_CANVAS_SOURCE_IMAGE => settings_image,
        _ => None, /* color-attribute modes read the mesh attribute — PORT_PENDING */
    }
}
