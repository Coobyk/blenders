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
