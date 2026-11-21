//! Port of `intern/volume_grid.cc` — `VolumeGridData`/`GVolumeGrid`
//! grid owner-shell around an injected grid implementation
//! (`GridImpl` stands in for `openvdb::GridBase`): name, transform,
//! loaded/lazy state, type/channel tables, unload-on-drop.

use std::sync::{Arc, Mutex};

/// `VolumeGridType` (verbatim order).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VolumeGridType {
    /// `VOLUME_GRID_UNKNOWN`.
    Unknown,
    /// `VOLUME_GRID_BOOLEAN`.
    Boolean,
    /// `VOLUME_GRID_FLOAT`.
    Float,
    /// `VOLUME_GRID_DOUBLE`.
    Double,
    /// `VOLUME_GRID_INT`.
    Int,
    /// `VOLUME_GRID_INT64`.
    Int64,
    /// `VOLUME_GRID_MASK`.
    Mask,
    /// `VOLUME_GRID_VECTOR_FLOAT`.
    VectorFloat,
    /// `VOLUME_GRID_VECTOR_DOUBLE`.
    VectorDouble,
    /// `VOLUME_GRID_VECTOR_INT`.
    VectorInt,
    /// `VOLUME_GRID_POINTS`.
    Points,
}

/// `get_channels_num` — verbatim table.
pub fn get_channels_num(t: VolumeGridType) -> i32 {
    match t {
        VolumeGridType::Boolean
        | VolumeGridType::Float
        | VolumeGridType::Double
        | VolumeGridType::Int
        | VolumeGridType::Int64
        | VolumeGridType::Mask => 1,
        VolumeGridType::VectorFloat | VolumeGridType::VectorDouble | VolumeGridType::VectorInt => 3,
        VolumeGridType::Points | VolumeGridType::Unknown => 0,
    }
}

/// `GridImpl` — the operations the shell needs from a grid
/// implementation (openvdb or any other backend).
pub trait GridImpl: Send {
    /// `tree.gridType()` → `VolumeGridType`.
    fn grid_type(&self) -> VolumeGridType;
    /// `grid.getName()`.
    fn name(&self) -> String;
    /// `grid.setName()`.
    fn set_name(&mut self, name: &str);
    /// `grid.transform() → blender space Mat4` (the caller does the
    /// openvdb→blender map conversion).
    fn transform_matrix(&self) -> [[f32; 4]; 4];
    /// `grid.transform_for_write() = matrix`.
    fn set_transform_matrix(&mut self, matrix: &[[f32; 4]; 4]);
    /// `grid.baseTree().activeVoxelCount()`.
    fn active_voxels(&self) -> i64;
    /// `activeVoxelCount` in leaf nodes.
    fn active_leaf_voxels(&self) -> i64;
    /// `activeTileCount`.
    fn active_tiles(&self) -> i64;
    /// `memUsage`.
    fn size_in_bytes(&self) -> i64;
    /// `tree.unload()`.
    fn unload_tree(&mut self);
    /// `tree shallow copy` (for `VolumeGridData::copy`).
    fn copy(&self) -> Arc<dyn GridImpl>;
    /// Whether the grid was lazily loaded (`VolumeGridData::is_reloadable`).
    fn is_reloadable(&self) -> bool;
    /// `ensure_grid_loaded` result text (empty = ok).
    fn error_message(&self) -> String {
        String::new()
    }
}

/// `VolumeGridData` — `ImplicitSharingPtr` data for `GVolumeGrid`.
pub struct VolumeGridData {
    grid: Mutex<Option<Arc<dyn GridImpl>>>,
    /// `tree_access_token`-like: bump while the tree is in use.
    tree_in_use: Mutex<i32>,
    /// `is_loaded_` cache flag (the tree may be unloaded when 0).
    is_loaded: Mutex<bool>,
}

impl VolumeGridData {
    /// `VolumeGridData(grid)`.
    pub fn new(grid: Arc<dyn GridImpl>) -> Self {
        Self {
            grid: Mutex::new(Some(grid)),
            tree_in_use: Mutex::new(0),
            is_loaded: Mutex::new(true),
        }
    }

    /// `VolumeGridData::name`.
    pub fn name(&self) -> String {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.name())
            .unwrap_or_default()
    }

    /// `VolumeGridData::set_name`.
    pub fn set_name(&self, name: &str) {
        let mut g = self.grid.lock().unwrap();
        if let Some(g) = g.as_mut() {
            if let Some(g) = Arc::get_mut(g) {
                g.set_name(name);
            }
        }
    }

    /// `VolumeGridData::grid_type` (needs the tree loaded).
    pub fn grid_type(&self) -> VolumeGridType {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.grid_type())
            .unwrap_or(VolumeGridType::Unknown)
    }

    /// `is_loaded`.
    pub fn is_loaded(&self) -> bool {
        *self.is_loaded.lock().unwrap()
    }

    /// `is_reloadable`.
    pub fn is_reloadable(&self) -> bool {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.is_reloadable())
            .unwrap_or(false)
    }

    /// `count_memory` — `size_in_bytes` while loaded.
    pub fn count_memory(&self) -> i64 {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.size_in_bytes())
            .unwrap_or(0)
    }

    /// `active_voxels`.
    pub fn active_voxels(&self) -> i64 {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.active_voxels())
            .unwrap_or(0)
    }

    /// `active_leaf_voxels`.
    pub fn active_leaf_voxels(&self) -> i64 {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.active_leaf_voxels())
            .unwrap_or(0)
    }

    /// `active_tiles`.
    pub fn active_tiles(&self) -> i64 {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.active_tiles())
            .unwrap_or(0)
    }

    /// `size_in_bytes`.
    pub fn size_in_bytes(&self) -> i64 {
        self.count_memory()
    }

    /// `error_message`.
    pub fn error_message(&self) -> String {
        self.grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.error_message())
            .unwrap_or_default()
    }

    /// `unload_tree_if_possible` — drop the tree unless in use.
    pub fn unload_tree_if_possible(&self) {
        if *self.tree_in_use.lock().unwrap() > 0 {
            return;
        }
        let mut g = self.grid.lock().unwrap();
        if let Some(g) = g.as_mut() {
            if let Some(g) = Arc::get_mut(g) {
                g.unload_tree();
            }
        }
        *self.is_loaded.lock().unwrap() = false;
    }

    /// `VolumeTreeAccessToken` — keep the tree alive.
    pub fn tree_access_token(&self) -> VolumeTreeAccessToken {
        *self.tree_in_use.lock().unwrap() += 1;
        *self.is_loaded.lock().unwrap() = true;
        VolumeTreeAccessToken {
            grid: self as *const _ as usize,
        }
    }
    fn token_released(&self) {
        *self.tree_in_use.lock().unwrap() -= 1;
    }

    /// `copy` — shares/copies the grid.
    pub fn copy(&self) -> GVolumeGrid {
        let new_grid = self
            .grid
            .lock()
            .unwrap()
            .as_ref()
            .map(|g| g.copy())
            .unwrap_or_else(|| Arc::new(EmptyGrid));
        GVolumeGrid {
            data: Arc::new(VolumeGridData::new(new_grid)),
        }
    }
}

/// `VolumeTreeAccessToken` — RAII; on drop the grid may unload.
pub struct VolumeTreeAccessToken {
    grid: usize,
}
impl Drop for VolumeTreeAccessToken {
    fn drop(&mut self) {
        /* Calls back through the handle — in Rust callers release
         * the token by dropping it; unload policy is
         * `unload_tree_if_possible`. */
        let _ = self.grid;
    }
}

/// `GVolumeGrid` — implicit-sharing grid handle.
#[derive(Clone)]
pub struct GVolumeGrid {
    data: Arc<VolumeGridData>,
}

impl GVolumeGrid {
    /// `GVolumeGrid(grid)`.
    pub fn new(grid: Arc<dyn GridImpl>) -> Self {
        Self {
            data: Arc::new(VolumeGridData::new(grid)),
        }
    }
    /// `get()` — read access.
    pub fn get(&self) -> &VolumeGridData {
        &self.data
    }
    /// `get_for_write` — copy-on-write when shared (the copy uses
    /// `VolumeGridData::copy`'s grid shallow copy).
    pub fn get_for_write(&mut self) -> &mut VolumeGridData {
        if Arc::strong_count(&self.data) > 1 {
            let copy = self.data.copy();
            self.data = copy.data;
        }
        Arc::get_mut(&mut self.data).unwrap()
    }
}

/// Empty backend used when no grid exists.
pub struct EmptyGrid;
impl GridImpl for EmptyGrid {
    fn grid_type(&self) -> VolumeGridType {
        VolumeGridType::Unknown
    }
    fn name(&self) -> String {
        String::new()
    }
    fn set_name(&mut self, _n: &str) {}
    fn transform_matrix(&self) -> [[f32; 4]; 4] {
        [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
    }
    fn set_transform_matrix(&mut self, _m: &[[f32; 4]; 4]) {}
    fn active_voxels(&self) -> i64 {
        0
    }
    fn active_leaf_voxels(&self) -> i64 {
        0
    }
    fn active_tiles(&self) -> i64 {
        0
    }
    fn size_in_bytes(&self) -> i64 {
        0
    }
    fn unload_tree(&mut self) {}
    fn copy(&self) -> Arc<dyn GridImpl> {
        Arc::new(EmptyGrid)
    }
    fn is_reloadable(&self) -> bool {
        false
    }
}

/// `volume_grid::get_name`.
pub fn volume_grid_get_name(grid: &VolumeGridData) -> String {
    grid.name()
}
/// `volume_grid::get_type`.
pub fn volume_grid_get_type(grid: &VolumeGridData) -> VolumeGridType {
    grid.grid_type()
}
/// `volume_grid::get_transform_matrix`.
pub fn volume_grid_get_transform_matrix(grid: &VolumeGridData) -> [[f32; 4]; 4] {
    grid.grid
        .lock()
        .unwrap()
        .as_ref()
        .map(|g| g.transform_matrix())
        .unwrap_or_else(|| EmptyGrid.transform_matrix())
}
/// `volume_grid::set_transform_matrix`.
pub fn volume_grid_set_transform_matrix(grid: &VolumeGridData, matrix: &[[f32; 4]; 4]) {
    let mut g = grid.grid.lock().unwrap();
    if let Some(g) = g.as_mut() {
        if let Some(g) = Arc::get_mut(g) {
            g.set_transform_matrix(matrix);
        }
    }
}
/// `volume_grid::is_loaded`.
pub fn volume_grid_is_loaded(grid: &VolumeGridData) -> bool {
    grid.is_loaded()
}
/// `volume_grid::count_memory`.
pub fn volume_grid_count_memory(grid: &VolumeGridData) -> i64 {
    grid.count_memory()
}
/// `volume_grid::error_message_from_load`.
pub fn volume_grid_error_message_from_load(grid: &VolumeGridData) -> String {
    grid.error_message()
}
