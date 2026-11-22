//! Port of `intern/multi_function_procedure_optimization.cc`
//! — `move_destructs_up` last-use relocation.

use crate::multi_function_procedure::{InstructionKind, Procedure};

/// `move_destructs_up` (verbatim — walk block
/// backwards; remember destruct per variable; when a
/// call uses it, unlink+insert right after call).
pub fn move_destructs_up(procedure: &mut Procedure, block_end_instr: usize) {
    let mut destruct_map: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    /* Backwards traversal — model the instr chain via
     * `prev` edges from block_end. */
    let mut current_instr = Some(block_end_instr);
    while let Some(ci) = current_instr {
        match &procedure.instructions[ci].kind {
            Some(InstructionKind::Destruct { variable }) => {
                destruct_map.insert(*variable, ci);
            }
            Some(InstructionKind::Call { params, .. }) => {
                let params = params.clone();
                for variable in params {
                    if variable == usize::MAX {
                        continue;
                    }
                    let Some(destruct_instr) = destruct_map.remove(&variable) else {
                        continue;
                    };
                    /* Unlink destruct from prev position. */
                    let after_destruct = procedure.instructions[destruct_instr].next.clone();
                    let prevs = procedure.instructions[destruct_instr].prev.clone();
                    for prev in prevs {
                        let next = &mut procedure.instructions[prev].next;
                        next.retain(|&n| n != destruct_instr);
                        next.extend(after_destruct.iter().copied());
                    }
                    /* Insert destruct right after call. */
                    let next_instr = procedure.instructions[ci].next.clone();
                    procedure.instructions[ci].next = vec![destruct_instr];
                    procedure.instructions[destruct_instr].next = next_instr;
                }
            }
            _ => {}
        }
        /* Step to first predecessor (linear chains). */
        current_instr = procedure.instructions[ci].prev.first().copied();
    }
}
