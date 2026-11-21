//! Port of `intern/light.cc` — Light ID type + `BKE_light_*`.
//!
//! The `IDTypeInfo` table is ported as data (like `speaker.rs`); the
//! node-tree/preview/colormanagement/BLO hooks are injected or
//! PORT_PENDING.

use crate::idtype::{IDTypeInfo, IDTYPE_FLAGS_APPEND_IS_REUSABLE};
use blenders_dna::id::{FILTER_ID_LA, FILTER_ID_TE, IndexID};
use blenders_dna::id_enums::ID_LA;
use blenders_dna::light_types::*;
use blenders_math::matrix::Mat4;
use blenders_math::matrix::Mat3;
use blenders_math::vector::Vec3;

/// `light_init_data` (`INIT_DEFAULT_STRUCT_AFTER`) → `Light::default`.
pub fn light_init_data() -> Light {
    Light::default()
}

/// `light_copy_data` — nodetree copy is injected (`BKE_id_copy_in_lib`
/// or `node_tree_localize` depending on `is_localized`); preview copy
/// via `previewimg_id_copy` (injected).
pub fn light_copy_data(
    la_dst: &mut Light,
    la_src: &Light,
    is_localized: bool,
    no_preview: bool,
    node_tree_copy: &mut dyn FnMut(usize, bool) -> usize,
    previewimg_id_copy: &mut dyn FnMut(),
) {
    if la_src.nodetree != 0 && la_src.nodetree != blenders_dna::id::NIL {
        la_dst.nodetree = node_tree_copy(la_src.nodetree, is_localized);
    }
    if !no_preview {
        previewimg_id_copy();
    } else {
        la_dst.preview = 0;
    }
}

/// `light_free_data` — `node_tree_free_embedded_tree` + delete injected.
pub fn light_free_data(la: &mut Light, node_tree_free: &mut dyn FnMut(usize)) {
    if la.nodetree != 0 && la.nodetree != blenders_dna::id::NIL {
        node_tree_free(la.nodetree);
        la.nodetree = 0;
    }
    /* `BKE_previewimg_id_free` + `BKE_icon_id_delete` — PORT_PENDING. */
}

/// `light_foreach_id` — calls `foreach_embedded` for `la.nodetree`.
pub fn light_foreach_id(la: &mut Light, foreach_embedded: &mut dyn FnMut(&mut usize)) {
    if la.nodetree != 0 && la.nodetree != blenders_dna::id::NIL {
        foreach_embedded(&mut la.nodetree);
    }
}

/// `light_foreach_working_space_color` — `fn.single(&la->r)`.
pub fn light_foreach_working_space_color(la: &mut Light, single: &mut dyn FnMut(&mut f32)) {
    single(&mut la.r);
    single(&mut la.g);
    single(&mut la.b);
}

/// `light_blend_write` energy pre-pass (`energy_deprecated` fill +
/// `use_nodes = true`) — the writer calls themselves are PORT_PENDING.
pub fn light_blend_write_prepare(la: &mut Light) {
    la.energy_deprecated = la.energy * la.exposure.exp2();
    if la.type_ == LA_AREA {
        la.energy_deprecated /= std::f32::consts::FRAC_PI_4;
    }
    la.use_nodes = 1;
}

/// `IDTypeInfo IDType_ID_LA` — ported as data.
pub static IDTYPE_ID_LA: IDTypeInfo = {
    let t = IDTypeInfo::new(
        ID_LA as i16,
        FILTER_ID_LA,
        FILTER_ID_TE,
        IndexID::INDEX_ID_LA as i32,
        core::mem::size_of::<Light>() as u32,
        "Light",
        "lights",
        "Light",
        IDTYPE_FLAGS_APPEND_IS_REUSABLE,
    );
    t
};

/// `BKE_light_add` — `BKE_id_new<Light>` + `node_tree_shader_default`
/// injected.
pub fn bke_light_add(
    id_new: &mut dyn FnMut(&str) -> Light,
    node_tree_shader_default: &mut dyn FnMut(&mut Light),
    name: &str,
) -> Light {
    let mut la = id_new(name);
    node_tree_shader_default(&mut la);
    la
}

/// `BKE_light_eval` — `DEG_debug_print_eval` only; no-op.
pub fn bke_light_eval(_la: &Light) {}

/// `BKE_light_power`.
pub fn bke_light_power(light: &Light) -> f32 {
    light.energy * light.exposure.exp2()
}

/// `BKE_light_color` — `IMB_colormanagement_blackbody_temperature_to_rgb`
/// injected.
pub fn bke_light_color(light: &Light, blackbody_to_rgb: &dyn Fn(f32) -> [f32; 4]) -> Vec3 {
    let mut color = Vec3::new(light.r, light.g, light.b);
    if light.mode & LA_USE_TEMPERATURE != 0 {
        let t = blackbody_to_rgb(light.temperature);
        color = Vec3::new(color.x * t[0], color.y * t[1], color.z * t[2]);
    }
    color
}

/// `BKE_light_area`.
pub fn bke_light_area(light: &Light, object_to_world: &Mat4) -> f32 {
    match light.type_ {
        LA_AREA => {
            /* `object_to_world.view<3,3>()` → top-left 3x3. */
            let m = Mat3::from_cols(
                Vec3::new(object_to_world[0].x, object_to_world[0].y, object_to_world[0].z),
                Vec3::new(object_to_world[1].x, object_to_world[1].y, object_to_world[1].z),
                Vec3::new(object_to_world[2].x, object_to_world[2].y, object_to_world[2].z),
            );
            let scale = m.to_scale();
            let size_x = light.area_size * scale.x;
            let size_y = (if light.area_shape == LA_AREA_RECT || light.area_shape == LA_AREA_ELLIPSE {
                light.area_sizey
            } else {
                light.area_size
            }) * scale.y;
            let mut area = size_x * size_y;
            if light.area_shape == LA_AREA_DISK || light.area_shape == LA_AREA_ELLIPSE {
                area *= std::f32::consts::FRAC_PI_4;
            }
            area
        }
        LA_LOCAL | LA_SPOT => {
            let radius = light.radius;
            if radius > 0.0 { 4.0 * std::f32::consts::PI * radius * radius } else { 4.0 }
        }
        LA_SUN => {
            let angle = light.sun_angle / 2.0;
            if angle > 0.0 { std::f32::consts::PI * angle.sin() * angle.sin() } else { 1.0 }
        }
        _ => 1.0,
    }
}
