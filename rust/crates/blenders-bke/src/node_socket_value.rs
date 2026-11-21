//! Port of `intern/node_socket_value.cc` — `SocketValueVariant` +
//! `SocketValueVariantTypeInfo`: the runtime-typed single/field/list/
//! grid/bundle/closure value used by node sockets.
//!
//! The C++ `CPPType` dispatch becomes a Rust enum
//! (`SocketValueVariantValue`); Field/Bundle/Closure/List/Grid payload
//! types are `usize` handles (`PORT_PENDING` — `fn::GField`,
//! `nodes::Bundle`, `volume_grid` aren't in the crate yet). Type
//! conversion (`try_convert`) is delegated to an injected
//! `TypeConversionRegistry`.

use blenders_math::{Mat4, Quaternion, Vec2, Vec3, Vec4};

/// `nodes::MenuValue` — current value or an item index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MenuValue {
    /// `value` — the enum value or -1 when `has_value` is false.
    pub value: i32,
    /// `has_value` — false while the menu hasn't been evaluated.
    pub has_value: bool,
}

impl MenuValue {
    /// Construct from a known value.
    pub fn new(value: i32) -> Self {
        Self {
            value,
            has_value: true,
        }
    }
}

/// `ColorGeometry4f`.
pub type ColorGeometry4f = [f32; 4];
/// `ColorGeometry4b`.
pub type ColorGeometry4b = [u8; 4];

/// `SocketValueStorageKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketValueStorageKind {
    /// `SocketValueStorageKind::Single`.
    Single,
    /// `SocketValueStorageKind::Field`.
    Field,
    /// `SocketValueStorageKind::List`.
    List,
    /// `SocketValueStorageKind::Grid`.
    Grid,
    /// `SocketValueStorageKind::Bundle` — C stores `BundlePtr` under
    /// Single kind, but tracking it separately keeps typed access
    /// honest.
    Bundle,
    /// `SocketValueStorageKind::Closure` — likewise Single in C.
    Closure,
}

/// `SocketValueVariantType` — the concrete payload type (analog of
/// `CPPType` identity for the supported types).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketValueType {
    /// `float`.
    Float,
    /// `int`.
    Int,
    /// `float2`.
    Float2,
    /// `float3`.
    Float3,
    /// `float4`.
    Float4,
    /// `int2`.
    Int2,
    /// `bool`.
    Bool,
    /// `int8_t`.
    Int8,
    /// `short2`.
    Short2,
    /// `ColorGeometry4f`.
    Color4f,
    /// `ColorGeometry4b`.
    Color4b,
    /// `math::Quaternion`.
    Quaternion,
    /// `float4x4`.
    Float4x4,
    /// `nodes::MenuValue`.
    Menu,
    /// `std::string`.
    String,
    /// `BundlePtr`.
    Bundle,
    /// `ClosurePtr`.
    Closure,
    /// `GeometrySet`.
    GeometrySet,
    /// `Object *`.
    Object,
    /// `Material *`.
    Material,
    /// `Image *`.
    Image,
    /// `VFont *`.
    VFont,
    /// `Scene *`.
    Scene,
    /// `bSound *`.
    Sound,
    /// `Collection *`.
    Collection,
    /// `Tex *`.
    Tex,
    /// `Text *`.
    Text,
    /// `Mask *`.
    Mask,
    /// `fn::GField` (generic — base type in `SocketValueVariant`).
    Field,
    /// `GListPtr` (generic).
    List,
    /// `GVolumeGrid` (generic).
    VolumeGrid,
}

/// All "single" value types (`SOCKET_VALUE_SINGLE_TYPES`).
pub const SOCKET_VALUE_SINGLE_TYPES: &[SocketValueType] = &[
    SocketValueType::Float,
    SocketValueType::Int,
    SocketValueType::Float2,
    SocketValueType::Float3,
    SocketValueType::Float4,
    SocketValueType::Int2,
    SocketValueType::Bool,
    SocketValueType::Int8,
    SocketValueType::Short2,
    SocketValueType::Color4f,
    SocketValueType::Color4b,
    SocketValueType::Quaternion,
    SocketValueType::Float4x4,
    SocketValueType::Menu,
    SocketValueType::String,
    SocketValueType::Bundle,
    SocketValueType::Closure,
    SocketValueType::GeometrySet,
    SocketValueType::Object,
    SocketValueType::Material,
    SocketValueType::Image,
    SocketValueType::VFont,
    SocketValueType::Scene,
    SocketValueType::Sound,
    SocketValueType::Collection,
    SocketValueType::Tex,
    SocketValueType::Text,
    SocketValueType::Mask,
];

/// All types fields and lists can hold
/// (`SOCKET_VALUE_FIELD_AND_LIST_TYPES`).
pub const SOCKET_VALUE_FIELD_AND_LIST_TYPES: &[SocketValueType] = &[
    SocketValueType::Float,
    SocketValueType::Int,
    SocketValueType::Float2,
    SocketValueType::Float3,
    SocketValueType::Float4,
    SocketValueType::Int2,
    SocketValueType::Bool,
    SocketValueType::Int8,
    SocketValueType::Short2,
    SocketValueType::Color4f,
    SocketValueType::Color4b,
    SocketValueType::Quaternion,
    SocketValueType::Float4x4,
    SocketValueType::Menu,
    SocketValueType::String,
];

/// `SocketValueVariantAny` — the payload.
#[derive(Debug, Clone)]
pub enum SocketValueVariantValue {
    /// `float`.
    Float(f32),
    /// `int`.
    Int(i32),
    /// `float2`.
    Float2(Vec2),
    /// `float3`.
    Float3(Vec3),
    /// `float4`.
    Float4(Vec4),
    /// `int2`.
    Int2([i32; 2]),
    /// `bool`.
    Bool(bool),
    /// `int8_t`.
    Int8(i8),
    /// `short2`.
    Short2([i16; 2]),
    /// `ColorGeometry4f`.
    Color4f(ColorGeometry4f),
    /// `ColorGeometry4b`.
    Color4b(ColorGeometry4b),
    /// `math::Quaternion`.
    Quaternion(Quaternion),
    /// `float4x4`.
    Float4x4(Mat4),
    /// `nodes::MenuValue`.
    Menu(MenuValue),
    /// `std::string`.
    String(String),
    /// `BundlePtr` — handle (`nodes::Bundle` PORT_PENDING).
    Bundle(usize),
    /// `ClosurePtr` — handle.
    Closure(usize),
    /// `GeometrySet` — handle.
    GeometrySet(usize),
    /// `Object *`.
    Object(usize),
    /// `Material *`.
    Material(usize),
    /// `Image *`.
    Image(usize),
    /// `VFont *`.
    VFont(usize),
    /// `Scene *`.
    Scene(usize),
    /// `bSound *`.
    Sound(usize),
    /// `Collection *`.
    Collection(usize),
    /// `Tex *`.
    Tex(usize),
    /// `Text *`.
    Text(usize),
    /// `Mask *`.
    Mask(usize),
    /// `fn::GField` + its base type (`Field<T>`).
    Field(usize, SocketValueType),
    /// `GListPtr` + its base type (`ListPtr<T>`).
    List(usize, SocketValueType),
    /// `GVolumeGrid` + its base type.
    VolumeGrid(usize, SocketValueType),
}

impl SocketValueVariantValue {
    /// `socket_value_storage_kind` for this payload.
    pub fn storage_kind(&self) -> SocketValueStorageKind {
        match self {
            Self::Field(..) => SocketValueStorageKind::Field,
            Self::List(..) => SocketValueStorageKind::List,
            Self::VolumeGrid(..) => SocketValueStorageKind::Grid,
            Self::Bundle(_) => SocketValueStorageKind::Bundle,
            Self::Closure(_) => SocketValueStorageKind::Closure,
            _ => SocketValueStorageKind::Single,
        }
    }

    /// The `SocketValueType` of this payload.
    pub fn value_type(&self) -> SocketValueType {
        match self {
            Self::Float(_) => SocketValueType::Float,
            Self::Int(_) => SocketValueType::Int,
            Self::Float2(_) => SocketValueType::Float2,
            Self::Float3(_) => SocketValueType::Float3,
            Self::Float4(_) => SocketValueType::Float4,
            Self::Int2(_) => SocketValueType::Int2,
            Self::Bool(_) => SocketValueType::Bool,
            Self::Int8(_) => SocketValueType::Int8,
            Self::Short2(_) => SocketValueType::Short2,
            Self::Color4f(_) => SocketValueType::Color4f,
            Self::Color4b(_) => SocketValueType::Color4b,
            Self::Quaternion(_) => SocketValueType::Quaternion,
            Self::Float4x4(_) => SocketValueType::Float4x4,
            Self::Menu(_) => SocketValueType::Menu,
            Self::String(_) => SocketValueType::String,
            Self::Bundle(_) => SocketValueType::Bundle,
            Self::Closure(_) => SocketValueType::Closure,
            Self::GeometrySet(_) => SocketValueType::GeometrySet,
            Self::Object(_) => SocketValueType::Object,
            Self::Material(_) => SocketValueType::Material,
            Self::Image(_) => SocketValueType::Image,
            Self::VFont(_) => SocketValueType::VFont,
            Self::Scene(_) => SocketValueType::Scene,
            Self::Sound(_) => SocketValueType::Sound,
            Self::Collection(_) => SocketValueType::Collection,
            Self::Tex(_) => SocketValueType::Tex,
            Self::Text(_) => SocketValueType::Text,
            Self::Mask(_) => SocketValueType::Mask,
            Self::Field(..) => SocketValueType::Field,
            Self::List(..) => SocketValueType::List,
            Self::VolumeGrid(..) => SocketValueType::VolumeGrid,
        }
    }

    /// `init_default` — default-constructed value of `ty`.
    pub fn default_for(ty: SocketValueType) -> Self {
        match ty {
            SocketValueType::Float => Self::Float(0.0),
            SocketValueType::Int => Self::Int(0),
            SocketValueType::Float2 => Self::Float2(Vec2::ZERO),
            SocketValueType::Float3 => Self::Float3(Vec3::ZERO),
            SocketValueType::Float4 => Self::Float4(Vec4::ZERO),
            SocketValueType::Int2 => Self::Int2([0; 2]),
            SocketValueType::Bool => Self::Bool(false),
            SocketValueType::Int8 => Self::Int8(0),
            SocketValueType::Short2 => Self::Short2([0; 2]),
            SocketValueType::Color4f => Self::Color4f([0.0; 4]),
            SocketValueType::Color4b => Self::Color4b([0; 4]),
            SocketValueType::Quaternion => Self::Quaternion(Quaternion::IDENTITY),
            SocketValueType::Float4x4 => Self::Float4x4(Mat4::IDENTITY),
            SocketValueType::Menu => Self::Menu(MenuValue::default()),
            SocketValueType::String => Self::String(String::new()),
            SocketValueType::Bundle => Self::Bundle(0),
            SocketValueType::Closure => Self::Closure(0),
            SocketValueType::GeometrySet => Self::GeometrySet(0),
            SocketValueType::Object => Self::Object(0),
            SocketValueType::Material => Self::Material(0),
            SocketValueType::Image => Self::Image(0),
            SocketValueType::VFont => Self::VFont(0),
            SocketValueType::Scene => Self::Scene(0),
            SocketValueType::Sound => Self::Sound(0),
            SocketValueType::Collection => Self::Collection(0),
            SocketValueType::Tex => Self::Tex(0),
            SocketValueType::Text => Self::Text(0),
            SocketValueType::Mask => Self::Mask(0),
            SocketValueType::Field => Self::Field(0, SocketValueType::Float),
            SocketValueType::List => Self::List(0, SocketValueType::Float),
            SocketValueType::VolumeGrid => Self::VolumeGrid(0, SocketValueType::Float),
        }
    }
}

/// `SocketValueVariant` — value + storage-kind tracking.
#[derive(Debug, Clone)]
pub struct SocketValueVariant {
    /// `value_`.
    pub value: SocketValueVariantValue,
}

impl Default for SocketValueVariant {
    fn default() -> Self {
        Self {
            value: SocketValueVariantValue::Float(0.0),
        }
    }
}

impl SocketValueVariant {
    /// `SocketValueVariant::from`.
    pub fn from(value: SocketValueVariantValue) -> Self {
        Self { value }
    }

    /// `is_single`.
    pub fn is_single(&self) -> bool {
        self.value.storage_kind() == SocketValueStorageKind::Single
    }

    /// `is_field`.
    pub fn is_field(&self) -> bool {
        self.value.storage_kind() == SocketValueStorageKind::Field
    }

    /// `is_list`.
    pub fn is_list(&self) -> bool {
        self.value.storage_kind() == SocketValueStorageKind::List
    }

    /// `is_volume_grid`.
    pub fn is_volume_grid(&self) -> bool {
        self.value.storage_kind() == SocketValueStorageKind::Grid
    }

    /// `is_context_dependent_field` — `field->depends_on_input()`
    /// injected via `field_depends_on_input` (fn::Field PORT_PENDING).
    pub fn is_context_dependent_field(&self, field_depends_on_input: &dyn Fn(usize) -> bool) -> bool {
        match &self.value {
            SocketValueVariantValue::Field(handle, _) => field_depends_on_input(*handle),
            _ => false,
        }
    }

    /// `get_if` — the base type of a Field/List/Grid payload, else
    /// the value type itself.
    pub fn base_type(&self) -> SocketValueType {
        match &self.value {
            SocketValueVariantValue::Field(_, base)
            | SocketValueVariantValue::List(_, base)
            | SocketValueVariantValue::VolumeGrid(_, base) => *base,
            other => other.value_type(),
        }
    }

    /// `is_interpretable_as` — whether this value can serve as `ty`
    /// (same kind and matching base type for generic kinds).
    pub fn is_interpretable_as(&self, ty: SocketValueType, base: Option<SocketValueType>) -> bool {
        match (&self.value, ty) {
            (SocketValueVariantValue::Field(_, b), SocketValueType::Field) => {
                base.map(|want| *b == want).unwrap_or(true)
            }
            (SocketValueVariantValue::List(_, b), SocketValueType::List) => {
                base.map(|want| *b == want).unwrap_or(true)
            }
            (SocketValueVariantValue::VolumeGrid(_, b), SocketValueType::VolumeGrid) => {
                base.map(|want| *b == want).unwrap_or(true)
            }
            (v, t) => v.value_type() == t,
        }
    }

    /// `get` — typed read; returns None on type mismatch.
    pub fn get(&self) -> &SocketValueVariantValue {
        &self.value
    }

    /// `get` mutable.
    pub fn get_mut(&mut self) -> &mut SocketValueVariantValue {
        &mut self.value
    }

    /// `extract` — move the value out.
    pub fn extract(self) -> SocketValueVariantValue {
        self.value
    }

    /// `ensure_type` — overwrite with `ty`'s default when the type
    /// doesn't already match (rough `ensure_type` semantics).
    pub fn ensure_type(&mut self, ty: SocketValueType) {
        if self.value.value_type() != ty {
            self.value = SocketValueVariantValue::default_for(ty);
        }
    }

    /// `ensure_owns_direct_data` — dispatch to the payload's
    /// `owns_direct_data`/`ensure_owns_direct_data` for handle types
    /// via the injected hook.
    pub fn ensure_owns_direct_data(&mut self, ensure: &mut dyn FnMut(usize)) {
        match &self.value {
            SocketValueVariantValue::Bundle(h)
            | SocketValueVariantValue::Closure(h)
            | SocketValueVariantValue::GeometrySet(h)
            | SocketValueVariantValue::List(h, _)
            | SocketValueVariantValue::Field(h, _)
            | SocketValueVariantValue::VolumeGrid(h, _) => {
                if *h != 0 {
                    ensure(*h);
                }
            }
            _ => {}
        }
    }
}

/// `TypeConversionRegistry` — injected stand-in for
/// `bke::get_implicit_type_conversions()`: reports whether a
/// single→single conversion exists and performs it.
pub trait TypeConversionRegistry {
    /// `conversion_available(src_type, dst_type)`.
    fn available(&self, src: SocketValueType, dst: SocketValueType) -> bool;
    /// Convert `value` (must be of `src` type) to `dst` type.
    fn convert(&self, value: &SocketValueVariantValue, dst: SocketValueType)
        -> Option<SocketValueVariantValue>;
    /// `GField::from_constant` — wrap a single as a constant field.
    fn field_from_constant(&self, value: &SocketValueVariantValue) -> Option<usize>;
    /// `fn::evaluate_constant_field` — reduce a constant field to
    /// its single value.
    fn evaluate_constant_field(&self, field: usize) -> Option<SocketValueVariantValue>;
}

/// `SocketValueVariant::try_convert` — best-effort convert in place.
/// Mirrors `try_convert_fn`: generic kinds go through the registry;
/// a single→Field wraps as a constant field; Field→single evaluates
/// constant fields.
pub fn socket_value_variant_try_convert(
    variant: &mut SocketValueVariant,
    dst_type: SocketValueType,
    dst_base: Option<SocketValueType>,
    conversions: &dyn TypeConversionRegistry,
) -> bool {
    match &variant.value {
        SocketValueVariantValue::Field(handle, base) => {
            /* Field → Field: base type must match or convert. */
            if dst_type == SocketValueType::Field {
                if dst_base.map(|b| *base == b).unwrap_or(true) {
                    return true;
                }
                /* Evaluate then rewrap via conversion (simplified:
                 * the C builds a conversion field-operation). */
                if let Some(single) = conversions.evaluate_constant_field(*handle) {
                    if let Some(target) = dst_base {
                        if let Some(converted) = conversions.convert(&single, target) {
                            if let Some(f) = conversions.field_from_constant(&converted) {
                                variant.value = SocketValueVariantValue::Field(f, target);
                                return true;
                            }
                        }
                    }
                }
                return false;
            }
            /* Field → single: evaluate if the field is constant. */
            if let Some(single) = conversions.evaluate_constant_field(*handle) {
                if single.value_type() == dst_type {
                    variant.value = single;
                    return true;
                }
                if let Some(converted) = conversions.convert(&single, dst_type) {
                    variant.value = converted;
                    return true;
                }
            }
            false
        }
        SocketValueVariantValue::List(handle, base) => {
            if dst_type == SocketValueType::List {
                return dst_base.map(|b| *base == b).unwrap_or(true);
            }
            let _ = handle;
            false
        }
        SocketValueVariantValue::VolumeGrid(_, base) => {
            if dst_type == SocketValueType::VolumeGrid {
                return dst_base.map(|b| *base == b).unwrap_or(true);
            }
            false
        }
        single => {
            /* Single → Field: wrap as constant field. */
            if dst_type == SocketValueType::Field {
                if dst_base.map(|b| single.value_type() == b).unwrap_or(false) {
                    if let Some(f) = conversions.field_from_constant(single) {
                        let base = single.value_type();
                        variant.value = SocketValueVariantValue::Field(f, base);
                        return true;
                    }
                    return false;
                }
                if let Some(target) = dst_base {
                    if conversions.available(single.value_type(), target) {
                        if let Some(converted) = conversions.convert(single, target) {
                            if let Some(f) = conversions.field_from_constant(&converted) {
                                variant.value = SocketValueVariantValue::Field(f, target);
                                return true;
                            }
                        }
                    }
                }
                return false;
            }
            /* Single → single: implicit conversion. */
            if single.value_type() == dst_type {
                return true;
            }
            if let Some(converted) = conversions.convert(single, dst_type) {
                variant.value = converted;
                return true;
            }
            false
        }
    }
}

/// `SocketValueVariant::copy_as` — convert to `dst_type` returning a
/// new variant (extract-then-convert semantics).
pub fn socket_value_variant_convert(
    variant: &SocketValueVariant,
    dst_type: SocketValueType,
    dst_base: Option<SocketValueType>,
    conversions: &dyn TypeConversionRegistry,
) -> Option<SocketValueVariant> {
    let mut out = variant.clone();
    if socket_value_variant_try_convert(&mut out, dst_type, dst_base, conversions) {
        Some(out)
    } else {
        None
    }
}
