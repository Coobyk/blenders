//! Port of `intern/callbacks.cc` — the global `callback_slots`
//! registry (`bCallbackFuncStore` per `eCbEvent`).
//!
//! `PointerRNA` payloads are `usize` handles; `Main *` likewise
//! (PORT_PENDING RNA). Slot storage is a `Mutex<Vec>` per event —
//! `BLI_addtail` order preserved; `funcstore->alloc` → owned flag.

use std::sync::Mutex;

/// `eCbEvent` (BKE_callbacks.hh) — indices into `CALLBACK_SLOTS`.
pub const BKE_CB_EVT_FRAME_CHANGE_PRE: usize = 0;
pub const BKE_CB_EVT_FRAME_CHANGE_POST: usize = 1;
pub const BKE_CB_EVT_RENDER_PRE: usize = 2;
pub const BKE_CB_EVT_RENDER_POST: usize = 3;
pub const BKE_CB_EVT_RENDER_WRITE: usize = 4;
pub const BKE_CB_EVT_RENDER_STATS: usize = 5;
pub const BKE_CB_EVT_RENDER_INIT: usize = 6;
pub const BKE_CB_EVT_RENDER_COMPLETE: usize = 7;
pub const BKE_CB_EVT_RENDER_CANCEL: usize = 8;
pub const BKE_CB_EVT_LOAD_PRE: usize = 9;
pub const BKE_CB_EVT_LOAD_POST: usize = 10;
pub const BKE_CB_EVT_LOAD_POST_FAIL: usize = 11;
pub const BKE_CB_EVT_SAVE_PRE: usize = 12;
pub const BKE_CB_EVT_SAVE_POST: usize = 13;
pub const BKE_CB_EVT_SAVE_POST_FAIL: usize = 14;
pub const BKE_CB_EVT_UNDO_PRE: usize = 15;
pub const BKE_CB_EVT_UNDO_POST: usize = 16;
pub const BKE_CB_EVT_REDO_PRE: usize = 17;
pub const BKE_CB_EVT_REDO_POST: usize = 18;
pub const BKE_CB_EVT_DEPSGRAPH_UPDATE_PRE: usize = 19;
pub const BKE_CB_EVT_DEPSGRAPH_UPDATE_POST: usize = 20;
pub const BKE_CB_EVT_VERSION_UPDATE: usize = 21;
pub const BKE_CB_EVT_LOAD_FACTORY_USERDEF_POST: usize = 22;
pub const BKE_CB_EVT_LOAD_FACTORY_STARTUP_POST: usize = 23;
pub const BKE_CB_EVT_XR_SESSION_START_PRE: usize = 24;
pub const BKE_CB_EVT_ANNOTATION_PRE: usize = 25;
pub const BKE_CB_EVT_ANNOTATION_POST: usize = 26;
pub const BKE_CB_EVT_OBJECT_BAKE_PRE: usize = 27;
pub const BKE_CB_EVT_OBJECT_BAKE_COMPLETE: usize = 28;
pub const BKE_CB_EVT_OBJECT_BAKE_CANCEL: usize = 29;
pub const BKE_CB_EVT_COMPOSITE_PRE: usize = 30;
pub const BKE_CB_EVT_COMPOSITE_POST: usize = 31;
pub const BKE_CB_EVT_COMPOSITE_CANCEL: usize = 32;
pub const BKE_CB_EVT_ANIMATION_PLAYBACK_PRE: usize = 33;
pub const BKE_CB_EVT_ANIMATION_PLAYBACK_POST: usize = 34;
pub const BKE_CB_EVT_TRANSLATION_UPDATE_POST: usize = 35;
pub const BKE_CB_EVT_EXTENSION_REPOS_UPDATE_PRE: usize = 36;
pub const BKE_CB_EVT_EXTENSION_REPOS_UPDATE_POST: usize = 37;
pub const BKE_CB_EVT_EXTENSION_REPOS_SYNC: usize = 38;
pub const BKE_CB_EVT_EXTENSION_REPOS_FILES_CLEAR: usize = 39;
pub const BKE_CB_EVT_BLENDIMPORT_PRE: usize = 40;
pub const BKE_CB_EVT_BLENDIMPORT_POST: usize = 41;
pub const BKE_CB_EVT_EXIT_PRE: usize = 42;
/// `BKE_CB_EVT_TOT`.
pub const BKE_CB_EVT_TOT: usize = 43;

/// `bCallbackFunc` signature — `(main, pointers, arg)`.
pub type BCallbackFunc = fn(usize, &[usize], usize);

/// `bCallbackFuncStore`.
pub struct BCallbackFuncStore {
    pub func: BCallbackFunc,
    /// `void *arg`.
    pub arg: usize,
    /// `short alloc` — whether the store was heap-allocated.
    pub alloc: bool,
}

struct Slots(Mutex<Vec<Vec<BCallbackFuncStore>>>);
static CALLBACK_SLOTS: Slots = Slots(Mutex::new(Vec::new()));
static INITIALIZED: Mutex<bool> = Mutex::new(false);

fn with_slots<R>(f: impl FnOnce(&mut Vec<Vec<BCallbackFuncStore>>) -> R) -> R {
    let mut slots = CALLBACK_SLOTS.0.lock().unwrap();
    if slots.is_empty() {
        *slots = (0..BKE_CB_EVT_TOT).map(|_| Vec::new()).collect();
    }
    f(&mut slots)
}

/// `BKE_callback_exec` — runs each store's func in list order.
/// Callback fns are `fn` pointers (not `FnMut`), so stores are copied
/// out first to avoid holding the lock across calls.
pub fn bke_callback_exec(bmain: usize, pointers: &[usize], evt: usize) {
    debug_assert!(*INITIALIZED.lock().unwrap(), "BKE_callback_global_init() first");
    /* Snapshot fn/arg pairs; stores may add/remove during exec in C
     * via deferred lists — simple snapshot here. */
    let snapshot: Vec<(BCallbackFunc, usize)> = with_slots(|s| {
        s[evt].iter().map(|fs| (fs.func, fs.arg)).collect()
    });
    for (func, arg) in snapshot {
        func(bmain, pointers, arg);
    }
}

/// `BKE_callback_exec_null`.
pub fn bke_callback_exec_null(bmain: usize, evt: usize) {
    bke_callback_exec(bmain, &[], evt);
}

/// `BKE_callback_exec_id` — `pointers[0]` is the ID's PointerRNA.
pub fn bke_callback_exec_id(bmain: usize, id_ptr: usize, evt: usize) {
    bke_callback_exec(bmain, &[id_ptr], evt);
}

/// `BKE_callback_exec_id_depsgraph` — `{id_ptr, depsgraph_ptr}`.
pub fn bke_callback_exec_id_depsgraph(bmain: usize, id_ptr: usize, depsgraph_ptr: usize, evt: usize) {
    bke_callback_exec(bmain, &[id_ptr, depsgraph_ptr], evt);
}

/// `BKE_callback_exec_boolean` — the bool's `PrimitiveBoolean` RNA ptr.
pub fn bke_callback_exec_boolean(bmain: usize, boolean_ptr: usize, evt: usize) {
    bke_callback_exec(bmain, &[boolean_ptr], evt);
}

/// `BKE_callback_exec_string`.
pub fn bke_callback_exec_string(bmain: usize, str_ptr: usize, evt: usize) {
    bke_callback_exec(bmain, &[str_ptr], evt);
}

/// `BKE_callback_add` (`BLI_addtail`).
pub fn bke_callback_add(funcstore: BCallbackFuncStore, evt: usize) {
    debug_assert!(*INITIALIZED.lock().unwrap(), "BKE_callback_global_init() first");
    with_slots(|s| s[evt].push(funcstore));
}

/// `BKE_callback_remove` — by identity of `func`+`arg`; removes the
/// first match (`BLI_remlink`), returns whether it was found.
pub fn bke_callback_remove(func: BCallbackFunc, arg: usize, evt: usize) -> bool {
    if !*INITIALIZED.lock().unwrap() {
        return false;
    }
    with_slots(|s| {
        if let Some(i) = s[evt].iter().position(|fs| fs.func == func && fs.arg == arg) {
            let _fs = s[evt].remove(i);
            /* `if (funcstore->alloc) MEM_delete` — dropped. */
            true
        } else {
            false
        }
    })
}

/// `BKE_callback_global_init`.
pub fn bke_callback_global_init() {
    *INITIALIZED.lock().unwrap() = true;
}

/// `BKE_callback_global_finalize` — remove every store in every slot.
pub fn bke_callback_global_finalize() {
    with_slots(|s| {
        for slot in s.iter_mut() {
            slot.clear();
        }
    });
    *INITIALIZED.lock().unwrap() = false;
}
