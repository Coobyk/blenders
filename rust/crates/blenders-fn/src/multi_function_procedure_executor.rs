//! Port of `intern/multi_function_procedure_executor.cc`
//! — ProcedureExecutor: index-mask instruction
//! scheduler + main execute loop.

use crate::multi_function_procedure::{InstructionKind, Procedure};

/// `IndexMask` minimal — list of indices.
pub type IndexMask = Vec<i64>;
/// `InstructionIndices` verbatim.
#[derive(Default)]
pub struct InstructionIndices {
    /// `referenced_indices`.
    pub referenced_indices: IndexMask,
}
/// `NextInstructionInfo` verbatim.
pub struct NextInstructionInfo {
    /// `instruction` index.
    pub instruction: usize,
    /// `indices`.
    pub indices: InstructionIndices,
}
/// `InstructionScheduler` verbatim — stack of
/// (instruction, mask) work items.
#[derive(Default)]
pub struct InstructionScheduler {
    /// `next_instructions_`.
    pub next_instructions: Vec<NextInstructionInfo>,
}
impl InstructionScheduler {
    /// `add_referenced_indices` (verbatim — skip
    /// empty).
    pub fn add_referenced_indices(&mut self, instruction: usize, mask: IndexMask) {
        if mask.is_empty() {
            return;
        }
        self.next_instructions.push(NextInstructionInfo {
            instruction,
            indices: InstructionIndices {
                referenced_indices: mask,
            },
        });
    }
    /// `is_done` (verbatim).
    pub fn is_done(&self) -> bool {
        self.next_instructions.is_empty()
    }
    /// `peek` (verbatim).
    pub fn peek(&self) -> &NextInstructionInfo {
        self.next_instructions.last().unwrap()
    }
    /// `update_instruction_pointer` (verbatim —
    /// advance same indices to next instr).
    pub fn update_instruction_pointer(&mut self, instruction: usize) {
        if let Some(top) = self.next_instructions.last_mut() {
            top.instruction = instruction;
        }
    }
    /// `pop` (verbatim).
    pub fn pop(&mut self) -> Option<NextInstructionInfo> {
        self.next_instructions.pop()
    }
}
/// `ProcedureExecutor::call` (verbatim — scheduler
/// loop: Call→exec+advance, Branch→split,
/// Destruct→destruct+advance, Dummy→advance,
/// Return→pop).
pub fn procedure_executor_call(
    procedure: &Procedure,
    entry: usize,
    full_mask: IndexMask,
    mut execute_call: impl FnMut(usize, &IndexMask),
    mut destruct: impl FnMut(usize, &IndexMask),
    mut split_mask: impl FnMut(usize, &IndexMask) -> (IndexMask, IndexMask),
) {
    let mut scheduler = InstructionScheduler::default();
    scheduler.add_referenced_indices(entry, full_mask);
    while !scheduler.is_done() {
        let (instr_i, mask) = {
            let info = scheduler.peek();
            (info.instruction, info.indices.referenced_indices.clone())
        };
        match &procedure.instructions[instr_i].kind {
            Some(InstructionKind::Call { .. }) => {
                (execute_call)(instr_i, &mask);
                let next = procedure.instructions[instr_i].next.first().copied();
                if let Some(n) = next {
                    scheduler.update_instruction_pointer(n);
                } else {
                    scheduler.pop();
                }
            }
            Some(InstructionKind::Branch {
                condition,
                branch_true,
                branch_false,
            }) => {
                let (t_idx, f_idx) = (split_mask)(*condition, &mask);
                scheduler.pop();
                scheduler.add_referenced_indices(*branch_false, f_idx);
                scheduler.add_referenced_indices(*branch_true, t_idx);
            }
            Some(InstructionKind::Destruct { variable }) => {
                (destruct)(*variable, &mask);
                let next = procedure.instructions[instr_i].next.first().copied();
                if let Some(n) = next {
                    scheduler.update_instruction_pointer(n);
                } else {
                    scheduler.pop();
                }
            }
            Some(InstructionKind::Dummy { .. }) => {
                let next = procedure.instructions[instr_i].next.first().copied();
                if let Some(n) = next {
                    scheduler.update_instruction_pointer(n);
                } else {
                    scheduler.pop();
                }
            }
            Some(InstructionKind::Return) | None => {
                scheduler.pop();
            }
        }
    }
}
/// `get_execution_hints` (verbatim — allocates,
/// grain 10000).
pub struct ExecutionHints {
    /// `allocates_array`/`min_grain_size`.
    pub allocates_array: bool,
    pub min_grain_size: usize,
}
/// verbatim hints.
pub fn execution_hints() -> ExecutionHints {
    ExecutionHints {
        allocates_array: true,
        min_grain_size: 10000,
    }
}
