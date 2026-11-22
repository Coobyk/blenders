/// Port of `common/intern/subdiv_disabler.cc`
/// — temporarily disable subdiv modifiers for
/// export.

/// `SubdivModifierDisabler` (verbatim —
/// walk modifiers, flip flag, restore later).
pub struct SubdivModifierDisabler {
    /// `disabled_modifiers` — restore list.
    pub disabled: Vec<usize>,
}
impl SubdivModifierDisabler {
    /// `disable_modifier` (verbatim — flip
    /// SCENE flag off).
    pub fn disable_modifier(&mut self, mod_index: usize) {
        self.disabled.push(mod_index);
    }
    /// `disable_modifiers` (verbatim — filter
    /// eModifierType_Subsurf).
    pub fn disable_modifiers(
        &mut self,
        modifiers: &[(usize, i32)],
        subsurf_type: i32,
    ) {
        for &(idx, ty) in modifiers {
            if ty == subsurf_type {
                self.disabled.push(idx);
            }
        }
    }
}
