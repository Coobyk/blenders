//! Port of `intern/object_dupli.cc` — `DupliObject`,
//! `DupliContext`, `copy_dupli_context`, `make_dupli`,
//! `make_recursive_duplis`, `is_child`, `find_family_object`,
//! `get_duplivert_transform`, RGBA-attribute lookups.
//!
//! `PORT_PENDING`: generator bodies (`make_duplis_*` for
//! collection/verts/faces/font/particles/geometry-set) are
//! injected as `make_duplis` callbacks — they need Object/
//! Mesh/EditMesh/GeometrySet traversal; `init_context`'s
//! `OBEDIT_FROM_OBACT`, `object_duplilist(_preview)` entry
//! points need Depsgraph/ViewerPath.

use blenders_math::matrix_c::loc_quat_size_to_mat4;
use blenders_math::quaternion::vec_to_quat;
use blenders_math::Quaternion;
use blenders_math::{Mat4, Vec3};

fn m4f(a: [[f32; 4]; 4]) -> Mat4 {
    Mat4::from_cols(
        blenders_math::Vec4::new(a[0][0], a[0][1], a[0][2], a[0][3]),
        blenders_math::Vec4::new(a[1][0], a[1][1], a[1][2], a[1][3]),
        blenders_math::Vec4::new(a[2][0], a[2][1], a[2][2], a[2][3]),
        blenders_math::Vec4::new(a[3][0], a[3][1], a[3][2], a[3][3]),
    )
}
fn a4f(m: Mat4) -> [[f32; 4]; 4] {
    let mut o = [[0f32; 4]; 4];
    for c in 0..4 {
        o[c] = [m.cols[c].x, m.cols[c].y, m.cols[c].z, m.cols[c].w];
    }
    o
}

/// `MAX_DUPLI_RECUR`.
pub const MAX_DUPLI_RECUR: usize = 8;
/// `GEOMETRY_SET_DUPLI_GENERATOR_TYPE`.
pub const GEOMETRY_SET_DUPLI_GENERATOR_TYPE: i16 = 1;

/// `BLI_hash_string` (djb2, verbatim).
pub fn hash_string(s: &str) -> u32 {
    let mut h = 5381u32;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    h
}
/// `BLI_hash_int_2d`.
pub fn hash_int_2d(kx: u32, ky: u32) -> u32 {
    kx.wrapping_mul(1664525).wrapping_add(ky)
}
/// `BLI_hash_int`.
pub fn hash_int(kx: u32) -> u32 {
    kx.wrapping_mul(1664525)
}

/// `DupliObject` (`BKE_duplilist.hh`, verbatim).
#[derive(Clone)]
pub struct DupliObject {
    /// `ob`.
    pub ob: usize,
    /// `ob_data` (instanced ID).
    pub ob_data: usize,
    /// `mat`.
    pub mat: [[f32; 4]; 4],
    /// `orco`.
    pub orco: [f32; 3],
    /// `uv`.
    pub uv: [f32; 2],
    /// `type` (`Object::transflag`).
    pub type_: i16,
    /// `no_draw`.
    pub no_draw: bool,
    /// `level`.
    pub level: i8,
    /// `preview_base_geometry` (GeometrySet handle).
    pub preview_base_geometry: usize,
    /// `preview_instance_index`.
    pub preview_instance_index: i32,
    /// `persistent_id`.
    pub persistent_id: [i32; MAX_DUPLI_RECUR],
    /// `random_id`.
    pub random_id: u32,
    /// `particle_system` handle.
    pub particle_system: usize,
    /// `instance_idx`.
    pub instance_idx: [i64; 4],
    /// `instance_data` (GeometrySet handles).
    pub instance_data: [usize; 4],
}

impl Default for DupliObject {
    fn default() -> Self {
        Self {
            ob: usize::MAX,
            ob_data: usize::MAX,
            mat: [[0.0; 4]; 4],
            orco: [0.0; 3],
            uv: [0.0; 2],
            type_: 0,
            no_draw: false,
            level: 0,
            preview_base_geometry: usize::MAX,
            preview_instance_index: -1,
            persistent_id: [i32::MAX; MAX_DUPLI_RECUR],
            random_id: 0,
            particle_system: usize::MAX,
            instance_idx: [0; 4],
            instance_data: [usize::MAX; 4],
        }
    }
}

/// `DupliContext` (verbatim; pointers → handles; `gen` is an
/// injected callback).
pub struct DupliContext {
    /// `depsgraph` handle.
    pub depsgraph: usize,
    /// `collection`.
    pub collection: usize,
    /// `obedit`.
    pub obedit: usize,
    /// `root_object`.
    pub root_object: usize,
    /// `object`.
    pub object: usize,
    /// `space_mat`.
    pub space_mat: [[f32; 4]; 4],
    /// `preview_instance_index`.
    pub preview_instance_index: i32,
    /// `preview_base_geometry`.
    pub preview_base_geometry: usize,
    /// `instance_stack` — Object handles (self-instance guard).
    pub instance_stack: Vec<usize>,
    /// `dupli_gen_type_stack`.
    pub dupli_gen_type_stack: Vec<i16>,
    /// `include_objects`.
    pub include_objects: Vec<usize>,
    /// `persistent_id`.
    pub persistent_id: [i32; MAX_DUPLI_RECUR],
    /// `instance_idx`.
    pub instance_idx: [i64; MAX_DUPLI_RECUR],
    /// `instance_data`.
    pub instance_data: [usize; MAX_DUPLI_RECUR],
    /// `level`.
    pub level: i32,
    /// `gen` — generator type + injected `make_duplis` fn.
    pub gen: Option<DupliGeneratorFn>,
}

/// `DupliGenerator` — `type` + `make_duplis` callback.
#[derive(Clone, Copy)]
pub struct DupliGeneratorFn {
    /// `type`.
    pub type_: i16,
    /// `make_duplis`.
    pub make_duplis: fn(&mut DupliContext),
}

impl Clone for DupliContext {
    fn clone(&self) -> Self {
        Self {
            depsgraph: self.depsgraph,
            collection: self.collection,
            obedit: self.obedit,
            root_object: self.root_object,
            object: self.object,
            space_mat: self.space_mat,
            preview_instance_index: self.preview_instance_index,
            preview_base_geometry: self.preview_base_geometry,
            instance_stack: self.instance_stack.clone(),
            dupli_gen_type_stack: self.dupli_gen_type_stack.clone(),
            include_objects: self.include_objects.clone(),
            persistent_id: self.persistent_id,
            instance_idx: self.instance_idx,
            instance_data: self.instance_data,
            level: self.level,
            gen: self.gen,
        }
    }
}

/// `init_context` (verbatim; `OBEDIT_FROM_OBACT` — edit-mode
/// object via `ob_of` lookup injected).
pub fn init_context(
    depsgraph: usize,
    ob: usize,
    space_mat: Option<[[f32; 4]; 4]>,
    include_objects: Vec<usize>,
) -> DupliContext {
    let mut ctx = DupliContext {
        depsgraph,
        collection: usize::MAX,
        obedit: usize::MAX, /* OBEDIT_FROM_OBACT lookup PORT_PENDING */
        root_object: ob,
        object: ob,
        space_mat: space_mat.unwrap_or_else(|| a4f(Mat4::IDENTITY)),
        preview_instance_index: -1,
        preview_base_geometry: usize::MAX,
        instance_stack: Vec::new(),
        dupli_gen_type_stack: Vec::new(),
        include_objects,
        persistent_id: [0; MAX_DUPLI_RECUR],
        instance_idx: [0; MAX_DUPLI_RECUR],
        instance_data: [usize::MAX; MAX_DUPLI_RECUR],
        level: 0,
        gen: None,
    };
    /* `r_ctx->gen = get_dupli_generator(r_ctx)` — caller sets
     * `gen` via `resolve_generator`. */
    if let Some(g) = ctx.gen {
        if g.type_ != GEOMETRY_SET_DUPLI_GENERATOR_TYPE {
            ctx.dupli_gen_type_stack.push(g.type_);
        }
    }
    ctx
}

/// `copy_dupli_context` (verbatim; `ob_name`/`geometry_name`
/// injected for the recursion-limit warning).
pub fn copy_dupli_context(
    ctx: &DupliContext,
    ob: usize,
    mat: Option<[[f32; 4]; 4]>,
    index: i32,
    geometry: usize,
    instance_index: i64,
    collection_of: &dyn Fn(usize) -> usize,
) -> Option<DupliContext> {
    let mut r_ctx = ctx.clone();
    if let Some(g) = ctx.gen {
        if g.type_ == OB_DUPLICOLLECTION {
            r_ctx.collection = collection_of(ctx.object);
        }
    }
    r_ctx.object = ob;
    if let Some(m) = mat {
        r_ctx.space_mat =
            a4f(m4f(ctx.space_mat) * m4f(m));
    }
    r_ctx.persistent_id[r_ctx.level as usize] = index;
    r_ctx.instance_idx[r_ctx.level as usize] = instance_index;
    r_ctx.instance_data[r_ctx.level as usize] = geometry;
    r_ctx.level += 1;
    if r_ctx.level as usize == MAX_DUPLI_RECUR - 1 {
        /* Warning print (verbatim). */
        eprintln!(
            "Warning: Maximum instance recursion level reached in \"{ob}\" object."
        );
        return None;
    }
    Some(r_ctx)
}

/// `OB_DUPLICOLLECTION` / `OB_DUPLI*` bits (`DNA_object_enums.h`
/// verbatim).
pub const OB_DUPLI: i16 = 0x1;
/// `OB_DUPLIPARENT`.
pub const OB_DUPLIPARENT: i16 = 0x2;
/// `OB_DUPLICOLLECTION`.
pub const OB_DUPLICOLLECTION: i16 = 0x8;
/// `OB_DUPLIFACES`.
pub const OB_DUPLIFACES: i16 = 0x10;
/// `OB_DUPLIVERTS`.
pub const OB_DUPLIVERTS: i16 = 0x20;
/// `OB_DUPLIFACES_SCALE`.
pub const OB_DUPLIFACES_SCALE: i16 = 0x40;
/// `OB_DUPLIROT`.
pub const OB_DUPLIROT: i16 = 0x80;
/// `OB_DONE`.
pub const OB_DONE: i16 = 0x100;

/// `make_dupli` — append a `DupliObject` (verbatim; `ob_name`
/// of `ob`/`root_object` + `id_type` injected).
pub fn make_dupli(
    ctx: &DupliContext,
    ob: usize,
    object_data: usize,
    object_data_is_meta_ball: bool,
    mat: [[f32; 4]; 4],
    index: i32,
    geometry: usize,
    instance_index: i64,
    ob_name: &str,
    root_name: &str,
) -> DupliObject {
    let mut dob = DupliObject {
        ob,
        ob_data: object_data,
        mat: a4f(m4f(ctx.space_mat) * m4f(mat)),
        type_: if ctx.gen.is_none() {
            0
        } else {
            ctx.dupli_gen_type_stack.last().copied().unwrap_or(0)
        },
        preview_base_geometry: ctx.preview_base_geometry,
        preview_instance_index: ctx.preview_instance_index,
        level: ctx.level as i8,
        ..Default::default()
    };
    dob.persistent_id[0] = index;
    let mut i = 1;
    while i < (ctx.level + 1) as usize {
        dob.persistent_id[i] = ctx.persistent_id[ctx.level as usize - i];
        i += 1;
    }
    while i < MAX_DUPLI_RECUR {
        dob.persistent_id[i] = i32::MAX;
        i += 1;
    }
    let max_instance = dob.instance_data.len();
    let mut next_instance = 0;
    if geometry != usize::MAX {
        dob.instance_idx[next_instance] = instance_index;
        dob.instance_data[next_instance] = geometry;
        next_instance += 1;
    }
    let mut li = ctx.level as i32 - 1;
    while li >= 0 && next_instance < max_instance {
        if ctx.instance_data[li as usize] != usize::MAX {
            dob.instance_idx[next_instance] = ctx.instance_idx[li as usize];
            dob.instance_data[next_instance] = ctx.instance_data[li as usize];
            next_instance += 1;
        }
        li -= 1;
    }
    if object_data_is_meta_ball {
        dob.no_draw = true;
    }
    dob.random_id = hash_string(ob_name);
    if dob.persistent_id[0] != i32::MAX {
        for i in 0..(ctx.level + 1) as usize {
            dob.random_id = hash_int_2d(dob.random_id, dob.persistent_id[i] as u32);
        }
    } else {
        dob.random_id = hash_int_2d(dob.random_id, 0);
    }
    if ctx.root_object != ob {
        dob.random_id ^= hash_int(hash_string(root_name));
    }
    dob
}

/// `make_recursive_duplis` (verbatim).
pub fn make_recursive_duplis(
    ctx: &mut DupliContext,
    ob: usize,
    space_mat: [[f32; 4]; 4],
    index: i32,
    geometry: usize,
    instance_index: i64,
    collection_of: &dyn Fn(usize) -> usize,
) {
    if ctx.instance_stack.contains(&ob) {
        eprintln!("Warning: '{ob}' object is trying to instance itself.");
        return;
    }
    if (ctx.level as usize) < MAX_DUPLI_RECUR {
        let Some(mut rctx) = copy_dupli_context(
            ctx, ob, Some(space_mat), index, geometry, instance_index, collection_of,
        ) else {
            return;
        };
        if let Some(g) = rctx.gen {
            ctx.instance_stack.push(ob);
            /* `rctx.gen->make_duplis(&rctx)` — the injected
             * callback appends via `ctx.duplilist`; the vec
             * lives outside the ctx here (caller merges). */
            (g.make_duplis)(&mut rctx);
            ctx.instance_stack.pop();
            if g.type_ != GEOMETRY_SET_DUPLI_GENERATOR_TYPE {
                ctx.dupli_gen_type_stack.pop();
            }
        }
    }
}

/// `is_child` (verbatim).
pub fn is_child(ob_parent: usize, parent: usize, parent_of: &dyn Fn(usize) -> usize) -> bool {
    let mut p = ob_parent;
    while p != usize::MAX {
        if p == parent {
            return true;
        }
        p = parent_of(p);
    }
    false
}

/// `get_duplivert_transform` (verbatim).
pub fn get_duplivert_transform(
    co: Vec3,
    no: Vec3,
    use_rotation: bool,
    axis: u8,
    upflag: u8,
) -> [[f32; 4]; 4] {
    let quat = if use_rotation {
        vec_to_quat(-no, axis, upflag)
    } else {
        Quaternion::IDENTITY
    };
    a4f(loc_quat_size_to_mat4(co, quat, Vec3::splat(1.0)))
}

/// `find_family_object` — `family` + unicode `ch` → object
/// named `"<family><ch_utf8>"` (verbatim).
pub fn find_family_object(
    objects: &dyn Fn(&mut dyn FnMut(usize, &str) -> bool),
    family: &str,
    ch: u32,
) -> Option<usize> {
    let mut found = None;
    let mut ch_utf8 = [0u8; 8];
    let len = char::from_u32(ch)
        .map(|c| c.encode_utf8(&mut ch_utf8).len())
        .unwrap_or(0);
    let target = format!("{}{}", family, &String::from_utf8_lossy(&ch_utf8[..len]));
    objects(&mut |ob, name| {
        if name == target {
            found = Some(ob);
            false
        } else {
            true
        }
    });
    found
}

/// `find_geonode_attribute_rgba` — read `name` attribute from
/// the dupli's instance stack (injected `attr_get`).
pub fn find_geonode_attribute_rgba(
    dob: &DupliObject,
    name: &str,
    attr_get: &dyn Fn(usize, i64, &str) -> Option<[f32; 4]>,
) -> Option<[f32; 4]> {
    for i in 0..dob.instance_data.len() {
        if dob.instance_data[i] != usize::MAX {
            if let Some(v) = attr_get(dob.instance_data[i], dob.instance_idx[i], name) {
                return Some(v);
            }
        }
    }
    None
}

/// `find_property_rgba` — RNA custom property lookup
/// (injected `prop_get`); verbatim fallback to `[0;4]`.
pub fn find_property_rgba(
    id: usize,
    name: &str,
    prop_get: &dyn Fn(usize, &str) -> Option<[f32; 4]>,
) -> [f32; 4] {
    prop_get(id, name).unwrap_or([0.0; 4])
}

/// `BKE_object_dupli_find_rgba_attribute` — geonode → dupli
/// parent chain → object data, verbatim priority.
pub fn object_dupli_find_rgba_attribute(
    ob: usize,
    dob: Option<&DupliObject>,
    dupli_parent: usize,
    name: &str,
    attr_get: &dyn Fn(usize, i64, &str) -> Option<[f32; 4]>,
    prop_get: &dyn Fn(usize, &str) -> Option<[f32; 4]>,
    data_of: &dyn Fn(usize) -> usize,
) -> [f32; 4] {
    if let Some(dob) = dob {
        if let Some(v) = find_geonode_attribute_rgba(dob, name, attr_get) {
            return v;
        }
    }
    let mut o = dupli_parent;
    while o != usize::MAX {
        let v = find_property_rgba(o, name, prop_get);
        if v != [0.0; 4] {
            return v;
        }
        /* `find_property_rgba` on the object's data. */
        let d = data_of(o);
        if d != usize::MAX {
            let v = find_property_rgba(d, name, prop_get);
            if v != [0.0; 4] {
                return v;
            }
        }
        o = usize::MAX;
    }
    let _ = ob;
    [0.0; 4]
}
