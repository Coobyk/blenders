//! Port of `intern/path_templates.cc` — `{variable:format}` path
//! template engine: `VariableMap`, tokenizer, format specifiers,
//! `eval_template`, escaping and error reporting.

use std::collections::HashMap;

/// `VariableMap` value — C stores a variant.
#[derive(Clone, Debug, PartialEq)]
pub enum VariableValue {
    /// String variable (sanitized as filename on use).
    String(String),
    /// Filepath variable (verbatim).
    Filepath(String),
    /// Integer variable.
    Integer(i64),
    /// Float variable.
    Float(f64),
}

/// `bke::path_templates::VariableMap`.
#[derive(Clone, Default)]
pub struct VariableMap {
    map: HashMap<String, VariableValue>,
}

impl VariableMap {
    /// `contains`.
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }
    /// `remove` — true when the variable existed.
    pub fn remove(&mut self, name: &str) -> bool {
        self.map.remove(name).is_some()
    }
    /// `add_string` — false when `name` already exists.
    pub fn add_string(&mut self, name: &str, value: &str) -> bool {
        if self.contains(name) {
            return false;
        }
        self.map
            .insert(name.to_string(), VariableValue::String(value.to_string()));
        true
    }
    /// `add_filepath`.
    pub fn add_filepath(&mut self, name: &str, value: &str) -> bool {
        if self.contains(name) {
            return false;
        }
        self.map
            .insert(name.to_string(), VariableValue::Filepath(value.to_string()));
        true
    }
    /// `add_integer`.
    pub fn add_integer(&mut self, name: &str, value: i64) -> bool {
        if self.contains(name) {
            return false;
        }
        self.map.insert(name.to_string(), VariableValue::Integer(value));
        true
    }
    /// `add_float`.
    pub fn add_float(&mut self, name: &str, value: f64) -> bool {
        if self.contains(name) {
            return false;
        }
        self.map.insert(name.to_string(), VariableValue::Float(value));
        true
    }
    /// `get_string`.
    pub fn get_string(&self, name: &str) -> Option<&str> {
        match self.map.get(name) {
            Some(VariableValue::String(s)) => Some(s),
            _ => None,
        }
    }
    /// `get_filepath`.
    pub fn get_filepath(&self, name: &str) -> Option<&str> {
        match self.map.get(name) {
            Some(VariableValue::Filepath(s)) => Some(s),
            _ => None,
        }
    }
    /// `get_integer`.
    pub fn get_integer(&self, name: &str) -> Option<i64> {
        match self.map.get(name) {
            Some(VariableValue::Integer(v)) => Some(*v),
            _ => None,
        }
    }
    /// `get_float`.
    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.map.get(name) {
            Some(VariableValue::Float(v)) => Some(*v),
            _ => None,
        }
    }
    /// `add_filename_only` — stores the basename of `path`.
    pub fn add_filename_only(&mut self, var_name: &str, path: &str) -> bool {
        let base = path.rsplit(['/', '\\']).next().unwrap_or(path);
        self.add_string(var_name, base)
    }
    /// `add_path_up_to_file` — stores `path` with the filename
    /// stripped (dir part).
    pub fn add_path_up_to_file(&mut self, var_name: &str, path: &str) -> bool {
        let end = path
            .rfind(['/', '\\'])
            .map(|i| i + 1)
            .unwrap_or(0);
        self.add_filepath(var_name, &path[..end])
    }
}

/// `ErrorType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorType {
    /// `UNESCAPED_CURLY_BRACE`.
    UnescapedCurlyBrace,
    /// `VARIABLE_SYNTAX`.
    VariableSyntax,
    /// `FORMAT_SPECIFIER`.
    FormatSpecifier,
    /// `UNKNOWN_VARIABLE`.
    UnknownVariable,
}

/// `bke::path_templates::Error`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Error {
    /// `type`.
    pub type_: ErrorType,
    /// `byte_range` — (start, size).
    pub byte_range: (usize, usize),
}

/// `ExpressionInfo` for `next_template_variable_expression`.
#[derive(Clone)]
pub struct ExpressionInfo {
    /// `byte_range` — (start, size).
    pub byte_range: (usize, usize),
    /// `variable_name` — byte range into text.
    pub variable_name: (usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum FormatSpecifierType {
    #[default]
    None,
    Integer,
    Float,
    SyntaxError,
}

#[derive(Clone, Default)]
struct FormatSpecifier {
    type_: FormatSpecifierType,
    integer_digit_count: Option<usize>,
    fractional_digit_count: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokenType {
    VariableExpression,
    LeftCurlyBrace,
    RightCurlyBrace,
    VariableSyntaxError,
    UnescapedCurlyBraceError,
}

struct Token {
    type_: TokenType,
    byte_range: (usize, usize), /* start, one-past-end. */
    variable_name: (usize, usize),
    format: FormatSpecifier,
}

const FORMAT_BUFFER_SIZE: usize = 512;

fn format_int_to_string(format: &FormatSpecifier, integer_value: i64) -> String {
    match format.type_ {
        FormatSpecifierType::None => format!("{}", integer_value),
        FormatSpecifierType::Integer => {
            format!("{:0>width$}", integer_value, width = format.integer_digit_count.unwrap())
        }
        FormatSpecifierType::Float => {
            let mut s = if let Some(w) = format.integer_digit_count {
                format!("{:0>width$}", integer_value, width = w)
            } else {
                format!("{}", integer_value)
            };
            s.push('.');
            for _ in 0..format.fractional_digit_count.unwrap().min(FORMAT_BUFFER_SIZE - 1) {
                s.push('0');
            }
            s
        }
        FormatSpecifierType::SyntaxError => String::new(),
    }
}

fn format_float_to_string(format: &FormatSpecifier, float_value: f64) -> String {
    match format.type_ {
        FormatSpecifierType::None => {
            let mut s = format!("{}", float_value);
            if s.chars().all(|c| "-0123456789".contains(c)) {
                s.push_str(".0");
            }
            s
        }
        FormatSpecifierType::Integer => format_int_to_string(format, float_value.round() as i64),
        FormatSpecifierType::Float => {
            let frac = format.fractional_digit_count.unwrap();
            if let Some(w) = format.integer_digit_count {
                format!("{:0>width$.frac$}", float_value, width = w, frac = frac)
            } else {
                format!("{:.frac$}", float_value, frac = frac)
            }
        }
        FormatSpecifierType::SyntaxError => String::new(),
    }
}

fn parse_format_specifier(spec: &str) -> FormatSpecifier {
    let mut format = FormatSpecifier::default();
    if spec.is_empty() {
        format.type_ = FormatSpecifierType::SyntaxError;
        return format;
    }
    if spec.chars().all(|c| c == '#') {
        format.integer_digit_count = Some(spec.len());
        format.type_ = FormatSpecifierType::Integer;
        return format;
    }
    let dot_index = spec.find('.');
    let only_one_dot = dot_index == spec.rfind('.');
    if spec.chars().all(|c| c == '.' || c == '#') && dot_index.is_some() && only_one_dot {
        let di = dot_index.unwrap();
        let left = &spec[..di];
        let right = &spec[di + 1..];
        if right.is_empty() {
            format.type_ = FormatSpecifierType::SyntaxError;
            return format;
        }
        if !left.is_empty() {
            format.integer_digit_count = Some(left.len());
        }
        format.fractional_digit_count = Some(right.len());
        format.type_ = FormatSpecifierType::Float;
        return format;
    }
    format.type_ = FormatSpecifierType::SyntaxError;
    format
}

fn next_token(path: &[u8], from_char: usize) -> Option<Token> {
    let n = path.len();
    let mut start: i64 = -1;
    let mut format_specifier_split: i64 = -1;
    let mut end: i64 = -1;

    let mut byte_index = from_char;
    while byte_index < n {
        let c = path[byte_index];
        if start == -1 && byte_index + 1 < n && c == b'{' && path[byte_index + 1] == b'{' {
            return Some(Token {
                type_: TokenType::LeftCurlyBrace,
                byte_range: (byte_index, byte_index + 2),
                variable_name: (0, 0),
                format: FormatSpecifier::default(),
            });
        }
        if start == -1 && byte_index + 1 < n && c == b'}' && path[byte_index + 1] == b'}' {
            return Some(Token {
                type_: TokenType::RightCurlyBrace,
                byte_range: (byte_index, byte_index + 2),
                variable_name: (0, 0),
                format: FormatSpecifier::default(),
            });
        }
        if start == -1 && c == b'}' {
            return Some(Token {
                type_: TokenType::UnescapedCurlyBraceError,
                byte_range: (byte_index, byte_index + 1),
                variable_name: (0, 0),
                format: FormatSpecifier::default(),
            });
        }
        if c == b'{' {
            if start != -1 {
                return Some(Token {
                    type_: TokenType::VariableSyntaxError,
                    byte_range: (start as usize, byte_index),
                    variable_name: (0, 0),
                    format: FormatSpecifier::default(),
                });
            }
            start = byte_index as i64;
            format_specifier_split = -1;
            byte_index += 1;
            continue;
        }
        if start == -1 {
            byte_index += 1;
            continue;
        }
        if c == b':' {
            if format_specifier_split == -1 {
                format_specifier_split = byte_index as i64;
            }
            byte_index += 1;
            continue;
        }
        if c == b'}' {
            end = byte_index as i64 + 1;
            break;
        }
        byte_index += 1;
    }

    if start == -1 {
        return None;
    }
    let start = start as usize;
    if end == -1 {
        return Some(Token {
            type_: TokenType::VariableSyntaxError,
            byte_range: (start, n),
            variable_name: (0, 0),
            format: FormatSpecifier::default(),
        });
    }
    let end = end as usize;
    let mut token = Token {
        type_: TokenType::VariableExpression,
        byte_range: (start, end),
        variable_name: (0, 0),
        format: FormatSpecifier::default(),
    };
    if format_specifier_split == -1 {
        token.variable_name = (start + 1, end - 1);
    } else {
        let fs = format_specifier_split as usize;
        token.variable_name = (start + 1, fs);
        token.format = parse_format_specifier(
            std::str::from_utf8(&path[fs + 1..end - 1]).unwrap_or(""),
        );
        if token.format.type_ == FormatSpecifierType::SyntaxError {
            token.type_ = TokenType::VariableSyntaxError;
            return Some(token);
        }
    }
    Some(token)
}

/// `bke::path_templates::next_template_variable_expression`.
pub fn next_template_variable_expression(text: &str, mut starting_offset: usize) -> Option<ExpressionInfo> {
    let path = text.as_bytes();
    while let Some(token) = next_token(path, starting_offset) {
        match token.type_ {
            TokenType::LeftCurlyBrace | TokenType::RightCurlyBrace => {
                starting_offset = token.byte_range.1;
            }
            TokenType::VariableExpression => {
                return Some(ExpressionInfo {
                    byte_range: (token.byte_range.0, token.byte_range.1 - token.byte_range.0),
                    variable_name: (
                        token.variable_name.0,
                        token.variable_name.1 - token.variable_name.0,
                    ),
                });
            }
            TokenType::VariableSyntaxError | TokenType::UnescapedCurlyBraceError => return None,
        }
    }
    None
}

fn parse_template(path: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut bytes_read = 0;
    while bytes_read < path.len() {
        match next_token(path, bytes_read) {
            Some(t) => {
                bytes_read = t.byte_range.1;
                tokens.push(t);
            }
            None => break,
        }
    }
    tokens
}

fn token_to_syntax_error(token: &Token) -> Option<Error> {
    match token.type_ {
        TokenType::VariableSyntaxError => {
            if token.format.type_ == FormatSpecifierType::SyntaxError {
                Some(Error {
                    type_: ErrorType::FormatSpecifier,
                    byte_range: token.byte_range,
                })
            } else {
                Some(Error {
                    type_: ErrorType::VariableSyntax,
                    byte_range: token.byte_range,
                })
            }
        }
        TokenType::UnescapedCurlyBraceError => Some(Error {
            type_: ErrorType::UnescapedCurlyBrace,
            byte_range: token.byte_range,
        }),
        _ => None,
    }
}

/// `BKE_path_contains_template_syntax`.
pub fn bke_path_contains_template_syntax(path: &str) -> bool {
    path.contains('{') || path.contains('}')
}

/// `BKE_path_template_escape`.
pub fn bke_path_template_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        escaped.push(c);
        if c == '{' || c == '}' {
            escaped.push(c);
        }
    }
    escaped
}

fn eval_template(r_out_path: Option<&mut String>, in_path: &str, vars: &VariableMap) -> Vec<Error> {
    let in_bytes = in_path.as_bytes();
    let tokens = parse_template(in_bytes);
    if tokens.is_empty() {
        if let Some(out) = r_out_path {
            out.push_str(in_path);
        }
        return Vec::new();
    }

    let mut errors = Vec::new();
    let mut in_head = 0;
    let mut out = String::new();

    for token in &tokens {
        if let Some(e) = token_to_syntax_error(token) {
            errors.push(e);
            continue;
        }
        let replacement: String = match token.type_ {
            TokenType::LeftCurlyBrace => "{".to_string(),
            TokenType::RightCurlyBrace => "}".to_string(),
            TokenType::VariableExpression => {
                let name = std::str::from_utf8(&in_bytes[token.variable_name.0..token.variable_name.1])
                    .unwrap_or("");
                if let Some(v) = vars.get_string(name) {
                    if token.format.type_ != FormatSpecifierType::None {
                        errors.push(Error {
                            type_: ErrorType::FormatSpecifier,
                            byte_range: token.byte_range,
                        });
                        continue;
                    }
                    /* `BLI_path_make_safe_filename`. */
                    v.chars()
                        .map(|c| if "<>:\"/\\|?* ".contains(c) { '_' } else { c })
                        .collect()
                } else if let Some(v) = vars.get_filepath(name) {
                    if token.format.type_ != FormatSpecifierType::None {
                        errors.push(Error {
                            type_: ErrorType::FormatSpecifier,
                            byte_range: token.byte_range,
                        });
                        continue;
                    }
                    v.to_string()
                } else if let Some(v) = vars.get_integer(name) {
                    format_int_to_string(&token.format, v)
                } else if let Some(v) = vars.get_float(name) {
                    format_float_to_string(&token.format, v)
                } else {
                    errors.push(Error {
                        type_: ErrorType::UnknownVariable,
                        byte_range: token.byte_range,
                    });
                    continue;
                }
            }
            _ => continue,
        };
        out.push_str(&in_path[in_head..token.byte_range.0]);
        out.push_str(&replacement);
        in_head = token.byte_range.1;
    }
    out.push_str(&in_path[in_head..]);

    if let Some(dst) = r_out_path {
        *dst = out;
    }
    errors
}

/// `BKE_path_validate_template` — returns errors only.
pub fn bke_path_validate_template(path: &str, vars: &VariableMap) -> Vec<Error> {
    eval_template(None, path, vars)
}

/// `BKE_path_apply_template` — `path` replaced when error-free.
pub fn bke_path_apply_template(path: &mut String, vars: &VariableMap) -> Vec<Error> {
    let mut evaluated = String::new();
    let errors = eval_template(Some(&mut evaluated), path, vars);
    if errors.is_empty() {
        *path = evaluated;
    }
    errors
}

/// `BKE_path_apply_template_alloc` — returns new path on success.
pub fn bke_path_apply_template_alloc(path: &str, vars: &VariableMap) -> Result<String, Vec<Error>> {
    let mut evaluated = String::new();
    let errors = eval_template(Some(&mut evaluated), path, vars);
    if errors.is_empty() {
        Ok(evaluated)
    } else {
        Err(errors)
    }
}

/// `BKE_path_template_error_to_string`.
pub fn bke_path_template_error_to_string(error: &Error, path: &str) -> String {
    let (start, end) = error.byte_range;
    let subpath = path.get(start..end.min(path.len())).unwrap_or("");
    match error.type_ {
        ErrorType::UnescapedCurlyBrace => format!("Unescaped curly brace '{}'.", subpath),
        ErrorType::VariableSyntax => {
            format!("Invalid or incomplete template expression '{}'.", subpath)
        }
        ErrorType::FormatSpecifier => {
            format!("Invalid format specifier in template expression '{}'.", subpath)
        }
        ErrorType::UnknownVariable => {
            format!("Unknown variable referenced in template expression '{}'.", subpath)
        }
    }
}

/// `BKE_report_path_template_errors`.
pub fn bke_report_path_template_errors(
    reports: &mut dyn FnMut(&str, &str),
    report_type: &str,
    path: &str,
    errors: &[Error],
) {
    let mut msg = format!("Parse errors in path '{}':", path);
    for e in errors {
        msg.push_str(&format!("\n- {}", bke_path_template_error_to_string(e, path)));
    }
    reports(report_type, &msg);
}

/// `BKE_path_template_format_float`.
pub fn bke_path_template_format_float(format_specifier: &str, value: f64) -> Option<String> {
    let format = parse_format_specifier(format_specifier);
    if format.type_ == FormatSpecifierType::SyntaxError {
        return None;
    }
    Some(format_float_to_string(&format, value))
}

/// `BKE_path_template_format_int`.
pub fn bke_path_template_format_int(format_specifier: &str, value: i64) -> Option<String> {
    let format = parse_format_specifier(format_specifier);
    if format.type_ == FormatSpecifierType::SyntaxError {
        return None;
    }
    Some(format_int_to_string(&format, value))
}
