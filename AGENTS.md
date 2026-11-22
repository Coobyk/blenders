# AGENTS.md — Blender → Rust + GPUI port ("blenders")

This repository is a public fork of upstream Blender (`blender/blender`) that is being
rewritten line-by-line in Rust + GPUI. The original C/C++ source stays in-tree as the
porting reference. If you are an agent working in this repo: port code, keep it faithful,
keep the build green, commit and push frequently.

## Mission

Every `.cc` file under `source/blender/` gets a faithful Rust counterpart under
`rust/crates/`, and the application UI is rebuilt on GPUI (`rust/src/main.rs`, the
`blenders` binary). The goal is a fully ported codebase — real algorithms, real math,
not stubs — while keeping upstream naming and structure so every line can be traced
back to its C/C++ original.

## Layout

```
source/blender/          upstream C/C++ (read-only porting reference)
rust/                    the Rust workspace
  Cargo.toml             workspace members (vendor/ patched deps)
  src/main.rs            GPUI app (workspace shell, viewport, orbit/zoom)
  PORTING.md             running port journal + module status
  crates/
    blenders-math        math_*.cc ports (Vec3/Mat3/Mat4/Quaternion, geom fns)
    blenders-bli         blenlib (containers, path_utils, scanfill, noise, ...)
    blenders-bmesh       bmesh kernel + tools + all BMO operator execs
    blenders-dna         DNA_*.h types + enums
    blenders-bke         blenkernel intern/*.cc
    blenders-deg         depsgraph
    blenders-wm          windowmanager + gizmos + xr + message bus
    blenders-ed          editors (transform system, view3d, space_*)
    blenders-nodes       geometry/shader/function nodes (SOCKS decls + exec)
    blenders-mod         modifiers (apply/deform bodies)
    blenders-draw        draw engine + eevee + workbench shells
    blenders-gpu         GPU module shells
    blenders-io          OBJ/STL/PLY/ABC/FBX/USD codecs
    blenders-cmp         compositor
    blenders-render      render engine glue
    blenders-seq         sequencer
    blenders-animrig/-asset/-blf/-blt/-blo/-fn/-geo/-ik/-imbuf/
    -makesdna/-makesrna/-py/-shaderfx/-sim/-thumb   the rest
```

## Porting rules (follow exactly)

- One Rust module per upstream file where practical: `foo_c.cc` → `foo_c.rs`,
  `BLI_foo.cc` → `foo.rs`, `tools/bmesh_x.cc` → `bmesh_x.rs`. Register every new
  file with `pub mod <name>;` in the crate's `lib.rs` — forgetting this breaks the build.
- Port the algorithm verbatim: same control flow, same constant values, same function
  names where sensible (`ed_view3d_project_internal`, `BM_face_triangulate`, ...).
- C pointers/handles → `usize`/`i32` indices into `Vec` arenas (`-1`/None for null).
  Never invent lifetimes where an index does the job.
- Un-ported callees (external engines: Bullet, AUD, Mantaflow, ImBuf, depsgraph, RNA):
  inject as `&mut dyn FnMut(...)` / `&dyn Fn(...)` params, or leave a
  `// PORT_PENDING: <what>` / `// PORT_NOTE: <why>` comment.
- `BLI_task_parallel_range` / TBB / SIMD → serial scalar code with a PORT_NOTE.
- `*_test.cc` → `#[cfg(test)]` deferred; only port enabled `#ifdef` branches.
- Enum/const names verbatim: `TFM_TRANSLATION`, `MOD_SCREW_NORMAL_FLIP`, `V3D_PROJ_RET_OK`, ...
- Keep comments short and cite the source (`/// Port of 'editors/transform/transform_mode_shear.cc' —`).

## Node-file skeleton (blenders-nodes)

Each node file keeps the same shape:
`SOCKS: &[(&str, i32, i32)]` (name, `SOCK_*` type, 1=in/2=out) →
`declare()` pushes `SocketDeclaration` into `decl.inputs`/`decl.outputs` →
`update()`/`exec()` carry the real node math (extracted verbatim per node type).

## Modifier skeleton (blenders-mod)

`<Mod>ModifierData` struct with verbatim field names → `apply()`/`deform` producing or
mutating `MeshChunks { verts, edges, corner_verts, face_offsets }` (shared output type).

## Build & test

```bash
cd rust
cargo build -p <crate>          # verify a crate compiles — ALWAYS before committing
cargo build --workspace         # full check
cargo test -p <crate>           # run tests
cargo build --bin blenders      # the GPUI app
```

The app cannot launch on headless VMs (`NoSupportedDeviceFound` — no GPU/X11);
`cargo build --bin blenders` is the verification there. It runs on a real desktop.

## Git workflow

- Branch: work locally on `main` in this checkout, push to
  `coobyk` (`github.com/Coobyk/blenders`) on ref `refs/heads/rust-work`.
- Stage per crate: `git add crates/<crate>` or `git add src/` — NEVER `git add .`
  (the tree contains the entire upstream source).
- Commit: `git -c core.hooksPath=/dev/null commit -qm "<crate>: <what was ported>"`
- Push: `git push -q coobyk HEAD:refs/heads/rust-work`
- Commit early and often — one file or one coherent batch per commit.
- Benign push noise: "unstaged changes" warning + LFS lock hint; pushes still succeed.
- `Cargo.lock` churn and auto-generated `lib.rs` mod-registration edits may stay
  unstaged unless they belong to your change.

## Useful crate APIs (already verified)

- `blenders_math`: `Vec3::new`, `Quaternion::from_axis_angle(AxisAngle { axis: Vec3::new(...), angle })`
  (struct literal — no `AxisAngle::new`), `Quaternion::new(w,x,y,z)`, `q1 * q2` (Mul trait),
  destructure via `[q.w, q.x, q.y, q.z]`.
- gpui 0.2.2: `canvas(prepaint, paint)`, `PathBuilder::stroke(px(1.0))` +
  `move_to/line_to/build()`, `window.paint_path(path, rgb(c))`, listeners need
  `.id()` + explicit param types (`|this: &mut V, e: &gpui::MouseDownEvent, _w, cx|`),
  `ScrollDelta::{Lines,Pixels}`, `.on_mouse_down(MouseButton::Middle, ...)`,
  `.on_scroll_wheel`, `.on_mouse_move`, `.when(cond, |d| ...)`.
- Borrow trap: a `|| { ... cx ... }` closure captures `&mut Context` and blocks later
  `cx` uses — convert to `|cx: &mut Context<V>| { ... }` and call `closure(cx)`.

## Current status

File-level coverage exists for every `source/blender` module. Ongoing work:
filling `PORT_PENDING` internals — remaining node exec bodies, remaining modifier
applies, editor internals (transform_convert_* extras, space_* ops, gizmos),
GPU/draw engine internals, makesrna register bodies — and growing the GPUI app
(per-workspace layouts, real node/UV editors, more viewport interactions).

See `rust/PORTING.md` for the per-module journal.
