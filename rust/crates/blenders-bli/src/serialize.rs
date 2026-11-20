//! Port of `serialize.cc` — `blender::io::serialize` Value/Array/Dictionary
//! model plus `JsonFormatter` (ordered-key JSON in/out).

use std::cell::RefCell;
use std::rc::Rc;

/// `serialize::Value::eValueType`.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Double(f64),
    Str(String),
    Array(ArrayValue),
    Dict(DictionaryValue),
}

/// `serialize::ArrayValue` — shared, appendable array.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrayValue(pub Rc<RefCell<Vec<Value>>>);

impl ArrayValue {
    pub fn new() -> Self {
        ArrayValue(Rc::new(RefCell::new(Vec::new())))
    }
    pub fn append(&self, v: Value) {
        self.0.borrow_mut().push(v);
    }
    /// `ArrayValue::append_bool/int/double/str/null`.
    pub fn append_bool(&self, v: bool) {
        self.append(Value::Bool(v));
    }
    pub fn append_int(&self, v: i64) {
        self.append(Value::Int(v));
    }
    pub fn append_double(&self, v: f64) {
        self.append(Value::Double(v));
    }
    pub fn append_str(&self, v: impl Into<String>) {
        self.append(Value::Str(v.into()));
    }
    pub fn append_null(&self) {
        self.append(Value::Null);
    }
    /// `ArrayValue::append_dict` — append a dict, return it for filling.
    pub fn append_dict(&self) -> DictionaryValue {
        let d = DictionaryValue::new();
        self.append(Value::Dict(d.clone()));
        d
    }
    /// `ArrayValue::append_array`.
    pub fn append_array(&self) -> ArrayValue {
        let a = ArrayValue::new();
        self.append(Value::Array(a.clone()));
        a
    }
    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// `serialize::DictionaryValue` — ordered key/value map (insertion order
/// like `nlohmann::ordered_json`).
#[derive(Clone, Debug, PartialEq)]
pub struct DictionaryValue(pub Rc<RefCell<Vec<(String, Value)>>>);

impl DictionaryValue {
    pub fn new() -> Self {
        DictionaryValue(Rc::new(RefCell::new(Vec::new())))
    }
    pub fn append(&self, key: impl Into<String>, v: Value) {
        let key = key.into();
        let mut m = self.0.borrow_mut();
        if let Some(e) = m.iter_mut().find(|(k, _)| *k == key) {
            e.1 = v;
        } else {
            m.push((key, v));
        }
    }
    pub fn append_int(&self, key: impl Into<String>, v: i64) {
        self.append(key, Value::Int(v));
    }
    pub fn append_bool(&self, key: impl Into<String>, v: bool) {
        self.append(key, Value::Bool(v));
    }
    pub fn append_double(&self, key: impl Into<String>, v: f64) {
        self.append(key, Value::Double(v));
    }
    pub fn append_str(&self, key: impl Into<String>, v: impl Into<String>) {
        self.append(key, Value::Str(v.into()));
    }
    pub fn append_dict(&self, key: impl Into<String>) -> DictionaryValue {
        let d = DictionaryValue::new();
        self.append(key, Value::Dict(d.clone()));
        d
    }
    pub fn append_array(&self, key: impl Into<String>) -> ArrayValue {
        let a = ArrayValue::new();
        self.append(key, Value::Array(a.clone()));
        a
    }
    fn lookup(&self, key: &str) -> Option<Value> {
        self.0
            .borrow()
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }
    /// `DictionaryValue::lookup_str/int/bool/double`.
    pub fn lookup_str(&self, key: &str) -> Option<String> {
        match self.lookup(key)? {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn lookup_int(&self, key: &str) -> Option<i64> {
        match self.lookup(key)? {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }
    pub fn lookup_bool(&self, key: &str) -> Option<bool> {
        match self.lookup(key)? {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }
    pub fn lookup_double(&self, key: &str) -> Option<f64> {
        match self.lookup(key)? {
            Value::Double(d) => Some(d),
            Value::Int(i) => Some(i as f64),
            _ => None,
        }
    }
}

// ----- JSON output (convert_to_json + dump) -----

fn escape_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn dump_value(v: &Value, indent: Option<usize>, depth: usize, out: &mut String) {
    let pad = |d: usize| indent.map(|n| " ".repeat(n * d)).unwrap_or_default();
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(i) => out.push_str(&i.to_string()),
        Value::Double(d) => {
            if d.fract() == 0.0 && d.is_finite() {
                out.push_str(&format!("{d:.1}"));
            } else {
                out.push_str(&format!("{d}"));
            }
        }
        Value::Str(s) => escape_str(s, out),
        Value::Array(a) => {
            let items = a.0.borrow();
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if indent.is_some() {
                    out.push('\n');
                    out.push_str(&pad(depth + 1));
                }
                dump_value(item, indent, depth + 1, out);
            }
            if indent.is_some() {
                out.push('\n');
                out.push_str(&pad(depth));
            }
            out.push(']');
        }
        Value::Dict(d) => {
            let items = d.0.borrow();
            if items.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            for (i, (k, val)) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if indent.is_some() {
                    out.push('\n');
                    out.push_str(&pad(depth + 1));
                }
                escape_str(k, out);
                out.push(':');
                if indent.is_some() {
                    out.push(' ');
                }
                dump_value(val, indent, depth + 1, out);
            }
            if indent.is_some() {
                out.push('\n');
                out.push_str(&pad(depth));
            }
            out.push('}');
        }
    }
}

// ----- JSON input (convert_from_json) -----

struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.pos < self.b.len() && self.b[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.b.get(self.pos).copied()
    }
    fn value(&mut self) -> Option<Value> {
        self.ws();
        match self.peek()? {
            b'{' => self.dict(),
            b'[' => self.array(),
            b'"' => self.string().map(Value::Str),
            b't' => self.lit("true").map(|_| Value::Bool(true)),
            b'f' => self.lit("false").map(|_| Value::Bool(false)),
            b'n' => self.lit("null").map(|_| Value::Null),
            _ => self.number(),
        }
    }
    fn lit(&mut self, s: &str) -> Option<()> {
        if self.b[self.pos..].starts_with(s.as_bytes()) {
            self.pos += s.len();
            Some(())
        } else {
            None
        }
    }
    fn string(&mut self) -> Option<String> {
        if self.peek()? != b'"' {
            return None;
        }
        self.pos += 1;
        let mut s = String::new();
        while let Some(c) = self.peek() {
            self.pos += 1;
            match c {
                b'"' => return Some(s),
                b'\\' => {
                    let e = self.peek()?;
                    self.pos += 1;
                    match e {
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'b' => s.push('\u{0008}'),
                        b'f' => s.push('\u{000c}'),
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.pos..self.pos + 4)?).ok()?;
                            let cp = u32::from_str_radix(hex, 16).ok()?;
                            self.pos += 4;
                            s.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                        }
                        _ => return None,
                    }
                }
                c if c < 0x80 => s.push(c as char),
                c => {
                    // UTF-8 multi-byte: copy bytes through.
                    let start = self.pos - 1;
                    let len = crate::string_utf8::str_utf8_size_safe(c).max(1) as usize;
                    let end = (start + len).min(self.b.len());
                    s.push_str(&String::from_utf8_lossy(&self.b[start..end]));
                    self.pos = end;
                }
            }
        }
        None
    }
    fn number(&mut self) -> Option<Value> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while self
            .peek()
            .map(|c| c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'+' || c == b'-')
            .unwrap_or(false)
        {
            self.pos += 1;
        }
        let txt = std::str::from_utf8(&self.b[start..self.pos]).ok()?;
        if txt.contains(['.', 'e', 'E']) {
            txt.parse::<f64>().ok().map(Value::Double)
        } else {
            txt.parse::<i64>().ok().map(Value::Int)
        }
    }
    fn dict(&mut self) -> Option<Value> {
        self.pos += 1; // '{'
        let d = DictionaryValue::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Some(Value::Dict(d));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            if self.peek()? != b':' {
                return None;
            }
            self.pos += 1;
            let v = self.value()?;
            d.append(key, v);
            self.ws();
            match self.peek()? {
                b',' => {
                    self.pos += 1;
                }
                b'}' => {
                    self.pos += 1;
                    return Some(Value::Dict(d));
                }
                _ => return None,
            }
        }
    }
    fn array(&mut self) -> Option<Value> {
        self.pos += 1; // '['
        let a = ArrayValue::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Some(Value::Array(a));
        }
        loop {
            let v = self.value()?;
            a.append(v);
            self.ws();
            match self.peek()? {
                b',' => {
                    self.pos += 1;
                }
                b']' => {
                    self.pos += 1;
                    return Some(Value::Array(a));
                }
                _ => return None,
            }
        }
    }
}

/// `serialize::JsonFormatter`.
pub struct JsonFormatter {
    /// `indentation_len` — pretty-print width; 0 = compact.
    pub indentation_len: usize,
}

impl JsonFormatter {
    /// `JsonFormatter::serialize` — dump to a string.
    pub fn serialize(&self, value: &Value) -> String {
        let mut out = String::new();
        let indent = (self.indentation_len > 0).then_some(self.indentation_len);
        dump_value(value, indent, 0, &mut out);
        out
    }
    /// `JsonFormatter::deserialize`.
    pub fn deserialize(&self, text: &str) -> Option<Value> {
        let mut p = Parser {
            b: text.as_bytes(),
            pos: 0,
        };
        p.value()
    }
}

/// `serialize::write_json_file`.
pub fn write_json_file(path: &str, value: &Value) -> std::io::Result<()> {
    let f = JsonFormatter { indentation_len: 0 };
    std::fs::write(path, f.serialize(value))
}

/// `serialize::read_json_file`.
pub fn read_json_file(path: &str) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    JsonFormatter { indentation_len: 0 }.deserialize(&text)
}
