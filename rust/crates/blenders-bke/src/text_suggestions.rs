//! Port of `intern/text_suggestions.cc` — text-completion suggestion list
//! (`suggestions` ListBase of `SuggItem`).
//!
//! Suggestion items live in a `Vec` arena; `firstmatch`/`lastmatch`/
//! `selected`/`top` indices keep the C semantics (`usize::MAX` = null).

use blenders_dna::id::NIL;

/// `SuggItem` (text_suggestions.cc) — a documentation/suggestion entry.
#[derive(Default)]
pub struct SuggItem {
    pub name: String,
    /// `char *documentation` — optional doc string.
    pub documentation: Option<String>,
}

/// `TextSuggestions` — the module-level `suggestions` ListBase + cursors.
#[derive(Default)]
pub struct TextSuggestions {
    pub items: Vec<SuggItem>,
    /// `firstmatch`/`lastmatch` — index range of current matches.
    pub firstmatch: usize,
    pub lastmatch: usize,
    /// `selected` — highlighted item.
    pub selected: usize,
    /// `top` — first visible item (scroll offset).
    pub top: usize,
    /// `documentation` — global doc string (`txttl_free_docs`).
    pub docs: Option<String>,
}

impl TextSuggestions {
    /// `txttl_free_suggest` — `MEM_delete` all + reset cursors.
    pub fn free_suggest(&mut self) {
        self.items.clear();
        self.firstmatch = NIL;
        self.lastmatch = NIL;
        self.selected = NIL;
        self.top = 0;
    }

    /// `txttl_free_docs`.
    pub fn free_docs(&mut self) {
        self.docs = None;
    }

    /// `free_texttools`.
    pub fn free_texttools(&mut self) {
        self.free_suggest();
        self.free_docs();
    }

    /// `txttl_suggest_add` — append (keeps list order).
    pub fn suggest_add(&mut self, name: String, docs: Option<String>) -> usize {
        self.items.push(SuggItem { name, documentation: docs });
        self.items.len() - 1
    }

    /// `txttl_suggest_last` — the last added item (ListBase `last`).
    pub fn suggest_last(&self) -> usize {
        if self.items.is_empty() { NIL } else { self.items.len() - 1 }
    }
}
