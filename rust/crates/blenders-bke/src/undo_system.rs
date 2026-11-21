//! Port of `intern/undo_system.cc` — `UndoType` registry and the
//! `UndoStack` mechanics: push/clear, step+memory limits, grouped
//! begin/end (skip flags), undo/redo step iteration and decode
//! driver (context/encode/decode injected).

/// `eUndoStepDir`.
pub type UndoStepDir = i32;
/// `STEP_UNDO`.
pub const STEP_UNDO: UndoStepDir = -1;
/// `STEP_REDO`.
pub const STEP_REDO: UndoStepDir = 1;
/// `STEP_INVALID`.
pub const STEP_INVALID: UndoStepDir = 0;

/// `UNDOTYPE_FLAG_*`.
pub const UNDOTYPE_FLAG_DECODE_ACTIVE_STEP: i32 = 1 << 0;
/// `UNDOTYPE_FLAG_NEED_CONTEXT_FOR_ENCODE`.
pub const UNDOTYPE_FLAG_NEED_CONTEXT_FOR_ENCODE: i32 = 1 << 1;

/// `eUndoPushReturn` bits.
pub const UNDO_PUSH_RET_FAILURE: i32 = 0;
/// `UNDO_PUSH_RET_OVERRIDE_CHANGED`.
pub const UNDO_PUSH_RET_OVERRIDE_CHANGED: i32 = 1 << 0;
/// `UNDO_PUSH_RET_SUCCESS`.
pub const UNDO_PUSH_RET_SUCCESS: i32 = 1 << 1;

/// `UndoType` — encode/decode behavior is injected.
#[derive(Clone)]
pub struct UndoType {
    /// `identifier`.
    pub identifier: String,
    /// `flags` — `UNDOTYPE_FLAG_*`.
    pub flags: i32,
    /// `step_size` — per-step user data size.
    pub step_size: usize,
}

/// `UndoStep` (arena-held; `next`/`prev` are indices).
#[derive(Clone, Default)]
pub struct UndoStep {
    /// `name`.
    pub name: String,
    /// `type` index into `g_undo_types`.
    pub type_: usize,
    /// `data_size`.
    pub data_size: usize,
    /// `skip`.
    pub skip: bool,
    /// `use_memfile_step`.
    pub use_memfile_step: bool,
    /// `use_old_bmain_data`.
    pub use_old_bmain_data: bool,
    /// `is_applied`.
    pub is_applied: bool,
    /// `next`/`prev` in `steps`.
    pub next: Option<usize>,
    /// `prev`.
    pub prev: Option<usize>,
    /// `elems` — opaque per-step data (`UndoType`-owned).
    pub elems: Vec<u8>,
}

/// `UndoStack`.
#[derive(Default)]
pub struct UndoStack {
    /// `steps` — doubly-linked via indices.
    pub steps: Vec<Option<UndoStep>>,
    /// head (first) and tail (last) indices.
    pub first: Option<usize>,
    /// `steps.last()` equivalent.
    pub last: Option<usize>,
    /// `step_active`.
    pub step_active: Option<usize>,
    /// `step_active_memfile`.
    pub step_active_memfile: Option<usize>,
    /// `step_init` — pending (not in list).
    pub step_init: Option<UndoStep>,
    /// `group_level` — nested begin/end depth.
    pub group_level: i32,
}

/// Global `UndoType` list (`g_undo_types`).
pub struct UndoTypeRegistry {
    /// Registered types, stable indices.
    pub types: Vec<UndoType>,
}

impl UndoTypeRegistry {
    /// `BKE_undosys_type_register`.
    pub fn register(&mut self, ut: UndoType) -> usize {
        self.types.push(ut);
        self.types.len() - 1
    }
    /// `BKE_undosys_type_unregister`.
    pub fn unregister(&mut self, index: usize) {
        self.types.remove(index);
    }
    /// `BKE_undosys_type_from_idname` — find by identifier.
    pub fn from_idname(&self, identifier: &str) -> Option<usize> {
        self.types.iter().position(|t| t.identifier == identifier)
    }
}

impl UndoStack {
    fn unlink(&mut self, i: usize) {
        if let Some(us) = self.steps.get(i).and_then(|s| s.as_ref()) {
            let (next, prev) = (us.next, us.prev);
            if let Some(p) = prev {
                if let Some(Some(s)) = self.steps.get_mut(p) {
                    s.next = next;
                }
            } else {
                self.first = next;
            }
            if let Some(nx) = next {
                if let Some(Some(s)) = self.steps.get_mut(nx) {
                    s.prev = prev;
                }
            } else {
                self.last = prev;
            }
        }
    }

    /// `undosys_step_free_and_unlink`.
    pub fn step_free_and_unlink(&mut self, i: usize) {
        self.unlink(i);
        self.steps[i] = None;
    }

    /// `undosys_stack_clear_all_last` — free all steps at/after `us`.
    pub fn stack_clear_all_last(&mut self, mut us: Option<usize>) {
        while let Some(i) = us {
            us = self.steps[i].as_ref().and_then(|s| s.next);
            self.step_free_and_unlink(i);
        }
    }

    /// `undosys_stack_clear_all_first` — free all steps at/before `us`,
    /// keeping `us_exclude` when set.
    pub fn stack_clear_all_first(&mut self, mut us: Option<usize>, us_exclude: Option<usize>) {
        while let Some(i) = us {
            us = self.steps[i].as_ref().and_then(|s| s.prev);
            if Some(i) == us_exclude {
                continue;
            }
            self.step_free_and_unlink(i);
        }
    }

    /// `BKE_undosys_stack_destroy` / `stack_clear`.
    pub fn stack_clear(&mut self) {
        self.stack_clear_all_last(self.first);
        self.step_active = None;
        self.step_active_memfile = None;
        self.step_init = None;
        self.steps.clear();
        self.first = None;
        self.last = None;
    }

    /// `BKE_undosys_stack_clear_active` — reset `step_active` to last.
    pub fn stack_clear_active(&mut self) {
        self.stack_clear_all_last(self.step_active);
        self.step_active = self.last;
        if self.step_active_memfile.is_some()
            && !self
                .step_active_memfile
                .map(|i| self.steps.get(i).is_some_and(|s| s.is_some()))
                .unwrap_or(false)
        {
            self.step_active_memfile = None;
        }
    }

    /// Append a step (tail).
    pub fn push_step(&mut self, step: UndoStep) -> usize {
        let i = self.steps.len();
        self.steps.push(Some(step));
        if let Some(last) = self.last {
            if let Some(Some(s)) = self.steps.get_mut(last) {
                s.next = Some(i);
            }
            if let Some(Some(s)) = self.steps.get_mut(i) {
                s.prev = Some(last);
            }
        } else {
            self.first = Some(i);
        }
        self.last = Some(i);
        i
    }

    /// `BKE_undosys_stack_has_undo` — first non-skipping step before
    /// active whose name matches.
    pub fn stack_has_undo(&self, name: &str) -> bool {
        let mut i = self.step_active;
        while let Some(idx) = i {
            let s = self.steps[idx].as_ref().unwrap();
            if s.name == name {
                return true;
            }
            i = s.prev;
        }
        false
    }

    /// `BKE_undosys_stack_has_redo`.
    pub fn stack_has_redo(&self) -> bool {
        match self.step_active {
            Some(i) => self.steps[i].as_ref().and_then(|s| s.next).is_some(),
            None => self.first.is_some(),
        }
    }

    /// `BKE_undosys_stack_limit_steps_and_memory` — `type_filter`
    /// returns true for `BKE_UNDOSYS_TYPE_MEMFILE`-equivalent steps.
    pub fn stack_limit_steps_and_memory(
        &mut self,
        steps: i32,
        memory_limit: usize,
        type_is_memfile: &dyn Fn(usize) -> bool,
    ) {
        if steps == -1 && memory_limit == 0 {
            return;
        }
        let mut data_size_all = 0usize;
        let mut us_count = 0usize;
        let mut us = self.last;
        let mut found: Option<usize> = None;
        while let Some(i) = us {
            let s = self.steps[i].as_ref().unwrap();
            if s.prev.is_none() {
                break;
            }
            if memory_limit != 0 {
                data_size_all += s.data_size;
                if data_size_all > memory_limit {
                    found = us;
                    break;
                }
            }
            if steps != -1 {
                if us_count == steps as usize {
                    found = us;
                    break;
                }
                if !s.skip {
                    us_count += 1;
                }
            }
            us = s.prev;
        }
        if let Some(i) = found {
            let (i_type, i_prev) = {
                let s = self.steps[i].as_ref().unwrap();
                (s.type_, s.prev)
            };
            let mut us_exclude = None;
            if !type_is_memfile(i_type) {
                let mut ex = i_prev;
                while let Some(e) = ex {
                    if type_is_memfile(self.steps[e].as_ref().unwrap().type_) {
                        us_exclude = Some(e);
                        break;
                    }
                    ex = self.steps[e].as_ref().unwrap().prev;
                }
                if let Some(e) = us_exclude {
                    if let Some(Some(st)) = self.steps.get_mut(e) {
                        st.skip = true;
                    }
                }
            }
            self.stack_clear_all_first(i_prev, us_exclude);
        }
    }

    /// `undosys_step_iter_first`.
    fn step_iter_first(&self, us_reference: usize, undo_dir: UndoStepDir, type_flags: i32) -> Option<usize> {
        let s = self.steps[us_reference].as_ref().unwrap();
        if type_flags & UNDOTYPE_FLAG_DECODE_ACTIVE_STEP != 0 {
            return if undo_dir == STEP_UNDO { Some(us_reference) } else { s.next };
        }
        if undo_dir == STEP_UNDO {
            s.prev
        } else {
            s.next
        }
    }

    /// `BKE_undosys_step_calc_direction` — compares positions.
    pub fn step_calc_direction(&self, us_target: usize, us_reference: usize) -> UndoStepDir {
        if us_target == us_reference {
            return STEP_UNDO;
        }
        /* Walk back from reference; if we reach target → undo, else redo. */
        let mut i = self.steps[us_reference].as_ref().unwrap().prev;
        while let Some(idx) = i {
            if idx == us_target {
                return STEP_UNDO;
            }
            i = self.steps[idx].as_ref().unwrap().prev;
        }
        STEP_REDO
    }

    /// `BKE_undosys_step_load_data_ex` — `decode` is the injected
    /// `undosys_step_decode` impl; `type_flags` returns the
    /// `UndoType.flags` for a step.
    pub fn step_load_data_ex(
        &mut self,
        us_target: usize,
        us_reference: Option<usize>,
        use_skip: bool,
        type_flags: &dyn Fn(usize) -> i32,
        decode: &mut dyn FnMut(&UndoStep, UndoStepDir, bool),
    ) -> bool {
        let us_reference = match us_reference.or(self.step_active) {
            Some(r) => r,
            None => return false,
        };
        let undo_dir = self.step_calc_direction(us_target, us_reference);
        debug_assert!(undo_dir != STEP_INVALID);

        let mut us_target_active = Some(us_target);
        if use_skip {
            let mut t = Some(us_target);
            while let Some(i) = t {
                if !self.steps[i].as_ref().unwrap().skip {
                    break;
                }
                t = if undo_dir == STEP_UNDO {
                    self.steps[i].as_ref().unwrap().prev
                } else {
                    self.steps[i].as_ref().unwrap().next
                };
            }
            if t.is_none() {
                return false;
            }
            us_target_active = t;
        }

        let mut is_processing_extra_skipped_steps = false;
        let mut us_iter = self.step_iter_first(us_reference, undo_dir, type_flags(us_reference));
        while let Some(i) = us_iter {
            let is_final = Some(i) == us_target_active;
            let s = self.steps[i].as_ref().unwrap();
            decode(s, undo_dir, is_final);
            self.step_active = Some(i);
            if i == us_target {
                is_processing_extra_skipped_steps = true;
            }
            if is_final {
                return true;
            }
            let _ = is_processing_extra_skipped_steps;
            us_iter = if undo_dir == STEP_UNDO {
                self.steps[i].as_ref().unwrap().prev
            } else {
                self.steps[i].as_ref().unwrap().next
            };
        }
        false
    }

    /// `BKE_undosys_step_undo` / `step_redo` — find the next
    /// non-skipped step in the direction and load it.
    pub fn step_undo_redo(
        &mut self,
        undo_dir: UndoStepDir,
        type_flags: &dyn Fn(usize) -> i32,
        decode: &mut dyn FnMut(&UndoStep, UndoStepDir, bool),
    ) -> bool {
        let active = match self.step_active {
            Some(i) => i,
            None => return false,
        };
        let mut target = if undo_dir == STEP_UNDO {
            self.steps[active].as_ref().unwrap().prev
        } else {
            self.steps[active].as_ref().unwrap().next
        };
        while let Some(i) = target {
            if !self.steps[i].as_ref().unwrap().skip {
                break;
            }
            target = if undo_dir == STEP_UNDO {
                self.steps[i].as_ref().unwrap().prev
            } else {
                self.steps[i].as_ref().unwrap().next
            };
        }
        match target {
            Some(t) => self.step_load_data_ex(t, Some(active), true, type_flags, decode),
            None => false,
        }
    }
}
