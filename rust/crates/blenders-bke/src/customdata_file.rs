//! Port of `intern/customdata_file.cc` — `CDataFile` (BCDF) binary
//! CustomData layer storage: header/layer structs and block io.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub const CDF_ENDIAN_LITTLE: u8 = 0;
pub const CDF_ENDIAN_BIG: u8 = 1;
pub const CDF_DATA_FLOAT: i32 = 0;
pub const CDF_VERSION: u8 = 0;
pub const CDF_SUBVERSION: u8 = 0;
pub const CDF_TILE_SIZE: i32 = 64;
pub const CDF_TYPE_IMAGE: i32 = 0;
pub const CDF_TYPE_MESH: i32 = 1;
pub const CDF_LAYER_NAME_MAX: usize = 64;

const HEADER_SIZE: usize = 16;
const IMAGE_HEADER_SIZE: usize = 16;
const MESH_HEADER_SIZE: usize = 4;
/* 4+4+8+4+64 = 84 packed. */
const LAYER_SIZE: usize = 84;

/// `CDataFileHeader`.
#[derive(Clone, Default)]
pub struct CDataFileHeader {
    pub id: [u8; 4],
    pub endian: u8,
    pub version: u8,
    pub subversion: u8,
    pub structbytes: i32,
    pub type_: i32,
    pub totlayer: i32,
}

/// `CDataFileImageHeader`.
#[derive(Clone, Default)]
pub struct CDataFileImageHeader {
    pub structbytes: i32,
    pub width: i32,
    pub height: i32,
    pub tile_size: i32,
}

/// `CDataFileMeshHeader`.
#[derive(Clone, Default)]
pub struct CDataFileMeshHeader {
    pub structbytes: i32,
}

/// `CDataFileLayer`.
#[derive(Clone, Default)]
pub struct CDataFileLayer {
    pub structbytes: i32,
    pub datatype: i32,
    pub datasize: u64,
    pub type_: i32,
    pub name: String,
}

impl CDataFileLayer {
    fn pack(&self) -> [u8; LAYER_SIZE] {
        let mut b = [0u8; LAYER_SIZE];
        b[0..4].copy_from_slice(&self.structbytes.to_le_bytes());
        b[4..8].copy_from_slice(&self.datatype.to_le_bytes());
        b[8..16].copy_from_slice(&self.datasize.to_le_bytes());
        b[16..20].copy_from_slice(&self.type_.to_le_bytes());
        let name = self.name.as_bytes();
        let n = name.len().min(CDF_LAYER_NAME_MAX - 1);
        b[20..20 + n].copy_from_slice(&name[..n]);
        b
    }

    fn unpack(b: &[u8]) -> Self {
        let name_bytes = &b[20..20 + CDF_LAYER_NAME_MAX];
        let end = name_bytes.iter().position(|&c| c == 0).unwrap_or(CDF_LAYER_NAME_MAX);
        Self {
            structbytes: i32::from_le_bytes(b[0..4].try_into().unwrap()),
            datatype: i32::from_le_bytes(b[4..8].try_into().unwrap()),
            datasize: u64::from_le_bytes(b[8..16].try_into().unwrap()),
            type_: i32::from_le_bytes(b[16..20].try_into().unwrap()),
            name: String::from_utf8_lossy(&name_bytes[..end]).to_string(),
        }
    }
}

/// `CDataFile`.
pub struct CDataFile {
    pub type_: i32,
    pub header: CDataFileHeader,
    pub image: CDataFileImageHeader,
    pub mesh: CDataFileMeshHeader,
    pub layer: Vec<CDataFileLayer>,
    pub totlayer: i32,
    readf: Option<File>,
    writef: Option<File>,
    pub dataoffset: u64,
}

fn cdf_endian() -> u8 {
    /* Blender only builds on little endian systems. */
    debug_assert!(cfg!(target_endian = "little"));
    CDF_ENDIAN_LITTLE
}

/// `cdf_create`.
pub fn cdf_create(type_: i32) -> CDataFile {
    CDataFile {
        type_,
        header: CDataFileHeader::default(),
        image: CDataFileImageHeader::default(),
        mesh: CDataFileMeshHeader::default(),
        layer: Vec::new(),
        totlayer: 0,
        readf: None,
        writef: None,
        dataoffset: 0,
    }
}

/// `cdf_free`.
pub fn cdf_free(cdf: CDataFile) {
    drop(cdf);
}

fn cdf_read_header(cdf: &mut CDataFile) -> bool {
    let f = cdf.readf.as_mut().unwrap();
    let mut offset: u64 = 0;
    let mut b16 = [0u8; 16];
    if f.read_exact(&mut b16).is_err() {
        return false;
    }
    let header = &mut cdf.header;
    header.id = b16[0..4].try_into().unwrap();
    header.endian = b16[4];
    header.version = b16[5];
    header.subversion = b16[6];
    header.structbytes = i32::from_le_bytes(b16[8..12].try_into().unwrap());
    header.type_ = i32::from_le_bytes(b16[12..16].try_into().unwrap());

    if header.id != *b"BCDF" {
        return false;
    }
    if header.version > CDF_VERSION {
        return false;
    }
    if header.endian != cdf_endian() {
        return false;
    }
    if ![CDF_TYPE_IMAGE, CDF_TYPE_MESH].contains(&header.type_) {
        return false;
    }

    offset += header.structbytes as u64;
    header.structbytes = HEADER_SIZE as i32;
    if f.seek(SeekFrom::Start(offset)).is_err() {
        return false;
    }

    if header.type_ == CDF_TYPE_IMAGE {
        let mut b = [0u8; IMAGE_HEADER_SIZE];
        if f.read_exact(&mut b).is_err() {
            return false;
        }
        cdf.image = CDataFileImageHeader {
            structbytes: i32::from_le_bytes(b[0..4].try_into().unwrap()),
            width: i32::from_le_bytes(b[4..8].try_into().unwrap()),
            height: i32::from_le_bytes(b[8..12].try_into().unwrap()),
            tile_size: i32::from_le_bytes(b[12..16].try_into().unwrap()),
        };
        offset += cdf.image.structbytes as u64;
        cdf.image.structbytes = IMAGE_HEADER_SIZE as i32;
    } else if header.type_ == CDF_TYPE_MESH {
        let mut b = [0u8; MESH_HEADER_SIZE];
        if f.read_exact(&mut b).is_err() {
            return false;
        }
        cdf.mesh = CDataFileMeshHeader {
            structbytes: i32::from_le_bytes(b[0..4].try_into().unwrap()),
        };
        offset += cdf.mesh.structbytes as u64;
        cdf.mesh.structbytes = MESH_HEADER_SIZE as i32;
    }

    if f.seek(SeekFrom::Start(offset)).is_err() {
        return false;
    }

    cdf.totlayer = cdf.header.totlayer;
    cdf.layer = Vec::with_capacity(cdf.totlayer.max(0) as usize);
    for _ in 0..cdf.header.totlayer.max(0) {
        let mut b = [0u8; LAYER_SIZE];
        if f.read_exact(&mut b).is_err() {
            return false;
        }
        let layer = CDataFileLayer::unpack(&b);
        if layer.datatype != CDF_DATA_FLOAT {
            return false;
        }
        offset += layer.structbytes as u64;
        cdf.layer.push(layer);
        if f.seek(SeekFrom::Start(offset)).is_err() {
            return false;
        }
    }

    cdf.dataoffset = offset;
    true
}

fn cdf_write_header(cdf: &mut CDataFile) -> bool {
    let f = cdf.writef.as_mut().unwrap();
    let mut b16 = [0u8; 16];
    let header = &cdf.header;
    b16[0..4].copy_from_slice(&header.id);
    b16[4] = header.endian;
    b16[5] = header.version;
    b16[6] = header.subversion;
    b16[8..12].copy_from_slice(&header.structbytes.to_le_bytes());
    b16[12..16].copy_from_slice(&header.type_.to_le_bytes());
    if f.write_all(&b16).is_err() {
        return false;
    }

    if header.type_ == CDF_TYPE_IMAGE {
        let mut b = [0u8; IMAGE_HEADER_SIZE];
        b[0..4].copy_from_slice(&cdf.image.structbytes.to_le_bytes());
        b[4..8].copy_from_slice(&cdf.image.width.to_le_bytes());
        b[8..12].copy_from_slice(&cdf.image.height.to_le_bytes());
        b[12..16].copy_from_slice(&cdf.image.tile_size.to_le_bytes());
        if f.write_all(&b).is_err() {
            return false;
        }
    } else if header.type_ == CDF_TYPE_MESH {
        let b = cdf.mesh.structbytes.to_le_bytes();
        if f.write_all(&b).is_err() {
            return false;
        }
    }

    for i in 0..cdf.header.totlayer.max(0) as usize {
        if f.write_all(&cdf.layer[i].pack()).is_err() {
            return false;
        }
    }
    true
}

/// `cdf_read_open`.
pub fn cdf_read_open(cdf: &mut CDataFile, filepath: &Path) -> bool {
    let Ok(f) = File::open(filepath) else {
        return false;
    };
    cdf.readf = Some(f);
    if !cdf_read_header(cdf) {
        cdf_read_close(cdf);
        return false;
    }
    if cdf.header.type_ != cdf.type_ {
        cdf_read_close(cdf);
        return false;
    }
    true
}

/// `cdf_read_layer` — seek to `blay` index's data.
pub fn cdf_read_layer(cdf: &mut CDataFile, blay: usize) -> bool {
    let mut offset = cdf.dataoffset;
    for (a, layer) in cdf.layer.iter().enumerate() {
        if a == blay {
            break;
        }
        offset += layer.datasize;
    }
    match cdf.readf.as_mut() {
        Some(f) => f.seek(SeekFrom::Start(offset)).is_ok(),
        None => false,
    }
}

/// `cdf_read_data`.
pub fn cdf_read_data(cdf: &mut CDataFile, data: &mut [u8]) -> bool {
    match cdf.readf.as_mut() {
        Some(f) => f.read_exact(data).is_ok(),
        None => false,
    }
}

/// `cdf_read_close`.
pub fn cdf_read_close(cdf: &mut CDataFile) {
    cdf.readf = None;
}

/// `cdf_write_open`.
pub fn cdf_write_open(cdf: &mut CDataFile, filepath: &Path) -> bool {
    let Ok(f) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filepath)
    else {
        return false;
    };
    cdf.writef = Some(f);

    /* Fill header. */
    let header = &mut cdf.header;
    header.id = *b"BCDF";
    header.endian = cdf_endian();
    header.version = CDF_VERSION;
    header.subversion = CDF_SUBVERSION;
    header.structbytes = HEADER_SIZE as i32;
    header.type_ = cdf.type_;
    header.totlayer = cdf.totlayer;

    if cdf.type_ == CDF_TYPE_IMAGE {
        cdf.image.structbytes = IMAGE_HEADER_SIZE as i32;
        cdf.image.tile_size = CDF_TILE_SIZE;
    } else if cdf.type_ == CDF_TYPE_MESH {
        cdf.mesh.structbytes = MESH_HEADER_SIZE as i32;
    }

    cdf_write_header(cdf)
}

/// `cdf_write_layer`.
pub fn cdf_write_layer(_cdf: &mut CDataFile, _blay: usize) -> bool {
    true
}

/// `cdf_write_data`.
pub fn cdf_write_data(cdf: &mut CDataFile, data: &[u8]) -> bool {
    match cdf.writef.as_mut() {
        Some(f) => f.write_all(data).is_ok(),
        None => false,
    }
}

/// `cdf_write_close`.
pub fn cdf_write_close(cdf: &mut CDataFile) {
    cdf.writef = None;
}

/// `cdf_remove`.
pub fn cdf_remove(filepath: &Path) {
    let _ = std::fs::remove_file(filepath);
}

/// `cdf_layer_find` — index of layer matching `(type, name)`.
pub fn cdf_layer_find(cdf: &CDataFile, type_: i32, name: &str) -> Option<usize> {
    (0..cdf.totlayer.max(0) as usize)
        .find(|&a| cdf.layer[a].type_ == type_ && cdf.layer[a].name == name)
}

/// `cdf_layer_add`.
pub fn cdf_layer_add(
    cdf: &mut CDataFile,
    type_: i32,
    name: &str,
    datasize: u64,
) -> usize {
    cdf.totlayer += 1;
    cdf.layer.push(CDataFileLayer {
        structbytes: LAYER_SIZE as i32,
        datatype: CDF_DATA_FLOAT,
        datasize,
        type_,
        name: name.to_string(),
    });
    cdf.layer.len() - 1
}
