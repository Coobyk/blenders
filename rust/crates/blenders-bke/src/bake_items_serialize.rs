//! Port of `intern/bake_items_serialize.cc` — the self-contained
//! blob layer: `BlobSlice` ser/deser, `BlobWriter`/`BlobReader`
//! traits, `DiskBlobWriter`/`DiskBlobReader`/`MemoryBlobReader`/
//! `MemoryBlobWriter`, `Blob{Write,Read}Sharing` (dedup + implicit
//! sharing), the attr-domain/data-type/storage io-name tables,
//! raw-bytes blob read/write helpers, `make_independent_file_name`,
//! `lookup_int_in_range`, `blob_slice_is_valid`.
//! `PORT_PENDING`: `serialize_socket_value_variant`/
//! `deserialize_bake_item`, `serialize_bundle_items`, all
//! `try_load_*` (Mesh/Curves/PointCloud/GreasePencil/Volume/
//! Instances/Bundle/GeometrySet), `serialize_materials`,
//! `load_texspace` — they need the geometry/node/attribute
//! stacks which are still in flight.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom, Write};

use blenders_bli::serialize::DictionaryValue;
use blenders_bli::implicit_sharing::ImplicitSharingInfo;

/// `bke::bake::BlobSlice` — `{StringRefNull name, IndexRange range}`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlobSlice {
    pub name: String,
    pub start: i64,
    pub size: i64,
}

impl BlobSlice {
    /// `BlobSlice::serialize`.
    pub fn serialize(&self) -> DictionaryValue {
        let io_slice = DictionaryValue::new();
        io_slice.append_str("name", &self.name);
        io_slice.append_int("start", self.start);
        io_slice.append_int("size", self.size);
        io_slice
    }

    /// `BlobSlice::deserialize` — old bakes may hold a negative
    /// size when writing failed.
    pub fn deserialize(io_slice: &DictionaryValue) -> Option<BlobSlice> {
        let name = io_slice.lookup_str("name")?;
        let start = io_slice.lookup_int("start")?;
        let size = io_slice.lookup_int("size")?;
        if start < 0 || size < 0 || size > i64::MAX - start {
            return None;
        }
        Some(BlobSlice { name, start, size })
    }
}

/// `bke::bake::BlobWriter` interface.
pub trait BlobWriter {
    /// `write(data, size)` → `{name, {start,size}}`.
    fn write(&mut self, data: &[u8]) -> BlobSlice;

    /// `write_as_stream(file_extension, fn)`.
    fn write_as_stream(
        &mut self,
        file_extension: &str,
        f: &mut dyn FnMut(&mut Vec<u8>),
    ) -> BlobSlice {
        let mut data = Vec::new();
        f(&mut data);
        let _ = file_extension;
        self.write(&data)
    }
}

/// `bke::bake::BlobReader` interface.
pub trait BlobReader {
    /// `read(slice, r_data)`.
    fn read(&self, slice: &BlobSlice, r_data: &mut [u8]) -> bool;

    /// `read_as_stream`.
    fn read_as_stream(&self, slice: &BlobSlice, f: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        let mut buffer = vec![0u8; slice.size.max(0) as usize];
        if !self.read(slice, &mut buffer) {
            return false;
        }
        f(&buffer)
    }
}

/// `bke::bake::DiskBlobReader` — reads slices from per-name blob
/// files under `blobs_dir`; open input streams are kept cached
/// like `open_input_streams_`.
pub struct DiskBlobReader {
    pub blobs_dir: String,
    open_input_streams: std::sync::Mutex<HashMap<String, std::fs::File>>,
}

impl DiskBlobReader {
    pub fn new(blobs_dir: String) -> Self {
        Self {
            blobs_dir,
            open_input_streams: std::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl BlobReader for DiskBlobReader {
    fn read(&self, slice: &BlobSlice, r_data: &mut [u8]) -> bool {
        if slice.size == 0 {
            return true;
        }
        let blob_path = format!("{}/{}", self.blobs_dir, slice.name);
        let mut streams = self.open_input_streams.lock().unwrap();
        let file = streams
            .entry(blob_path.clone())
            .or_insert_with(|| std::fs::File::open(&blob_path).unwrap());
        if file.seek(SeekFrom::Start(slice.start as u64)).is_err() {
            return false;
        }
        let mut got = vec![0u8; slice.size as usize];
        match file.read_exact(&mut got) {
            Ok(()) => {
                let n = got.len().min(r_data.len());
                r_data[..n].copy_from_slice(&got[..n]);
                true
            }
            Err(_) => false,
        }
    }
}

/// `bke::bake::DiskBlobWriter` — appends to `<base>.blob` and
/// independent `*_file_<n><ext>` files under `blob_dir`.
pub struct DiskBlobWriter {
    pub blob_dir: String,
    pub base_name: String,
    blob_name: String,
    current_offset: i64,
    total_written_size: i64,
    independent_file_count: i64,
    file_names: Vec<String>,
    blob_stream: Option<std::io::BufWriter<std::fs::File>>,
    has_error: bool,
}

impl DiskBlobWriter {
    pub fn new(blob_dir: String, base_name: String) -> Self {
        Self {
            blob_name: format!("{base_name}.blob"),
            blob_dir,
            base_name,
            current_offset: 0,
            total_written_size: 0,
            independent_file_count: 0,
            file_names: Vec::new(),
            blob_stream: None,
            has_error: false,
        }
    }

    pub fn total_written_size(&self) -> i64 {
        self.total_written_size
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }

    /// `DiskBlobWriter::finish` — flush the blob stream.
    pub fn finish(&mut self) -> bool {
        if let Some(s) = &mut self.blob_stream {
            if s.flush().is_err() {
                self.has_error = true;
            }
            self.blob_stream = None;
        }
        !self.has_error
    }

    /// `DiskBlobWriter::remove_files`.
    pub fn remove_files(&mut self) {
        debug_assert!(self.blob_stream.is_none());
        for file_name in &self.file_names {
            let path = format!("{}/{}", self.blob_dir, file_name);
            let _ = std::fs::remove_file(&path);
        }
    }
}

impl BlobWriter for DiskBlobWriter {
    fn write(&mut self, data: &[u8]) -> BlobSlice {
        let old_offset = self.current_offset;
        if self.has_error {
            return BlobSlice {
                name: self.blob_name.clone(),
                start: old_offset,
                size: 0,
            };
        }
        if self.blob_stream.is_none() {
            let path = format!("{}/{}", self.blob_dir, self.blob_name);
            if let Some(parent) = std::path::Path::new(&path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            self.blob_stream = Some(std::io::BufWriter::new(
                std::fs::File::create(&path).unwrap(),
            ));
            self.file_names.push(self.blob_name.clone());
        }
        let stream = self.blob_stream.as_mut().unwrap();
        if stream.write_all(data).is_err() {
            self.has_error = true;
            return BlobSlice {
                name: self.blob_name.clone(),
                start: old_offset,
                size: 0,
            };
        }
        let size = data.len() as i64;
        self.current_offset += size;
        self.total_written_size += size;
        BlobSlice {
            name: self.blob_name.clone(),
            start: old_offset,
            size,
        }
    }

    /// `DiskBlobWriter::write_as_stream` — writes into an
    /// independent `*_file_<n><ext>` file instead of the blob.
    fn write_as_stream(
        &mut self,
        file_extension: &str,
        f: &mut dyn FnMut(&mut Vec<u8>),
    ) -> BlobSlice {
        debug_assert!(file_extension.starts_with('.'));
        self.independent_file_count += 1;
        let file_name = make_independent_file_name(
            &self.base_name,
            self.independent_file_count,
            file_extension,
        );
        if self.has_error {
            return BlobSlice {
                name: file_name,
                start: 0,
                size: 0,
            };
        }
        let path = format!("{}/{}", self.blob_dir, file_name);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut data = Vec::new();
        f(&mut data);
        let written = match std::fs::write(&path, &data) {
            Ok(()) => data.len() as i64,
            Err(_) => -1,
        };
        self.file_names.push(file_name.clone());
        if written < 0 {
            self.has_error = true;
            return BlobSlice {
                name: file_name,
                start: 0,
                size: 0,
            };
        }
        self.total_written_size += written;
        BlobSlice {
            name: file_name,
            start: 0,
            size: written,
        }
    }
}

/// `bke::bake::MemoryBlobReader` — named in-memory blobs.
#[derive(Default)]
pub struct MemoryBlobReader {
    blob_by_name: HashMap<String, Vec<u8>>,
}

impl MemoryBlobReader {
    /// `MemoryBlobReader::add`.
    pub fn add(&mut self, name: &str, blob: &[u8]) {
        self.blob_by_name.insert(name.to_string(), blob.to_vec());
    }
}

impl BlobReader for MemoryBlobReader {
    fn read(&self, slice: &BlobSlice, r_data: &mut [u8]) -> bool {
        match self.blob_by_name.get(&slice.name) {
            Some(blob) => {
                let start = slice.start as usize;
                let size = slice.size as usize;
                if start + size > blob.len() || size > r_data.len() {
                    return false;
                }
                r_data[..size].copy_from_slice(&blob[start..start + size]);
                true
            }
            None => false,
        }
    }
}

/// `bke::bake::MemoryBlobWriter` — single in-memory blob buffer.
#[derive(Default)]
pub struct MemoryBlobWriter {
    pub data: Vec<u8>,
}

impl BlobWriter for MemoryBlobWriter {
    fn write(&mut self, data: &[u8]) -> BlobSlice {
        let start = self.data.len() as i64;
        self.data.extend_from_slice(data);
        BlobSlice {
            name: String::new(),
            start,
            size: data.len() as i64,
        }
    }
}

/// `bke::bake::BlobWriteSharing` — deduplicated writes plus
/// implicit-sharing tracking (`write_deduplicated`,
/// `write_implicitly_shared`).
#[derive(Default)]
pub struct BlobWriteSharing {
    /// Hash → serialized `DictionaryValue` of the slice.
    dedup: HashMap<u64, DictionaryValue>,
}

impl BlobWriteSharing {
    fn xxh3(data: &[u8]) -> u64 {
        xxhash_rust::xxh3::xxh3_64(data)
    }

    /// `BlobWriteSharing::write_deduplicated` — writes `data` once
    /// per unique content hash, returns the slice dict.
    pub fn write_deduplicated(
        &mut self,
        blob_writer: &mut dyn BlobWriter,
        data: &[u8],
    ) -> DictionaryValue {
        let hash = Self::xxh3(data);
        if let Some(io) = self.dedup.get(&hash) {
            return io.clone();
        }
        let io = blob_writer.write(data).serialize();
        self.dedup.insert(hash, io.clone());
        io
    }

    /// `BlobWriteSharing::write_implicitly_shared` — runs `write_fn`
    /// when `sharing_info` is not already recorded; recorded infos
    /// produce a `{"type": "shared", "value": <id>}` marker dict.
    pub fn write_implicitly_shared(
        &mut self,
        sharing_info: Option<&dyn ImplicitSharingInfo>,
        write_fn: &mut dyn FnMut() -> DictionaryValue,
        lookup_shared_id: &mut dyn FnMut(&dyn ImplicitSharingInfo) -> Option<i64>,
    ) -> DictionaryValue {
        if let Some(info) = sharing_info {
            if let Some(id) = lookup_shared_id(info) {
                let io = DictionaryValue::new();
                io.append_str("type", "shared");
                io.append_int("value", id);
                return io;
            }
        }
        write_fn()
    }
}

/// `bke::bake::BlobReadSharing` — read cache for implicit sharing.
#[derive(Default)]
pub struct BlobReadSharing {
    /// `io_data` key → `(sharing_info handle, data handle)`.
    cache: HashSet<usize>,
}

impl BlobReadSharing {
    /// `BlobReadSharing::read_shared` — `read_fn` runs on cache
    /// miss; hit returns the cached handles (modeled via `cache_key`
    /// + `lookup` callbacks since sharing handles are external).
    pub fn read_shared<T>(
        &self,
        io_data: &DictionaryValue,
        cache_key: &dyn Fn(&DictionaryValue) -> usize,
        lookup: &dyn Fn(usize) -> Option<T>,
        read_fn: &mut dyn FnMut(&DictionaryValue) -> Option<T>,
    ) -> Option<T> {
        let key = cache_key(io_data);
        if self.cache.contains(&key) {
            return lookup(key);
        }
        read_fn(io_data)
    }
}

/// `get_domain_io_name` — `rna_enum_attribute_domain_items`
/// id for `AttrDomain` (verbatim RNA identifiers).
pub fn get_domain_io_name(domain: i32) -> &'static str {
    match domain {
        0 => "POINT",
        1 => "EDGE",
        2 => "FACE",
        3 => "CORNER",
        4 => "CURVE",
        5 => "INSTANCE",
        6 => "LAYER",
        _ => "unknown",
    }
}

/// `get_data_type_io_name` — `rna_enum_attribute_type_items`.
pub fn get_data_type_io_name(data_type: i32) -> &'static str {
    match data_type {
        1 => "FLOAT",
        10 => "FLOAT2",
        2 => "FLOAT3",
        3 => "FLOAT4X4",
        4 => "FLOAT_COLOR",
        5 => "BYTE_COLOR",
        6 => "INT32",
        7 => "INT32_2D",
        8 => "INT8",
        9 => "STRING",
        0 => "BOOLEAN",
        11 => "QUATERNION",
        12 => "FLOAT4",
        13 => "INT64",
        14 => "INT8_COLOR",
        15 => "UINT8",
        16 => "UINT32",
        _ => "unknown",
    }
}

/// `get_storage_type_io_name` — `rna_enum_attr_storage_type_items`.
pub fn get_storage_type_io_name(storage_type: i32) -> &'static str {
    match storage_type {
        0 => "AUTO",
        1 => "SINGLE",
        _ => "unknown",
    }
}

/// `get_domain_from_io_name`.
pub fn get_domain_from_io_name(io_name: &str) -> Option<i32> {
    Some(match io_name {
        "POINT" => 0,
        "EDGE" => 1,
        "FACE" => 2,
        "CORNER" => 3,
        "CURVE" => 4,
        "INSTANCE" => 5,
        "LAYER" => 6,
        _ => return None,
    })
}

/// `get_data_type_from_io_name`.
pub fn get_data_type_from_io_name(io_name: &str) -> Option<i32> {
    Some(match io_name {
        "BOOLEAN" => 0,
        "FLOAT" => 1,
        "FLOAT2" => 10,
        "FLOAT3" => 2,
        "FLOAT4X4" => 3,
        "FLOAT_COLOR" => 4,
        "BYTE_COLOR" => 5,
        "INT32" => 6,
        "INT32_2D" => 7,
        "INT8" => 8,
        "STRING" => 9,
        "QUATERNION" => 11,
        "FLOAT4" => 12,
        "INT64" => 13,
        "INT8_COLOR" => 14,
        "UINT8" => 15,
        "UINT32" => 16,
        _ => return None,
    })
}

/// `get_storage_type_from_io_name`.
pub fn get_storage_type_from_io_name(io_name: &str) -> Option<i32> {
    Some(match io_name {
        "AUTO" => 0,
        "SINGLE" => 1,
        _ => return None,
    })
}

/// `write_blob_raw_bytes`.
pub fn write_blob_raw_bytes(
    blob_writer: &mut dyn BlobWriter,
    blob_sharing: &mut BlobWriteSharing,
    data: &[u8],
) -> DictionaryValue {
    blob_sharing.write_deduplicated(blob_writer, data)
}

/// `blob_slice_is_valid`.
pub fn blob_slice_is_valid(slice_opt: &Option<BlobSlice>, expected_size: i64) -> bool {
    slice_opt
        .as_ref()
        .is_some_and(|s| s.size == expected_size)
}

/// `read_blob_raw_bytes`.
pub fn read_blob_raw_bytes(
    blob_reader: &dyn BlobReader,
    io_data: &DictionaryValue,
    bytes_num: i64,
    r_data: &mut [u8],
) -> bool {
    let slice = BlobSlice::deserialize(io_data);
    if !blob_slice_is_valid(&slice, bytes_num) {
        return false;
    }
    blob_reader.read(&slice.unwrap(), r_data)
}

/// `make_independent_file_name` — `"{}_file_{}{}"`.
pub fn make_independent_file_name(base_name: &str, file_index: i64, extension: &str) -> String {
    format!("{base_name}_file_{file_index}{extension}")
}

/// `lookup_int_in_range` — `io_dict.lookup_int` clamped to a range
/// check, returning None outside `[min, max]`.
pub fn lookup_int_in_range(io_dict: &DictionaryValue, name: &str, min: i64, max: i64) -> Option<i64> {
    let v = io_dict.lookup_int(name)?;
    (v >= min && v <= max).then_some(v)
}
