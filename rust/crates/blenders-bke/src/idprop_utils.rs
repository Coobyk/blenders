//! Port of `intern/idprop_utils.cc` — `IDP_repr_fn`/`IDP_reprN`/
//! `IDP_print`/`IDP_type_str` (Python-literal repr of IDProperties).
//!
//! `ReprState` collapses into a `String` sink. `IDP_ID` repr needs
//! `BKE_idtype_idcode_to_name_plural` + the ID's name → injected
//! `id_info` returning `(plural, name)`.

use crate::idprop::*;

/// `%g` formatting — Rust lacks %g; emit the shortest round-trip form.
fn fmt_g(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e16 {
        return format!("{:.0}", v);
    }
    let s = format!("{:e}", v);
    /* %g picks %e below 1e-4 or >= 1e6-ish; keep it simple: shortest. */
    let s2 = format!("{}", v);
    if s2.len() <= s.len() { s2 } else { s }
}

/// `idp_str_append_escape` — escape `"`/`\\`/control chars, with
/// optional surrounding quotes.
fn append_escape(out: &mut String, s: &[u8], quote: bool) {
    if quote { out.push('"'); }
    for &c in s {
        match c {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            c if c < 32 => out.push_str(&format!("\\x{:02x}", c)),
            c => out.push(c as char),
        }
    }
    if quote { out.push('"'); }
}

/// `idp_repr_fn_recursive` — `arena` resolves `IDPArray`/`Group`
/// handles (index into the same arena `idp_add_to_group` builds).
pub fn idp_repr_fn_recursive(
    prop: &IDProperty,
    arena: &[IDProperty],
    id_info: &dyn Fn(usize) -> Option<(String, String)>,
    enum_item_name: &dyn Fn(&IDProperty) -> Option<String>,
    out: &mut String,
) {
    match prop.type_ {
        IDP_STRING => {
            let s = idp_string_get(prop);
            let n = (prop.len - 1).max(0) as usize;
            append_escape(out, &s[..n.min(s.len())], true);
        }
        IDP_INT => {
            if let Some(name) = if prop.ui_data != 0 { enum_item_name(prop) } else { None } {
                append_escape(out, name.as_bytes(), true);
            } else if let IDPData::Val(v) = prop.data {
                out.push_str(&format!("{}", v as i32));
            }
        }
        IDP_FLOAT => {
            if let IDPData::Val(v) = prop.data {
                out.push_str(&fmt_g(f32::from_bits(v as u32) as f64));
            }
        }
        IDP_DOUBLE => {
            if let IDPData::Val(v) = prop.data {
                out.push_str(&fmt_g(f64::from_bits(v as u64)));
            }
        }
        IDP_BOOLEAN => {
            let b = matches!(prop.data, IDPData::Val(v) if v != 0);
            out.push_str(if b { "True" } else { "False" });
        }
        IDP_ARRAY => {
            out.push('[');
            if let IDPData::Array(buf) = &prop.data {
                let n = prop.len as usize;
                match prop.subtype {
                    IDP_INT => {
                        for i in 0..n {
                            if i > 0 { out.push_str(", "); }
                            let v = i32::from_ne_bytes(buf[i * 4..i * 4 + 4].try_into().unwrap());
                            out.push_str(&format!("{}", v));
                        }
                    }
                    IDP_FLOAT => {
                        for i in 0..n {
                            if i > 0 { out.push_str(", "); }
                            let v = f32::from_ne_bytes(buf[i * 4..i * 4 + 4].try_into().unwrap());
                            out.push_str(&fmt_g(v as f64));
                        }
                    }
                    IDP_DOUBLE => {
                        for i in 0..n {
                            if i > 0 { out.push_str(", "); }
                            let v = f64::from_ne_bytes(buf[i * 8..i * 8 + 8].try_into().unwrap());
                            out.push_str(&fmt_g(v));
                        }
                    }
                    IDP_BOOLEAN => {
                        for i in 0..n {
                            if i > 0 { out.push_str(", "); }
                            out.push_str(if buf[i] != 0 { "True" } else { "False" });
                        }
                    }
                    _ => {}
                }
            }
            out.push(']');
        }
        IDP_IDPARRAY => {
            out.push('[');
            if let IDPData::IDPArray(items) = &prop.data {
                for (i, &h) in items.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    if let Some(sub) = arena.get(h) {
                        idp_repr_fn_recursive(sub, arena, id_info, enum_item_name, out);
                    }
                }
            }
            out.push(']');
        }
        IDP_GROUP => {
            out.push('{');
            if let IDPData::Group(items) = &prop.data {
                for (i, &h) in items.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    if let Some(sub) = arena.get(h) {
                        append_escape(out, sub.name.as_bytes(), true);
                        out.push_str(": ");
                        idp_repr_fn_recursive(sub, arena, id_info, enum_item_name, out);
                    }
                }
            }
            out.push('}');
        }
        IDP_ID => {
            if let IDPData::Val(id) = prop.data {
                if let Some((plural, name)) = id_info(id as usize) {
                    out.push_str("bpy.data.");
                    out.push_str(&plural);
                    out.push('[');
                    append_escape(out, name.as_bytes(), true);
                    out.push(']');
                }
            }
        }
        _ => {}
    }
}

/// `IDP_reprN`.
pub fn idp_reprn(
    prop: &IDProperty,
    arena: &[IDProperty],
    id_info: &dyn Fn(usize) -> Option<(String, String)>,
    enum_item_name: &dyn Fn(&IDProperty) -> Option<String>,
) -> String {
    let mut out = String::new();
    idp_repr_fn_recursive(prop, arena, id_info, enum_item_name, &mut out);
    out
}

/// `IDP_type_str(type, subtype)`.
pub fn idp_type_str(type_: i8, sub_type: i8) -> &'static str {
    match type_ {
        IDP_STRING => match sub_type {
            IDP_STRING_SUB_UTF8 => "String",
            IDP_STRING_SUB_BYTE => "Bytes",
            _ => "String",
        },
        IDP_INT => "Int",
        IDP_FLOAT => "Float",
        IDP_ARRAY => match sub_type {
            IDP_INT => "Array (Int)",
            IDP_FLOAT => "Array (Float)",
            IDP_DOUBLE => "Array (Double)",
            IDP_BOOLEAN => "Array (Boolean)",
            _ => "Array",
        },
        IDP_GROUP => "Group",
        IDP_ID => "ID",
        IDP_DOUBLE => "Double",
        IDP_IDPARRAY => "Array of Properties",
        IDP_BOOLEAN => "Boolean",
        _ => "Unknown", /* IDP_ANY_STRUCT_TYPE_CASE → struct info PORT_PENDING */
    }
}

/// `IDP_type_str(prop)`.
pub fn idp_type_str_prop(prop: &IDProperty) -> &'static str {
    idp_type_str(prop.type_, prop.subtype)
}
