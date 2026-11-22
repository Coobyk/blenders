//! Port of `intern/action.cc` — Action/Layer/Strip/
//! Slot/Channelbag container core.

/// Slot/Layer defaults verbatim.
pub const SLOT_DEFAULT_NAME: &str = "Slot";
pub const SLOT_UNTYPED_PREFIX: &str = "XX";
pub const LAYER_DEFAULT_NAME: &str = "Layer";
/// `ActionSlot::unassigned` handle verbatim.
pub const SLOT_HANDLE_UNASSIGNED: i64 = 0;
/// `ACT_FRAME_RANGE`/`ACT_CYCLIC` verbatim.
pub const ACT_FRAME_RANGE: i32 = 1 << 7;
pub const ACT_CYCLIC: i32 = 1 << 2;
/// `MINAFRAMEF`/`MAXFRAMEF` verbatim.
pub const MINAFRAMEF: f32 = -1_000_000.0;
pub const MAXFRAMEF: f32 = 1_000_000.0;

/// `Slot` verbatim fields.
#[derive(Default)]
pub struct ActionSlot {
    /// `handle`.
    pub handle: i64,
    /// `identifier` (2-char prefix + display name).
    pub identifier: String,
    /// `idtype`.
    pub idtype: i16,
    /// `slot_flags` (expanded/selected/active).
    pub slot_flags: i32,
}
impl ActionSlot {
    /// `is_expanded`/`set_expanded` verbatim.
    pub fn is_expanded(&self) -> bool {
        (self.slot_flags & 1) != 0
    }
    /// `set_expanded`.
    pub fn set_expanded(&mut self, expanded: bool) {
        if expanded {
            self.slot_flags |= 1;
        } else {
            self.slot_flags &= !1;
        }
    }
    /// `is_selected`.
    pub fn is_selected(&self) -> bool {
        (self.slot_flags & 2) != 0
    }
    /// `is_active`.
    pub fn is_active(&self) -> bool {
        (self.slot_flags & 4) != 0
    }
    /// `identifier_ensure_prefix` — first 2 chars become
    /// idtype code (verbatim).
    pub fn identifier_ensure_prefix(&mut self, idtype_code: &str) {
        let mut ident = format!("{}{}", idtype_code, &self.identifier[self.identifier.len().min(2)..]);
        if ident.len() < 2 {
            ident.push_str(SLOT_UNTYPED_PREFIX);
        }
        self.identifier = ident;
    }
}

/// `Action` verbatim fields.
#[derive(Default)]
pub struct Action {
    /// `flag`.
    pub flag: i32,
    /// `idroot`.
    pub idroot: i16,
    /// `frame_start`/`frame_end`.
    pub frame_start: f32,
    pub frame_end: f32,
    /// `last_slot_handle`.
    pub last_slot_handle: i64,
    /// slot list.
    pub slots: Vec<ActionSlot>,
    /// layer handles.
    pub layers: Vec<usize>,
}
impl Action {
    /// `slot_for_handle` (verbatim — linear scan,
    /// unassigned → None).
    pub fn slot_for_handle(&self, handle: i64) -> Option<usize> {
        if handle == SLOT_HANDLE_UNASSIGNED {
            return None;
        }
        self.slots.iter().position(|s| s.handle == handle)
    }
    /// `slot_find_by_identifier` (verbatim).
    pub fn slot_find_by_identifier(&self, identifier: &str) -> Option<usize> {
        self.slots.iter().position(|s| s.identifier == identifier)
    }
    /// `slot_allocate` (verbatim — ++last_slot_handle,
    /// expanded default).
    pub fn slot_allocate(&mut self) -> ActionSlot {
        self.last_slot_handle += 1;
        let mut slot = ActionSlot {
            handle: self.last_slot_handle,
            ..Default::default()
        };
        slot.set_expanded(true);
        slot
    }
    /// `slot_add` (verbatim — XX prefix + default name +
    /// unique + idroot=0).
    pub fn slot_add(&mut self) -> usize {
        let mut slot = self.slot_allocate();
        slot.identifier = format!("{}{}", SLOT_UNTYPED_PREFIX, SLOT_DEFAULT_NAME);
        self.slots.push(slot);
        let idx = self.slots.len() - 1;
        slot_identifier_ensure_unique(self, idx);
        self.idroot = 0;
        idx
    }
    /// `is_cyclic` (verbatim).
    pub fn is_cyclic(&self) -> bool {
        (self.flag & ACT_FRAME_RANGE) != 0 && (self.flag & ACT_CYCLIC) != 0
    }
    /// `get_frame_range` (verbatim — manual or computed).
    pub fn get_frame_range(
        &self,
        computed: [f32; 2],
    ) -> [f32; 2] {
        if (self.flag & ACT_FRAME_RANGE) != 0 {
            [self.frame_start, self.frame_end]
        } else {
            computed
        }
    }
}

/// `slot_identifier_ensure_unique` (verbatim —
/// XX-prefix idtype string + uniquename on rest).
pub fn slot_identifier_ensure_unique(action: &mut Action, slot_idx: usize) {
    let base = action.slots[slot_idx].identifier.clone();
    let mut ident = base.clone();
    let mut suffix = 1u32;
    loop {
        let taken = action
            .slots
            .iter()
            .enumerate()
            .any(|(i, s)| i != slot_idx && s.identifier == ident);
        if !taken {
            break;
        }
        ident = format!("{}.{:03}", base, suffix);
        suffix += 1;
    }
    action.slots[slot_idx].identifier = ident;
}

/// `array_shift_range` rotate (verbatim —
/// std::rotate on [to, range_start, range_end]).
pub fn array_shift_range<T>(array: &mut Vec<T>, range_start: usize, range_end: usize, to: usize) {
    if to == range_start || to == range_end {
        return;
    }
    if to < range_start {
        array[to..range_end].rotate_left(range_start - to);
    } else {
        let len = range_end - range_start;
        array[range_start..to + len].rotate_right(len);
    }
}

/// `Action::has_single_frame` (verbatim — every fcurve
/// ≤1 key; all equal within 1e-3).
pub fn has_single_frame(fcurve_key_frames: &[Option<f32>]) -> bool {
    let mut found_key = false;
    let mut found_key_frame = 0.0f32;
    for &kf in fcurve_key_frames {
        let Some(this_key_frame) = kf else {
            continue;
        };
        if !found_key {
            found_key = true;
            found_key_frame = this_key_frame;
            continue;
        }
        if (found_key_frame - this_key_frame).abs() > 0.001 {
            return false;
        }
    }
    found_key
}

/// `get_frame_range_of_fcurves` (verbatim — min/max of
/// key extents, limit/cycle modifier extremes).
pub fn get_frame_range_of_fcurves(
    fcurve_ranges: &[(f32, f32)],
    cycle_before_none: bool,
    cycle_after_none: bool,
) -> [f32; 2] {
    let mut min = 999_999_999.0f32;
    let mut max = -999_999_999.0f32;
    let mut found = false;
    for &(nmin, nmax) in fcurve_ranges {
        min = min.min(nmin);
        max = max.max(nmax);
        found = true;
    }
    if !cycle_before_none {
        min = MINAFRAMEF;
    }
    if !cycle_after_none {
        max = MAXFRAMEF;
    }
    if !found {
        return [0.0, 0.0];
    }
    [min, max]
}
