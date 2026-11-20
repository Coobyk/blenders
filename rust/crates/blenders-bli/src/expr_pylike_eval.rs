//! Port of `expr_pylike_eval.cc` — `BLI_expr_pylike_*` python-like
//! expression parser + bytecode evaluator (driver expressions).
//! PORT_NOTE: FP exception flags (fetestexcept) →
//! `result.is_finite()` + explicit div-by-zero checks where upstream
//! relies on FE_DIVBYZERO/FE_INVALID.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum OpCode {
    Const,
    Func1,
    Func2,
    Func3,
    Parameter,
    Min,
    Max,
    Jmp,
    JmpElse,
    JmpOr,
    JmpAnd,
    CmpChain,
}

type UnaryOpFunc = fn(f64) -> f64;
type BinaryOpFunc = fn(f64, f64) -> f64;
type TernaryOpFunc = fn(f64, f64, f64) -> f64;

#[derive(Clone, Copy)]
enum OpArg {
    I(i32),
    D(f64),
    F1(UnaryOpFunc),
    F2(BinaryOpFunc),
    F3(TernaryOpFunc),
}

#[derive(Clone, Copy)]
struct ExprOp {
    opcode: OpCode,
    jmp_offset: i32,
    arg: OpArg,
}

/// `ExprPyLike_Parsed`.
pub struct ExprPyLikeParsed {
    ops: Vec<ExprOp>,
    max_stack: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EvalStatus {
    Success,
    Invalid,
    DivByZero,
    MathError,
    FatalError,
}

impl ExprPyLikeParsed {
    /// `BLI_expr_pylike_is_valid`.
    pub fn is_valid(&self) -> bool {
        !self.ops.is_empty()
    }
    /// `BLI_expr_pylike_is_constant`.
    pub fn is_constant(&self) -> bool {
        self.ops.len() == 1 && self.ops[0].opcode == OpCode::Const
    }
    /// `BLI_expr_pylike_is_using_param`.
    pub fn is_using_param(&self, index: i32) -> bool {
        self.ops
            .iter()
            .any(|op| op.opcode == OpCode::Parameter && matches!(op.arg, OpArg::I(i) if i == index))
    }

    /// `BLI_expr_pylike_eval` — returns the result on success.
    pub fn eval(&self, param_values: &[f64]) -> Result<f64, EvalStatus> {
        if !self.is_valid() || self.max_stack <= 0 || self.max_stack > 1000 {
            return Err(EvalStatus::Invalid);
        }
        let mut stack = vec![0.0f64; self.max_stack as usize];
        let mut sp = 0usize;
        let mut pc: i64 = 0;
        let mut div_zero = false;
        let mut math_err = false;
        let mut check = |v: f64, div_zero: &mut bool, math_err: &mut bool| {
            if v.is_nan() || (v.is_infinite()) {
                *math_err = true;
            }
        };
        while pc >= 0 && (pc as usize) < self.ops.len() {
            let op = self.ops[pc as usize];
            match op.opcode {
                OpCode::Const => {
                    if sp >= self.max_stack as usize {
                        return Err(EvalStatus::FatalError);
                    }
                    if let OpArg::D(d) = op.arg {
                        stack[sp] = d;
                    }
                    sp += 1;
                }
                OpCode::Parameter => {
                    if let OpArg::I(i) = op.arg {
                        if sp >= self.max_stack as usize || i as usize >= param_values.len() {
                            return Err(EvalStatus::FatalError);
                        }
                        stack[sp] = param_values[i as usize];
                    }
                    sp += 1;
                }
                OpCode::Func1 => {
                    if sp < 1 {
                        return Err(EvalStatus::FatalError);
                    }
                    if let OpArg::F1(f) = op.arg {
                        let a = stack[sp - 1];
                        stack[sp - 1] = f(a);
                        if a != 0.0 && (stack[sp - 1]).is_infinite() && f(a) == a / 0.0 {
                            div_zero = true;
                        }
                        check(stack[sp - 1], &mut div_zero, &mut math_err);
                    }
                }
                OpCode::Func2 => {
                    if sp < 2 {
                        return Err(EvalStatus::FatalError);
                    }
                    if let OpArg::F2(f) = op.arg {
                        stack[sp - 2] = f(stack[sp - 2], stack[sp - 1]);
                        check(stack[sp - 2], &mut div_zero, &mut math_err);
                    }
                    sp -= 1;
                }
                OpCode::Func3 => {
                    if sp < 3 {
                        return Err(EvalStatus::FatalError);
                    }
                    if let OpArg::F3(f) = op.arg {
                        stack[sp - 3] = f(stack[sp - 3], stack[sp - 2], stack[sp - 1]);
                        check(stack[sp - 3], &mut div_zero, &mut math_err);
                    }
                    sp -= 2;
                }
                OpCode::Min => {
                    if let OpArg::I(n) = op.arg {
                        if sp < n as usize {
                            return Err(EvalStatus::FatalError);
                        }
                        for _ in 1..n {
                            stack[sp - 2] = stack[sp - 2].min(stack[sp - 1]);
                            sp -= 1;
                        }
                    }
                }
                OpCode::Max => {
                    if let OpArg::I(n) = op.arg {
                        if sp < n as usize {
                            return Err(EvalStatus::FatalError);
                        }
                        for _ in 1..n {
                            stack[sp - 2] = stack[sp - 2].max(stack[sp - 1]);
                            sp -= 1;
                        }
                    }
                }
                OpCode::Jmp => pc += op.jmp_offset as i64,
                OpCode::JmpElse => {
                    if sp < 1 {
                        return Err(EvalStatus::FatalError);
                    }
                    sp -= 1;
                    if stack[sp] == 0.0 {
                        pc += op.jmp_offset as i64;
                    }
                }
                OpCode::JmpOr | OpCode::JmpAnd => {
                    if sp < 1 {
                        return Err(EvalStatus::FatalError);
                    }
                    if !(stack[sp - 1] != 0.0) == !(op.opcode == OpCode::JmpOr) {
                        pc += op.jmp_offset as i64;
                    } else {
                        sp -= 1;
                    }
                }
                OpCode::CmpChain => {
                    if sp < 2 {
                        return Err(EvalStatus::FatalError);
                    }
                    if let OpArg::F2(f) = op.arg {
                        if f(stack[sp - 2], stack[sp - 1]) == 0.0 {
                            stack[sp - 2] = 0.0;
                            pc += op.jmp_offset as i64;
                        } else {
                            stack[sp - 2] = stack[sp - 1];
                        }
                    }
                    sp -= 1;
                }
            }
            pc += 1;
        }
        if sp != 1 || pc as usize != self.ops.len() {
            return Err(EvalStatus::FatalError);
        }
        if math_err {
            return Err(EvalStatus::MathError);
        }
        if div_zero {
            return Err(EvalStatus::DivByZero);
        }
        Ok(stack[0])
    }
}

// ---- builtin ops ----

fn op_negate(a: f64) -> f64 {
    -a
}
fn op_mul(a: f64, b: f64) -> f64 {
    a * b
}
fn op_div(a: f64, b: f64) -> f64 {
    a / b
}
fn op_add(a: f64, b: f64) -> f64 {
    a + b
}
fn op_sub(a: f64, b: f64) -> f64 {
    a - b
}
fn op_radians(a: f64) -> f64 {
    a * std::f64::consts::PI / 180.0
}
fn op_degrees(a: f64) -> f64 {
    a * 180.0 / std::f64::consts::PI
}
fn op_log2(a: f64, b: f64) -> f64 {
    a.ln() / b.ln()
}
fn op_lerp(a: f64, b: f64, x: f64) -> f64 {
    a * (1.0 - x) + b * x
}
fn op_clamp(a: f64) -> f64 {
    a.clamp(0.0, 1.0)
}
fn op_clamp3(a: f64, minv: f64, maxv: f64) -> f64 {
    a.clamp(minv, maxv)
}
fn op_smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn op_not(a: f64) -> f64 {
    if a != 0.0 { 0.0 } else { 1.0 }
}
fn op_eq(a: f64, b: f64) -> f64 {
    (a == b) as i32 as f64
}
fn op_ne(a: f64, b: f64) -> f64 {
    (a != b) as i32 as f64
}
fn op_lt(a: f64, b: f64) -> f64 {
    (a < b) as i32 as f64
}
fn op_le(a: f64, b: f64) -> f64 {
    (a <= b) as i32 as f64
}
fn op_gt(a: f64, b: f64) -> f64 {
    (a > b) as i32 as f64
}
fn op_ge(a: f64, b: f64) -> f64 {
    (a >= b) as i32 as f64
}

const BUILTIN_CONSTS: &[(&str, f64)] =
    &[("pi", std::f64::consts::PI), ("True", 1.0), ("False", 0.0)];

#[derive(Clone, Copy)]
enum BuiltinFunc {
    F1(UnaryOpFunc),
    F2(BinaryOpFunc),
    F3(TernaryOpFunc),
}
impl BuiltinFunc {
    fn arg_count(&self) -> i32 {
        match self {
            BuiltinFunc::F1(_) => 1,
            BuiltinFunc::F2(_) => 2,
            BuiltinFunc::F3(_) => 3,
        }
    }
}

const BUILTIN_OPS: &[(&str, BuiltinFunc)] = &[
    ("radians", BuiltinFunc::F1(op_radians)),
    ("degrees", BuiltinFunc::F1(op_degrees)),
    ("abs", BuiltinFunc::F1(f64::abs)),
    ("fabs", BuiltinFunc::F1(f64::abs)),
    ("floor", BuiltinFunc::F1(f64::floor)),
    ("ceil", BuiltinFunc::F1(f64::ceil)),
    ("trunc", BuiltinFunc::F1(f64::trunc)),
    ("round", BuiltinFunc::F1(f64::round)),
    ("int", BuiltinFunc::F1(f64::trunc)),
    ("sin", BuiltinFunc::F1(f64::sin)),
    ("cos", BuiltinFunc::F1(f64::cos)),
    ("tan", BuiltinFunc::F1(f64::tan)),
    ("asin", BuiltinFunc::F1(f64::asin)),
    ("acos", BuiltinFunc::F1(f64::acos)),
    ("atan", BuiltinFunc::F1(f64::atan)),
    ("atan2", BuiltinFunc::F2(f64::atan2)),
    ("exp", BuiltinFunc::F1(f64::exp)),
    ("log", BuiltinFunc::F1(f64::ln)),
    ("log", BuiltinFunc::F2(op_log2)),
    ("sqrt", BuiltinFunc::F1(f64::sqrt)),
    ("pow", BuiltinFunc::F2(f64::powf)),
    ("fmod", BuiltinFunc::F2(|a, b| a % b)),
    ("lerp", BuiltinFunc::F3(op_lerp)),
    ("clamp", BuiltinFunc::F1(op_clamp)),
    ("clamp", BuiltinFunc::F3(op_clamp3)),
    ("smoothstep", BuiltinFunc::F3(op_smoothstep)),
];

// Tokens: 2-char ids via MAKE_CHAR2; single chars = their byte.
const fn mk2(a: u8, b: u8) -> i16 {
    ((a as i16) << 8) | b as i16
}
const TOKEN_ID: i16 = mk2(b'I', b'D');
const TOKEN_NUMBER: i16 = mk2(b'0', b'0');
const TOKEN_GE: i16 = mk2(b'>', b'=');
const TOKEN_LE: i16 = mk2(b'<', b'=');
const TOKEN_NE: i16 = mk2(b'!', b'=');
const TOKEN_EQ: i16 = mk2(b'=', b'=');
const TOKEN_AND: i16 = mk2(b'A', b'N');
const TOKEN_OR: i16 = mk2(b'O', b'R');
const TOKEN_NOT: i16 = mk2(b'N', b'O');
const TOKEN_IF: i16 = mk2(b'I', b'F');
const TOKEN_ELSE: i16 = mk2(b'E', b'L');

const TOKEN_EQ_CHARS: &[u8] = b"!=><";
const TOKEN_CHARS: &[u8] = b"~`!@#$%^&*+-=/\\?:;<>(){}[]|.,\"'";
const KEYWORDS: &[(&str, i16)] = &[
    ("and", TOKEN_AND),
    ("or", TOKEN_OR),
    ("not", TOKEN_NOT),
    ("if", TOKEN_IF),
    ("else", TOKEN_ELSE),
];

struct ParseState<'a> {
    param_names: &'a [&'a str],
    expr: &'a [u8],
    cur: usize,
    token: i16,
    tokenbuf: Vec<u8>,
    tokenval: f64,
    last_jmp: usize,
    ops: Vec<ExprOp>,
    stack_ptr: i32,
    max_stack: i32,
}

impl<'a> ParseState<'a> {
    fn peek(&self) -> u8 {
        *self.expr.get(self.cur).unwrap_or(&0)
    }
    fn at(&self, off: usize) -> u8 {
        *self.expr.get(self.cur + off).unwrap_or(&0)
    }
    fn token_str(&self) -> &str {
        std::str::from_utf8(&self.tokenbuf).unwrap_or("")
    }

    fn add_op(&mut self, code: OpCode, stack_delta: i32) {
        self.stack_ptr = (self.stack_ptr + stack_delta).max(0);
        self.max_stack = self.max_stack.max(self.stack_ptr);
        self.ops.push(ExprOp {
            opcode: code,
            jmp_offset: 0,
            arg: OpArg::I(0),
        });
    }
    fn add_jump(&mut self, code: OpCode) -> usize {
        self.add_op(code, -1);
        self.last_jmp = self.ops.len();
        self.ops.len()
    }
    fn set_jump(&mut self, jump: usize) {
        self.last_jmp = self.ops.len();
        self.ops[jump - 1].jmp_offset = (self.ops.len() - jump) as i32;
    }

    /// `parse_add_func` — with const-folding of builtin ops.
    fn add_func(&mut self, f: BuiltinFunc) -> bool {
        let jmp_gap = self.ops.len() - self.last_jmp;
        let const_at = |ops: &Vec<ExprOp>, i: usize| -> Option<f64> {
            let op = ops[i];
            if op.opcode == OpCode::Const {
                if let OpArg::D(d) = op.arg {
                    return Some(d);
                }
            }
            None
        };
        match f {
            BuiltinFunc::F1(func) => {
                if jmp_gap >= 1 {
                    if let Some(a) = const_at(&self.ops, self.ops.len() - 1) {
                        let r = func(a);
                        if r.is_finite() {
                            self.ops.last_mut().unwrap().arg = OpArg::D(r);
                            return true;
                        }
                    }
                }
                self.add_op(OpCode::Func1, 0);
                self.ops.last_mut().unwrap().arg = OpArg::F1(func);
            }
            BuiltinFunc::F2(func) => {
                if jmp_gap >= 2 {
                    if let (Some(a), Some(b)) = (
                        const_at(&self.ops, self.ops.len() - 2),
                        const_at(&self.ops, self.ops.len() - 1),
                    ) {
                        let r = func(a, b);
                        if r.is_finite() {
                            let n = self.ops.len();
                            self.ops[n - 2].arg = OpArg::D(r);
                            self.ops.pop();
                            self.stack_ptr -= 1;
                            return true;
                        }
                    }
                }
                self.add_op(OpCode::Func2, -1);
                self.ops.last_mut().unwrap().arg = OpArg::F2(func);
            }
            BuiltinFunc::F3(func) => {
                if jmp_gap >= 3 {
                    if let (Some(a), Some(b), Some(c)) = (
                        const_at(&self.ops, self.ops.len() - 3),
                        const_at(&self.ops, self.ops.len() - 2),
                        const_at(&self.ops, self.ops.len() - 1),
                    ) {
                        let r = func(a, b, c);
                        if r.is_finite() {
                            let n = self.ops.len();
                            self.ops[n - 3].arg = OpArg::D(r);
                            self.ops.pop();
                            self.ops.pop();
                            self.stack_ptr -= 2;
                            return true;
                        }
                    }
                }
                self.add_op(OpCode::Func3, -2);
                self.ops.last_mut().unwrap().arg = OpArg::F3(func);
            }
        }
        true
    }

    fn next_token(&mut self) -> bool {
        while self.peek().is_ascii_whitespace() {
            self.cur += 1;
        }
        let c = self.peek();
        if c == 0 {
            self.token = 0;
            return true;
        }
        if c.is_ascii_digit() || (c == b'.' && self.at(1).is_ascii_digit()) {
            self.tokenbuf.clear();
            let mut is_float = false;
            while self.peek().is_ascii_digit() {
                self.tokenbuf.push(self.peek());
                self.cur += 1;
            }
            if self.peek() == b'.' {
                is_float = true;
                self.tokenbuf.push(b'.');
                self.cur += 1;
                while self.peek().is_ascii_digit() {
                    self.tokenbuf.push(self.peek());
                    self.cur += 1;
                }
            }
            if self.peek() == b'e' || self.peek() == b'E' {
                is_float = true;
                self.tokenbuf.push(self.peek());
                self.cur += 1;
                if self.peek() == b'+' || self.peek() == b'-' {
                    self.tokenbuf.push(self.peek());
                    self.cur += 1;
                }
                if !self.peek().is_ascii_digit() {
                    return false;
                }
                while self.peek().is_ascii_digit() {
                    self.tokenbuf.push(self.peek());
                    self.cur += 1;
                }
            }
            if !is_float && self.tokenbuf.first() == Some(&b'0') {
                if self.tokenbuf[1..].iter().any(|&b| b != b'0') {
                    return false;
                }
            }
            self.token = TOKEN_NUMBER;
            self.tokenval = self.token_str().parse::<f64>().unwrap_or(0.0);
            return true;
        }
        if self.at(1) == b'=' && TOKEN_EQ_CHARS.contains(&c) {
            self.token = mk2(c, b'=');
            self.cur += 2;
            return true;
        }
        if TOKEN_CHARS.contains(&c) {
            self.token = c as i16;
            self.cur += 1;
            return true;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            self.tokenbuf.clear();
            while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
                self.tokenbuf.push(self.peek());
                self.cur += 1;
            }
            for (name, tok) in KEYWORDS {
                if self.token_str() == *name {
                    self.token = *tok;
                    return true;
                }
            }
            self.token = TOKEN_ID;
            return true;
        }
        false
    }

    fn parse_function_args(&mut self) -> i32 {
        if !self.next_token() || self.token != b'(' as i16 || !self.next_token() {
            return -1;
        }
        let mut count = 0;
        loop {
            if !self.parse_expr() {
                return -1;
            }
            count += 1;
            match self.token {
                t if t == b',' as i16 => {
                    if !self.next_token() {
                        return -1;
                    }
                }
                t if t == b')' as i16 => {
                    if !self.next_token() {
                        return -1;
                    }
                    return count;
                }
                _ => return -1,
            }
        }
    }

    fn parse_unary(&mut self) -> bool {
        match self.token {
            t if t == b'+' as i16 => self.next_token() && self.parse_unary(),
            t if t == b'-' as i16 => {
                if !(self.next_token() && self.parse_unary()) {
                    return false;
                }
                self.add_func(BuiltinFunc::F1(op_negate))
            }
            t if t == b'(' as i16 => {
                self.next_token()
                    && self.parse_expr()
                    && self.token == b')' as i16
                    && self.next_token()
            }
            TOKEN_NUMBER => {
                let v = self.tokenval;
                self.add_op(OpCode::Const, 1);
                self.ops.last_mut().unwrap().arg = OpArg::D(v);
                self.next_token()
            }
            TOKEN_ID => {
                for (i, name) in self.param_names.iter().enumerate().rev() {
                    if self.token_str() == *name {
                        self.add_op(OpCode::Parameter, 1);
                        self.ops.last_mut().unwrap().arg = OpArg::I(i as i32);
                        return self.next_token();
                    }
                }
                for (name, val) in BUILTIN_CONSTS {
                    if self.token_str() == *name {
                        self.add_op(OpCode::Const, 1);
                        self.ops.last_mut().unwrap().arg = OpArg::D(*val);
                        return self.next_token();
                    }
                }
                for i in 0..BUILTIN_OPS.len() {
                    if self.token_str() == BUILTIN_OPS[i].0 {
                        let args = self.parse_function_args();
                        let mut i = i;
                        if args != BUILTIN_OPS[i].1.arg_count() {
                            let name = BUILTIN_OPS[i].0;
                            for j in i + 1..BUILTIN_OPS.len() {
                                if BUILTIN_OPS[j].0 == name
                                    && BUILTIN_OPS[j].1.arg_count() == args
                                {
                                    i = j;
                                    break;
                                }
                            }
                        }
                        if args != BUILTIN_OPS[i].1.arg_count() {
                            return false;
                        }
                        return self.add_func(BUILTIN_OPS[i].1);
                    }
                }
                if self.token_str() == "min" {
                    let count = self.parse_function_args();
                    if count <= 0 {
                        return false;
                    }
                    self.add_op(OpCode::Min, 1 - count);
                    self.ops.last_mut().unwrap().arg = OpArg::I(count);
                    return true;
                }
                if self.token_str() == "max" {
                    let count = self.parse_function_args();
                    if count <= 0 {
                        return false;
                    }
                    self.add_op(OpCode::Max, 1 - count);
                    self.ops.last_mut().unwrap().arg = OpArg::I(count);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn parse_mul(&mut self) -> bool {
        if !self.parse_unary() {
            return false;
        }
        loop {
            match self.token {
                t if t == b'*' as i16 => {
                    if !(self.next_token() && self.parse_unary()) {
                        return false;
                    }
                    self.add_func(BuiltinFunc::F2(op_mul));
                }
                t if t == b'/' as i16 => {
                    if !(self.next_token() && self.parse_unary()) {
                        return false;
                    }
                    self.add_func(BuiltinFunc::F2(op_div));
                }
                _ => return true,
            }
        }
    }

    fn parse_add(&mut self) -> bool {
        if !self.parse_mul() {
            return false;
        }
        loop {
            match self.token {
                t if t == b'+' as i16 => {
                    if !(self.next_token() && self.parse_mul()) {
                        return false;
                    }
                    self.add_func(BuiltinFunc::F2(op_add));
                }
                t if t == b'-' as i16 => {
                    if !(self.next_token() && self.parse_mul()) {
                        return false;
                    }
                    self.add_func(BuiltinFunc::F2(op_sub));
                }
                _ => return true,
            }
        }
    }

    fn parse_cmp_chain(&mut self, cur_func: BinaryOpFunc) -> bool {
        if let Some(next_func) = parse_get_cmp_func(self.token) {
            self.add_op(OpCode::CmpChain, -1);
            self.ops.last_mut().unwrap().arg = OpArg::F2(cur_func);
            let jump = {
                self.last_jmp = self.ops.len();
                self.ops.len()
            };
            if !(self.next_token() && self.parse_add()) {
                return false;
            }
            if !self.parse_cmp_chain(next_func) {
                return false;
            }
            self.set_jump(jump);
        } else {
            self.add_func(BuiltinFunc::F2(cur_func));
        }
        true
    }

    fn parse_cmp(&mut self) -> bool {
        if !self.parse_add() {
            return false;
        }
        if let Some(func) = parse_get_cmp_func(self.token) {
            if !(self.next_token() && self.parse_add()) {
                return false;
            }
            return self.parse_cmp_chain(func);
        }
        true
    }

    fn parse_not(&mut self) -> bool {
        if self.token == TOKEN_NOT {
            if !(self.next_token() && self.parse_not()) {
                return false;
            }
            self.add_func(BuiltinFunc::F1(op_not));
            return true;
        }
        self.parse_cmp()
    }

    fn parse_and(&mut self) -> bool {
        if !self.parse_not() {
            return false;
        }
        if self.token == TOKEN_AND {
            let jump = self.add_jump(OpCode::JmpAnd);
            if !(self.next_token() && self.parse_and()) {
                return false;
            }
            self.set_jump(jump);
        }
        true
    }

    fn parse_or(&mut self) -> bool {
        if !self.parse_and() {
            return false;
        }
        if self.token == TOKEN_OR {
            let jump = self.add_jump(OpCode::JmpOr);
            if !(self.next_token() && self.parse_or()) {
                return false;
            }
            self.set_jump(jump);
        }
        true
    }

    fn parse_expr(&mut self) -> bool {
        let prev_last_jmp = self.last_jmp;
        let start = {
            self.last_jmp = self.ops.len();
            self.ops.len()
        };
        if !self.parse_or() {
            return false;
        }
        if self.token == TOKEN_IF {
            let size = self.ops.len() - start;
            let body: Vec<ExprOp> = self.ops[start..].to_vec();
            self.ops.truncate(start);
            self.last_jmp = start;
            self.stack_ptr -= 1;
            if !self.next_token()
                || !self.parse_or()
                || self.token != TOKEN_ELSE
                || !self.next_token()
            {
                return false;
            }
            let jmp_else = self.add_jump(OpCode::JmpElse);
            self.ops.extend_from_slice(&body);
            self.stack_ptr += 1;
            let jmp_end = self.add_jump(OpCode::Jmp);
            self.set_jump(jmp_else);
            if !self.parse_expr() {
                return false;
            }
            self.set_jump(jmp_end);
        } else if self.last_jmp == start {
            self.last_jmp = prev_last_jmp;
        }
        true
    }
}

fn parse_get_cmp_func(token: i16) -> Option<BinaryOpFunc> {
    Some(match token {
        TOKEN_EQ => op_eq,
        TOKEN_NE => op_ne,
        t if t == b'>' as i16 => op_gt,
        TOKEN_GE => op_ge,
        t if t == b'<' as i16 => op_lt,
        TOKEN_LE => op_le,
        _ => return None,
    })
}

/// `BLI_expr_pylike_parse` — None on parse failure.
pub fn expr_pylike_parse(expression: &str, param_names: &[&str]) -> Option<ExprPyLikeParsed> {
    let mut state = ParseState {
        param_names,
        expr: expression.as_bytes(),
        cur: 0,
        token: 0,
        tokenbuf: Vec::new(),
        tokenval: 0.0,
        last_jmp: 0,
        ops: Vec::new(),
        stack_ptr: 0,
        max_stack: 0,
    };
    if !state.next_token() || !state.parse_expr() || state.token != 0 {
        return None;
    }
    Some(ExprPyLikeParsed {
        ops: state.ops,
        max_stack: state.max_stack,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arithmetic_and_logic() {
        let e = expr_pylike_parse("1 + 2 * 3", &[]).unwrap();
        assert_eq!(e.eval(&[]).unwrap(), 7.0);
        let e = expr_pylike_parse("frame * 2", &["frame"]).unwrap();
        assert_eq!(e.eval(&[21.0]).unwrap(), 42.0);
        let e = expr_pylike_parse("x if x > 0 else -x", &["x"]).unwrap();
        assert_eq!(e.eval(&[-3.0]).unwrap(), 3.0);
        let e = expr_pylike_parse("min(a, 3, 9) + max(0, b)", &["a", "b"]).unwrap();
        assert_eq!(e.eval(&[5.0, -2.0]).unwrap(), 3.0);
        let e = expr_pylike_parse("a > 1 and a < 3 or b", &["a", "b"]).unwrap();
        assert_eq!(e.eval(&[2.0, 0.0]).unwrap(), 1.0);
        assert!(expr_pylike_parse("1 +", &[]).is_none());
    }
}
