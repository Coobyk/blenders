//! Port of `intern/multi_function_procedure.cc` —
//! Procedure (variables + instructions + cursors).

/// `Variable` verbatim.
#[derive(Default)]
pub struct Variable {
    /// `type` index.
    pub type_: usize,
    /// `name`.
    pub name: String,
}
/// `Instruction` kind verbatim.
pub enum InstructionKind {
    /// `CallInstruction` — fn + param variables.
    Call {
        /// `fn_` handle.
        fn_: usize,
        /// `params_` variable indices.
        params: Vec<usize>,
    },
    /// `DestructInstruction`.
    Destruct {
        /// `variable_`.
        variable: usize,
    },
    /// `DummyInstruction` (named target).
    Dummy {
        /// `name_`.
        name: String,
    },
    /// `ReturnInstruction`.
    Return,
    /// `BranchInstruction` (condition variable).
    Branch {
        /// `condition_` variable.
        condition: usize,
        /// `branch_true`/`branch_false` instruction.
        branch_true: usize,
        branch_false: usize,
    },
}
/// `Instruction` verbatim.
#[derive(Default)]
pub struct Instruction {
    /// `kind`.
    pub kind: Option<InstructionKind>,
    /// `next_` linear next instruction.
    pub next: Vec<usize>,
    /// `prev_` for cursor linking.
    pub prev: Vec<usize>,
}
/// `InstructionCursor` verbatim.
#[derive(Clone, Copy)]
pub struct InstructionCursor {
    /// `instruction` index.
    pub instruction: usize,
}
/// `Procedure` verbatim.
#[derive(Default)]
pub struct Procedure {
    /// `variables_`.
    pub variables: Vec<Variable>,
    /// `instructions_`.
    pub instructions: Vec<Instruction>,
    /// `entry_instruction_`.
    pub entry: Option<usize>,
}
impl Procedure {
    /// `new_variable`.
    pub fn new_variable(&mut self, type_: usize, name: &str) -> usize {
        self.variables.push(Variable { type_, name: name.to_string() });
        self.variables.len() - 1
    }
    /// `new_*_instruction` family.
    pub fn new_call_instruction(&mut self, fn_: usize) -> usize {
        self.instructions.push(Instruction {
            kind: Some(InstructionKind::Call { fn_, params: Vec::new() }),
            next: Vec::new(),
            prev: Vec::new(),
        });
        self.instructions.len() - 1
    }
    /// `new_destruct_instruction`.
    pub fn new_destruct_instruction(&mut self) -> usize {
        self.instructions.push(Instruction {
            kind: Some(InstructionKind::Destruct { variable: usize::MAX }),
            next: Vec::new(),
            prev: Vec::new(),
        });
        self.instructions.len() - 1
    }
    /// `new_return_instruction`.
    pub fn new_return_instruction(&mut self) -> usize {
        self.instructions.push(Instruction {
            kind: Some(InstructionKind::Return),
            next: Vec::new(),
            prev: Vec::new(),
        });
        self.instructions.len() - 1
    }
}
/// `ProcedureBuilder` verbatim — instruction-append
/// helpers with cursor chaining.
pub struct ProcedureBuilder<'a> {
    /// `procedure_`.
    pub procedure: &'a mut Procedure,
    /// `cursors_`.
    pub cursors: Vec<InstructionCursor>,
}
impl<'a> ProcedureBuilder<'a> {
    /// `link_to_cursors` (verbatim — chain all
    /// cursors to the new instruction).
    pub fn link_to_cursors(&mut self, instruction: usize) {
        for c in &self.cursors {
            self.procedure.instructions[c.instruction].next.push(instruction);
            self.procedure.instructions[instruction].prev.push(c.instruction);
        }
    }
    /// `add_destruct` (verbatim — destruct + link +
    /// single cursor).
    pub fn add_destruct(&mut self, variable: usize) {
        let instr = self.procedure.new_destruct_instruction();
        if let Some(InstructionKind::Destruct { variable: v }) =
            &mut self.procedure.instructions[instr].kind
        {
            *v = variable;
        }
        self.link_to_cursors(instr);
        self.cursors = vec![InstructionCursor { instruction: instr }];
    }
    /// `add_return` (verbatim — link, clear cursors).
    pub fn add_return(&mut self) -> usize {
        let instr = self.procedure.new_return_instruction();
        self.link_to_cursors(instr);
        self.cursors.clear();
        instr
    }
    /// `add_call_with_no_variables` (verbatim).
    pub fn add_call_with_no_variables(&mut self, fn_: usize) -> usize {
        let instr = self.procedure.new_call_instruction(fn_);
        self.link_to_cursors(instr);
        self.cursors = vec![InstructionCursor { instruction: instr }];
        instr
    }
    /// `add_call_with_all_variables` (verbatim —
    /// call then set params).
    pub fn add_call_with_all_variables(&mut self, fn_: usize, param_variables: Vec<usize>) -> usize {
        let instr = self.add_call_with_no_variables(fn_);
        if let Some(InstructionKind::Call { params, .. }) =
            &mut self.procedure.instructions[instr].kind
        {
            *params = param_variables;
        }
        instr
    }
}
