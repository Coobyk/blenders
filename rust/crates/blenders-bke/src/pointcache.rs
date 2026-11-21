//! Port of `intern/pointcache.cc` — the generic point-cache
//! layer: `BPHYS_DATA_*` tables, `PTCacheFile` `.bphys` header
//! format, `PTCacheMem` frame cache list ops
//! (`mem_index_find`/`mem_pointers_init`/`insert`/`firstlast`/
//! `frame_exists`/`find_frame_around`), `BKE_ptcache_make_
//! particle_key`, the particle Catmull-Rom `ptcache_particle_
//! interpolate`, filename frame parsing, and `PTCacheID`.
//!
//! Per-physics `void *` glue (softbody/particles/cloth/dynamic
//! paint/rigidbody `write`/`read`/`totpoint`/`error` calls and
//! `BKE_ptcache_id_from_*`/`foreach_object_ptcache`) is injected
//! — `PORT_PENDING` until those systems are ported.

use std::io::{Read, Write};

use blenders_dna::particle_types::ParticleKey;
use blenders_math::quaternion::vec_to_quat;
use blenders_math::Vec3;
fn blenox(v: [f32;3]) -> Vec3 { Vec3::new(v[0], v[1], v[2]) }

/// `BPHYS_DATA_INDEX`.
pub const BPHYS_DATA_INDEX: usize = 0;
/// `BPHYS_DATA_LOCATION` (also `SMOKE_LOW`).
pub const BPHYS_DATA_LOCATION: usize = 1;
/// `BPHYS_DATA_VELOCITY` (also `SMOKE_HIGH`).
pub const BPHYS_DATA_VELOCITY: usize = 2;
/// `BPHYS_DATA_ROTATION` (also `DYNAMICPAINT`).
pub const BPHYS_DATA_ROTATION: usize = 3;
/// `BPHYS_DATA_AVELOCITY` (also `XCONST` for cloth).
pub const BPHYS_DATA_AVELOCITY: usize = 4;
/// `BPHYS_DATA_XCONST`.
pub const BPHYS_DATA_XCONST: usize = 4;
/// `BPHYS_DATA_SIZE`.
pub const BPHYS_DATA_SIZE: usize = 5;
/// `BPHYS_DATA_TIMES`.
pub const BPHYS_DATA_TIMES: usize = 6;
/// `BPHYS_DATA_BOIDS`.
pub const BPHYS_DATA_BOIDS: usize = 7;
/// `BPHYS_TOT_DATA`.
pub const BPHYS_TOT_DATA: usize = 8;

/// `ptcache_data_size[]` — bytes per element per data type.
pub const PTCACHE_DATA_SIZE: [usize; BPHYS_TOT_DATA] = [
    4,  /* INDEX: uint */
    12, /* LOCATION: float[3] */
    12, /* VELOCITY: float[3] */
    16, /* ROTATION: float[4] */
    12, /* AVELOCITY/XCONST: float[3] */
    4,  /* SIZE: float */
    12, /* TIMES: float[3] */
    36, /* BOIDS: BoidData */
];

/// `ptcache_extra_datasize[]` — extra-data element sizes
/// (0=nothing, 1=ParticleSpring, 2=float[3]).
pub const PTCACHE_EXTRA_DATASIZE: [usize; 3] = [0, 28, 12];

/// `BKE_ptcache_data_size`.
pub fn ptcache_data_size(data_type: usize) -> usize {
    PTCACHE_DATA_SIZE[data_type]
}

/// `PTCACHE_TYPEFLAG_*` — `.bphys` typeflag bit ranges.
pub const PTCACHE_TYPEFLAG_TYPEMASK: u32 = 0x0000FFFF;
/// `PTCACHE_TYPEFLAG_FLAGMASK`.
pub const PTCACHE_TYPEFLAG_FLAGMASK: u32 = 0xFFFF0000;

/// `PTCACHE_FILE_PTCACHE` file type + `PTCACHE_EXT` extension.
pub const PTCACHE_FILE_PTCACHE: u32 = 0;
/// `PTCACHE_EXT`.
pub const PTCACHE_EXT: &str = ".bphys";

/// `PTCacheFile` — `.bphys` reader/writer state.
pub struct PTCacheFile<'a> {
    /// `fp` → a stream.
    pub fp: &'a mut (dyn Read + 'a),
    /// `type` — the `PTCACHE_TYPE_*` of data in this file.
    pub type_: u32,
    /// `flag` — `PTCACHE_FILE_*` flag bits.
    pub flag: u32,
    /// `totpoint` — count in the current frame.
    pub totpoint: u32,
    /// `data_types` — `BPHYS_DATA_*` bitmask.
    pub data_types: u32,
    /// `data` — scratch row being read/written.
    pub data: PTCacheFileData,
}

/// `PTCacheFileData` — one decoded row (verbatim `pf->data`).
#[derive(Default)]
pub struct PTCacheFileData {
    /// `index`.
    pub index: u32,
    /// `loc`.
    pub loc: [f32; 3],
    /// `vel`.
    pub vel: [f32; 3],
    /// `rot`.
    pub rot: [f32; 4],
    /// `ave`.
    pub ave: [f32; 3],
    /// `size`.
    pub size: f32,
    /// `times`.
    pub times: [f32; 3],
    /// `boids`.
    pub boids: [f32; 9],
}

/// `PTCacheMem` — one cached frame in the `mem_cache` list.
#[derive(Default)]
pub struct PTCacheMem {
    /// `next`, `prev` → list order in `PointCache::mem_cache`.
    pub frame: i32,
    /// `totpoint`.
    pub totpoint: u32,
    /// `data_types`.
    pub data_types: u32,
    /// `flag` — `PTCACHE_MEM_*` bits.
    pub flag: u32,
    /// `data[BPHYS_TOT_DATA]` — per-type flat f32/u32 arrays.
    pub data: [Option<Vec<u8>>; BPHYS_TOT_DATA],
    /// `extradata` list — (`type`, `totdata`, `data`).
    pub extradata: Vec<PTCacheExtraData>,
}

/// `PTCacheExtraData` — spring/BoidData extras (verbatim).
pub struct PTCacheExtraData {
    /// `type` — `PTCACHE_EXTRA_*`.
    pub type_: u32,
    /// `totdata`.
    pub totdata: u32,
    /// `data`.
    pub data: Vec<u8>,
}

/// `ptcache_add_extra_data`.
pub fn ptcache_add_extra_data(
    pm: &mut PTCacheMem,
    type_: u32,
    totdata: u32,
    data: Vec<u8>,
) {
    pm.extradata.push(PTCacheExtraData { type_, totdata, data });
}

/// `BKE_ptcache_mem_index_find` — binary search index array.
pub fn ptcache_mem_index_find(pm: &PTCacheMem, index: u32) -> i32 {
    if pm.totpoint > 0 {
        if let Some(raw) = &pm.data[BPHYS_DATA_INDEX] {
            let data: &[u32] = bytemuck_cast(raw);
            let (mut low, mut high) = (0i64, data.len() as i64 - 1);
            if index < data[0] || index > data[high as usize] {
                return -1;
            }
            /* Continuous indexes fast path. */
            let di = index.wrapping_sub(data[0]) as usize;
            if di < data.len() && data[di] == index {
                return di as i32;
            }
            while low <= high {
                let mid = (low + high) / 2;
                let v = data[mid as usize];
                if v > index {
                    high = mid - 1;
                } else if v < index {
                    low = mid + 1;
                } else {
                    return mid as i32;
                }
            }
            return -1;
        }
        if index < pm.totpoint {
            return index as i32;
        }
    }
    -1
}

fn bytemuck_cast<T: Copy>(raw: &[u8]) -> &[T] {
    /* `pm->data[i]` buffers are `ptcache_data_size[i]` aligned. */
    assert!(raw.len() % std::mem::size_of::<T>() == 0);
    unsafe {
        std::slice::from_raw_parts(raw.as_ptr() as *const T, raw.len() / std::mem::size_of::<T>())
    }
}

/// `BKE_ptcache_mem_pointers_init` — returns data-type flag of
/// `pm` filtered by `data_types`.
pub fn ptcache_mem_pointers_init(pm: &PTCacheMem) -> [bool; BPHYS_TOT_DATA] {
    let mut out = [false; BPHYS_TOT_DATA];
    for i in 0..BPHYS_TOT_DATA {
        out[i] = (pm.data_types & (1 << i)) != 0 && pm.data[i].is_some();
    }
    out
}

/// `BKE_ptcache_mem_insert` — ordered insert into `mem_cache`
/// (sorted by frame).
pub fn ptcache_mem_insert(mem_cache: &mut Vec<PTCacheMem>, pm: PTCacheMem) {
    let pos = mem_cache
        .iter()
        .position(|p| p.frame >= pm.frame)
        .unwrap_or(mem_cache.len());
    mem_cache.insert(pos, pm);
}

/// `BKE_ptcache_firstlast` — `(first, last)` frames.
pub fn ptcache_firstlast(mem_cache: &[PTCacheMem]) -> Option<(i32, i32)> {
    Some((mem_cache.first()?.frame, mem_cache.last()?.frame))
}

/// `BKE_ptcache_frame_exists` — exact frame lookup.
pub fn ptcache_frame_exists(mem_cache: &[PTCacheMem], frame: i32) -> bool {
    mem_cache.iter().any(|pm| pm.frame == frame)
}

/// `ptcache_find_frame_around` — `(pm_before, pm_after)` around
/// `frame` (nearest each side).
pub fn ptcache_find_frame_around(
    mem_cache: &[PTCacheMem],
    frame: i32,
) -> (Option<usize>, Option<usize>) {
    let mut before = None;
    let mut after = None;
    for (i, pm) in mem_cache.iter().enumerate() {
        if pm.frame <= frame {
            before = Some(i);
        }
        if pm.frame >= frame && after.is_none() {
            after = Some(i);
        }
    }
    (before, after)
}

/// `ptcache_file_header_begin_write` / `_read` — `"BPHYSICS"`
/// + u32 typeflag.
pub fn ptcache_file_header_begin_write(w: &mut dyn Write, type_: u32, flag: u32) -> bool {
    if w.write_all(b"BPHYSICS").is_err() {
        return false;
    }
    let typeflag = type_ + flag;
    w.write_all(&typeflag.to_le_bytes()).is_ok()
}
/// `ptcache_file_header_begin_read`.
pub fn ptcache_file_header_begin_read(r: &mut dyn Read) -> Option<(u32, u32)> {
    let mut bphysics = [0u8; 8];
    r.read_exact(&mut bphysics).ok()?;
    if &bphysics != b"BPHYSICS" {
        return None;
    }
    let mut tb = [0u8; 4];
    r.read_exact(&mut tb).ok()?;
    let typeflag = u32::from_le_bytes(tb);
    Some((
        typeflag & PTCACHE_TYPEFLAG_TYPEMASK,
        typeflag & PTCACHE_TYPEFLAG_FLAGMASK,
    ))
}

/// `ptcache_basic_header_write` / `_read` — `totpoint`,
/// `data_types` u32s.
pub fn ptcache_basic_header_write(w: &mut dyn Write, totpoint: u32, data_types: u32) -> bool {
    w.write_all(&totpoint.to_le_bytes()).is_ok()
        && w.write_all(&data_types.to_le_bytes()).is_ok()
}
/// `ptcache_basic_header_read`.
pub fn ptcache_basic_header_read(r: &mut dyn Read) -> Option<(u32, u32)> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b).ok()?;
    Some((
        u32::from_le_bytes(b[0..4].try_into().unwrap()),
        u32::from_le_bytes(b[4..8].try_into().unwrap()),
    ))
}

/// `ptcache_file_extension`.
pub fn ptcache_file_extension(file_type: u32) -> &'static str {
    match file_type {
        PTCACHE_FILE_PTCACHE => PTCACHE_EXT,
        _ => PTCACHE_EXT,
    }
}

/// `ptcache_frame_from_filename` — parse the 6-digit frame
/// before `ext` in `filename` (stack index suffix allowed).
pub fn ptcache_frame_from_filename(filename: &str, ext: &str) -> i32 {
    let frame_len = 6;
    let ext_len = frame_len + ext.len();
    let len = filename.len();
    if len > ext_len {
        let num = &filename[len - ext_len..len - ext.len()];
        return num.trim().parse::<i32>().unwrap_or(-1);
    }
    -1
}

/// `OB_NEGX`/`OB_POSZ` axis codes used by `vec_to_quat`.
pub const OB_NEGX: u8 = 0;
/// `OB_POSZ`.
pub const OB_POSZ: u8 = 5;

/// `BKE_ptcache_make_particle_key` — verbatim field extraction
/// from the `data` pointer array (typed slots here).
pub fn ptcache_make_particle_key(
    key: &mut ParticleKey,
    index: usize,
    loc: Option<&[[f32; 3]]>,
    vel: Option<&[[f32; 3]]>,
    rot: Option<&[[f32; 4]]>,
    ave: Option<&[[f32; 3]]>,
    time: f32,
) {
    if let Some(l) = loc {
        key.co = l[index];
    }
    if let Some(v) = vel {
        key.vel = v[index];
    }
    if let Some(r) = rot {
        key.rot = r[index];
    } else {
        /* No rotation info — make something nice up. */
        let q = vec_to_quat(blenox(key.vel), OB_NEGX, OB_POSZ);
        key.rot = [q.w, q.x, q.y, q.z];
    }
    if let Some(a) = ave {
        key.ave = a[index];
    }
    key.time = time;
}

/// `ptcache_particle_interpolate` — Catmull-Rom blend of
/// `keys[1]`→`keys[2]` at `cfra` between `cfra1`/`cfra2` with
/// `old_data`/`keys[0]`/`keys[3]` neighbors. `data` slots give
/// cached rows (loc/vel/rot/ave).
pub fn ptcache_particle_interpolate(
    keys1: &mut ParticleKey,
    keys2: &ParticleKey,
    keys0: Option<&ParticleKey>,
    keys3: Option<&ParticleKey>,
    cfra: f32,
    cfra1: f32,
    cfra2: f32,
    timestep: f32,
    has_velocity: bool,
) {
    if cfra1 == cfra2 {
        return;
    }

    /* Determine velocity from previous location. */
    let mut keys2m = keys2.clone();
    if !has_velocity {
        if keys1.time > keys2m.time {
            for i in 0..3 {
                keys2m.vel[i] = keys1.co[i] - keys2m.co[i];
                keys2m.vel[i] *= (keys1.time - keys2m.time) * timestep;
            }
        } else {
            for i in 0..3 {
                keys2m.vel[i] = keys2m.co[i] - keys1.co[i];
                keys2m.vel[i] *= (keys2m.time - keys1.time) * timestep;
            }
        }
    }

    /* Catmull-Rom position interpolation across [cfra1, cfra2]. */
    let dfra = cfra2 - cfra1;
    let t = if dfra != 0.0 { (cfra - cfra1) / dfra } else { 0.0 };
    let k0 = keys0.cloned().unwrap_or_else(|| keys1.clone());
    let k3 = keys3.cloned().unwrap_or_else(|| keys2m.clone());
    keys1.co = catmull_rom_v3(k0.co, keys1.co, keys2m.co, k3.co, t);
    keys1.time = cfra;
    if !has_velocity {
        keys1.vel = catmull_rom_v3(k0.vel, keys1.vel, keys2m.vel, k3.vel, t);
    }
}

fn catmull_rom_v3(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    let t2 = t * t;
    let t3 = t2 * t;
    let mut out = [0f32; 3];
    for i in 0..3 {
        out[i] = 0.5
            * ((2.0 * p1[i])
                + (-p0[i] + p2[i]) * t
                + (2.0 * p0[i] - 5.0 * p1[i] + 4.0 * p2[i] - p3[i]) * t2
                + (-p0[i] + 3.0 * p1[i] - 3.0 * p2[i] + p3[i]) * t3);
    }
    out
}

/// `PTCacheID` — `{ob, scene, cache, cal_data, data_types,
/// file_type, stack_index, flag, default_step, info_types,
/// totpoint, totwrite, write, read, interpolate, error}` — the
/// callbacks are system handles here (`usize`) or injected.
pub struct PTCacheID {
    /// `ob`.
    pub ob: usize,
    /// `scene`.
    pub scene: usize,
    /// `cache` — `PointCache` arena handle.
    pub cache: usize,
    /// `cal_data` — the physics system's data pointer.
    pub cal_data: usize,
    /// `data_types` — `BPHYS_DATA_*` bitmask.
    pub data_types: u32,
    /// `info_types` — extra `BPHYS_DATA_*` written.
    pub info_types: u32,
    /// `file_type` — `PTCACHE_FILE_*`.
    pub file_type: u32,
    /// `stack_index` — cache stack slot.
    pub stack_index: i32,
    /// `flag` — `PTCACHE_*` bits.
    pub flag: u32,
    /// `default_step`.
    pub default_step: u32,
}

/// `BKE_ptcache_id_to_filename` — `"{name}_{frame:06}_{stack:02}"`.
pub fn ptcache_id_to_filename(name: &str, frame: i32, stack_index: i32) -> String {
    format!("{}_{:06}_{:02}", name, frame, stack_index)
}
