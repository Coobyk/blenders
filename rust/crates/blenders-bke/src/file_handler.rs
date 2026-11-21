//! Port of `intern/file_handler.cc` — `bke::file_handler_*` (drag-drop file
//! handler registry).

use std::sync::Mutex;

use blenders_bli::path_utils;
use blenders_bli::string::strcaseeq;

/// `FileHandlerType` (BKE_file_handler.hh) — `poll_drop` is an optional
/// injected predicate (the `bContext` arg stays an opaque `usize`).
pub struct FileHandlerType {
    pub idname: String,
    /// `file_extensions_str` — raw `;`-joined source string.
    pub file_extensions_str: String,
    /// `file_extensions` — parsed by `file_handler_add`.
    pub file_extensions: Vec<String>,
    /// `poll_drop(C, handler)` — `bContext*` stays `usize`.
    pub poll_drop: Option<Box<dyn FnMut(usize) -> bool + Send>>,
    /// `label` — UI name (`label_with_extensions` suffixes the globs).
    pub label: String,
}

impl FileHandlerType {
    /// `FileHandlerType::filter_supported_paths` — indices into `paths`.
    pub fn filter_supported_paths(&self, paths: &[String]) -> Vec<usize> {
        let mut indices = Vec::new();
        for (idx, path) in paths.iter().enumerate() {
            let Some(extension) = path_utils::extension(path) else {
                continue;
            };
            if self.file_extensions.iter().any(|e| strcaseeq(extension, e)) {
                indices.push(idx);
            }
        }
        indices
    }
    /// `FileHandlerType::get_default_filename` — "Untitled" on blank names.
    pub fn get_default_filename(&self, name: &str) -> String {
        let all_blank = name.is_empty() || name.chars().all(|c| c == ' ');
        let mut filename = if all_blank { "Untitled".to_string() } else { name.to_string() };
        path_utils::extension_ensure(
            &mut filename,
            if self.file_extensions.is_empty() { "" } else { &self.file_extensions[0] },
        );
        filename
    }
}

fn file_handlers_vector() -> &'static Mutex<Vec<FileHandlerType>> {
    static HANDLERS: Mutex<Vec<FileHandlerType>> = Mutex::new(Vec::new());
    &HANDLERS
}

/// `file_handlers` — index-keyed view (the Vec replaces pointer identity).
pub fn file_handlers() -> Vec<String> {
    file_handlers_vector().lock().unwrap().iter().map(|h| h.idname.clone()).collect()
}

/// `file_handler_find` — idname lookup, returns the handler index.
pub fn file_handler_find(idname: &str) -> Option<usize> {
    file_handlers_vector().lock().unwrap().iter().position(|h| h.idname == idname)
}

/// `file_handler_add` — parses `file_extensions_str` on ';' (verbatim: only
/// multi-char segments are kept).
pub fn file_handler_add(mut file_handler: FileHandlerType) {
    let mut handlers = file_handlers_vector().lock().unwrap();
    debug_assert!(!handlers.iter().any(|h| h.idname == file_handler.idname));
    /* C keeps segments longer than 1 char (char_end - char_begin > 1). */
    for part in file_handler.file_extensions_str.split(';') {
        if part.len() > 1 {
            file_handler.file_extensions.push(part.to_string());
        }
    }
    handlers.push(file_handler);
}

/// `file_handler_remove` — by index.
pub fn file_handler_remove(index: usize) {
    file_handlers_vector().lock().unwrap().remove(index);
}

/// `file_handlers_poll_file_drop` — handlers matching the paths' extensions
/// whose `poll_drop` accepts. Returns their indices.
pub fn file_handlers_poll_file_drop(ctx: usize, paths: &[String]) -> Vec<usize> {
    let mut path_extensions: Vec<String> = Vec::new();
    for path in paths {
        if let Some(extension) = path_utils::extension(path) {
            let e = extension.to_string();
            if !path_extensions.contains(&e) {
                path_extensions.push(e);
            }
        }
    }
    let mut result = Vec::new();
    let mut handlers = file_handlers_vector().lock().unwrap();
    for (idx, file_handler) in handlers.iter_mut().enumerate() {
        let support_any_extension = path_extensions.iter().any(|extension| {
            file_handler.file_extensions.iter().any(|e| strcaseeq(extension, e))
        });
        if !support_any_extension {
            continue;
        }
        let Some(poll_drop) = &mut file_handler.poll_drop else {
            continue;
        };
        if !poll_drop(ctx) {
            continue;
        }
        result.push(idx);
    }
    result
}

/// `extensions_common_prefix`.
fn extensions_common_prefix(extensions: &[String]) -> String {
    let mut prefix = extensions[0].clone();
    for extension in &extensions[1..] {
        let max_len = prefix.len().min(extension.len());
        let mut i = 0;
        while i < max_len && prefix.as_bytes()[i] == extension.as_bytes()[i] {
            i += 1;
        }
        prefix.truncate(i);
    }
    prefix
}

/// `extensions_group_is_collapsible` — shared prefix of >= 3 letters and each
/// extension at most one char longer.
fn extensions_group_is_collapsible(extensions: &[String]) -> bool {
    let prefix = extensions_common_prefix(extensions);
    let prefix_letters = prefix.len() - usize::from(prefix.starts_with('.'));
    prefix_letters >= 3 && extensions.iter().all(|e| e.len() <= prefix.len() + 1)
}

impl FileHandlerType {
    /// `FileHandlerType::label_with_extensions` — "Label (.ext/.pr*)" style.
    pub fn label_with_extensions(&self) -> String {
        if self.file_extensions.is_empty() {
            return self.label.clone();
        }
        let all = &self.file_extensions;
        let mut extensions = String::new();
        let mut group_start = 0usize;
        while group_start < all.len() {
            let mut group_size = 1usize;
            while group_start + group_size < all.len()
                && extensions_group_is_collapsible(&all[group_start..group_start + group_size + 1])
            {
                group_size += 1;
            }
            let group = &all[group_start..group_start + group_size];
            if !extensions.is_empty() {
                extensions.push('/');
            }
            if group.len() > 1 {
                extensions.push_str(&extensions_common_prefix(group));
                extensions.push('*');
            } else {
                extensions.push_str(&group[0]);
            }
            group_start += group_size;
        }
        format!("{} ({})", self.label, extensions)
    }

    /// `FileHandlerType::filter_glob_from_extensions` — "*.ext;*.ext2".
    pub fn filter_glob_from_extensions(&self) -> Option<String> {
        if self.file_extensions.is_empty() {
            return None;
        }
        let mut filter_glob = String::new();
        let mut is_first = true;
        for extension in &self.file_extensions {
            if !is_first {
                filter_glob.push(';');
            }
            filter_glob.push_str(if extension.starts_with('.') { "*" } else { "*." });
            filter_glob.push_str(extension);
            is_first = false;
        }
        Some(filter_glob)
    }
}
