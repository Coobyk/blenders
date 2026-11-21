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
