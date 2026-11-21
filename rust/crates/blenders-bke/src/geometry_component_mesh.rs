//! Port of `intern/geometry_component_mesh.cc` — `MeshComponent`
//! methods + `bke::mesh_normals_varray`.
//!
//! `MeshComponent` itself lives in `geometry_components`; this file
//! adds the cc's remaining methods (`copy`/`owns_direct_data`/
//! `ensure_owns_direct_data`/`count_memory`) as free fns with
//! `BKE_mesh_copy_for_eval`/`Mesh::count_memory` injected, plus the
//! normals VArray (as a `Vec<Vec3>` materialized per IndexMask).

use blenders_dna::id::NIL;
use blenders_math::vector::Vec3;

use crate::attribute_legacy_convert::AttrDomain;
use crate::geometry_components::{GeometryOwnershipType, MeshComponent};

/// `MeshComponent::copy` — `copy_for_eval` injected.
pub fn mesh_component_copy(
    c: &MeshComponent,
    copy_for_eval: &mut dyn FnMut(usize) -> usize,
) -> MeshComponent {
    let mut new_component = MeshComponent::new();
    if c.mesh != NIL {
        new_component.mesh = copy_for_eval(c.mesh);
        new_component.ownership = GeometryOwnershipType::Owned;
    }
    new_component
}

/// `owns_direct_data`.
pub fn mesh_component_owns_direct_data(c: &MeshComponent) -> bool {
    c.ownership == GeometryOwnershipType::Owned
}

/// `ensure_owns_direct_data` — ReadOnly → copy_for_eval.
pub fn mesh_component_ensure_owns_direct_data(
    c: &mut MeshComponent,
    copy_for_eval: &mut dyn FnMut(usize) -> usize,
) {
    if c.ownership != GeometryOwnershipType::Owned {
        if c.mesh != NIL {
            c.mesh = copy_for_eval(c.mesh);
        }
        c.ownership = GeometryOwnershipType::Owned;
    }
}

/// `count_memory` — `Mesh::count_memory` injected.
pub fn mesh_component_count_memory(c: &MeshComponent, count_memory: &mut dyn FnMut(usize)) {
    if c.mesh != NIL {
        count_memory(c.mesh);
    }
}

/// `MeshComponent::attributes` / `attributes_for_write` — the
/// `AttributeAccessor` machinery isn't ported (PORT_PENDING:
/// `mesh_attribute_accessor_functions`).

/// `mesh_normals_varray` — normals per `domain`/`mask`. Spans for
/// the true and evaluated normal caches + the edges are passed in;
/// `adapt_domain` (Face→Corner) is injected (returns corner normals
/// given the face normals).
#[allow(clippy::too_many_arguments)]
pub fn mesh_normals_varray(
    domain: AttrDomain,
    mask: &[i32],
    min_array_size: usize,
    vert_normals: &[Vec3],
    vert_normals_true: &[Vec3],
    face_normals: &[Vec3],
    face_normals_true: &[Vec3],
    corner_normals: &[Vec3],
    edges: &[[i32; 2]],
    no_corner_normals: bool,
    true_normals: bool,
    adapt_domain_face_to_corner: &mut dyn FnMut(&[Vec3]) -> Vec<Vec3>,
) -> Vec<Vec3> {
    match domain {
        AttrDomain::Face => {
            (if true_normals { face_normals_true } else { face_normals }).to_vec()
        }
        AttrDomain::Point => {
            (if true_normals { vert_normals_true } else { vert_normals }).to_vec()
        }
        AttrDomain::Edge => {
            let vn = if true_normals { vert_normals_true } else { vert_normals };
            let mut edge_normals = vec![Vec3::ZERO; min_array_size.max(mask.len())];
            for &i in mask {
                let e = edges[i as usize];
                let a = vn[e[0] as usize];
                let b = vn[e[1] as usize];
                /* `math::normalize(math::interpolate(a, b, 0.5))`. */
                let v = Vec3::new(
                    (a.x + b.x) * 0.5,
                    (a.y + b.y) * 0.5,
                    (a.z + b.z) * 0.5,
                );
                edge_normals[i as usize] = v.normalized();
            }
            edge_normals
        }
        AttrDomain::Corner => {
            if no_corner_normals || true_normals {
                let f = if true_normals { face_normals_true } else { face_normals };
                adapt_domain_face_to_corner(f)
            } else {
                corner_normals.to_vec()
            }
        }
        _ => Vec::new(),
    }
}
