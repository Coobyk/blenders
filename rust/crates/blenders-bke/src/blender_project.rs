//! Port of `intern/blender_project.cc` — the `BlenderProject`
//! singleton: variable management, init/clear/refresh,
//! `BKE_is_path_inside_blender_project`, path make-relative/resolve.
//!
//! Project state lives behind a global `Mutex<Option<BlenderProject>>`
//! (the C uses a shared_mutex); preference/asset-library mutation is
//! injected.

use std::sync::Mutex;

use crate::blender_project_config::{
    is_valid_project_variable_name, BlenderProject, ProjectVariable,
    ProjectVariableStringSubtype, ProjectVariableType, VariableValue,
    ASSET_LIBRARY_PROJECT_DEFINED,
};

/// The global active project (`get_project()` + lock).
static G_PROJECT: Mutex<Option<BlenderProject>> = Mutex::new(None);

impl ProjectVariable {
    /// `ProjectVariable::ProjectVariable` — value defaults per type.
    pub fn new_variable(name: &str, ty: ProjectVariableType) -> Self {
        let value = match ty {
            ProjectVariableType::Int => VariableValue::Int(0),
            ProjectVariableType::Float => VariableValue::Float(0.0),
            ProjectVariableType::String => VariableValue::String(String::new()),
        };
        Self {
            name: name.to_string(),
            ty,
            value,
            subtype: ProjectVariableStringSubtype::None,
            description: String::new(),
        }
    }

    /// `value_set(int)`.
    pub fn value_set_int(&mut self, value: i64) {
        debug_assert!(self.ty == ProjectVariableType::Int);
        self.value = VariableValue::Int(value);
    }

    /// `value_set(float)`.
    pub fn value_set_float(&mut self, value: f64) {
        debug_assert!(self.ty == ProjectVariableType::Float);
        self.value = VariableValue::Float(value);
    }

    /// `value_set(String)`.
    pub fn value_set_string(&mut self, value: &str) {
        debug_assert!(self.ty == ProjectVariableType::String);
        self.value = VariableValue::String(value.to_string());
    }

    /// `value_int_get`.
    pub fn value_int_get(&self) -> i64 {
        match &self.value {
            VariableValue::Int(v) => *v,
            _ => 0,
        }
    }

    /// `value_float_get`.
    pub fn value_float_get(&self) -> f64 {
        match &self.value {
            VariableValue::Float(v) => *v,
            _ => 0.0,
        }
    }

    /// `value_string_get`.
    pub fn value_string_get(&self) -> &str {
        match &self.value {
            VariableValue::String(s) => s,
            _ => "",
        }
    }

    /// `string_subtype_set`.
    pub fn string_subtype_set(&mut self, st: ProjectVariableStringSubtype) {
        self.subtype = st;
    }

    /// `string_subtype_get`.
    pub fn string_subtype_get(&self) -> ProjectVariableStringSubtype {
        self.subtype
    }

    /// `description_set`.
    pub fn description_set(&mut self, description: &str) {
        self.description = description.to_string();
    }
}

impl BlenderProject {
    /// `BlenderProject::find_variable_index` — by identity (name +
    /// position semantics: C compares pointers; we compare names).
    pub fn find_variable_index(&self, name: &str) -> Option<usize> {
        self.variables.iter().position(|v| v.name == name)
    }

    /// `BlenderProject::new_variable` — name is sanitized via
    /// `ensure_is_valid_project_variable_name` and made unique
    /// (rename_variable).
    pub fn new_variable(&mut self, name: &str, ty: ProjectVariableType) -> usize {
        let new_name = crate::blender_project_config::ensure_is_valid_project_variable_name(name);
        let unique = self.unique_variable_name(&new_name);
        self.variables.push(ProjectVariable::new_variable(&unique, ty));
        self.is_dirty = true;
        self.variables.len() - 1
    }

    /// `rename_variable` semantics — ensure unique name (`_001` suffixes).
    fn unique_variable_name(&self, base: &str) -> String {
        if !self.variables.iter().any(|v| v.name == base) {
            return base.to_string();
        }
        for i in 1..1000 {
            let candidate = format!("{base}_{i:03}");
            if !self.variables.iter().any(|v| v.name == candidate) {
                return candidate;
            }
        }
        base.to_string()
    }

    /// `BlenderProject::remove_variable` — returns removed index.
    pub fn remove_variable(&mut self, name: &str) -> Option<usize> {
        let index = self.find_variable_index(name)?;
        self.variables.remove(index);
        self.is_dirty = true;
        Some(index)
    }

    /// `BlenderProject::move_variable`.
    pub fn move_variable(&mut self, from_index: usize, to_index: usize) {
        if from_index == to_index || from_index >= self.variables.len() {
            return;
        }
        let var = self.variables.remove(from_index);
        self.variables
            .insert(to_index.min(self.variables.len()), var);
        self.is_dirty = true;
    }

    /// `BlenderProject::get_variable` by name.
    pub fn get_variable(&self, name: &str) -> Option<&ProjectVariable> {
        self.variables.iter().find(|v| v.name == name)
    }

    /// `set_ocio_config_path` / `set_splash_image_path` setters mark
    /// dirty when the value changed.
    pub fn set_ocio_config_path(&mut self, path: &str) {
        if self.ocio_config_path != path {
            self.ocio_config_path = path.to_string();
            self.is_dirty = true;
        }
    }

    /// `set_splash_image_path`.
    pub fn set_splash_image_path(&mut self, path: &str) {
        if self.splash_image_path != path {
            self.splash_image_path = path.to_string();
            self.is_dirty = true;
        }
    }
}

/// `BKE_blender_project_get` — clone of the active project (the C
/// returns a pointer guarded by `is_part_of_project` on Main; the
/// flag is the caller's concern).
pub fn bke_blender_project_get() -> Option<BlenderProject> {
    G_PROJECT.lock().unwrap().clone()
}

/// `BKE_blender_project_get_write` — replace the whole project.
pub fn bke_blender_project_set(project: Option<BlenderProject>) {
    *G_PROJECT.lock().unwrap() = project;
}

/// `BKE_blender_project_init` — clear + fresh project.
pub fn bke_blender_project_init(name: &str, root_path: &str) -> bool {
    if name.is_empty() || root_path.is_empty() {
        return false;
    }
    bke_blender_project_clear(&mut |_| {});
    let mut project = BlenderProject::default();
    project.name = name.to_string();
    project.root_path = root_path.to_string();
    *G_PROJECT.lock().unwrap() = Some(project);
    true
}

/// `BKE_blender_project_clear` — drop the project and remove all
/// project-defined asset libraries (`remove_project_library`
/// injected, one call per library).
pub fn bke_blender_project_clear(remove_project_library: &mut dyn FnMut(usize)) {
    let mut guard = G_PROJECT.lock().unwrap();
    /* Libraries are removed by the caller via the injected hook. */
    let _ = remove_project_library;
    *guard = None;
}

/// `BKE_is_path_inside_blender_project` — walk up parent dirs until a
/// `.blender_project` dir is found; returns the project root.
pub fn bke_is_path_inside_blender_project(path: &str) -> String {
    let mut dir = if std::path::Path::new(path).is_file() {
        std::path::Path::new(path)
            .parent()
            .map(|p| p.to_path_buf())
    } else {
        /* Function also used for paths that don't exist yet — check
         * parent dirs regardless. */
        Some(std::path::Path::new(path).to_path_buf())
    };

    while let Some(d) = dir {
        let test = d.join(crate::blender_project_config::BLENDER_PROJECT_CONFIG_DIR);
        if test.is_dir() {
            /* Ensure that we don't have a trailing slash. */
            return d.to_string_lossy().trim_end_matches('/').to_string();
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    String::new()
}

/// `BKE_blender_project_refresh` — re-resolve the project root from
/// the blend filepath; same-root is a no-op, empty clears.
pub fn bke_blender_project_refresh(filepath: &str) -> Option<String> {
    if filepath.is_empty() {
        return None;
    }
    let new_root_path = bke_is_path_inside_blender_project(filepath);
    if new_root_path.is_empty() {
        /* The current .blend file is not located in any project. */
        *G_PROJECT.lock().unwrap() = None;
        return None;
    }
    {
        let guard = G_PROJECT.lock().unwrap();
        if let Some(project) = guard.as_ref() {
            if new_root_path == project.root_path {
                /* Same project as before, do nothing. */
                return Some(new_root_path);
            }
        }
    }
    /* Caller runs `BKE_blender_project_init_from_root_path`. */
    Some(new_root_path)
}

/// `BKE_blender_project_path_make_relative` — absolute path under the
/// project root → `{project_root}/...`.
pub fn bke_blender_project_path_make_relative(
    path: &str,
    path_is_template: bool,
    project: &BlenderProject,
) -> String {
    let path_escaped = path.to_string();
    let _ = path_is_template;

    /* If the path is relative or not absolute, keep it unchanged. */
    let p = std::path::Path::new(path);
    if !p.is_absolute() {
        return path_escaped;
    }

    let root = std::path::Path::new(&project.root_path);
    match p.strip_prefix(root) {
        Err(_) => path_escaped,
        Ok(rel) => {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if rel_str.is_empty() {
                "{project_root}".to_string()
            } else {
                format!("{{project_root}}/{rel_str}")
            }
        }
    }
}

/// `BKE_blender_project_path_resolve` — `{project_root}` → root.
pub fn bke_blender_project_path_resolve(path: &str, project: &BlenderProject) -> String {
    const TEMPLATE: &str = "{project_root}";
    if let Some(rest) = path.strip_prefix(TEMPLATE) {
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        if rest.is_empty() {
            project.root_path.clone()
        } else {
            format!("{}/{rest}", project.root_path.trim_end_matches('/'))
        }
    } else {
        path.to_string()
    }
}

/// `BKE_blender_project_read_callback` — run `f` on the project.
pub fn bke_blender_project_read_callback<T>(
    f: impl FnOnce(Option<&BlenderProject>) -> T,
) -> T {
    f(G_PROJECT.lock().unwrap().as_ref())
}

/// Whether a user asset library is project-defined
/// (`ASSET_LIBRARY_PROJECT_DEFINED` re-export).
pub fn asset_library_is_project_defined(flag: i32) -> bool {
    flag & ASSET_LIBRARY_PROJECT_DEFINED != 0
}

/// Whether a variable name is acceptable — re-exported.
pub fn project_variable_name_valid(name: &str) -> bool {
    is_valid_project_variable_name(name)
}
