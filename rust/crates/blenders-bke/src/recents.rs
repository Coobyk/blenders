//! Port of `intern/recents.cc` — `Section` backed by a TOML file
//! (`BLENDER_RECENTS_FILE` under `BLENDER_USER_CONFIG`).
//!
//! toml11 → `toml` crate; appdir/`G.background`/file-io injected.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};

/// `BLENDER_RECENTS_FILE`.
pub const BLENDER_RECENTS_FILE: &str = "recents.toml";
/// `BLENDER_USER_CONFIG` appdir id.
pub const BLENDER_USER_CONFIG: i32 = 2;

static RECENTS_MUTEX: Mutex<()> = Mutex::new(());
static RECENTS_INIT: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());
static RECENTS_READY: AtomicBool = AtomicBool::new(false);
static INIT_ONCE: OnceLock<()> = OnceLock::new();

/// `defaults_get()` — the embedded default document.
fn defaults_get() -> &'static toml::Value {
    static VALUE: OnceLock<toml::Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        toml::from_str(
            r#"
title = "Saved UI State Settings"
name = "Blender"

["temp.window.dimensions"]
_default = [100.0, 900.0, 200.0, 800.0]
PREFERENCES = [100.0, 940.0, 350.0, 900.0]
FILE_BROWSER = [100.0, 1160.0, 350.0, 950.0]
IMAGE_EDITOR = [50.0, 1360.0, 50.0, 830.0]
GRAPH_EDITOR = [50.0, 950.0, 200.0, 780.0]
INFO = [100.0, 1000.0, 300.0, 880.0]
OUTLINER = [100.0, 550.0, 350.0, 800.0]
PROJECT = [100.0, 800.0, 500.0, 850.0]

["panel.sortorder"]
_default = -1

["panel.open"]
_default = true
"#,
        )
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            toml::Value::Table(toml::map::Map::new())
        })
    })
}

static CURRENT: OnceLock<Mutex<toml::Value>> = OnceLock::new();

fn current_cell() -> &'static Mutex<toml::Value> {
    CURRENT.get_or_init(|| Mutex::new(defaults_get().clone()))
}

/// `recents_init_impl` — parse file if present (background skips io).
fn recents_init_impl(
    background: &dyn Fn() -> bool,
    file_path: &dyn Fn(bool) -> Option<String>,
) {
    let _ = defaults_get();

    /* Load from on-disk file if found. In background mode avoid any
     * file I/O or console error output. The in-memory defaults are
     * still parsed above so API calls will work. */
    if !background() {
        if let Some(path) = file_path(false) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                match toml::from_str::<toml::Value>(&content) {
                    Ok(v) => {
                        let _lock = RECENTS_MUTEX.lock().unwrap();
                        *current_cell().lock().unwrap() = v;
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }
        }
    }

    /* Mark ready and wake any waiters. */
    {
        let mut g = RECENTS_INIT.0.lock().unwrap();
        *g = true;
        RECENTS_READY.store(true, Ordering::Release);
    }
    RECENTS_INIT.1.notify_all();
}

/// `init`.
pub fn init(background: &dyn Fn() -> bool, file_path: &dyn Fn(bool) -> Option<String>) {
    recents_init_impl(background, file_path);
}

/// `init_async` — one-shot spawn.
pub fn init_async(
    background: &'static (dyn Fn() -> bool + Sync),
    file_path: &'static (dyn Fn(bool) -> Option<String> + Sync),
) {
    INIT_ONCE.get_or_init(|| {
        std::thread::spawn(move || recents_init_impl(background, file_path));
    });
}

/// `ensure_init` — wait for ready (sync init if async never ran).
pub fn ensure_init(
    background: &dyn Fn() -> bool,
    file_path: &dyn Fn(bool) -> Option<String>,
) {
    if RECENTS_READY.load(Ordering::Acquire) {
        return;
    }

    /* If async init wasn't started (e.g. background mode), run
     * synchronously now. */
    INIT_ONCE.get_or_init(|| {
        recents_init_impl(background, file_path);
    });

    if RECENTS_READY.load(Ordering::Acquire) {
        return;
    }

    let mut g = RECENTS_INIT.0.lock().unwrap();
    while !RECENTS_READY.load(Ordering::Acquire) {
        g = RECENTS_INIT.1.wait(g).unwrap();
    }
}

/// `save` — serialize to the recents file. `blender_version` stamped.
pub fn save(
    background: &dyn Fn() -> bool,
    file_path: &dyn Fn(bool) -> Option<String>,
    blender_version: i64,
) -> bool {
    let _lock = RECENTS_MUTEX.lock().unwrap();
    let mut current = current_cell().lock().unwrap();
    if let toml::Value::Table(t) = &*current {
        if t.is_empty() {
            return false;
        }
    } else {
        return false;
    }

    /* Stamp the Blender version that wrote this file, so a future
     * version can tell how old the on-disk data is. */
    if let toml::Value::Table(t) = &mut *current {
        t.insert("blender_version".to_string(), toml::Value::Integer(blender_version));
    }

    /* Don't write files when running in background/headless mode. */
    if background() {
        return false;
    }

    let Some(path) = file_path(true) else {
        return false;
    };
    match toml::to_string_pretty(&*current) {
        Ok(s) => std::fs::write(path, s).is_ok(),
        Err(_) => false,
    }
}

/// `recents_find_in` — section.key lookup.
fn recents_find_in<'a>(
    root: &'a toml::Value,
    section_name: &str,
    item_key: &str,
) -> Option<&'a toml::Value> {
    let root_table = root.as_table()?;
    if section_name.is_empty() {
        return root_table.get(item_key);
    }
    let section = root_table.get(section_name)?;
    section.as_table()?.get(item_key)
}

/// `TomlGet` — `toml::get_or` typed extraction.
pub trait TomlGet: Sized {
    fn get_or(v: &toml::Value, default: Self) -> Self;
    fn into_toml(self) -> toml::Value;
}

macro_rules! impl_num {
    ($($t:ty),*) => {$(
        impl TomlGet for $t {
            fn get_or(v: &toml::Value, default: Self) -> Self {
                match v {
                    toml::Value::Integer(i) => *i as $t,
                    toml::Value::Float(f) => *f as $t,
                    _ => default,
                }
            }
            fn into_toml(self) -> toml::Value {
                toml::Value::Integer(self as i64)
            }
        }
    )*};
}
impl_num!(i8, i16, i32, i64, u8, u16, u32, u64);

impl TomlGet for f32 {
    fn get_or(v: &toml::Value, default: Self) -> Self {
        match v {
            toml::Value::Float(f) => *f as f32,
            toml::Value::Integer(i) => *i as f32,
            _ => default,
        }
    }
    fn into_toml(self) -> toml::Value {
        toml::Value::Float(self as f64)
    }
}

impl TomlGet for f64 {
    fn get_or(v: &toml::Value, default: Self) -> Self {
        match v {
            toml::Value::Float(f) => *f,
            toml::Value::Integer(i) => *i as f64,
            _ => default,
        }
    }
    fn into_toml(self) -> toml::Value {
        toml::Value::Float(self)
    }
}

impl TomlGet for bool {
    fn get_or(v: &toml::Value, default: Self) -> Self {
        v.as_bool().unwrap_or(default)
    }
    fn into_toml(self) -> toml::Value {
        toml::Value::Boolean(self)
    }
}

impl TomlGet for char {
    fn get_or(v: &toml::Value, default: Self) -> Self {
        match v {
            toml::Value::Integer(i) => (*i as u8) as char,
            _ => default,
        }
    }
    fn into_toml(self) -> toml::Value {
        toml::Value::Integer(self as i64)
    }
}

impl TomlGet for String {
    fn get_or(v: &toml::Value, default: Self) -> Self {
        v.as_str().map(|s| s.to_string()).unwrap_or(default)
    }
    fn into_toml(self) -> toml::Value {
        toml::Value::String(self)
    }
}

impl TomlGet for BTreeMap<String, String> {
    fn get_or(v: &toml::Value, default: Self) -> Self {
        match v {
            toml::Value::Table(t) => t
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect(),
            _ => default,
        }
    }
    fn into_toml(self) -> toml::Value {
        toml::Value::Table(
            self.into_iter()
                .map(|(k, v)| (k, toml::Value::String(v)))
                .collect(),
        )
    }
}

impl<T: TomlGet + Default> TomlGet for Vec<T> {
    fn get_or(v: &toml::Value, default: Self) -> Self {
        match v {
            toml::Value::Array(a) => {
                a.iter().map(|e| T::get_or(e, T::default())).collect()
            }
            _ => default,
        }
    }
    fn into_toml(self) -> toml::Value {
        toml::Value::Array(self.into_iter().map(|e| e.into_toml()).collect())
    }
}

/// `Section` — `{section_}`-scoped typed get/set over the TOML store.
pub struct Section {
    section_: String,
}

impl Section {
    /// `Section(name)`.
    pub fn new(section_: &str) -> Self {
        Self {
            section_: section_.to_string(),
        }
    }

    /// `Section::get<T>` — user value → per-key default → user
    /// section `_default` → builtin section `_default` → `T{}`.
    pub fn get<T: TomlGet + Default>(
        &self,
        item_key: &str,
        background: &dyn Fn() -> bool,
        file_path: &dyn Fn(bool) -> Option<String>,
    ) -> T {
        ensure_init(background, file_path);
        let _lock = RECENTS_MUTEX.lock().unwrap();
        let default_value = T::default();
        let current = current_cell().lock().unwrap();
        let current = &*current;

        /* Try user value first. */
        if let Some(v) = recents_find_in(current, &self.section_, item_key) {
            return T::get_or(v, default_value);
        }
        let defaults = defaults_get();
        /* Per-key default value. */
        if let Some(v) = recents_find_in(defaults, &self.section_, item_key) {
            return T::get_or(v, default_value);
        }
        /* User's section-level `_default` (user override). */
        if !self.section_.is_empty() {
            if let Some(v) = recents_find_in(current, &self.section_, "_default") {
                return T::get_or(v, default_value);
            }
        }
        /* Builtin section-level `_default`. */
        if !self.section_.is_empty() {
            if let Some(v) = recents_find_in(defaults, &self.section_, "_default") {
                return T::get_or(v, default_value);
            }
        }
        default_value
    }

    /// `Section::set<T>`.
    pub fn set<T: TomlGet>(
        &self,
        item_key: &str,
        value: T,
        background: &dyn Fn() -> bool,
        file_path: &dyn Fn(bool) -> Option<String>,
    ) {
        ensure_init(background, file_path);
        let _lock = RECENTS_MUTEX.lock().unwrap();
        let mut guard = current_cell().lock().unwrap();
        let current = &mut *guard;
        if !current.is_table() {
            *current = toml::Value::Table(toml::map::Map::new());
        }
        let table = if self.section_.is_empty() {
            current.as_table_mut().unwrap()
        } else {
            let root = current.as_table_mut().unwrap();
            if !matches!(root.get(&self.section_), Some(v) if v.is_table()) {
                /* Replace with empty table if it's a different data
                 * type, to avoid exception. */
                root.insert(self.section_.clone(), toml::Value::Table(toml::map::Map::new()));
            }
            root.get_mut(&self.section_).unwrap().as_table_mut().unwrap()
        };
        table.insert(item_key.to_string(), value.into_toml());
    }

    /// `Section::remove`.
    pub fn remove(
        &self,
        item_key: &str,
        background: &dyn Fn() -> bool,
        file_path: &dyn Fn(bool) -> Option<String>,
    ) {
        ensure_init(background, file_path);
        let _lock = RECENTS_MUTEX.lock().unwrap();
        let mut guard = current_cell().lock().unwrap();
        let current = &mut *guard;
        if !current.is_table() {
            return;
        }
        let root_table = current.as_table_mut().unwrap();
        if self.section_.is_empty() {
            root_table.remove(item_key);
            return;
        }
        let Some(section_value) = root_table.get_mut(&self.section_) else {
            return;
        };
        if let Some(section_table) = section_value.as_table_mut() {
            section_table.remove(item_key);
        }
    }

    /// `Section::remove_section`.
    pub fn remove_section(
        &self,
        background: &dyn Fn() -> bool,
        file_path: &dyn Fn(bool) -> Option<String>,
    ) {
        ensure_init(background, file_path);
        let _lock = RECENTS_MUTEX.lock().unwrap();
        let mut guard = current_cell().lock().unwrap();
        let current = &mut *guard;
        if self.section_.is_empty() || !current.is_table() {
            return;
        }
        current.as_table_mut().unwrap().remove(&self.section_);
    }
}
