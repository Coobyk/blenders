//! Port of `intern/blender.cc` — `Global G`/`UserDef U`/`G_MAIN`
//! singletons, version strings, userdef swap/free, atexit registry.

use std::sync::Mutex;

use blenders_dna::userdef_types::UserDef;

/// `BLENDER_VERSION` / `_PATCH` / `_CYCLE` / `_SUFFIX` for the port.
pub const BLENDER_VERSION: i32 = 500;
pub const BLENDER_VERSION_PATCH: i32 = 0;
/// `BLENDER_VERSION_CYCLE` — "alpha"/"beta"/"rc"/"release".
pub const BLENDER_VERSION_CYCLE: &str = "alpha";
/// `BLENDER_VERSION_SUFFIX` — "" or "LTS".
pub const BLENDER_VERSION_SUFFIX: &str = "";

/// `Global` (BKE_global.hh subset) — the fields `blender.cc` touches.
#[derive(Default)]
pub struct Global {
    /// `G.f` — `G_FLAG_*` bits.
    pub f: i32,
    /// `G.filepath_last_image`.
    pub filepath_last_image: String,
    /// `G.log.level` (`CLG_LEVEL_*`).
    pub log_level: i32,
    /// `G.log.file` — flag only (the FILE* isn't ported).
    pub log_file_open: bool,
}
/// `G_FLAG_SCRIPT_AUTOEXEC` (BKE_global.hh).
pub const G_FLAG_SCRIPT_AUTOEXEC: i32 = 1 << 2;

/// The `G`/`U`/`G_MAIN` triple; `G_MAIN` is a `Main` handle (`usize`).
pub struct BlenderGlobals {
    /* `Mutex<BlenderGlobals>` needs Send — the `ListBase` raw pointers
     * inside `UserDef` are handles, never dereferenced off-thread. */
    pub g: Global,
    /// `U` — `UserDef` (huge; boxed to keep the struct small).
    pub u: Box<UserDef>,
    /// `G_MAIN` — `Main*` handle (`usize::MAX` = none).
    pub g_main: usize,
    /// `G_MAIN->is_global_main` (the Main itself holds the flag;
    /// tracked here since Main isn't a struct yet).
    pub g_main_is_global: bool,
}

unsafe impl Send for BlenderGlobals {}
unsafe impl Send for Global {}

static GLOBALS: Mutex<Option<Box<BlenderGlobals>>> = Mutex::new(None);
static VERSION_STRING: Mutex<Option<String>> = Mutex::new(None);
static VERSION_STRING_COMPACT: Mutex<Option<String>> = Mutex::new(None);

/// `with_globals` — access the singleton block (created lazily by
/// `bke_blender_globals_init`).
pub fn with_globals<R>(f: impl FnOnce(&mut BlenderGlobals) -> R) -> Option<R> {
    let mut g = GLOBALS.lock().unwrap();
    g.as_mut().map(|b| f(&mut **b))
}

/// `blender_version_init`.
fn blender_version_init() {
    let (version_cycle, version_cycle_compact) = match BLENDER_VERSION_CYCLE {
        "alpha" => (" Alpha", " a"),
        "beta" => (" Beta", " b"),
        "rc" => (" Release Candidate", " RC"),
        "release" => ("", ""),
        _ => unreachable!("Invalid Blender version cycle"),
    };
    let version_suffix = if bke_blender_version_is_lts() { " LTS" } else { "" };
    *VERSION_STRING.lock().unwrap() = Some(format!(
        "{}.{:01}{}{}{}",
        BLENDER_VERSION / 100,
        BLENDER_VERSION % 100,
        BLENDER_VERSION_PATCH,
        version_suffix,
        version_cycle
    ));
    *VERSION_STRING_COMPACT.lock().unwrap() = Some(format!(
        "{}.{:01}{}{}",
        BLENDER_VERSION / 100,
        BLENDER_VERSION % 100,
        BLENDER_VERSION_PATCH,
        version_cycle_compact
    ));
}

/// `BKE_blender_version_string` (lazy-init safe).
pub fn bke_blender_version_string() -> String {
    let mut g = VERSION_STRING.lock().unwrap();
    if g.is_none() {
        drop(g);
        blender_version_init();
        g = VERSION_STRING.lock().unwrap();
    }
    g.clone().unwrap()
}

/// `BKE_blender_version_string_compact`.
pub fn bke_blender_version_string_compact() -> String {
    let mut g = VERSION_STRING_COMPACT.lock().unwrap();
    if g.is_none() {
        drop(g);
        blender_version_init();
        g = VERSION_STRING_COMPACT.lock().unwrap();
    }
    g.clone().unwrap()
}

/// `BKE_blender_version_blendfile_string_from_values` — returns the
/// formatted string (`SNPRINTF` target).
pub fn bke_blender_version_blendfile_string_from_values(
    file_version: i16,
    file_subversion: i16,
) -> String {
    let file_version_major = file_version / 100;
    let file_version_minor = file_version % 100;
    if file_subversion >= 0 {
        format!("{}.{} (sub {})", file_version_major, file_version_minor, file_subversion)
    } else {
        format!("{}.{}", file_version_major, file_version_minor)
    }
}

/// `BKE_blender_version_is_alpha`.
pub fn bke_blender_version_is_alpha() -> bool {
    BLENDER_VERSION_CYCLE == "alpha"
}

/// `BKE_blender_version_is_lts`.
pub fn bke_blender_version_is_lts() -> bool {
    BLENDER_VERSION_SUFFIX == "LTS"
}

/// `BKE_blender_globals_init` — `BKE_main_new` injected (returns the
/// Main handle); `G_FLAG_SCRIPT_AUTOEXEC` set (no
/// `WITH_PYTHON_SECURITY`); `CLG_LEVEL_WARN` = 2.
pub fn bke_blender_globals_init(main_new: &mut dyn FnMut() -> usize) {
    blender_version_init();
    let g = Global {
        f: G_FLAG_SCRIPT_AUTOEXEC,
        filepath_last_image: "//".to_string(),
        log_level: 2,
        ..Default::default()
    };
    let mut u = Box::<UserDef>::default();
    u.savetime = 1;
    let mut gs = GLOBALS.lock().unwrap();
    *gs = Some(Box::new(BlenderGlobals {
        g,
        u,
        g_main: 0,
        g_main_is_global: false,
    }));
    bke_blender_globals_main_replace(main_new());
}

/// `BKE_blender_globals_clear` — `BKE_main_free(G_MAIN)` injected.
pub fn bke_blender_globals_clear(main_free: &mut dyn FnMut(usize)) {
    let (main, was_global) = {
        let mut gs = GLOBALS.lock().unwrap();
        match gs.as_mut() {
            Some(g) if g.g_main != usize::MAX => (g.g_main, g.g_main_is_global),
            _ => return,
        }
    };
    debug_assert!(was_global);
    main_free(main);
    if let Some(g) = GLOBALS.lock().unwrap().as_mut() {
        g.g_main = usize::MAX;
    }
}

/// `BKE_blender_globals_main_replace`.
pub fn bke_blender_globals_main_replace(bmain: usize) {
    let mut gs = GLOBALS.lock().unwrap();
    if let Some(g) = gs.as_mut() {
        /* clear() when an old global main exists (asserts inside the
         * caller — `main_free` isn't available here; the C frees it,
         * callers must clear first). */
        debug_assert!(!g.g_main_is_global || g.g_main == usize::MAX);
        g.g_main = bmain;
        g.g_main_is_global = true;
    }
}

/// `BKE_blender_globals_main_swap`.
pub fn bke_blender_globals_main_swap(new_gmain: usize) -> usize {
    let mut gs = GLOBALS.lock().unwrap();
    let g = gs.as_mut().expect("globals not initialized");
    debug_assert!(g.g_main_is_global);
    let old = g.g_main;
    g.g_main = new_gmain;
    old
}

/// `BKE_blender_globals_crash_path_get` — `<tempdir>/<basename>.crash.txt`.
pub fn bke_blender_globals_crash_path_get(
    tempdir_base: &str,
    main_filepath: Option<&str>,
) -> String {
    match main_filepath {
        None => blenders_bli::path_utils::join(&[tempdir_base, "blender.crash.txt"]),
        Some(fp) => {
            let mut p = blenders_bli::path_utils::join(&[
                tempdir_base,
                &blenders_bli::path_utils::basename(fp),
            ]);
            p = format!("{}.crash.txt", p.trim_end_matches(".blend").trim_end_matches('.'));
            p
        }
    }
}

/// `BKE_blender_userdef_data_swap` — `dna::shallow_swap` → `mem::swap`.
pub fn bke_blender_userdef_data_swap(userdef_a: &mut UserDef, userdef_b: &mut UserDef) {
    std::mem::swap(userdef_a, userdef_b);
}

/// `BKE_blender_userdef_data_set` — swaps global `U` with `userdef`,
/// then `userdef_data_free(userdef)` (injected: keymaps/menus/addons/
/// fonts/asset shelves have subsystem free-fns — PORT_PENDING).
pub fn bke_blender_userdef_data_set(
    userdef: &mut Box<UserDef>,
    userdef_data_free: &mut dyn FnMut(&mut UserDef, bool),
) {
    let mut gs = GLOBALS.lock().unwrap();
    let g = gs.as_mut().expect("globals not initialized");
    bke_blender_userdef_data_swap(&mut g.u, userdef);
    drop(gs);
    userdef_data_free(userdef, true);
}

/// `BKE_blender_userdef_data_set_and_free` — `data_set` + drop.
pub fn bke_blender_userdef_data_set_and_free(
    userdef: Box<UserDef>,
    userdef_data_free: &mut dyn FnMut(&mut UserDef, bool),
) {
    let mut userdef = userdef;
    bke_blender_userdef_data_set(&mut userdef, userdef_data_free);
}

/// `BKE_blender_userdef_app_template_data_swap` — swaps the listed
/// fields verbatim (Value/Data/Flag swap set).
pub fn bke_blender_userdef_app_template_data_swap(
    userdef_a: &mut UserDef,
    userdef_b: &mut UserDef,
) {
    macro_rules! value_swap { ($f:ident) => {{ std::mem::swap(&mut userdef_a.$f, &mut userdef_b.$f); }}}
    macro_rules! data_swap { ($f:ident) => {{ std::mem::swap(&mut userdef_a.$f, &mut userdef_b.$f); }}}
    macro_rules! flag_swap { ($f:ident, $flags:expr) => {{
        let f = $flags;
        let a = userdef_a.$f; let b = userdef_b.$f;
        userdef_a.$f = (a & !f) | (b & f);
        userdef_b.$f = (b & !f) | (a & f);
    }}}
    value_swap!(uistyles);
    value_swap!(uifonts);
    value_swap!(themes);
    value_swap!(addons);
    value_swap!(user_keymaps);
    value_swap!(user_keyconfig_prefs);
    data_swap!(font_path_ui);
    data_swap!(font_path_ui_mono);
    data_swap!(keyconfigstr);
    data_swap!(gizmo_flag);
    data_swap!(app_flag);
    flag_swap!(
        uiflag,
        blenders_dna::userdef_types::USER_SAVE_PROMPT
            | blenders_dna::userdef_types::USER_SPLASH_DISABLE
            | blenders_dna::userdef_types::USER_SHOW_GIZMO_NAVIGATE
    );
    data_swap!(ui_scale);
}

/// `BKE_blender_userdef_app_template_data_set` / `_and_free`.
pub fn bke_blender_userdef_app_template_data_set(
    userdef: &mut Box<UserDef>,
    userdef_data_free: &mut dyn FnMut(&mut UserDef, bool),
) {
    let mut gs = GLOBALS.lock().unwrap();
    let g = gs.as_mut().expect("globals not initialized");
    bke_blender_userdef_app_template_data_swap(&mut g.u, userdef);
    drop(gs);
    userdef_data_free(userdef, true);
}
pub fn bke_blender_userdef_app_template_data_set_and_free(
    userdef: Box<UserDef>,
    userdef_data_free: &mut dyn FnMut(&mut UserDef, bool),
) {
    let mut userdef = userdef;
    bke_blender_userdef_app_template_data_set(&mut userdef, userdef_data_free);
}

/// `AtExitData` registry — `Vec<(func, user_data)>` (LIFO like the C list).
type AtExitFunc = fn(usize);
static AT_EXIT: Mutex<Vec<(AtExitFunc, usize)>> = Mutex::new(Vec::new());

/// `BKE_blender_atexit_register`.
pub fn bke_blender_atexit_register(func: AtExitFunc, user_data: usize) {
    AT_EXIT.lock().unwrap().push((func, user_data));
}

/// `BKE_blender_atexit_unregister`.
pub fn bke_blender_atexit_unregister(func: AtExitFunc, user_data: usize) {
    let mut v = AT_EXIT.lock().unwrap();
    if let Some(i) = v.iter().position(|&(f, d)| f == func && d == user_data) {
        v.remove(i);
    }
}

/// `BKE_blender_atexit` — pops and calls each entry (LIFO).
pub fn bke_blender_atexit() {
    let mut v = AT_EXIT.lock().unwrap();
    while let Some((func, data)) = v.pop() {
        func(data);
    }
}

/// `BKE_blender_free` — subsystem exits in C order (all injected).
pub fn bke_blender_free(exit_steps: &mut dyn FnMut(&str)) {
    /* studiolight_free → globals_clear → fclose(G.log.file) →
     * spacetypes_free → IMB_exit → DEG_free_node_types →
     * brush_system_exit → RE_texture_rng_exit →
     * callback_global_finalize → IMB_cache_destruct →
     * seq::fontmap_clear → MOV_exit → node_system_exit. */
    for step in [
        "BKE_studiolight_free",
        "BKE_blender_globals_clear",
        "BKE_spacetypes_free",
        "IMB_exit",
        "DEG_free_node_types",
        "BKE_brush_system_exit",
        "RE_texture_rng_exit",
        "BKE_callback_global_finalize",
        "IMB_cache_destruct",
        "seq::fontmap_clear",
        "MOV_exit",
        "bke::node_system_exit",
    ] {
        exit_steps(step);
    }
}
