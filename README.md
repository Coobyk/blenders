Blenders
========

Blender rewritten in Rust and GPUI (port of blender/blender).

- Upstream sources (C/C++): kept in-tree as the porting reference.
- Rust port: `rust/` — a Cargo workspace. `cargo run -p blenders` opens the app shell (GPUI).
- Ported so far: `blenders-math` (vectors/matrices/quaternions/eulers — `BLI_math_*`) and `blenders-core` (`ID`, `Main`, `Object` — BKE/DNA base layer).
- Porting convention: matrices are `mat[col][row]` with translation at `mat[3]`; quaternions `(w,x,y,z)`; docs in each file cite the upstream symbol.

Upstream README (for reference)
===============================

Blender is the free and open source 3D creation suite.
It supports the entirety of the 3D pipeline—modeling, rigging, animation, simulation, rendering, compositing,
motion tracking and video editing.

- [Main Website](https://www.blender.org)
- [Build Instructions](https://developer.blender.org/docs/handbook/building_blender/)
- [Code Review & Bug Tracker](https://projects.blender.org)
- [Developer Forum](https://devtalk.blender.org)
