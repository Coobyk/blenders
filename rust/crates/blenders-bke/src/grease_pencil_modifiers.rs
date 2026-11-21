//! Port of `intern/grease_pencil_modifiers.cc` — grease-pencil
//! modifier type registry: `gpencil_modifier_type_valid`,
//! `gpencil_modifier_struct_size`, `gpencil_modifier_free_data`,
//! `gpencil_modifier_foreach_ID_link`, `BKE_grease_pencil_modifier_*`
//! free/foreach/blend_read helpers.
//!
//! Modifier payloads are a tagged `GpencilModifier` enum; CurveMapping/
//! ColorBand/deform-cache frees are injected hooks.

/// `GpencilModifierType` (DNA_grease_pencil_modifier_types.h).
pub mod gpencil_modifier_type {
    pub const NONE: i32 = 0;
    pub const NOISE: i32 = 1;
    pub const SUBDIV: i32 = 2;
    pub const THICK: i32 = 3;
    pub const TINT: i32 = 4;
    pub const ARRAY: i32 = 5;
    pub const BUILD: i32 = 6;
    pub const OPACITY: i32 = 7;
    pub const COLOR: i32 = 8;
    pub const LATTICE: i32 = 9;
    pub const SIMPLIFY: i32 = 10;
    pub const SMOOTH: i32 = 11;
    pub const HOOK: i32 = 12;
    pub const OFFSET: i32 = 13;
    pub const MIRROR: i32 = 14;
    pub const ARMATURE: i32 = 15;
    pub const TIME: i32 = 16;
    pub const MULTIPLY: i32 = 17;
    pub const TEXTURE: i32 = 18;
    pub const LINEART: i32 = 19;
    pub const LENGTH: i32 = 20;
    pub const WEIGHT_PROXIMITY: i32 = 21;
    pub const DASH: i32 = 22;
    pub const WEIGHT_ANGLE: i32 = 23;
    pub const SHRINKWRAP: i32 = 24;
    pub const ENVELOPE: i32 = 25;
    pub const OUTLINE: i32 = 26;
    pub const NUM_GREASEPENCIL_MODIFIER_TYPES: i32 = 27;
}
use gpencil_modifier_type as t;

/// `eGpencilModifierFlag_OverrideLibrary_Local`.
pub const GPENCIL_MODIFIER_FLAG_OVERRIDE_LIBRARY_LOCAL: i32 = 1;

/// `gpencil_modifier_type_valid` — check if the type value is valid.
pub fn gpencil_modifier_type_valid(ty: i32) -> bool {
    ty > 0 && ty < t::NUM_GREASEPENCIL_MODIFIER_TYPES
}

/// `GpencilModifier` — the tagged payloads `gpencil_modifier_*`
/// dispatch over. Fields named `*_id` are `ID*` handles, `*_cm` are
/// `CurveMapping` handles, `cache_data` are runtime caches.
#[derive(Default)]
pub struct GpencilModifier {
    /// `GpencilModifierData` header.
    pub ty: i32,
    pub flag: i32,
    /// `md->error` — runtime, cleared on read.
    pub error: Option<String>,
    /// Payload (per-type fields the free/link/read paths touch).
    pub data: GpencilModifierData,
}

/// The modifier payloads.
#[derive(Default)]
pub enum GpencilModifierData {
    #[default]
    None,
    /// `NoiseGpencilModifierData`.
    Noise {
        material: usize,
        curve_intensity: usize,
    },
    /// `SubdivGpencilModifierData`.
    Subdiv { material: usize },
    /// `ThickGpencilModifierData`.
    Thick {
        material: usize,
        curve_thickness: usize,
    },
    /// `TintGpencilModifierData`.
    Tint {
        material: usize,
        object: usize,
        colorband: usize,
        curve_intensity: usize,
    },
    /// `ArrayGpencilModifierData`.
    Array { material: usize, object: usize },
    /// `BuildGpencilModifierData`.
    Build { object: usize },
    /// `OpacityGpencilModifierData`.
    Opacity {
        material: usize,
        curve_intensity: usize,
    },
    /// `ColorGpencilModifierData`.
    Color {
        material: usize,
        curve_intensity: usize,
    },
    /// `LatticeGpencilModifierData`.
    Lattice {
        material: usize,
        object: usize,
        cache_data: usize,
    },
    /// `SimplifyGpencilModifierData`.
    Simplify { material: usize },
    /// `SmoothGpencilModifierData`.
    Smooth {
        material: usize,
        curve_intensity: usize,
    },
    /// `HookGpencilModifierData`.
    Hook {
        material: usize,
        object: usize,
        curfalloff: usize,
    },
    /// `OffsetGpencilModifierData`.
    Offset { material: usize },
    /// `MirrorGpencilModifierData`.
    Mirror { material: usize, object: usize },
    /// `ArmatureGpencilModifierData`.
    Armature { object: usize },
    /// `TimeGpencilModifierData`.
    Time {
        material: usize,
        segments_len: usize,
    },
    /// `MultiplyGpencilModifierData`.
    Multiply { material: usize },
    /// `TextureGpencilModifierData`.
    Texture { material: usize },
    /// `LineartGpencilModifierData`.
    Lineart {
        target_material: usize,
        source_collection: usize,
        source_object: usize,
        source_camera: usize,
        light_contour_object: usize,
    },
    /// `LengthGpencilModifierData`.
    Length { material: usize },
    /// `WeightProximityGpencilModifierData`.
    WeightProximity { material: usize, object: usize },
    /// `DashGpencilModifierData`.
    Dash {
        material: usize,
        segments_len: usize,
    },
    /// `WeightAngleGpencilModifierData`.
    WeightAngle { material: usize },
    /// `ShrinkwrapGpencilModifierData`.
    Shrinkwrap {
        material: usize,
        target: usize,
        aux_target: usize,
        cache_data: usize,
    },
    /// `EnvelopeGpencilModifierData`.
    Envelope { material: usize },
    /// `OutlineGpencilModifierData`.
    Outline {
        material: usize,
        outline_material: usize,
        object: usize,
    },
}

/// `gpencil_modifier_struct_size` — nominal sizes aren't meaningful
/// in Rust; keep the C-type-name mapping for blend-read dispatch.
pub fn gpencil_modifier_struct_name(ty: i32) -> &'static str {
    match ty {
        t::NONE => "GpencilModifierData",
        t::NOISE => "NoiseGpencilModifierData",
        t::SUBDIV => "SubdivGpencilModifierData",
        t::THICK => "ThickGpencilModifierData",
        t::TINT => "TintGpencilModifierData",
        t::ARRAY => "ArrayGpencilModifierData",
        t::BUILD => "BuildGpencilModifierData",
        t::OPACITY => "OpacityGpencilModifierData",
        t::COLOR => "ColorGpencilModifierData",
        t::LATTICE => "LatticeGpencilModifierData",
        t::SIMPLIFY => "SimplifyGpencilModifierData",
        t::SMOOTH => "SmoothGpencilModifierData",
        t::HOOK => "HookGpencilModifierData",
        t::OFFSET => "OffsetGpencilModifierData",
        t::MIRROR => "MirrorGpencilModifierData",
        t::ARMATURE => "ArmatureGpencilModifierData",
        t::TIME => "TimeGpencilModifierData",
        t::MULTIPLY => "MultiplyGpencilModifierData",
        t::TEXTURE => "TextureGpencilModifierData",
        t::LINEART => "LineartGpencilModifierData",
        t::LENGTH => "LengthGpencilModifierData",
        t::WEIGHT_PROXIMITY => "WeightProximityGpencilModifierData",
        t::DASH => "DashGpencilModifierData",
        t::WEIGHT_ANGLE => "WeightAngleGpencilModifierData",
        t::SHRINKWRAP => "ShrinkwrapGpencilModifierData",
        t::ENVELOPE => "EnvelopeGpencilModifierData",
        t::OUTLINE => "OutlineGpencilModifierData",
        _ => "GpencilModifierData",
    }
}

/// `GpencilModifierFreeHooks` — payload frees the C calls out to.
pub struct GpencilModifierFreeHooks<'a> {
    pub curvemapping_free: &'a dyn Fn(usize),
    pub colorband_free: &'a dyn Fn(usize),
    /// `BKE_lattice_deform_data_destroy`.
    pub lattice_deform_data_destroy: &'a dyn Fn(usize),
    /// `BKE_shrinkwrap_free_tree` + `MEM_delete`.
    pub shrinkwrap_free_tree: &'a dyn Fn(usize),
}

/// `gpencil_modifier_free_data` — per-type payload frees
/// (curve/colorband/deform caches; other fields are plain data).
pub fn gpencil_modifier_free_data(md: &mut GpencilModifier, hooks: &GpencilModifierFreeHooks) {
    match &mut md.data {
        GpencilModifierData::Noise {
            curve_intensity, ..
        } => {
            if *curve_intensity != 0 {
                (hooks.curvemapping_free)(*curve_intensity);
                *curve_intensity = 0;
            }
        }
        GpencilModifierData::Thick {
            curve_thickness, ..
        } => {
            if *curve_thickness != 0 {
                (hooks.curvemapping_free)(*curve_thickness);
                *curve_thickness = 0;
            }
        }
        GpencilModifierData::Tint {
            colorband,
            curve_intensity,
            ..
        } => {
            if *colorband != 0 {
                (hooks.colorband_free)(*colorband);
                *colorband = 0;
            }
            if *curve_intensity != 0 {
                (hooks.curvemapping_free)(*curve_intensity);
                *curve_intensity = 0;
            }
        }
        GpencilModifierData::Opacity {
            curve_intensity, ..
        }
        | GpencilModifierData::Color {
            curve_intensity, ..
        }
        | GpencilModifierData::Smooth {
            curve_intensity, ..
        } => {
            if *curve_intensity != 0 {
                (hooks.curvemapping_free)(*curve_intensity);
                *curve_intensity = 0;
            }
        }
        GpencilModifierData::Lattice { cache_data, .. } => {
            /* free deform data */
            if *cache_data != 0 {
                (hooks.lattice_deform_data_destroy)(*cache_data);
                *cache_data = 0;
            }
        }
        GpencilModifierData::Hook { curfalloff, .. } => {
            if *curfalloff != 0 {
                (hooks.curvemapping_free)(*curfalloff);
                *curfalloff = 0;
            }
        }
        GpencilModifierData::Time { segments_len, .. }
        | GpencilModifierData::Dash { segments_len, .. } => {
            *segments_len = 0;
        }
        GpencilModifierData::Shrinkwrap { cache_data, .. } => {
            if *cache_data != 0 {
                (hooks.shrinkwrap_free_tree)(*cache_data);
                *cache_data = 0;
            }
        }
        _ => {
            /* No allocated data. */
        }
    }
}

/// One `walk(...)` target inside `gpencil_modifier_foreach_ID_link`.
pub struct IdLinkField {
    /// Field label (for debugging/borrow-correctness).
    pub name: &'static str,
    /// `IDWALK_CB_*` flags (USER=1<<0, NOP=0).
    pub flag: i32,
}

/// `IDWALK_CB_USER`.
pub const IDWALK_CB_USER: i32 = 1;
/// `IDWALK_CB_NOP`.
pub const IDWALK_CB_NOP: i32 = 0;

/// `gpencil_modifier_foreach_ID_link` — table of ID-pointer fields
/// per modifier type with their `IDWALK_CB_*` flags, matching the
/// C switch exactly.
pub fn gpencil_modifier_id_link_fields(ty: i32) -> &'static [IdLinkField] {
    macro_rules! f {
        ($($n:literal => $f:expr),+ $(,)?) => {
            &[$(IdLinkField { name: $n, flag: $f }),+]
        };
    }
    match ty {
        t::NONE => &[],
        t::NOISE | t::SUBDIV | t::THICK | t::OPACITY | t::COLOR | t::SIMPLIFY | t::SMOOTH
        | t::OFFSET | t::MULTIPLY | t::LENGTH | t::DASH | t::WEIGHT_ANGLE | t::TEXTURE
        | t::ARMATURE => match ty {
            t::ARMATURE => f!("object" => IDWALK_CB_NOP),
            _ => f!("material" => IDWALK_CB_USER),
        },
        t::TINT => f!("material" => IDWALK_CB_USER, "object" => IDWALK_CB_NOP),
        t::ARRAY => f!("material" => IDWALK_CB_USER, "object" => IDWALK_CB_NOP),
        t::BUILD => f!("object" => IDWALK_CB_NOP),
        t::LATTICE => f!("material" => IDWALK_CB_USER, "object" => IDWALK_CB_NOP),
        t::HOOK => f!("material" => IDWALK_CB_USER, "object" => IDWALK_CB_NOP),
        t::MIRROR => f!("material" => IDWALK_CB_USER, "object" => IDWALK_CB_NOP),
        t::TIME => f!("material" => IDWALK_CB_USER),
        t::LINEART => f!(
            "target_material" => IDWALK_CB_USER,
            "source_collection" => IDWALK_CB_NOP,
            "source_object" => IDWALK_CB_NOP,
            "source_camera" => IDWALK_CB_NOP,
            "light_contour_object" => IDWALK_CB_NOP
        ),
        t::WEIGHT_PROXIMITY => f!("material" => IDWALK_CB_USER, "object" => IDWALK_CB_NOP),
        t::SHRINKWRAP => f!(
            "target" => IDWALK_CB_NOP,
            "aux_target" => IDWALK_CB_NOP,
            "material" => IDWALK_CB_USER
        ),
        t::ENVELOPE => f!("material" => IDWALK_CB_USER),
        t::OUTLINE => f!(
            "material" => IDWALK_CB_USER,
            "outline_material" => IDWALK_CB_USER,
            "object" => IDWALK_CB_NOP
        ),
        _ => &[],
    }
}

/// `gpencil_modifier_foreach_ID_link` — invoke `walk(field, &mut id)`
/// for each ID field of `md`, in C order.
pub fn gpencil_modifier_foreach_id_link(
    md: &mut GpencilModifier,
    walk: &mut dyn FnMut(&'static str, &mut usize, i32),
) {
    for field in gpencil_modifier_id_link_fields(md.ty) {
        let slot: &mut usize = match (field.name, &mut md.data) {
            ("material", GpencilModifierData::Noise { material, .. })
            | ("material", GpencilModifierData::Subdiv { material })
            | ("material", GpencilModifierData::Thick { material, .. })
            | ("material", GpencilModifierData::Tint { material, .. })
            | ("material", GpencilModifierData::Array { material, .. })
            | ("material", GpencilModifierData::Opacity { material, .. })
            | ("material", GpencilModifierData::Color { material, .. })
            | ("material", GpencilModifierData::Lattice { material, .. })
            | ("material", GpencilModifierData::Simplify { material })
            | ("material", GpencilModifierData::Smooth { material, .. })
            | ("material", GpencilModifierData::Hook { material, .. })
            | ("material", GpencilModifierData::Offset { material })
            | ("material", GpencilModifierData::Mirror { material, .. })
            | ("material", GpencilModifierData::Time { material, .. })
            | ("material", GpencilModifierData::Multiply { material })
            | ("material", GpencilModifierData::Texture { material })
            | ("material", GpencilModifierData::Length { material })
            | ("material", GpencilModifierData::WeightProximity { material, .. })
            | ("material", GpencilModifierData::Dash { material, .. })
            | ("material", GpencilModifierData::Shrinkwrap { material, .. })
            | ("material", GpencilModifierData::Envelope { material })
            | ("material", GpencilModifierData::Outline { material, .. }) => material,
            ("outline_material", GpencilModifierData::Outline {
                outline_material, ..
            }) => outline_material,
            ("target_material", GpencilModifierData::Lineart {
                target_material, ..
            }) => target_material,
            ("object", GpencilModifierData::Tint { object, .. })
            | ("object", GpencilModifierData::Array { object, .. })
            | ("object", GpencilModifierData::Build { object })
            | ("object", GpencilModifierData::Lattice { object, .. })
            | ("object", GpencilModifierData::Hook { object, .. })
            | ("object", GpencilModifierData::Mirror { object, .. })
            | ("object", GpencilModifierData::Armature { object })
            | ("object", GpencilModifierData::WeightProximity { object, .. })
            | ("object", GpencilModifierData::Outline { object, .. }) => object,
            ("source_collection", GpencilModifierData::Lineart {
                source_collection, ..
            }) => source_collection,
            ("source_object", GpencilModifierData::Lineart {
                source_object, ..
            }) => source_object,
            ("source_camera", GpencilModifierData::Lineart {
                source_camera, ..
            }) => source_camera,
            ("light_contour_object", GpencilModifierData::Lineart {
                light_contour_object,
                ..
            }) => light_contour_object,
            ("target", GpencilModifierData::Shrinkwrap { target, .. }) => target,
            ("aux_target", GpencilModifierData::Shrinkwrap {
                aux_target, ..
            }) => aux_target,
            _ => continue,
        };
        walk(field.name, slot, field.flag);
    }
}

/// `BKE_grease_pencil_modifier_free_ex` — `flag` selects whether to
/// free payload data and/or unlink the struct.
pub fn bke_grease_pencil_modifier_free_ex(
    md: &mut GpencilModifier,
    flag: i32,
    hooks: &GpencilModifierFreeHooks,
    id_us_decrement: &mut dyn FnMut(usize),
) {
    const LIB_ID_CREATE_NO_USER_REFCOUNT: i32 = 1 << 0;
    if flag & LIB_ID_CREATE_NO_USER_REFCOUNT == 0 {
        /* modifier_free_data_id_us_cb */
        gpencil_modifier_foreach_id_link(md, &mut |_, id, walk_flag| {
            if walk_flag & IDWALK_CB_USER != 0 && *id != 0 {
                id_us_decrement(*id);
            }
        });
    }
    gpencil_modifier_free_data(md, hooks);
}

/// `BKE_grease_pencil_modifier_free` — full free.
pub fn bke_grease_pencil_modifier_free(
    md: &mut GpencilModifier,
    hooks: &GpencilModifierFreeHooks,
    id_us_decrement: &mut dyn FnMut(usize),
) {
    bke_grease_pencil_modifier_free_ex(md, 0, hooks, id_us_decrement);
}

/// `BKE_grease_pencil_modifiers_foreach_ID_link` — all modifiers on
/// the object's gpencil modifier list.
pub fn bke_grease_pencil_modifiers_foreach_id_link(
    modifiers: &mut [GpencilModifier],
    walk: &mut dyn FnMut(&'static str, &mut usize, i32),
) {
    for md in modifiers {
        gpencil_modifier_foreach_id_link(md, walk);
    }
}

/// `BKE_grease_pencil_modifier_blend_read_data` — post-load fixups:
/// error clear, type validity, linked-library flag clear, per-type
/// runtime fields.
///
/// `is_linked` = `ID_IS_LINKED(ob)`; curve-mapping fields need a
/// `curvemapping_init` after their own read (caller side).
pub fn bke_grease_pencil_modifier_blend_read_data(
    md: &mut GpencilModifier,
    is_linked: bool,
) {
    md.error = None;

    /* if modifiers disappear, or for upward compatibility */
    if !gpencil_modifier_type_valid(md.ty) {
        md.ty = t::NONE;
        md.data = GpencilModifierData::None;
    }

    /* If linking from a library, clear 'local' library override flag. */
    if is_linked {
        md.flag &= !GPENCIL_MODIFIER_FLAG_OVERRIDE_LIBRARY_LOCAL;
    }

    match &mut md.data {
        GpencilModifierData::Lattice { cache_data, .. }
        | GpencilModifierData::Shrinkwrap { cache_data, .. } => {
            *cache_data = 0;
        }
        _ => {}
    }
}
