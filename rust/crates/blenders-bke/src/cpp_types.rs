//! Port of `intern/cpp_types.cc` — `BKE_cpp_types_init` registration list.
//!
//! The runtime-type registry lives in `blenders_bli::cpp_type`; this module
//! holds the exact registration set the C++ file declares. Types whose Rust
//! counterpart does not exist yet register under a name marker so the table
//! is complete and auditable.

/// `CPPTypeFlags` (BLI_cpp_type.hh).
pub mod cpp_type_flags {
    pub const NONE: u32 = 0;
    pub const BASIC_TYPE: u32 = 1 << 0;
    pub const PRINTABLE: u32 = 1 << 1;
    pub const EQUALITY_COMPARABLE: u32 = 1 << 2;
    pub const HASHABLE: u32 = 1 << 3;
}

use cpp_type_flags::*;

/// One `BLI_CPP_TYPE_REGISTER(type, flags)` entry.
pub struct CppTypeRegistration {
    pub name: &'static str,
    pub flags: u32,
}

/// The registration set from `BKE_cpp_types_init`, verbatim.
pub static CPP_TYPE_REGISTRATIONS: &[CppTypeRegistration] = &[
    CppTypeRegistration { name: "bke::GeometrySet", flags: PRINTABLE | EQUALITY_COMPARABLE },
    CppTypeRegistration { name: "bke::InstanceReference", flags: NONE },
    CppTypeRegistration { name: "Object *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "Collection *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "Tex *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "Image *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "Material *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "VFont *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "Scene *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "Text *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "Mask *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "bSound *", flags: BASIC_TYPE },
    CppTypeRegistration { name: "MStringProperty", flags: NONE },
    CppTypeRegistration { name: "nodes::MenuValue", flags: HASHABLE | EQUALITY_COMPARABLE },
    CppTypeRegistration { name: "nodes::BundlePtr", flags: EQUALITY_COMPARABLE },
    CppTypeRegistration { name: "nodes::ClosurePtr", flags: EQUALITY_COMPARABLE },
    CppTypeRegistration { name: "bke::GeometryNodesReferenceSet", flags: NONE },
    CppTypeRegistration { name: "bke::SocketValueVariant", flags: NONE },
    CppTypeRegistration { name: "nodes::GeoNodesMultiInput<bke::SocketValueVariant>", flags: NONE },
    CppTypeRegistration { name: "nodes::BundleItemValue", flags: NONE },
    CppTypeRegistration { name: "fn::GField", flags: NONE },
    CppTypeRegistration { name: "fn::Field<float>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<float2>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<float3>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<float4>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<int>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<int2>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<bool>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<int8_t>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<short2>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<ColorGeometry4f>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<ColorGeometry4b>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<math::Quaternion>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<float4x4>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<nodes::MenuValue>", flags: NONE },
    CppTypeRegistration { name: "fn::Field<std::string>", flags: NONE },
    CppTypeRegistration { name: "nodes::GListPtr", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<float>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<float2>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<float3>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<float4>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<int>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<int2>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<bool>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<int8_t>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<short2>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<ColorGeometry4f>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<ColorGeometry4b>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<math::Quaternion>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<float4x4>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<nodes::MenuValue>", flags: NONE },
    CppTypeRegistration { name: "nodes::ListPtr<std::string>", flags: NONE },
    /* WITH_OPENVDB volume grids. */
    CppTypeRegistration { name: "bke::volume_grid::GVolumeGrid", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<float>", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<float3>", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<bool>", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<double>", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<int>", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<int64_t>", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<int3>", flags: NONE },
    CppTypeRegistration { name: "bke::volume_grid::VolumeGrid<double3>", flags: NONE },
];
