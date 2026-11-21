//! Port of `intern/shader_fx.cc` — `BKE_shaderfx_*` (shader-FX type
//! registry + generic fx lifecycle).
//!
//! `shader_fx_types[]` → `static Mutex<[Option<ShaderFxTypeInfo>]>`
//! (NUM_SHADER_FX_TYPES slots; `shaderfx_type_init` from
//! FX_shader_util fills them — injected via `register`).

use std::sync::Mutex;

use blenders_dna::id::NIL;

/// `NUM_SHADER_FX_TYPES`.
pub const NUM_SHADER_FX_TYPES: usize = 32;
/// `eShaderFxType_GpencilType`.
pub const SHADERFX_TYPE_GPENCIL: i32 = 1;
/// `eShaderFxMode_*`.
pub const SHADERFX_MODE_REALTIME: i32 = 1;
pub const SHADERFX_MODE_RENDER: i32 = 2;
pub const SHADERFX_MODE_EDITMODE: i32 = 4;
/// `eShaderFxTypeFlag_EnableInEditmode`.
pub const SHADERFX_TYPEFLAG_ENABLE_IN_EDITMODE: i32 = 1;
/// `eShaderFxFlag_OverrideLibrary_Local`.
pub const SHADERFX_FLAG_OVERRIDE_LIBRARY_LOCAL: i32 = 1;
/// `UI_PANEL_DATA_EXPAND_ROOT`.
pub const UI_PANEL_DATA_EXPAND_ROOT: i32 = 1;
/// `SHADERFX_TYPE_PANEL_PREFIX`.
pub const SHADERFX_TYPE_PANEL_PREFIX: &str = "SHADERFX_PT_";
/// `LIB_ID_CREATE_NO_USER_REFCOUNT`.
pub const LIB_ID_CREATE_NO_USER_REFCOUNT: i32 = 1 << 7;
/// `IDWALK_CB_USER`.
pub const IDWALK_CB_USER: i32 = 1;

/// `ShaderFxData` — the common header every fx struct starts with.
#[derive(Clone, Default)]
pub struct ShaderFxData {
    pub next: usize,
    pub prev: usize,
    /// `type` (`ShaderFxType`).
    pub type_: i32,
    /// `mode` (`eShaderFxMode` mask).
    pub mode: i32,
    /// `flag` (`eShaderFxFlag` mask).
    pub flag: i32,
    /// `ui_expand_flag`.
    pub ui_expand_flag: i32,
    /// `name[64]`.
    pub name: String,
    /// `error` — free-string, `NIL` = none.
    pub error: usize,
    /// Payload area (`fxi->struct_size - sizeof(ShaderFxData)` raw
    /// bytes — the `copydata_generic` memcpy target).
    pub payload: Vec<u8>,
}

/// `ShaderFxTypeInfo`.
#[derive(Clone, Default)]
pub struct ShaderFxTypeInfo {
    /// `struct_size`/`struct_name`.
    pub struct_size: usize,
    pub struct_name: &'static str,
    /// `type` (`eShaderFxType`: Gpencil/Mesh).
    pub type_: i32,
    /// `flags` (`eShaderFxTypeFlag`).
    pub flags: i32,
    /// `name[64]` (empty = slot unused).
    pub name: String,
    /// `init_data`.
    pub init_data: Option<fn(&mut ShaderFxData)>,
    /// `copy_data`.
    pub copy_data: Option<fn(&ShaderFxData, &mut ShaderFxData)>,
    /// `free_data`.
    pub free_data: Option<fn(&mut ShaderFxData)>,
    /// `depends_on_time`.
    pub depends_on_time: Option<fn(&ShaderFxData) -> bool>,
    /// `foreach_ID_link` — `(fx, id_walk_cb)` where cb receives each
    /// `(id_handle, cb_flag)`.
    pub foreach_id_link: Option<fn(&mut ShaderFxData, &mut dyn FnMut(usize, i32))>,
}

/// `shader_fx_types[NUM_SHADER_FX_TYPES]`.
pub static SHADER_FX_TYPES: Mutex<Vec<Option<ShaderFxTypeInfo>>> = Mutex::new(Vec::new());

fn types_lock() -> std::sync::MutexGuard<'static, Vec<Option<ShaderFxTypeInfo>>> {
    let mut g = SHADER_FX_TYPES.lock().unwrap();
    if g.is_empty() {
        g.resize_with(NUM_SHADER_FX_TYPES, || None);
    }
    g
}

/// `BKE_shaderfx_init` — `shaderfx_type_init(types)` injected.
pub fn bke_shaderfx_init(fill: &mut dyn FnMut(&mut Vec<Option<ShaderFxTypeInfo>>)) {
    let mut g = types_lock();
    fill(&mut g);
}

/// `BKE_shaderfx_get_info` — `name[0] == '\0'` → None.
pub fn bke_shaderfx_get_info(type_: i32) -> Option<ShaderFxTypeInfo> {
    let g = types_lock();
    let t = type_ as usize;
    if type_ > 0 && t < NUM_SHADER_FX_TYPES {
        if let Some(fxi) = &g[t] {
            if !fxi.name.is_empty() {
                return Some(fxi.clone());
            }
        }
    }
    None
}

/// `BKE_shaderfx_has_gpencil`.
pub fn bke_shaderfx_has_gpencil(shader_fx: &[ShaderFxData]) -> bool {
    shader_fx.iter().any(|fx| {
        bke_shaderfx_get_info(fx.type_).map(|f| f.type_ == SHADERFX_TYPE_GPENCIL).unwrap_or(false)
    })
}

/// `BKE_shaderfx_new`.
pub fn bke_shaderfx_new(type_: i32) -> Option<ShaderFxData> {
    let fxi = bke_shaderfx_get_info(type_)?;
    let mut fx = ShaderFxData {
        type_,
        name: fxi.name.clone(),
        mode: SHADERFX_MODE_REALTIME | SHADERFX_MODE_RENDER,
        flag: SHADERFX_FLAG_OVERRIDE_LIBRARY_LOCAL,
        ui_expand_flag: UI_PANEL_DATA_EXPAND_ROOT,
        payload: vec![0u8; fxi.struct_size],
        ..Default::default()
    };
    if fxi.flags & SHADERFX_TYPEFLAG_ENABLE_IN_EDITMODE != 0 {
        fx.mode |= SHADERFX_MODE_EDITMODE;
    }
    if let Some(init) = fxi.init_data {
        init(&mut fx);
    }
    Some(fx)
}

/// `BKE_shaderfx_free_ex` — `id_us_min` injected; `error`/`fx`
/// freed by drop.
pub fn bke_shaderfx_free_ex(fx: &mut ShaderFxData, flag: i32, id_us_min: &mut dyn FnMut(usize)) {
    let Some(fxi) = bke_shaderfx_get_info(fx.type_) else { return };
    if flag & LIB_ID_CREATE_NO_USER_REFCOUNT == 0 {
        if let Some(each) = fxi.foreach_id_link {
            each(fx, &mut |id, cb_flag| {
                if id != NIL && cb_flag & IDWALK_CB_USER != 0 {
                    id_us_min(id);
                }
            });
        }
    }
    if let Some(free) = fxi.free_data {
        free(fx);
    }
}

/// `BKE_shaderfx_unique_name` — `BLI_uniquename` injected (needs
/// sibling names from the ListBase).
pub fn bke_shaderfx_unique_name(fx: &mut ShaderFxData, uniquename: &mut dyn FnMut(&mut String)) {
    uniquename(&mut fx.name);
}

/// `BKE_shaderfx_depends_ontime`.
pub fn bke_shaderfx_depends_ontime(fx: &ShaderFxData) -> bool {
    bke_shaderfx_get_info(fx.type_)
        .and_then(|f| f.depends_on_time)
        .map(|f| f(fx))
        .unwrap_or(false)
}

/// `BKE_shaderfx_is_nonlocal_in_liboverride`.
pub fn bke_shaderfx_is_nonlocal_in_liboverride(ob_is_override_library: bool, shaderfx: Option<&ShaderFxData>) -> bool {
    ob_is_override_library
        && shaderfx.map(|fx| fx.flag & SHADERFX_FLAG_OVERRIDE_LIBRARY_LOCAL == 0).unwrap_or(true)
}

/// `BKE_shaderfxType_panel_id` — `SHADERFX_PT_ + fxi->name`.
pub fn bke_shaderfxtype_panel_id(type_: i32) -> String {
    match bke_shaderfx_get_info(type_) {
        Some(fxi) => format!("{}{}", SHADERFX_TYPE_PANEL_PREFIX, fxi.name),
        None => String::new(),
    }
}

/// `BKE_shaderfx_panel_expand`.
pub fn bke_shaderfx_panel_expand(fx: &mut ShaderFxData) {
    fx.ui_expand_flag |= UI_PANEL_DATA_EXPAND_ROOT;
}

/// `BKE_shaderfx_copydata_generic` — free dst payload + memcpy the
/// payload region (`struct_size - sizeof(ShaderFxData)` → the whole
/// `payload` vec in the port).
pub fn bke_shaderfx_copydata_generic(fx_src: &ShaderFxData, fx_dst: &mut ShaderFxData) {
    if let Some(free) = bke_shaderfx_get_info(fx_src.type_).and_then(|f| f.free_data) {
        free(fx_dst);
    }
    fx_dst.payload = fx_src.payload.clone();
}

/// `BKE_shaderfx_copydata_ex` — `id_us_plus` injected.
pub fn bke_shaderfx_copydata_ex(
    fx: &ShaderFxData,
    target: &mut ShaderFxData,
    flag: i32,
    id_us_plus: &mut dyn FnMut(usize),
) {
    let Some(fxi) = bke_shaderfx_get_info(fx.type_) else { return };
    target.mode = fx.mode;
    target.flag = fx.flag;
    target.ui_expand_flag = fx.ui_expand_flag;
    if let Some(copy) = fxi.copy_data {
        copy(fx, target);
    }
    if flag & LIB_ID_CREATE_NO_USER_REFCOUNT == 0 {
        if let Some(each) = fxi.foreach_id_link {
            each(target, &mut |id, cb_flag| {
                if id != NIL && cb_flag & IDWALK_CB_USER != 0 {
                    id_us_plus(id);
                }
            });
        }
    }
}

/// `BKE_shaderfx_copy` — duplicatelist: dst gets fresh clones then
/// copydata per pair.
pub fn bke_shaderfx_copy(src: &[ShaderFxData], id_us_plus: &mut dyn FnMut(usize)) -> Vec<ShaderFxData> {
    src.iter()
        .map(|fx| {
            let mut dst = fx.clone();
            bke_shaderfx_copydata_ex(fx, &mut dst, 0, id_us_plus);
            dst
        })
        .collect()
}

/// `BKE_shaderfx_findby_type` / `findby_name`.
pub fn bke_shaderfx_findby_type<'a>(shader_fx: &'a [ShaderFxData], type_: i32) -> Option<&'a ShaderFxData> {
    shader_fx.iter().find(|fx| fx.type_ == type_)
}
pub fn bke_shaderfx_findby_name<'a>(shader_fx: &'a [ShaderFxData], name: &str) -> Option<&'a ShaderFxData> {
    shader_fx.iter().find(|fx| fx.name == name)
}

/// `BKE_shaderfx_blend_write` — `writer.write_struct_by_name` injected.
pub fn bke_shaderfx_blend_write(fxbase: Option<&[ShaderFxData]>, write_struct: &mut dyn FnMut(&str, &ShaderFxData)) {
    let Some(fxbase) = fxbase else { return };
    for fx in fxbase {
        let Some(fxi) = bke_shaderfx_get_info(fx.type_) else { return };
        write_struct(fxi.struct_name, fx);
    }
}

/// `BKE_shaderfx_blend_read_data` — drops fxs whose type info is gone
/// (upward compat), clears the override-local flag for linked ob.
pub fn bke_shaderfx_blend_read_data(fxbase: &mut Vec<ShaderFxData>, ob_is_linked: bool) {
    for fx in fxbase.iter_mut() {
        fx.error = NIL;
    }
    fxbase.retain(|fx| bke_shaderfx_get_info(fx.type_).is_some());
    if ob_is_linked {
        for fx in fxbase.iter_mut() {
            fx.flag &= !SHADERFX_FLAG_OVERRIDE_LIBRARY_LOCAL;
        }
    }
}
