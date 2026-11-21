//! Port of `intern/editlattice.cc` — `BKE_editlattice_{free,make,load}`.
//!
//! `Lattice::def`/`dvert` are owned `Vec`s in the DNA port, so the
//! `MEM_dupalloc`/defvert copies become clones. `EditLatt` lives in a
//! caller-owned arena (`Vec<EditLatt>`) since `editlatt` is a `usize` handle.

use blenders_dna::id::NIL;
use blenders_dna::lattice_types::Lattice;

/// `EditLatt` (DNA_editlattice... actually `struct EditLatt` in lattice types):
/// `{Lattice *latt; int shapenr;}`.
#[derive(Clone)]
pub struct EditLatt {
    pub latt: Lattice,
    pub shapenr: i32,
}

/// `BKE_editlattice_free(Object *ob)` — frees `lt->editlatt` chain
/// (`editlt->def`, `editlt->dvert`, `editlt`, `editlatt` itself).
/// `edits` is the EditLatt arena; the slot is cleared and the handle nulled.
pub fn BKE_editlattice_free(lt: &mut Lattice, edits: &mut Vec<Option<EditLatt>>) {
    if lt.editlatt != NIL {
        if let Some(slot) = edits.get_mut(lt.editlatt) {
            *slot = None; // MEM_delete chain.
        }
        lt.editlatt = NIL;
    }
}

/// `BKE_editlattice_make(Object *obedit)` — allocates `lt->editlatt`, dup's the
/// lattice and its `def`/`dvert` arrays; copies `shapenr` when `lt->key` set.
/// `BKE_keyblock_from_object`/`BKE_keyblock_convert_to_lattice` — PORT_PENDING.
pub fn BKE_editlattice_make(
    lt: &mut Lattice,
    edits: &mut Vec<Option<EditLatt>>,
    ob_shapenr: i32,
) {
    BKE_editlattice_free(lt, edits);
    // MEM_dupalloc(lt) + MEM_dupalloc(lt->def) + BKE_defvert_array_copy —
    // all covered by the deep clone (def/dvert are owned Vecs).
    let mut copy = lt.clone();
    copy.editlatt = NIL;
    let shapenr = if lt.key != NIL { ob_shapenr } else { 0 };
    edits.push(Some(EditLatt { latt: copy, shapenr }));
    lt.editlatt = edits.len() - 1;
}

/// `BKE_editlattice_load(Object *obedit)` — copies the edit lattice's scalar
/// fields back onto `lt` and rebuilds `def`/key data.
/// `editlt` is the editlatt's inner `Lattice`; `shapenr` its key index.
pub fn BKE_editlattice_load(lt: &mut Lattice, editlt: &Lattice, _shapenr: i32) {
    lt.def = editlt.def.clone(); // MEM_delete(lt->def); dupalloc.
    lt.flag = editlt.flag;
    lt.pntsu = editlt.pntsu;
    lt.pntsv = editlt.pntsv;
    lt.pntsw = editlt.pntsw;
    lt.typeu = editlt.typeu;
    lt.typev = editlt.typev;
    lt.typew = editlt.typew;
    lt.actbp = editlt.actbp;
    lt.fu = editlt.fu;
    lt.fv = editlt.fv;
    lt.fw = editlt.fw;
    lt.du = editlt.du;
    lt.dv = editlt.dv;
    lt.dw = editlt.dw;
    // KeyBlock data rebuild (actkey->data floats) — PORT_PENDING (Key runtime).
    lt.dvert = editlt.dvert.clone(); // BKE_defvert_array_free + copy.
}
