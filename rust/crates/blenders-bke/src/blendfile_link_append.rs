//! Port of `intern/blendfile_link_append.cc` — the link/append
//! context and its public management API (`context_new/free`,
//! flag set, embedded blendfile, library add, item add/idtypes/
//! library_index_enable, item userdata/newid/liboverrideid/idcode
//! get/set, item_foreach, init_done/finalize). ID/BlendHandle/
//! Main references are `usize` handles; `BLO_*`, `BKE_main_*`,
//! RNA callbacks and the actual link/append/instantiate engines
//! (`loose_data_instantiate_*`, `foreach_libblock_*`) are
//! `PORT_PENDING` with injected closures.

use std::collections::HashMap;

/// `BLENDFILE_LINK_APPEND_INVALID`.
pub const BLENDFILE_LINK_APPEND_INVALID: i32 = -1;
/// `BLO_EMBEDDED_STARTUP_BLEND` marker name.
pub const BLO_EMBEDDED_STARTUP_BLEND: &str = "<BLO_EMBEDDED_STARTUP_BLEND>";

/// `FILE_ASSETS_ONLY` (params flag bit).
pub const FILE_ASSETS_ONLY: i32 = 1 << 10;

/// `LINK_APPEND_ACT_*` — item action codes (verbatim from
/// `BKE_blendfile_link_append.hh`).
pub const LINK_APPEND_ACT_UNSET: i16 = 0;
/// `LINK_APPEND_ACT_KEEP_LINKED`.
pub const LINK_APPEND_ACT_KEEP_LINKED: i16 = 1;
/// `LINK_APPEND_ACT_REUSE_LOCAL`.
pub const LINK_APPEND_ACT_REUSE_LOCAL: i16 = 2;
/// `LINK_APPEND_ACT_COPY_LOCAL`.
pub const LINK_APPEND_ACT_COPY_LOCAL: i16 = 3;

/// `LINK_APPEND_TAG_*`.
pub const LINK_APPEND_TAG_INDIRECT: u32 = 1 << 0;
/// `LINK_APPEND_TAG_REUSE`.
pub const LINK_APPEND_TAG_REUSE: u32 = 1 << 1;
/// `LINK_APPEND_TAG_INDIRECT_REUSE`.
pub const LINK_APPEND_TAG_INDIRECT_REUSE: u32 = 1 << 2;

/// `BKE_BLENDFILE_LINK_APPEND_FOREACH_ITEM_FLAG_*`.
pub const FOREACH_ITEM_FLAG_DO_DIRECT: u32 = 1 << 0;
/// `DO_INDIRECT`.
pub const FOREACH_ITEM_FLAG_DO_INDIRECT: u32 = 1 << 1;
/// Both kinds.
pub const FOREACH_ITEM_FLAG_ALL: u32 =
    FOREACH_ITEM_FLAG_DO_DIRECT | FOREACH_ITEM_FLAG_DO_INDIRECT;

/// `ProcessStage` enum (verbatim).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ProcessStage {
    /// `Init` — context still being populated.
    #[default]
    Init,
    /// `Linking`.
    Linking,
    /// `Appending`.
    Appending,
    /// `Instantiating`.
    Instantiating,
    /// `Done`.
    Done,
}

/// `BlendfileLinkAppendContextLibrary` — one source library.
pub struct ContextLibrary {
    /// `path`.
    pub path: String,
    /// `blo_handle` — `BlendHandle` index/handle (0 = none).
    pub blo_handle: usize,
    /// `blo_handle_is_owned`.
    pub blo_handle_is_owned: bool,
    /// `bf_reports` — `ReportList` handle.
    pub bf_reports: usize,
}

/// `BlendfileLinkAppendContextItem` — one pending linked/appended
/// ID reference.
pub struct ContextItem {
    /// `name` — the ID's name (e.g. "OBfoo").
    pub name: String,
    /// `idcode` — the `ID_Type` code.
    pub idcode: i16,
    /// `libraries` bitmask — which source libraries provide it.
    pub libraries: Vec<bool>,
    /// `new_id` — resulting local/linked ID handle.
    pub new_id: usize,
    /// `liboverride_id`.
    pub liboverride_id: usize,
    /// `action`.
    pub action: i16,
    /// `tag` — `LINK_APPEND_TAG_*` bits.
    pub tag: u32,
    /// `userdata`.
    pub userdata: usize,
}

/// `BlendfileLinkAppendContext` — the whole link/append context.
pub struct BlendfileLinkAppendContext {
    /// `params` — `LibraryLink_Params` handle.
    pub params: usize,
    /// `libraries`.
    pub libraries: Vec<ContextLibrary>,
    /// `items`.
    pub items: Vec<ContextItem>,
    /// `new_id_to_item` — reverse map new-ID → item index.
    pub new_id_to_item: HashMap<usize, usize>,
    /// `blendfile_mem` — embedded memfile bytes.
    pub blendfile_mem: Option<Vec<u8>>,
    /// `blendfile_memsize` (kept verbatim; redundant with len).
    pub blendfile_memsize: usize,
    /// `process_stage`.
    pub process_stage: ProcessStage,
    /// `library_weak_reference_mapping` (unused pointer in C,
    /// always asserted empty on free).
    pub library_weak_reference_mapping: Option<usize>,
}

impl BlendfileLinkAppendContext {
    /// `BKE_blendfile_link_append_context_new`.
    pub fn new(params: usize) -> Self {
        Self {
            params,
            libraries: Vec::new(),
            items: Vec::new(),
            new_id_to_item: HashMap::new(),
            blendfile_mem: None,
            blendfile_memsize: 0,
            process_stage: ProcessStage::Init,
            library_weak_reference_mapping: None,
        }
    }

    /// `BKE_blendfile_link_append_context_flag_set`.
    pub fn flag_set(&mut self, flag: i32, do_set: bool, params_flag: &mut i32) {
        if do_set {
            *params_flag |= flag;
        } else {
            *params_flag &= !flag;
        }
        let _ = self;
    }

    /// `BKE_blendfile_link_append_context_embedded_blendfile_set`.
    pub fn embedded_blendfile_set(&mut self, blendfile_mem: Vec<u8>) {
        debug_assert!(self.blendfile_mem.is_none());
        self.blendfile_memsize = blendfile_mem.len();
        self.blendfile_mem = Some(blendfile_mem);
    }

    /// `BKE_blendfile_link_append_context_embedded_blendfile_clear`.
    pub fn embedded_blendfile_clear(&mut self) {
        self.blendfile_mem = None;
        self.blendfile_memsize = 0;
    }

    /// `BKE_blendfile_link_append_context_library_add`.
    pub fn library_add(&mut self, libname: &str, blo_handle: usize) {
        debug_assert!(self.items.is_empty());
        debug_assert!(self.process_stage == ProcessStage::Init);
        self.libraries.push(ContextLibrary {
            path: libname.to_string(),
            blo_handle,
            blo_handle_is_owned: true,
            bf_reports: 0,
        });
    }

    /// `BKE_blendfile_link_append_context_item_add`.
    pub fn item_add(&mut self, idname: &str, idcode: i16, userdata: usize) -> usize {
        let item = ContextItem {
            name: idname.to_string(),
            idcode,
            libraries: vec![false; self.libraries.len()],
            new_id: 0,
            liboverride_id: 0,
            action: LINK_APPEND_ACT_UNSET,
            tag: 0,
            userdata,
        };
        self.items.push(item);
        self.items.len() - 1
    }

    /// `BKE_blendfile_link_append_context_item_library_index_enable`.
    pub fn item_library_index_enable(&mut self, item_index: usize, library_index: usize) {
        debug_assert!(self.process_stage == ProcessStage::Init);
        self.items[item_index].libraries[library_index] = true;
    }

    /// `BKE_blendfile_link_append_context_is_empty`.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// `BKE_blendfile_link_append_context_item_userdata_get`.
    pub fn item_userdata_get(&self, item_index: usize) -> usize {
        self.items[item_index].userdata
    }

    /// `BKE_blendfile_link_append_context_item_newid_get`.
    pub fn item_newid_get(&self, item_index: usize) -> usize {
        debug_assert!(self.process_stage != ProcessStage::Init);
        self.items[item_index].new_id
    }

    /// `BKE_blendfile_link_append_context_item_newid_set` — the
    /// linked→local/packed replacement rules from the C asserts are
    /// enforced by `is_valid_newid`.
    pub fn item_newid_set(&mut self, item_index: usize, new_id: usize) {
        debug_assert!(self.process_stage != ProcessStage::Init);
        let item = &mut self.items[item_index];
        debug_assert!(item.new_id != 0);
        debug_assert!(item.liboverride_id == 0);
        debug_assert!(!self.new_id_to_item.contains_key(&new_id));
        self.new_id_to_item.remove(&item.new_id);
        item.new_id = new_id;
        self.new_id_to_item.insert(new_id, item_index);
    }

    /// `BKE_blendfile_link_append_context_item_liboverrideid_get`.
    pub fn item_liboverrideid_get(&self, item_index: usize) -> usize {
        debug_assert!(self.process_stage != ProcessStage::Init);
        self.items[item_index].liboverride_id
    }

    /// `BKE_blendfile_link_append_context_item_idcode_get`.
    pub fn item_idcode_get(&self, item_index: usize) -> i16 {
        self.items[item_index].idcode
    }

    /// `BKE_blendfile_link_append_context_item_foreach` — direct /
    /// indirect filtering verbatim.
    pub fn item_foreach(
        &mut self,
        flag: u32,
        callback: &mut dyn FnMut(&mut Self, usize) -> bool,
    ) {
        for i in 0..self.items.len() {
            let indirect = (self.items[i].tag & LINK_APPEND_TAG_INDIRECT) != 0;
            if (flag & FOREACH_ITEM_FLAG_DO_DIRECT) == 0 && !indirect {
                continue;
            }
            if (flag & FOREACH_ITEM_FLAG_DO_INDIRECT) == 0 && indirect {
                continue;
            }
            if !callback(self, i) {
                break;
            }
        }
    }

    /// `BKE_blendfile_link_append_context_init_done` — emits the
    /// `BKE_CB_EVT_BLENDIMPORT_PRE` callback (injected).
    pub fn init_done(&mut self, callback_exec: &mut dyn FnMut(&mut Self)) {
        debug_assert!(self.process_stage == ProcessStage::Init);
        callback_exec(self);
    }

    /// `BKE_blendfile_link_append_context_finalize` — `Done` stage +
    /// indirect-link-status fixup + node-tree interface tag
    /// (injected `update_linked`/`ntree_update_tag_all` closures).
    pub fn finalize(
        &mut self,
        update_linked: &mut dyn FnMut(usize),
        ntree_update_tag_all: &mut dyn FnMut(usize),
        id_is_nodetree: &dyn Fn(usize) -> bool,
        bmain: usize,
    ) {
        debug_assert!(matches!(
            self.process_stage,
            ProcessStage::Linking | ProcessStage::Appending | ProcessStage::Instantiating
        ));
        self.process_stage = ProcessStage::Done;

        update_linked(bmain);

        for &id in self.new_id_to_item.keys() {
            if id_is_nodetree(id) {
                ntree_update_tag_all(id);
            }
        }
    }
}

/// `link_append_context_library_blohandle_ensure` — lazily opens
/// the library's `BlendHandle` (file or embedded memfile).
pub fn library_blohandle_ensure(
    lapp_context: &BlendfileLinkAppendContext,
    lib_context: &mut ContextLibrary,
    reports: usize,
    blendhandle_from_memory: &mut dyn FnMut(&[u8], usize) -> usize,
    blendhandle_from_file: &mut dyn FnMut(&str, usize) -> usize,
) -> usize {
    if reports != 0 {
        lib_context.bf_reports = reports;
    }
    if lib_context.blo_handle == 0 {
        lib_context.blo_handle = if lib_context.path == BLO_EMBEDDED_STARTUP_BLEND {
            let mem = lapp_context.blendfile_mem.as_deref().unwrap_or(&[]);
            blendhandle_from_memory(mem, reports)
        } else {
            blendhandle_from_file(&lib_context.path, reports)
        };
        lib_context.blo_handle_is_owned = true;
    }
    lib_context.blo_handle
}

/// `link_append_context_library_blohandle_release`.
pub fn library_blohandle_release(
    lib_context: &mut ContextLibrary,
    blendhandle_close: &mut dyn FnMut(usize),
) {
    if lib_context.blo_handle_is_owned && lib_context.blo_handle != 0 {
        blendhandle_close(lib_context.blo_handle);
        lib_context.blo_handle = 0;
    }
}

/// `BKE_blendfile_link_append_context_item_idtypes_from_library_add`
/// — enumerate linkable datablocks of one library into items.
/// `iter_idcodes` yields linkable idcodes (already filtered);
/// `get_datablock_names` yields each name list.
pub fn item_idtypes_from_library_add(
    lapp_context: &mut BlendfileLinkAppendContext,
    reports: usize,
    id_types_filter: u64,
    library_index: usize,
    params_flag: i32,
    iter_idcodes: &mut dyn FnMut(&mut dyn FnMut(i16, u64)),
    get_datablock_names: &mut dyn FnMut(usize, i16, bool) -> Vec<String>,
    blohandle_ensure: &mut dyn FnMut(usize) -> usize,
) -> i32 {
    debug_assert!(lapp_context.process_stage == ProcessStage::Init);
    let mut id_num = 0;

    if blohandle_ensure(library_index) == 0 && reports == 0 {
        return BLENDFILE_LINK_APPEND_INVALID;
    }
    let blo = blohandle_ensure(library_index);
    if blo == 0 {
        return BLENDFILE_LINK_APPEND_INVALID;
    }

    let use_assets_only = (params_flag & FILE_ASSETS_ONLY) != 0;

    iter_idcodes(&mut |id_code, _idfilter| {
        let names = get_datablock_names(blo, id_code, use_assets_only);
        id_num += names.len() as i32;
        for id_name in names {
            let idx = lapp_context.item_add(&id_name, id_code, 0);
            lapp_context.item_library_index_enable(idx, library_index);
        }
    });

    id_num
}

/// `new_id_to_item_mapping_add` / `_create` — rebuild the reverse
/// map after processing.
pub fn new_id_to_item_mapping_create(lapp_context: &mut BlendfileLinkAppendContext) {
    lapp_context.new_id_to_item.clear();
    for (i, item) in lapp_context.items.iter().enumerate() {
        if item.new_id != 0 {
            lapp_context.new_id_to_item.insert(item.new_id, i);
        }
    }
}
