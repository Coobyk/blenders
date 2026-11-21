//! Port of `intern/idprop_serialize.cc` — IDProperty ⇄
//! `serialize::DictionaryValue` (JSON-able) conversion.
//!
//! Arena-based: group/IDPArray children are `Vec<IDProperty>`
//! handles, matching `idprop.rs`.

use blenders_bli::serialize::{ArrayValue, DictionaryValue, Value};

use crate::idprop::{
    idp_new_string, IDPData, IDProperty, IDP_ARRAY,  IDP_BOOLEAN, IDP_DOUBLE,
    IDP_FLOAT, IDP_GROUP, IDP_INT, IDP_STRING,
};

/// `IDP_KEY_NAME`/`TYPE`/`SUBTYPE`/`VALUE`.
const IDP_KEY_NAME: &str = "name";
const IDP_KEY_TYPE: &str = "type";
const IDP_KEY_SUBTYPE: &str = "subtype";
const IDP_KEY_VALUE: &str = "value";

const TN_STRING: &str = "IDP_STRING";
const TN_BOOL: &str = "IDP_BOOL";
const TN_INT: &str = "IDP_INT";
const TN_FLOAT: &str = "IDP_FLOAT";
const TN_DOUBLE: &str = "IDP_DOUBLE";
const TN_ARRAY: &str = "IDP_ARRAY";
const TN_GROUP: &str = "IDP_GROUP";
const TN_UNKNOWN: &str = "IDP_UNKNOWN";

/// `serializer_for(type).type_name()`.
fn typename_for(property_type: i8) -> &'static str {
    match property_type {
        IDP_STRING => TN_STRING,
        IDP_BOOLEAN => TN_BOOL,
        IDP_INT => TN_INT,
        IDP_FLOAT => TN_FLOAT,
        IDP_DOUBLE => TN_DOUBLE,
        IDP_ARRAY => TN_ARRAY,
        IDP_GROUP => TN_GROUP,
        _ => TN_UNKNOWN,
    }
}

/// `serializer_for(StringRef).property_type()`.
fn type_for_typename(name: &str) -> Option<i8> {
    match name {
        TN_STRING => Some(IDP_STRING),
        TN_BOOL => Some(IDP_BOOLEAN),
        TN_INT => Some(IDP_INT),
        TN_FLOAT => Some(IDP_FLOAT),
        TN_DOUBLE => Some(IDP_DOUBLE),
        TN_ARRAY => Some(IDP_ARRAY),
        TN_GROUP => Some(IDP_GROUP),
        _ => None,
    }
}

fn create_dictionary(id_property: &IDProperty) -> DictionaryValue {
    let result = DictionaryValue::new();
    result.append_str(IDP_KEY_NAME, id_property.name.clone());
    result.append_str(IDP_KEY_TYPE, typename_for(id_property.type_));
    result
}

fn array_get(array: &ArrayValue, i: usize) -> Option<Value> {
    array.0.borrow().get(i).cloned()
}

fn i64_to_bytes(v: &[i64]) -> Vec<u8> {
    v.iter().flat_map(|x| (*x as i32).to_le_bytes()).collect()
}
fn f64_to_f32_bytes(v: &[f64]) -> Vec<u8> {
    v.iter().flat_map(|x| (*x as f32).to_le_bytes()).collect()
}
fn f64_to_bytes(v: &[f64]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// `idprop_to_dictionary` — dispatch on `id_property.type_`.
/// `arena` resolves group/array child handles.
/// Returns `None` for unserializable types (C: `!supports_serializing`).
pub fn idprop_to_dictionary(id_property: &IDProperty, arena: &[IDProperty]) -> Option<DictionaryValue> {
    let result = create_dictionary(id_property);
    match id_property.type_ {
        IDP_STRING => {
            let s = crate::idprop::idp_string_get(id_property);
            let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
            result.append_str(IDP_KEY_VALUE, String::from_utf8_lossy(&s[..end]).into_owned());
        }
        IDP_BOOLEAN => {
            result.append_bool(IDP_KEY_VALUE, matches!(id_property.data, IDPData::Val(v) if v != 0));
        }
        IDP_INT => {
            let v = match id_property.data {
                IDPData::Val(v) => v,
                _ => 0,
            };
            result.append_int(IDP_KEY_VALUE, v);
        }
        IDP_FLOAT => {
            let v = match id_property.data {
                IDPData::Val(v) => v,
                _ => 0,
            };
            result.append_double(IDP_KEY_VALUE, f32::from_bits(v as u32) as f64);
        }
        IDP_DOUBLE => {
            let v = match id_property.data {
                IDPData::Val(v) => v,
                _ => 0,
            };
            result.append_double(IDP_KEY_VALUE, f64::from_bits(v as u64));
        }
        IDP_ARRAY => {
            result.append_str(IDP_KEY_SUBTYPE, typename_for(id_property.subtype));
            let array = result.append_array(IDP_KEY_VALUE);
            match &id_property.data {
                IDPData::Array(bytes) => match id_property.subtype {
                    IDP_INT => {
                        for c in bytes.chunks_exact(4) {
                            array.append(Value::Int(i32::from_le_bytes(
                                c.try_into().unwrap(),
                            ) as i64));
                        }
                    }
                    IDP_FLOAT => {
                        for c in bytes.chunks_exact(4) {
                            array.append(Value::Double(f32::from_le_bytes(
                                c.try_into().unwrap(),
                            ) as f64));
                        }
                    }
                    IDP_DOUBLE => {
                        for c in bytes.chunks_exact(8) {
                            array.append(Value::Double(f64::from_le_bytes(
                                c.try_into().unwrap(),
                            )));
                        }
                    }
                    _ => {}
                },
                IDPData::IDPArray(children) => {
                    /* C stores IDP_GROUP subtype arrays of linked
                     * IDProperty; serialize each child dict. */
                    for &h in children {
                        let child = arena.get(h)?;
                        if let Some(d) = idprop_to_dictionary(child, arena) {
                            array.append(Value::Dict(d));
                        }
                    }
                }
                _ => {}
            }
        }
        IDP_GROUP => {
            let array = result.append_array(IDP_KEY_VALUE);
            if let IDPData::Group(children) = &id_property.data {
                for &h in children {
                    if let Some(child) = arena.get(h) {
                        if let Some(d) = idprop_to_dictionary(child, arena) {
                            array.append(Value::Dict(d));
                        }
                    }
                }
            }
        }
        _ => return None,
    }
    Some(result)
}

fn get_name(dict: &DictionaryValue) -> Option<String> {
    dict.lookup_str(IDP_KEY_NAME)
}
fn get_type(dict: &DictionaryValue) -> Option<i8> {
    type_for_typename(&dict.lookup_str(IDP_KEY_TYPE)?)
}
fn get_subtype(dict: &DictionaryValue) -> Option<i8> {
    type_for_typename(&dict.lookup_str(IDP_KEY_SUBTYPE)?)
}
fn get_array_value(dict: &DictionaryValue) -> Option<ArrayValue> {
    dict.0
        .borrow()
        .iter()
        .find(|(k, _)| k == IDP_KEY_VALUE)
        .and_then(|(_, v)| match v {
            Value::Array(a) => Some(a.clone()),
            _ => None,
        })
}

fn array_ints(a: &ArrayValue) -> Option<Vec<i64>> {
    a.0.borrow()
        .iter()
        .map(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })
        .collect()
}
fn array_doubles(a: &ArrayValue) -> Option<Vec<f64>> {
    a.0.borrow()
        .iter()
        .map(|v| match v {
            Value::Double(d) => Some(*d),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        })
        .collect()
}

fn new_prop(name: &str, type_: i8, subtype: i8, data: IDPData, len: i32) -> IDProperty {
    IDProperty {
        type_,
        subtype,
        name: name.to_string(),
        data,
        len,
        totallen: len,
        ..Default::default()
    }
}

/// `idprop_from_value(DictionaryValue)` — child handles are
/// appended to `arena`.
pub fn idprop_from_value(dict: &DictionaryValue, arena: &mut Vec<IDProperty>) -> Option<usize> {
    let type_ = get_type(dict)?;
    let name = get_name(dict)?;
    let prop = match type_ {
        IDP_STRING => {
            let v = dict.lookup_str(IDP_KEY_VALUE)?;
            idp_new_string(&v, &name, 0)
        }
        IDP_BOOLEAN => {
            let v = dict.lookup_bool(IDP_KEY_VALUE)?;
            new_prop(&name, IDP_BOOLEAN, 0, IDPData::Val(v as i64), 1)
        }
        IDP_INT => {
            let v = dict.lookup_int(IDP_KEY_VALUE)?;
            new_prop(&name, IDP_INT, 0, IDPData::Val(v), 1)
        }
        IDP_FLOAT => {
            let v = dict.lookup_double(IDP_KEY_VALUE)?;
            new_prop(&name, IDP_FLOAT, 0, IDPData::Val((v as f32).to_bits() as i64), 1)
        }
        IDP_DOUBLE => {
            let v = dict.lookup_double(IDP_KEY_VALUE)?;
            new_prop(&name, IDP_DOUBLE, 0, IDPData::Val(v.to_bits() as i64), 1)
        }
        IDP_ARRAY => {
            let subtype = get_subtype(dict)?;
            let a = get_array_value(dict)?;
            match subtype {
                IDP_INT => {
                    let v = array_ints(&a)?;
                    new_prop(&name, IDP_ARRAY, IDP_INT, IDPData::Array(i64_to_bytes(&v)), v.len() as i32)
                }
                IDP_FLOAT => {
                    let v = array_doubles(&a)?;
                    new_prop(&name, IDP_ARRAY, IDP_FLOAT, IDPData::Array(f64_to_f32_bytes(&v)), v.len() as i32)
                }
                IDP_DOUBLE => {
                    let v = array_doubles(&a)?;
                    new_prop(&name, IDP_ARRAY, IDP_DOUBLE, IDPData::Array(f64_to_bytes(&v)), v.len() as i32)
                }
                _ => return None,
            }
        }
        IDP_GROUP => {
            let a = get_array_value(dict)?;
            let elements: Vec<Value> = a.0.borrow().clone();
            let mut children = Vec::new();
            for el in elements {
                if let Value::Dict(sub) = el {
                    if let Some(h) = idprop_from_value(&sub, arena) {
                        children.push(h);
                    }
                }
            }
            let n = children.len() as i32;
            new_prop(&name, IDP_GROUP, 0, IDPData::Group(children), n)
        }
        _ => return None,
    };
    arena.push(prop);
    Some(arena.len() - 1)
}

/// `idprop_from_value(ArrayValue)` — returns the linked list of
/// created properties (as arena handles in order).
pub fn idprops_from_array(value: &ArrayValue, arena: &mut Vec<IDProperty>) -> Vec<usize> {
    let elements: Vec<Value> = value.0.borrow().clone();
    let mut out = Vec::new();
    for el in elements {
        if let Value::Dict(dict) = el {
            if let Some(h) = idprop_from_value(&dict, arena) {
                out.push(h);
            }
        }
    }
    out
}

/// `convert_from_serialize_value` — only accepts ArrayValue.
pub fn convert_from_serialize_value(value: &Value, arena: &mut Vec<IDProperty>) -> Vec<usize> {
    match value {
        Value::Array(a) => idprops_from_array(a, arena),
        _ => Vec::new(),
    }
}
