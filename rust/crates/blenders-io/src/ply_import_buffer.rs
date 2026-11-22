/// Port of `io_ply/importer/ply_import_buffer.cc` —
/// `PlyReadBuffer` — ascii + binary LE/BE
/// element parsing per PLY header spec.
/// `PlyPropertyType` (verbatim).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlyPropertyType {
    Char,
    Uchar,
    Short,
    Ushort,
    Int,
    Uint,
    Float,
    Double,
}
/// `PlyElementSpec` — element block header
/// (`element name count` + props).
#[derive(Clone)]
pub struct PlyElementSpec {
    /// `name`.
    pub name: String,
    /// `count`.
    pub count: usize,
    /// `properties` — (name, type, list?).
    pub properties: Vec<(String, PlyPropertyType, Option<PlyPropertyType>)>,
}
/// `PlyHeader` — parsed PLY header.
#[derive(Clone, Default)]
pub struct PlyHeader {
    /// `format` — 0=ascii 1=binLE 2=binBE.
    pub format: u8,
    /// `elements`.
    pub elements: Vec<PlyElementSpec>,
    /// `comments`.
    pub comments: Vec<String>,
    /// `body_offset` — byte offset of body.
    pub body_offset: usize,
}
/// `parse_header` (verbatim — `ply` magic,
/// `format`/`element`/`property`/`comment`
/// keywords → PlyHeader).
pub fn parse_header(data: &[u8]) -> Option<PlyHeader> {
    let text = String::from_utf8_lossy(data);
    let mut hdr = PlyHeader::default();
    let mut in_header = true;
    let mut off = 0usize;
    for line in text.lines() {
        if !in_header {
            break;
        }
        let t = line.trim();
        if t == "ply" {
            off += line.len() + 1;
            continue;
        }
        if t.starts_with("format") {
            let parts: Vec<&str> = t.split_whitespace().collect();
            if parts.len() >= 3 {
                hdr.format = match parts[1] {
                    "ascii" => 0,
                    "binary_little_endian" => 1,
                    "binary_big_endian" => 2,
                    _ => 0,
                };
            }
        } else if t.starts_with("comment") {
            hdr.comments.push(t["comment".len()..].trim().to_string());
        } else if t.starts_with("element") {
            let parts: Vec<&str> = t.split_whitespace().collect();
            if parts.len() >= 3 {
                hdr.elements.push(PlyElementSpec {
                    name: parts[1].to_string(),
                    count: parts[2].parse().unwrap_or(0),
                    properties: Vec::new(),
                });
            }
        } else if t.starts_with("property") {
            let parts: Vec<&str> = t.split_whitespace().collect();
            if parts.len() >= 3 {
                let ty = |s: &str| -> PlyPropertyType {
                    match s {
                        "char" | "int8" => PlyPropertyType::Char,
                        "uchar" | "uint8" => PlyPropertyType::Uchar,
                        "short" | "int16" => PlyPropertyType::Short,
                        "ushort" | "uint16" => PlyPropertyType::Ushort,
                        "int" | "int32" => PlyPropertyType::Int,
                        "uint" | "uint32" => PlyPropertyType::Uint,
                        "float" | "float32" => PlyPropertyType::Float,
                        "double" | "float64" => PlyPropertyType::Double,
                        _ => PlyPropertyType::Float,
                    }
                };
                if let Some(el) = hdr.elements.last_mut() {
                    if parts[1] == "list" && parts.len() >= 5 {
                        el.properties.push((
                            parts[4].to_string(),
                            ty(parts[3]),
                            Some(ty(parts[2])),
                        ));
                    } else {
                        el.properties
                            .push((parts[2].to_string(), ty(parts[1]), None));
                    }
                }
            }
        } else if t == "end_header" {
            off += line.len() + 1;
            in_header = false;
            hdr.body_offset = off;
            break;
        }
        off += line.len() + 1;
    }
    if hdr.elements.is_empty() {
        return None;
    }
    Some(hdr)
}
/// `PlyReadBuffer` — read one element value.
/// `read_ascii` (verbatim — whitespace-
/// separated scalar per property).
pub fn read_ascii_property(
    ty: PlyPropertyType,
    token: &str,
) -> f64 {
    let _ = ty;
    token.parse().unwrap_or(0.0)
}
/// `read_binary` (verbatim — LE/BE sized
/// reads per type).
pub fn read_binary_property(
    ty: PlyPropertyType,
    data: &[u8],
    off: usize,
    big_endian: bool,
) -> (f64, usize) {
    let be = big_endian;
    let g = |n: usize| -> Option<&[u8]> {
        data.get(off..off + n)
    };
    match ty {
        PlyPropertyType::Char => (
            g(1).map(|b| b[0] as i8 as f64).unwrap_or(0.0),
            1,
        ),
        PlyPropertyType::Uchar => (
            g(1).map(|b| b[0] as f64).unwrap_or(0.0),
            1,
        ),
        PlyPropertyType::Short => (
            g(2).map(|b| {
                if be { i16::from_be_bytes([b[0], b[1]]) } else { i16::from_le_bytes([b[0], b[1]]) } as f64
            }).unwrap_or(0.0),
            2,
        ),
        PlyPropertyType::Ushort => (
            g(2).map(|b| {
                if be { u16::from_be_bytes([b[0], b[1]]) } else { u16::from_le_bytes([b[0], b[1]]) } as f64
            }).unwrap_or(0.0),
            2,
        ),
        PlyPropertyType::Int => (
            g(4).map(|b| {
                if be { i32::from_be_bytes([b[0], b[1], b[2], b[3]]) } else { i32::from_le_bytes([b[0], b[1], b[2], b[3]]) } as f64
            }).unwrap_or(0.0),
            4,
        ),
        PlyPropertyType::Uint => (
            g(4).map(|b| {
                if be { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) } else { u32::from_le_bytes([b[0], b[1], b[2], b[3]]) } as f64
            }).unwrap_or(0.0),
            4,
        ),
        PlyPropertyType::Float => (
            g(4).map(|b| {
                if be { f32::from_be_bytes([b[0], b[1], b[2], b[3]]) } else { f32::from_le_bytes([b[0], b[1], b[2], b[3]]) } as f64
            }).unwrap_or(0.0),
            4,
        ),
        PlyPropertyType::Double => (
            g(8).map(|b| {
                if be {
                    f64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
                } else {
                    f64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
                }
            }).unwrap_or(0.0),
            8,
        ),
    }
}
