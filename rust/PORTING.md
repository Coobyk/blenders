# Porting conventions — Blender C/C++ → Rust + GPUI

Layout: upstream source stays in-tree under `source/`; ports live in `rust/crates/`:
`blenders-math` (math_* files), `blenders-bli` (blenlib), `blenders-core` (DNA/RNA/kernel),
`blenders` (GPUI app).

Rules:
- One Rust module per upstream .cc where practical; `foo_c.cc` → `foo_c.rs`, `BLI_foo.cc` → `foo.rs`.
- C pointer graphs → index arenas (`Vec<T>` + `i32`/`usize` indices, `-1`/`None` for null).
- Callbacks → `&mut dyn FnMut(...)` params or `Rc<RefCell<dyn FnMut>>` for shared/lifetimed ones.
- `BLI_task_parallel_range` / TBB / SSE2 / AVX2 → serial scalar code (PORT_NOTE).
- Upstream `#ifdef`s: grep the file for `#define USE_*`/`#if` before porting — port only branches
  that are actually enabled upstream; mark the rest `// PORT_NOTE: ...` or `// PORT_PENDING: ...`.
- Windows-only files (`winstuff*.cc`, `system_win32.cc`) → cfg-gated stubs returning upstream's
  non-Windows behavior.
- Static tables → extracted verbatim into `*_tables.rs` (scripted, e.g. smaa_textures, noise_tables).
- Naming: keep upstream function names where sensible (`bvhtree_*`, `BLI_x` → `x` in the
  `fileops_c` module, etc.).
- Every module must build clean (`cargo build -p <crate>`); add a small unit test for
  algorithmic ports. Commit + push to `coobyk/rust-work` after each file.

## bmesh (source/blender/bmesh) — in progress

Crate: `crates/blenders-bmesh`. Index-arena model: pools are `Vec`, links are `usize` indices, `BM_NIL = usize::MAX`; killed elements have `head.htype == 0` (tombstones, no compaction). BMO oflags ride on `head.api_flag`. CustomData/flag layers/toolflags: PORT_PENDING.

- bmesh_class.hh → `lib.rs` (BMHeader, BMVert, BMEdge, BMLoop, BMFace, BMesh)
- bmesh_structure.cc → `structure.rs` (disk/radial/cycle ops)
- bmesh_core.cc → `core.rs` (create/kill; kernel split/join/splice PORT_PENDING)
- bmesh_mesh.cc → `mesh.rs` (index ensure, at_index_find, coords apply; remap/toolflags PORT_PENDING)
- bmesh_iterators.cc → `iterators.rs` (iterator structs)
- bmesh_construct.cc → `construct.rs` (quad_tri/ngon/ensure/copy; CustomData PORT_PENDING)
- bmesh_delete.cc → `delete.rs` (hflag + oflag contexts)
- Remaining: query, marking, interp, mesh_convert, mesh_normals, operators/opdefines, walkers, uvselect, polygon(+edgenet), log, mods, edgeloop, callback_generic, kernel splits/joins in core.

- bmesh_bevel.cc -> `tools/bmesh_bevel.rs` FULL port (6.7k lines): BM_mesh_bevel wired in bmo_bevel
- makesdna DNA headers underway in `blenders-dna`: DNA_ID.h, DNA_ID_enums.h, DNA_vec_types.h, DNA_customdata_types.h, DNA_meshdata_types.h, DNA_mesh_types.h, DNA_object_types.h, DNA_object_enums.h, DNA_collection_types.h, DNA_layer_types.h, DNA_{world,colorband,text,packedFile,uuid,image_enums,gpu,session_uid,view3d_enums,attribute}_types.h ported

## makesdna (source/blender/makesdna) — COMPLETE (97/97 headers)

All `DNA_*.h` headers are ported into `crates/blenders-dna` via the
`/tmp/struct_conv.py` + `/tmp/enum_conv.py` converters (struct→`#[derive(Clone)]`
+ `impl Default`, enums→`pub type` alias + verbatim `pub const` members).
Conventions: pointers→`usize` arena idx (`id::NIL`), `char*`→`Option<String>`,
`char name[N]`→`[u8;N]`, `ListBaseT`→`blenders_bli::listbase::ListBase`,
`void*`/runtime ptrs→usize, `T *p` owned arrays→`Vec<T>`, `T **p`→`Vec<usize>`.
Notable: `screen_types` (bScreen/ARegion/Panel/uiList), `space_types` (44
editor structs), `node_types` (236 structs), `modifier_types` (102 structs +
~40 GP modifiers), `userdef_types`, `sequence_types`, `windowmanager_types`,
`gpencil_legacy_types`, `particle_types`, `fluid_types`, `linestyle_types`
(41 Freestyle modifiers), `node_tree_interface_types`, `sdna_types`+`genfile`
(SDNA tables + `dna_elem_type_size`), `effect_types` (legacy particles).

Next: `blenkernel/intern/*.cc` (~330k lines) into `blenders-bke`.

## blenkernel → blenders-bke (first batch)

`blenders-bke` hosts `source/blender/blenkernel/intern/*.cc` ports. Handles:
`Main`, `bContext`, `Depsgraph`, RNA, and foreign-subsystem pointers are
`usize` handles; code paths gated on those keep their structure and carry
`PORT_PENDING` notes naming the missing system. Fully ported logic includes:

- `appdir.cc` — the whole folder-id search table, env-var precedence,
  portable-layout, program-path init, tempdir, app-template discovery
  (GHOST → platform conventions).
- `autoexec.cc`, `bake_items_paths.cc`, `bake_data_block_map.cc`,
  `callbacks.cc` (all 43 `eCbEvent` codes), `cpp_types.cc`,
  `mesh_flip_faces.cc`, `mesh_merge_customdata.cc` (UV weld),
  `mesh_topology_state.cc`, `editlattice.cc`, `editmesh_cache.cc`
  (against `blenders-bmesh` normals/center fns), `icons_rasterize.cc`
  (triangle rasterizer + box downscale), `armature_selection.cc`,
  `multires_subdiv.cc`, `multires_reshape_{ccg,subdivide,versioning}.cc`,
  `subdiv_{converter,displacement,topology,deform,stats}.cc`,
  `node_tree_dot_export.cc`, `node_enum_definition.cc`,
  `main_invariants.cc`, `addon.cc`, `blender_user_menu.cc`,
  `blender_undo.cc`, `blender_copybuffer.cc`, `anim_data_bmain_utils.cc`,
  `curve_convert.cc`, `action_bones.cc`, `lib_id_remapper.cc`,
  `sound_reader_cache.cc`, `volume_grid_fields.cc`,
  `geometry_component_{mesh,curves,instances,volume,pointcloud,grease_pencil,edit_data}.cc`,
  `idtype.cc`/`speaker.cc` (`IDTypeInfo` + `IDTYPE_ID_SPK`),
  `scene_context.cc`, `view3d_runtime.cc`, `paint_runtime.cc`,
  `paint_canvas.cc`, `wm_runtime.cc`, `mesh_debug.cc`,
  `vfontdata_freetype.cc`, `text_suggestions.cc`,
  `lib_id_eval.cc`.
