//! Port of `BLI_resource_scope.hh` + `resource_scope.cc` —
//! `blender::ResourceScope`: owns resources freed in reverse order when the
//! scope destructs. C stores `void*` + free fn in a `LinearAllocator` list;
//! here boxed `FnOnce` destructors give the same lifetime semantics.

/// `ResourceScope::ResourceData` — one resource + its free function.
pub struct ResourceData {
    /// `free` closure that releases the resource.
    free: Box<dyn FnOnce() + Send>,
}

/// `ResourceScope`.
pub struct ResourceScope {
    resources: Vec<ResourceData>,
}

impl ResourceScope {
    /// `ResourceScope()` — empty scope.
    pub fn new() -> Self {
        ResourceScope {
            resources: Vec::new(),
        }
    }
    /// `ResourceScope(initial_size)` — pre-reserves capacity like the C
    /// initial-allocator hint.
    pub fn with_capacity(initial_size: usize) -> Self {
        ResourceScope {
            resources: Vec::with_capacity(initial_size),
        }
    }
    /// `scope.add(data, free)` — register `free()` to run on destruct.
    pub fn add<F: FnOnce() + Send + 'static>(&mut self, free: F) {
        self.resources.push(ResourceData {
            free: Box::new(free),
        });
    }
    /// `scope.add_owned(value)` — drop `value` when the scope destructs.
    pub fn add_owned<T: Send + 'static>(&mut self, value: T) {
        self.resources.push(ResourceData {
            free: Box::new(move || drop(value)),
        });
    }
    /// `scope.release_as()`-style: hand out the list of resources.
    pub fn len(&self) -> usize {
        self.resources.len()
    }
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

impl Default for ResourceScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ResourceScope {
    /// `~ResourceScope` — resources freed in reverse order of registration.
    fn drop(&mut self) {
        while let Some(res) = self.resources.pop() {
            (res.free)();
        }
    }
}
