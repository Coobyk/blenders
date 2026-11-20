//! Port of `BLI_args.cc` — `bArgs` command-line argument parser with
//! pass-based matching and per-arg documentation.

use std::collections::HashMap;

/// `BA_ArgCallback` — returns consumed arg count (>=0), -1 to stop.
/// Shared (Rc<RefCell>) because one callback registers under both
/// short and long spellings like the C function pointer.
pub type ArgCallback = std::rc::Rc<std::cell::RefCell<dyn FnMut(&[String]) -> i32>>;
/// `bArgPrintFn` — printf sink; we pass formatted strings.
pub type ArgPrintFn<'a> = &'a mut dyn FnMut(&str);

const NO_DOCS: &str = "NO DOCUMENTATION SPECIFIED";

/// `bArgDoc` — doc entry for one argument pair.
pub struct ArgDoc {
    pub short_arg: Option<String>,
    pub long_arg: Option<String>,
    pub documentation: String,
    pub done: bool,
}

/// `bAKey` — (arg, pass, case_str) lookup key; pass -1 = wildcard.
#[derive(Hash, PartialEq, Eq)]
struct AKey {
    arg: String,
    pass: i64,
    case_str: bool,
}

struct BArgument {
    key_arg: String,
    key_pass: i64,
    func_idx: usize,
    doc_idx: usize,
}

/// `bArgs`.
pub struct BArgs<'a> {
    docs: Vec<ArgDoc>,
    /// arg string -> list of (pass, case_str, argument index).
    items: HashMap<String, Vec<(i64, bool, usize)>>,
    arguments: Vec<BArgument>,
    callbacks: Vec<(ArgCallback, usize)>, // (cb, user-data tag)
    argc: usize,
    argv: Vec<String>,
    passes: Vec<i32>,
    print_fn: Option<ArgPrintFn<'a>>,
    current_pass: i64,
    pass_max: i64,
}

impl<'a> BArgs<'a> {
    /// `BLI_args_create(argc, argv)`.
    pub fn create(argv: Vec<String>) -> Self {
        let argc = argv.len();
        BArgs {
            docs: Vec::new(),
            items: HashMap::new(),
            arguments: Vec::new(),
            callbacks: Vec::new(),
            argc,
            argv,
            passes: vec![0; argc],
            print_fn: None,
            current_pass: 0,
            pass_max: 0,
        }
    }
    /// `BLI_args_print_fn_set`.
    pub fn set_print_fn(&mut self, f: ArgPrintFn<'a>) {
        self.print_fn = Some(f);
    }
    /// `BLI_args_pass_set` — set the pass new args are registered on.
    pub fn pass_set(&mut self, current_pass: i32) {
        self.current_pass = current_pass as i64;
        self.pass_max = self.pass_max.max(self.current_pass);
    }
    /// `BLI_args_printf` — formatted print via the print fn.
    pub fn printf(&mut self, s: &str) {
        if let Some(f) = self.print_fn.as_mut() {
            f(s);
        } else {
            print!("{s}");
        }
    }

    fn look_up(&self, arg: &str, pass: i64, case_str: i32) -> Option<usize> {
        // Exact-case-sensitive first, then case-insensitive, honoring
        // pass wildcards like the C keycmp.
        if let Some(list) = self.items.get(arg) {
            for &(p, _cs, idx) in list {
                if p == pass || p == -1 || pass == -1 {
                    return Some(idx);
                }
            }
        }
        if case_str != 0 {
            let lower = arg.to_lowercase();
            for (k, list) in &self.items {
                if k.to_lowercase() == lower {
                    for &(p, _cs, idx) in list {
                        if p == pass || p == -1 || pass == -1 {
                            return Some(idx);
                        }
                    }
                }
            }
        }
        None
    }

    fn internal_docs(
        &mut self,
        short_arg: Option<&str>,
        long_arg: Option<&str>,
        doc: Option<&str>,
    ) -> usize {
        self.docs.push(ArgDoc {
            short_arg: short_arg.map(|s| s.to_string()),
            long_arg: long_arg.map(|s| s.to_string()),
            documentation: doc.unwrap_or(NO_DOCS).to_string(),
            done: false,
        });
        self.docs.len() - 1
    }

    fn internal_add(
        &mut self,
        arg: &str,
        case_str: bool,
        cb: ArgCallback,
        data_tag: usize,
        doc_idx: usize,
    ) {
        let pass = self.current_pass;
        if self.look_up(arg, pass, case_str as i32).is_some() {
            self.printf(&format!(
                "WARNING: conflicting argument\n\ttrying to add '{arg}' on pass {pass}\n"
            ));
        }
        let idx = self.arguments.len();
        self.arguments.push(BArgument {
            key_arg: arg.to_string(),
            key_pass: pass,
            func_idx: self.callbacks.len(),
            doc_idx,
        });
        self.callbacks.push((cb, data_tag));
        self.items
            .entry(arg.to_string())
            .or_default()
            .push((pass, case_str, idx));
    }

    /// `BLI_args_add_case`.
    pub fn add_case(
        &mut self,
        short_arg: Option<&str>,
        short_case: bool,
        long_arg: Option<&str>,
        long_case: bool,
        doc: Option<&str>,
        cb: ArgCallback,
        data_tag: usize,
    ) {
        let d = self.internal_docs(short_arg, long_arg, doc);
        // One callback for both spellings (C shares it too).
        if let Some(s) = short_arg {
            self.internal_add(s, short_case, cb.clone(), data_tag, d);
        }
        if let Some(l) = long_arg {
            self.internal_add(l, long_case, cb, data_tag, d);
        }
    }

    /// `BLI_args_add` — case-insensitive short/long registration.
    pub fn add(
        &mut self,
        short_arg: Option<&str>,
        long_arg: Option<&str>,
        doc: Option<&str>,
        cb: ArgCallback,
        data_tag: usize,
    ) {
        self.add_case(short_arg, false, long_arg, false, doc, cb, data_tag);
    }

    fn internal_doc_print(&mut self, d: usize) {
        let doc = &self.docs[d];
        let mut s = String::new();
        match (&doc.short_arg, &doc.long_arg) {
            (Some(sa), Some(la)) => s.push_str(&format!("{sa} or {la}")),
            (Some(sa), None) => s.push_str(sa),
            (None, Some(la)) => s.push_str(la),
            (None, None) => {}
        }
        s.push_str(&format!(" {}\n\n", doc.documentation));
        self.printf(&s);
    }

    /// `BLI_args_print_arg_doc`.
    pub fn print_arg_doc(&mut self, arg: &str) {
        let mut found = None;
        for pass in 0..=self.pass_max {
            if let Some(idx) = self.look_up(arg, pass, -1) {
                let d = self.arguments[idx].doc_idx;
                if !self.docs[d].done {
                    found = Some(d);
                    break;
                }
            }
        }
        if let Some(d) = found {
            self.internal_doc_print(d);
            self.docs[d].done = true;
        }
    }

    /// `BLI_args_print_other_doc`.
    pub fn print_other_doc(&mut self) {
        for i in 0..self.docs.len() {
            if !self.docs[i].done {
                self.internal_doc_print(i);
            }
        }
    }

    /// `BLI_args_has_other_doc`.
    pub fn has_other_doc(&self) -> bool {
        self.docs.iter().any(|d| !d.done)
    }

    /// `BLI_args_parse` — run `pass`; unmatched args go to `default_cb`.
    pub fn parse(
        &mut self,
        pass: i32,
        mut default_cb: Option<ArgCallback>,
    ) {
        debug_assert!(pass != 0 && pass >= -1);
        let mut i = 1usize;
        while i < self.argc {
            if self.passes[i] == 0 {
                let idx = self.look_up(&self.argv[i], pass as i64, -1);
                let retval = if let Some(idx) = idx {
                    let (cb, _) = &self.callbacks[self.arguments[idx].func_idx];
                    Some(((cb.borrow_mut())(&self.argv[i..]), idx))
                } else if let Some(cb) = default_cb.as_mut() {
                    Some(((cb.borrow_mut())(&self.argv[i..]), usize::MAX))
                } else {
                    None
                };
                if let Some((retval, idx)) = retval {
                    if retval >= 0 {
                        for j in 0..=retval as usize {
                            self.passes[i + j] = pass;
                        }
                        i += retval as usize;
                    } else if retval == -1 {
                        if idx != usize::MAX && self.arguments[idx].key_pass != -1 {
                            self.passes[i] = pass;
                        }
                        break;
                    }
                }
            }
            i += 1;
        }
    }
}
