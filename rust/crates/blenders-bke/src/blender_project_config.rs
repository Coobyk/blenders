//! Port of `intern/blender_project_config.cc` — Blender project
//! config (`<root>/.blender_project/project.toml`): TOML schema
//! read/write, `TomlProjectVariable`/`TomlAssetLibrary` parsing,
//! autosave + init-from-root flow.
//!
//! Uses the `toml` crate (the C uses toml11). RNA enum identifier
//! lookups are mapped to fixed tables.

use std::collections::BTreeMap;

/// `BLENDER_PROJECT_CONFIG_DIR`.
pub const BLENDER_PROJECT_CONFIG_DIR: &str = ".blender_project";
/// `BLENDER_PROJECT_CONFIG_FILE`.
pub const BLENDER_PROJECT_CONFIG_FILE: &str = "project.toml";
/// `project_schema_version` — bump on breaking schema changes.
pub const PROJECT_SCHEMA_VERSION: i64 = 1;

/// `ASSET_LIBRARY_*` flags (DNA_asset_types.h).
pub const ASSET_LIBRARY_RELATIVE_PATH: i32 = 1 << 0;
pub const ASSET_LIBRARY_DISABLED: i32 = 1 << 1;
pub const ASSET_LIBRARY_USE_REMOTE_URL: i32 = 1 << 2;
pub const ASSET_LIBRARY_USE_AUTH_TOKEN: i32 = 1 << 3;
pub const ASSET_LIBRARY_PROJECT_DEFINED: i32 = 1 << 4;

/// `eAssetImportMethod` (DNA_asset_types.h).
pub const ASSET_IMPORT_LINK: i32 = 0;
pub const ASSET_IMPORT_APPEND: i32 = 1;
pub const ASSET_IMPORT_APPEND_REUSE: i32 = 2;
pub const ASSET_IMPORT_PACK: i32 = 3;

/// `rna_enum_asset_import_method_items` — identifier↔value.
pub fn asset_import_method_from_identifier(id: &str) -> Option<i32> {
    match id {
        "LINK" => Some(ASSET_IMPORT_LINK),
        "APPEND" => Some(ASSET_IMPORT_APPEND),
        "APPEND_REUSE" => Some(ASSET_IMPORT_APPEND_REUSE),
        "PACK" => Some(ASSET_IMPORT_PACK),
        _ => None,
    }
}

/// `RNA_enum_id_from_value` — inverse of the above.
pub fn asset_import_method_identifier(value: i32) -> Option<&'static str> {
    match value {
        ASSET_IMPORT_LINK => Some("LINK"),
        ASSET_IMPORT_APPEND => Some("APPEND"),
        ASSET_IMPORT_APPEND_REUSE => Some("APPEND_REUSE"),
        ASSET_IMPORT_PACK => Some("PACK"),
        _ => None,
    }
}

/// `ProjectVariableType` (re-uses eIDPropertyType values).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectVariableType {
    /// `IDP_STRING`.
    String,
    /// `IDP_INT`.
    Int,
    /// `IDP_FLOAT`.
    Float,
}

/// `rna_enum_project_variable_type_items` identifiers.
pub fn project_variable_type_identifier(ty: ProjectVariableType) -> &'static str {
    match ty {
        ProjectVariableType::Int => "INTEGER",
        ProjectVariableType::Float => "FLOAT",
        ProjectVariableType::String => "STRING",
    }
}

/// `ProjectVariableStringSubtype`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectVariableStringSubtype {
    /// `PROP_NONE`.
    None,
    /// `PROP_FILEPATH`.
    Filepath,
}

/// `rna_enum_project_variable_string_subtype_items`.
pub fn project_variable_string_subtype_identifier(
    st: ProjectVariableStringSubtype,
) -> &'static str {
    match st {
        ProjectVariableStringSubtype::None => "NONE",
        ProjectVariableStringSubtype::Filepath => "FILEPATH",
    }
}

/// `toml_type_name`.
pub fn toml_type_name(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::String(_) => "string",
        toml::Value::Integer(_) => "integer",
        toml::Value::Float(_) => "float",
        toml::Value::Boolean(_) => "boolean",
        toml::Value::Datetime(d) => {
            if d.date.is_some() && d.time.is_some() {
                if d.offset.is_some() {
                    "offset datetime"
                } else {
                    "local datetime"
                }
            } else if d.date.is_some() {
                "local date"
            } else {
                "local time"
            }
        }
        toml::Value::Array(_) => "array",
        toml::Value::Table(_) => "table",
    }
}

/// `ensure_is_valid_project_variable_name` (blender_project.cc —
/// ported here as the config needs it).
pub fn ensure_is_valid_project_variable_name(name: &str) -> String {
    let mut bytes = name.as_bytes().to_vec();

    /* Shouldn't be empty. */
    if bytes.is_empty() {
        bytes.push(b'_');
    }

    /* Shouldn't start with a numerical digit. */
    if bytes[0].is_ascii_digit() {
        bytes[0] = b'_';
    }

    /* All characters should be alphanumeric or underscore.
     * Utf8-safe: non-ascii bytes each become '_'. */
    for b in bytes.iter_mut() {
        let c = *b;
        let is_valid_identifier_char = c.is_ascii_alphanumeric() || c == b'_';
        if !is_valid_identifier_char {
            *b = b'_';
        }
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

/// `is_valid_project_variable_name`.
pub fn is_valid_project_variable_name(name: &str) -> bool {
    name == ensure_is_valid_project_variable_name(name)
}

/// `TomlProjectVariable` — one entry of `[[variables]]`.
pub struct TomlProjectVariable {
    pub name: String,
    pub ty: ProjectVariableType,
    pub value: toml::Value,
    pub subtype: ProjectVariableStringSubtype,
    pub description: String,
}

impl TomlProjectVariable {
    /// `TomlProjectVariable::from_toml` — validate name/type/value
    /// (+subtype for strings).
    pub fn from_toml(v: &toml::Value) -> Result<Self, String> {
        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("missing variable name")?
            .to_string();
        if !is_valid_project_variable_name(&name) {
            return Err(format!(
                "Invalid variable name: \"{name}\". Variable names can not be empty, must not \
                 start with a digit, and must contain only alphanumeric characters and underscores."
            ));
        }

        let type_str = v
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or("missing variable type")?;
        let ty = match type_str {
            "INTEGER" => ProjectVariableType::Int,
            "FLOAT" => ProjectVariableType::Float,
            "STRING" => ProjectVariableType::String,
            _ => {
                return Err(format!(
                    "invalid value for 'type' expected \"INTEGER\", \"FLOAT\", or \"STRING\", \
                     but got \"{type_str}\""
                ))
            }
        };

        let value = v.get("value").cloned().ok_or("missing variable value")?;
        if ty == ProjectVariableType::Int && value.as_integer().is_none() {
            return Err(format!(
                "Actual and declared type of project variable \"value\" do not match. Got \
                 \"{}\", expected an integer",
                toml_type_name(&value)
            ));
        }
        if ty == ProjectVariableType::Float && value.as_float().is_none() {
            return Err(format!(
                "Actual and declared type of project variable \"value\" do not match. Got \
                 \"{}\", expected a float",
                toml_type_name(&value)
            ));
        }

        let mut subtype = ProjectVariableStringSubtype::None;
        if ty == ProjectVariableType::String {
            if value.as_str().is_none() {
                return Err(format!(
                    "Actual and declared type of project variable \"value\" do not match. Got \
                     \"{}\", expected a string",
                    toml_type_name(&value)
                ));
            }
            let subtype_str = v
                .get("subtype")
                .and_then(|t| t.as_str())
                .ok_or("missing variable subtype")?;
            subtype = match subtype_str {
                "NONE" => ProjectVariableStringSubtype::None,
                "FILEPATH" => ProjectVariableStringSubtype::Filepath,
                _ => {
                    return Err(format!(
                        "invalid value for 'subtype' expected \"NONE\" or \"FILEPATH\", but \
                         got \"{subtype_str}\""
                    ))
                }
            };
        }

        let description = v
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        Ok(Self {
            name,
            ty,
            value,
            subtype,
            description,
        })
    }
}

/// `TomlAssetLibrary` — one entry of `[[asset_libraries]]`.
#[derive(Default)]
pub struct TomlAssetLibrary {
    pub name: String,
    /// `eAssetImportMethod`.
    pub import_method: i32,
    /// Only used to keep track if "url" was defined when parsing.
    pub is_remote_asset_lib: bool,
    pub url: String,
    pub access_token: String,
    pub path: String,
    pub use_relative_path: bool,
    pub uuid: String,
}

impl TomlAssetLibrary {
    /// `TomlAssetLibrary::from_toml`.
    pub fn from_toml(v: &toml::Value) -> Result<Self, String> {
        let mut lib = TomlAssetLibrary {
            name: v
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or("missing library name")?
                .to_string(),
            ..Default::default()
        };

        let import_method_str = v
            .get("import_method")
            .and_then(|s| s.as_str())
            .ok_or("missing import_method")?;
        lib.import_method = asset_import_method_from_identifier(import_method_str).ok_or_else(
            || {
                format!(
                    "invalid value for 'import_method. \"{import_method_str}\" is not a valid \
                     import method"
                )
            },
        )?;

        if v.get("url").is_some() {
            lib.is_remote_asset_lib = true;
            lib.url = v
                .get("url")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            lib.access_token = v
                .get("access_token")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            return Ok(lib);
        }

        lib.path = v
            .get("path")
            .and_then(|s| s.as_str())
            .ok_or("missing library path")?
            .to_string();
        lib.use_relative_path = v
            .get("use_relative_path")
            .and_then(|s| s.as_bool())
            .ok_or("missing use_relative_path")?;
        lib.uuid = v
            .get("uuid")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        Ok(lib)
    }
}

/// `ProjectVariable` — the runtime variable (subset of
/// `BlenderProject`'s list entries).
pub struct ProjectVariable {
    pub name: String,
    pub ty: ProjectVariableType,
    pub value: VariableValue,
    pub subtype: ProjectVariableStringSubtype,
    pub description: String,
}

/// `ProjectVariable::value_*`.
pub enum VariableValue {
    Int(i64),
    Float(f64),
    String(String),
}

/// `project_variable_add` — copy a parsed variable into the project.
pub fn project_variable_add(variables: &mut Vec<ProjectVariable>, v: TomlProjectVariable) {
    let value = match v.ty {
        ProjectVariableType::Int => {
            VariableValue::Int(v.value.as_integer().unwrap_or(0))
        }
        ProjectVariableType::Float => {
            VariableValue::Float(v.value.as_float().unwrap_or(0.0))
        }
        ProjectVariableType::String => {
            VariableValue::String(v.value.as_str().unwrap_or("").to_string())
        }
    };
    let mut var = ProjectVariable {
        name: v.name,
        ty: v.ty,
        value,
        subtype: v.subtype,
        description: String::new(),
    };
    if !v.description.is_empty() {
        var.description = v.description;
    }
    variables.push(var);
}

/// `BlenderProject` — the fields the config touches.
#[derive(Default)]
pub struct BlenderProject {
    pub name: String,
    pub root_path: String,
    pub ocio_config_path: String,
    pub splash_image_path: String,
    pub variables: Vec<ProjectVariable>,
    pub is_dirty: bool,
}

/// `project_asset_library_add` — map the parsed library onto a
/// `bUserAssetLibrary` flag/name/path set (returned for the caller to
/// store in preferences).
pub struct ParsedAssetLibrary {
    pub name: String,
    pub import_method: i32,
    pub flag: i32,
    /// Remote url or local dirpath.
    pub location: String,
    pub auth_token: Option<String>,
    pub uuid: Option<String>,
    /// Preserved invalid uuid text for UI notification.
    pub invalid_uuid: Option<String>,
}

/// `project_asset_library_add`.
pub fn project_asset_library_add(lib: &TomlAssetLibrary) -> ParsedAssetLibrary {
    if lib.is_remote_asset_lib {
        let flag = ASSET_LIBRARY_USE_REMOTE_URL
            | ASSET_LIBRARY_PROJECT_DEFINED
            | if lib.access_token.is_empty() {
                0
            } else {
                ASSET_LIBRARY_USE_AUTH_TOKEN
            };
        return ParsedAssetLibrary {
            name: lib.name.clone(),
            import_method: lib.import_method,
            flag,
            location: lib.url.clone(),
            auth_token: if lib.access_token.is_empty() {
                None
            } else {
                Some(lib.access_token.clone())
            },
            uuid: None,
            invalid_uuid: None,
        };
    }

    let mut flag = ASSET_LIBRARY_PROJECT_DEFINED;
    /* User passed an invalid UUID → disable the library, keep the
     * string for a UI notice. */
    let invalid_uuid = if !lib.uuid.is_empty() && !is_valid_uuid(&lib.uuid) {
        flag |= ASSET_LIBRARY_DISABLED;
        Some(lib.uuid.clone())
    } else {
        None
    };
    if lib.use_relative_path {
        flag |= ASSET_LIBRARY_RELATIVE_PATH;
    }
    ParsedAssetLibrary {
        name: lib.name.clone(),
        import_method: lib.import_method,
        flag,
        location: lib.path.clone(),
        auth_token: None,
        uuid: if lib.uuid.is_empty() {
            None
        } else {
            Some(lib.uuid.clone())
        },
        invalid_uuid,
    }
}

/// `UUID` validity — `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` hex.
fn is_valid_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && [8usize, 4, 4, 4, 12]
            .iter()
            .zip(parts.iter())
            .all(|(&len, p)| p.len() == len && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// `create_blender_project_from_toml` — parse + apply. Returns the
/// project and its project-defined asset libraries.
pub fn create_blender_project_from_toml(
    toml_path: &str,
    project_root_path: &str,
) -> Result<(BlenderProject, Vec<ParsedAssetLibrary>), String> {
    let text = std::fs::read_to_string(toml_path)
        .map_err(|e| format!("Failed to open file: {toml_path}: {e}"))?;
    let config_values: toml::Value =
        toml::from_str(&text).map_err(|e| format!("Failed to parse {toml_path}: {e}"))?;

    let version = config_values
        .get("schema_version")
        .and_then(|v| v.as_integer())
        .ok_or("missing schema_version")?;
    if version > PROJECT_SCHEMA_VERSION {
        return Err(format!(
            "expected a value equal or below: {PROJECT_SCHEMA_VERSION}\""
        ));
    }
    /* Version < PROJECT_SCHEMA_VERSION: versioning code here. */

    let name = config_values
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Err("Couldn't initialize the project because the project name was empty".into());
    }

    let mut project = BlenderProject {
        name,
        root_path: project_root_path.to_string(),
        ..Default::default()
    };
    let mut libraries = Vec::new();

    if let Some(ocio) = config_values.get("ocio_config").and_then(|v| v.as_str()) {
        project.ocio_config_path = ocio.to_string();
    }

    if let Some(toml::Value::Array(vars)) = config_values.get("variables") {
        for variable in vars {
            let project_variable = TomlProjectVariable::from_toml(variable)?;
            project_variable_add(&mut project.variables, project_variable);
        }
    }

    if let Some(toml::Value::Array(libs)) = config_values.get("asset_libraries") {
        for library in libs {
            let project_asset_library = TomlAssetLibrary::from_toml(library)?;
            libraries.push(project_asset_library_add(&project_asset_library));
        }
    }

    if let Some(splash) = config_values.get("splash_image").and_then(|v| v.as_str()) {
        project.splash_image_path = splash.to_string();
    }

    /* Done loading, mark project as non dirty. */
    project.is_dirty = false;
    Ok((project, libraries))
}

/// `project_variables_to_toml`.
pub fn project_variables_to_toml(project: &BlenderProject) -> Vec<toml::Value> {
    let mut toml_array = Vec::new();
    for project_variable in &project.variables {
        let mut variable_data: BTreeMap<String, toml::Value> = BTreeMap::new();
        variable_data.insert("name".into(), toml::Value::String(project_variable.name.clone()));
        variable_data.insert(
            "type".into(),
            toml::Value::String(project_variable_type_identifier(project_variable.ty).into()),
        );
        match &project_variable.value {
            VariableValue::Int(v) => {
                variable_data.insert("value".into(), toml::Value::Integer(*v));
            }
            VariableValue::Float(v) => {
                variable_data.insert("value".into(), toml::Value::Float(*v));
            }
            VariableValue::String(s) => {
                variable_data.insert("value".into(), toml::Value::String(s.clone()));
                variable_data.insert(
                    "subtype".into(),
                    toml::Value::String(
                        project_variable_string_subtype_identifier(project_variable.subtype)
                            .into(),
                    ),
                );
            }
        }
        if !project_variable.description.is_empty() {
            variable_data.insert(
                "description".into(),
                toml::Value::String(project_variable.description.clone()),
            );
        }
        toml_array.push(toml::Value::Table(
            variable_data.into_iter().collect(),
        ));
    }
    toml_array
}

/// `bUserAssetLibrary` — what `project_asset_libraries_to_toml`
/// reads from preferences.
pub struct UserAssetLibrary {
    pub name: String,
    pub flag: i32,
    pub import_method: i32,
    /// Remote URL or local dirpath.
    pub location: String,
    pub auth_token: Option<String>,
    pub uuid: String,
}

/// `project_asset_libraries_to_toml` — only project-defined
/// libraries.
pub fn project_asset_libraries_to_toml(
    user_libraries: &[UserAssetLibrary],
) -> Vec<toml::Value> {
    let mut toml_array = Vec::new();
    for user_library in user_libraries {
        if user_library.flag & ASSET_LIBRARY_PROJECT_DEFINED == 0 {
            continue;
        }
        let import_method =
            asset_import_method_identifier(user_library.import_method).unwrap_or("APPEND");
        let mut asset_library_data: BTreeMap<String, toml::Value> = BTreeMap::new();
        asset_library_data.insert("name".into(), toml::Value::String(user_library.name.clone()));
        asset_library_data.insert(
            "import_method".into(),
            toml::Value::String(import_method.into()),
        );
        if user_library.flag & ASSET_LIBRARY_USE_REMOTE_URL != 0 {
            /* remote asset library */
            asset_library_data
                .insert("url".into(), toml::Value::String(user_library.location.clone()));
            if user_library.flag & ASSET_LIBRARY_USE_AUTH_TOKEN != 0 {
                if let Some(token) = &user_library.auth_token {
                    asset_library_data
                        .insert("access_token".into(), toml::Value::String(token.clone()));
                }
            }
        } else {
            /* Local asset library */
            asset_library_data
                .insert("path".into(), toml::Value::String(user_library.location.clone()));
            asset_library_data.insert(
                "use_relative_path".into(),
                toml::Value::Boolean(user_library.flag & ASSET_LIBRARY_RELATIVE_PATH != 0),
            );
            asset_library_data
                .insert("uuid".into(), toml::Value::String(user_library.uuid.clone()));
        }
        toml_array.push(toml::Value::Table(
            asset_library_data.into_iter().collect(),
        ));
    }
    toml_array
}

/// `create_toml_from_blender_project`.
pub fn create_toml_from_blender_project(
    project: &BlenderProject,
    user_libraries: &[UserAssetLibrary],
) -> toml::Value {
    let mut toml_table: BTreeMap<String, toml::Value> = BTreeMap::new();
    toml_table.insert(
        "schema_version".into(),
        toml::Value::Integer(PROJECT_SCHEMA_VERSION),
    );
    toml_table.insert("name".into(), toml::Value::String(project.name.clone()));

    if !project.ocio_config_path.is_empty() {
        toml_table.insert(
            "ocio_config".into(),
            toml::Value::String(project.ocio_config_path.clone()),
        );
    }

    let variables = project_variables_to_toml(project);
    if !variables.is_empty() {
        toml_table.insert("variables".into(), toml::Value::Array(variables));
    }

    let asset_libraries = project_asset_libraries_to_toml(user_libraries);
    if !asset_libraries.is_empty() {
        toml_table.insert(
            "asset_libraries".into(),
            toml::Value::Array(asset_libraries),
        );
    }

    if !project.splash_image_path.is_empty() {
        toml_table.insert(
            "splash_image".into(),
            toml::Value::String(project.splash_image_path.clone()),
        );
    }

    toml::Value::Table(toml_table.into_iter().collect())
}

/// `BKE_blender_project_config_file_write` — create the config dir
/// and serialize.
pub fn bke_blender_project_config_file_write(
    project: &mut BlenderProject,
    user_libraries: &[UserAssetLibrary],
) -> bool {
    /* Create the directory for new projects. */
    let config_dir = format!("{}/{BLENDER_PROJECT_CONFIG_DIR}", project.root_path);
    if std::fs::create_dir_all(&config_dir).is_err() {
        return false;
    }
    let toml_path = format!("{config_dir}/{BLENDER_PROJECT_CONFIG_FILE}");
    let toml_config = create_toml_from_blender_project(project, user_libraries);
    let Ok(text) = toml::to_string(&toml_config) else {
        return false;
    };
    if std::fs::write(&toml_path, text).is_err() {
        return false;
    }
    project.is_dirty = false;
    true
}

/// `BKE_blender_project_config_autosave` — write when dirty and
/// `USER_PREF_FLAG_PROJECT_SAVE` is set.
pub fn bke_blender_project_config_autosave(
    project: &mut BlenderProject,
    user_pref_flag_project_save: bool,
    user_libraries: &[UserAssetLibrary],
) {
    if !user_pref_flag_project_save {
        return;
    }
    if project.is_dirty {
        bke_blender_project_config_file_write(project, user_libraries);
    }
}

/// `BKE_blender_project_init_from_root_path` — validate dirs + parse.
pub fn bke_blender_project_init_from_root_path(
    project_root_path: &str,
) -> Result<(BlenderProject, Vec<ParsedAssetLibrary>), String> {
    if !std::path::Path::new(project_root_path).is_dir() {
        return Err(format!(
            "Project root path '{project_root_path}', is not a directory"
        ));
    }
    let toml_path = format!(
        "{project_root_path}/{BLENDER_PROJECT_CONFIG_DIR}/{BLENDER_PROJECT_CONFIG_FILE}"
    );
    if !std::path::Path::new(&toml_path).is_file() {
        return Err("Project has no project.toml file".into());
    }
    create_blender_project_from_toml(&toml_path, project_root_path)
}
