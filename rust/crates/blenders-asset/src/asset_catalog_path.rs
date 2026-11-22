//! Port of `intern/asset_catalog_path.cc` — `/`-joined
//! catalog path with cleanup/rebase/parent ops.

/// `AssetCatalogPath::SEPARATOR` verbatim.
pub const SEPARATOR: char = '/';

/// `AssetCatalogPath` verbatim.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetCatalogPath {
    /// `path_`.
    pub path: String,
}
impl AssetCatalogPath {
    /// ctor.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
    /// `length`.
    pub fn length(&self) -> usize {
        self.path.len()
    }
    /// `is_empty` (operator bool inverted).
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
    /// `name` (verbatim — after last '/').
    pub fn name(&self) -> &str {
        match self.path.rfind(SEPARATOR) {
            None => &self.path,
            Some(i) => &self.path[i + 1..],
        }
    }
    /// `operator/` (verbatim — join with '/' unless
    /// either empty).
    pub fn join(&self, other: &AssetCatalogPath) -> AssetCatalogPath {
        if self.path.is_empty() {
            return other.clone();
        }
        if other.path.is_empty() {
            return self.clone();
        }
        AssetCatalogPath::new(format!("{}{}{}", self.path, SEPARATOR, other.path))
    }
    /// `cleanup_component` (verbatim — trim + ':'→'-').
    pub fn cleanup_component(component_name: &str) -> String {
        component_name.trim().replace(':', "-")
    }
    /// `iterate_components` (verbatim — '/' or '\' sep,
    /// yields (component, is_last)).
    pub fn iterate_components(&self, mut callback: impl FnMut(&str, bool)) {
        let bytes = self.path.as_bytes();
        let mut start = 0usize;
        loop {
            if start >= bytes.len() {
                break;
            }
            let mut i = start;
            while i < bytes.len() && bytes[i] != b'/' && bytes[i] != b'\\' {
                i += 1;
            }
            let is_last = i >= bytes.len();
            callback(&self.path[start..i], is_last);
            if is_last {
                break;
            }
            start = i + 1;
        }
    }
    /// `cleanup` (verbatim — drop empty comps, rejoin).
    pub fn cleanup(&self) -> AssetCatalogPath {
        let mut cleaned = String::new();
        let mut first = true;
        self.iterate_components(|component, _is_last| {
            let c = Self::cleanup_component(component);
            if c.is_empty() {
                return;
            }
            if !first {
                cleaned.push(SEPARATOR);
            }
            first = false;
            cleaned.push_str(&c);
        });
        AssetCatalogPath::new(cleaned)
    }
    /// `from_user_input` (verbatim — cleanup()).
    pub fn from_user_input(path: &str) -> AssetCatalogPath {
        AssetCatalogPath::new(path).cleanup()
    }
    /// `is_contained_in` (verbatim — empty→all, equal,
    /// prefix + '/' boundary, ≥len+2).
    pub fn is_contained_in(&self, other: &AssetCatalogPath) -> bool {
        if other.path.is_empty() {
            return true;
        }
        if self.path == other.path {
            return true;
        }
        if self.path.len() < other.path.len() + 2 {
            return false;
        }
        self.path.starts_with(&other.path)
            && self.path.as_bytes()[other.path.len()] == SEPARATOR as u8
    }
    /// `parent` (verbatim — strip last component).
    pub fn parent(&self) -> AssetCatalogPath {
        if self.path.is_empty() {
            return AssetCatalogPath::new("");
        }
        match self.path.rfind(SEPARATOR) {
            None => AssetCatalogPath::new(""),
            Some(i) => AssetCatalogPath::new(&self.path[..i]),
        }
    }
    /// `rebase` (verbatim — from-contained → to/suffix).
    pub fn rebase(&self, from_path: &AssetCatalogPath, to_path: &AssetCatalogPath) -> AssetCatalogPath {
        if from_path.path.is_empty() {
            if to_path.path.is_empty() {
                return AssetCatalogPath::new("");
            }
            return to_path.join(self);
        }
        if !self.is_contained_in(from_path) {
            return AssetCatalogPath::new("");
        }
        if self == from_path {
            return to_path.clone();
        }
        let suffix = &self.path[from_path.length() + 1..];
        to_path.join(&AssetCatalogPath::new(suffix))
    }
}
