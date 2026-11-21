//! Port of `source/blender/blenkernel/intern/volume_grid_file_cache.cc`.
//!
//! Global file→grids cache. OpenVDB types are opaque `usize` handles
//! and all VDB I/O (file open, metadata read, grid read, simplify)
//! is injected — `WITH_OPENVDB` (`PORT_PENDING` until `volume_openvdb`
//! is ported).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// `openvdb::GridBase::Ptr` — opaque grid handle.
pub type GridPtr = usize;
/// `openvdb::MetaMap` — opaque file-metadata handle.
pub type MetaMap = usize;
/// `GVolumeGrid` — shared grid wrapper (`VolumeGridData` handle).
pub type GVolumeGrid = usize;
/// `LazyLoadedGrid` — grid + tree-sharing-info pair.
pub type LazyLoadedGrid = (GridPtr, usize);

/// `NIL` handle.
pub const NIL_GRID: usize = usize::MAX;

/// `GridCache` — cache for a single grid stored in a file.
#[derive(Default)]
pub struct GridCache {
    /// Grid returned by `readAllGridMetadata` — meta-data and
    /// transform only, not the tree.
    pub meta_data_grid: GridPtr,
    /// Cached simplify levels.
    pub grid_by_simplify_level: HashMap<i32, GVolumeGrid>,
    /// `meta_data_grid->getName()`.
    pub name: String,
}

/// `FileCache` — cache for a file that contains potentially multiple
/// grids.
#[derive(Default)]
pub struct FileCache {
    /// Empty on success, otherwise an error message generated when
    /// trying to load the file.
    pub error_message: String,
    /// Meta data of the file (not of an individual grid).
    pub meta_data: MetaMap,
    /// Caches for grids in the same order they are stored in the file.
    pub grids: Vec<GridCache>,
}

impl FileCache {
    /// `grid_cache_by_name`.
    pub fn grid_cache_by_name(&mut self, name: &str) -> Option<&mut GridCache> {
        self.grids.iter_mut().find(|g| g.name == name)
    }
}

/// `GlobalCache` — singleton cache shared application-wide.
#[derive(Default)]
pub struct GlobalCache {
    pub file_map: HashMap<String, FileCache>,
}

static GLOBAL_CACHE: OnceLock<Mutex<GlobalCache>> = OnceLock::new();

/// `get_global_cache` — construct-on-first-use idiom.
pub fn get_global_cache() -> &'static Mutex<GlobalCache> {
    GLOBAL_CACHE.get_or_init(|| Mutex::new(GlobalCache::default()))
}

/// `create_file_cache` — read meta-data of the file (not trees).
/// `open_fn` performs `openvdb::io::File::open + readAllGridMetadata
/// + getMetadata` → `(meta_map, [(name, meta_grid)])` or an error
/// message.
pub fn create_file_cache(
    file_path: &str,
    open_fn: &dyn Fn(&str) -> Result<(MetaMap, Vec<(String, GridPtr)>), String>,
) -> FileCache {
    let mut file_cache = FileCache::default();
    match open_fn(file_path) {
        Ok((meta, vdb_grids)) => {
            file_cache.meta_data = meta;
            for (name, vdb_grid) in vdb_grids {
                if vdb_grid == NIL_GRID {
                    continue;
                }
                file_cache.grids.push(GridCache {
                    meta_data_grid: vdb_grid,
                    name,
                    ..Default::default()
                });
            }
        }
        Err(e) => {
            file_cache.error_message = e;
        }
    }
    file_cache
}

/// `GridReadKey` — identifies a grid in the global memory cache.
#[derive(Hash, PartialEq, Eq)]
pub struct GridReadKey {
    pub file_path: String,
    pub grid_name: String,
    pub simplify_level: i32,
}

/// `get_file_cache` — assumes `global_cache` lock is held.
pub fn get_file_cache<'a>(
    global_cache: &'a mut GlobalCache,
    file_path: &str,
    open_fn: &dyn Fn(&str) -> Result<(MetaMap, Vec<(String, GridPtr)>), String>,
) -> &'a mut FileCache {
    global_cache
        .file_map
        .entry(file_path.to_string())
        .or_insert_with(|| create_file_cache(file_path, open_fn))
}

/// `get_grid_from_file` — load a named grid (meta-data now, tree
/// on-demand via `make_grid`).
///
/// `make_grid` wraps `MEM_new<VolumeGridData>(load_grid_fn,
/// meta_data_and_transform_grid)` and `main_grid_of` resolves a
/// simplify-level-0 grid for `BKE_volume_grid_create_with_changed_
/// resolution` — both injected.
pub fn get_grid_from_file(
    file_path: &str,
    grid_name: &str,
    simplify_level: i32,
    open_fn: &dyn Fn(&str) -> Result<(MetaMap, Vec<(String, GridPtr)>), String>,
    load_grid_fn: &dyn Fn(&str, &str, i32) -> Option<LazyLoadedGrid>,
    make_grid: &dyn Fn(&dyn Fn() -> Option<LazyLoadedGrid>, GridPtr) -> GVolumeGrid,
) -> GVolumeGrid {
    let mut cache = get_global_cache().lock().unwrap();
    let file_cache = get_file_cache(&mut cache, file_path, open_fn);
    if let Some(grid_cache) = file_cache.grid_cache_by_name(grid_name) {
        return get_cached_grid(
            file_path,
            grid_cache,
            simplify_level,
            load_grid_fn,
            make_grid,
        );
    }
    NIL_GRID
}

/// `get_cached_grid` — per-simplify-level grid with lazy tree.
fn get_cached_grid(
    file_path: &str,
    grid_cache: &mut GridCache,
    simplify_level: i32,
    load_grid_fn: &dyn Fn(&str, &str, i32) -> Option<LazyLoadedGrid>,
    make_grid: &dyn Fn(&dyn Fn() -> Option<LazyLoadedGrid>, GridPtr) -> GVolumeGrid,
) -> GVolumeGrid {
    if let Some(&grid) = grid_cache.grid_by_simplify_level.get(&simplify_level) {
        return grid;
    }
    /* A callback that actually loads the full grid including the
     * tree when it's accessed. */
    let file_path = file_path.to_string();
    let grid_name = grid_cache.name.clone();
    let load = move || -> Option<LazyLoadedGrid> {
        load_grid_fn(&file_path, &grid_name, simplify_level)
    };
    /* This allows the returned grid to already contain meta-data and
     * transforms, even if the tree is not loaded yet. */
    let meta_data_and_transform_grid = if simplify_level == 0 {
        /* Only pass the meta-data grid when there is no
         * simplification for now. For simplified grids, the transform
         * would have to be updated here already. */
        grid_cache.meta_data_grid
    } else {
        NIL_GRID
    };
    let grid = make_grid(&load, meta_data_and_transform_grid);
    grid_cache.grid_by_simplify_level.insert(simplify_level, grid);
    grid
}

/// `GridsFromFile` — `get_all_grids_from_file` result.
#[derive(Default)]
pub struct GridsFromFile {
    pub error_message: String,
    /// `std::shared_ptr<openvdb::MetaMap>` handle.
    pub file_meta_data: MetaMap,
    pub grids: Vec<GVolumeGrid>,
}

/// `get_all_grids_from_file`.
pub fn get_all_grids_from_file(
    file_path: &str,
    simplify_level: i32,
    open_fn: &dyn Fn(&str) -> Result<(MetaMap, Vec<(String, GridPtr)>), String>,
    load_grid_fn: &dyn Fn(&str, &str, i32) -> Option<LazyLoadedGrid>,
    make_grid: &dyn Fn(&dyn Fn() -> Option<LazyLoadedGrid>, GridPtr) -> GVolumeGrid,
) -> GridsFromFile {
    let mut result = GridsFromFile::default();
    let mut cache = get_global_cache().lock().unwrap();
    let file_cache = get_file_cache(&mut cache, file_path, open_fn);

    if !file_cache.error_message.is_empty() {
        result.error_message = file_cache.error_message.clone();
        return result;
    }
    result.file_meta_data = file_cache.meta_data;
    for grid_cache in file_cache.grids.iter_mut() {
        let g = get_cached_grid(
            file_path,
            grid_cache,
            simplify_level,
            load_grid_fn,
            make_grid,
        );
        result.grids.push(g);
    }
    result
}

/// `unload_unused` — drop simplify-level grids whose data is mutable
/// (i.e. uniquely owned → evictable).
pub fn unload_unused(is_mutable: &dyn Fn(GVolumeGrid) -> bool) {
    let mut cache = get_global_cache().lock().unwrap();
    for file_cache in cache.file_map.values_mut() {
        for grid_cache in file_cache.grids.iter_mut() {
            grid_cache
                .grid_by_simplify_level
                .retain(|_, grid| !is_mutable(*grid));
        }
    }
}

/* PORT_PENDING: `openvdb::io::File`, `GridBase` copy/transform,
 * `OpenvdbTreeSharingInfo`, `BKE_volume_grid_create_with_changed_
 * resolution`, `VolumeGridData` construction — all injected above
 * until `volume_openvdb.cc` and the VDB backend arrive. */
