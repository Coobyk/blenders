//! Port of `intern/node_socket_value_iter.cc` — `socket_value_visitor`:
//! recursive check/edit walk over `SocketValueVariant`, `GeometrySet`,
//! `nodes::Bundle`/`Closure`/`GList`, components and GP layers.
//!
//! `GPointer`/`CPPType` → `(CppTypeTag, usize)`; `nodes::Bundle`,
//! `Closure`, `GList` are modeled as owned value trees so the
//! traversal is self-contained; attribute accessors are injected.

/// `CPPType::is<T>` discrimination — the closed set this file tests.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CppTypeTag {
    GeometrySet,
    BundlePtr,
    ClosurePtr,
    SocketValueVariant,
    Other,
}

/// `GPointer`/`GMutablePointer` — typed pointer to a value.
#[derive(Clone, Copy)]
pub struct GPtr {
    pub type_tag: CppTypeTag,
    /// Handle to the pointed-to value (GeometrySet/Bundle/Closure
    /// resolved by the caller's store).
    pub value: usize,
}

/// `nodes::BundleItem` — only `BundleItemSocketValue` is visited;
/// other variants pass through untouched.
#[derive(Clone)]
pub enum BundleItemValue {
    SocketValue(SocketValueVariant),
    Other,
}

/// `nodes::Bundle::items()`.
#[derive(Clone, Default)]
pub struct Bundle {
    /// `(name, value)` pairs (name unused by the visitor).
    pub items: Vec<(String, BundleItemValue)>,
}

/// `nodes::Closure`.
#[derive(Clone, Default)]
pub struct Closure {
    pub captured_values: Vec<SocketValueVariant>,
    pub default_input_values: Vec<SocketValueVariant>,
}

/// `nodes::GList` — homogeneous typed list.
#[derive(Clone)]
pub enum GList {
    SocketValueVariant(Vec<SocketValueVariant>),
    GeometrySet(Vec<GeometrySetData>),
    BundlePtr(Vec<Option<usize>>),
    ClosurePtr(Vec<Option<usize>>),
    Other,
}

/// `SocketValueVariant` — single value / field / list.
#[derive(Clone)]
pub enum SocketValueVariant {
    Single(GPtr),
    /// `fn::GField` handle.
    Field(usize),
    List(Box<GList>),
}

impl SocketValueVariant {
    pub fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }
    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field(_))
    }
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }
}

/// `GeometryComponent::Type`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComponentType {
    Mesh,
    PointCloud,
    Instance,
    Volume,
    Curve,
    Edit,
    GreasePencil,
}

const ALL_COMPONENT_TYPES: [ComponentType; 7] = [
    ComponentType::Mesh,
    ComponentType::PointCloud,
    ComponentType::Instance,
    ComponentType::Volume,
    ComponentType::Curve,
    ComponentType::Edit,
    ComponentType::GreasePencil,
];

/// `GeometryComponent` — attribute accessor + optional payloads
/// (Instances references, GP layers) resolved through handles.
#[derive(Clone)]
pub struct GeometryComponentData {
    pub type_: ComponentType,
    /// `bke::AttributeAccessor` handle.
    pub attributes: Option<usize>,
    /// `Instances::references()` — InstanceReference payloads.
    pub instance_references: Vec<InstanceReference>,
    /// `greasepencil::Drawing` strokes attribute handles per layer.
    pub grease_pencil_drawings: Vec<usize>,
}

/// `bke::InstanceReference`.
#[derive(Clone)]
pub enum InstanceReference {
    /// `Type::GeometrySet`.
    GeometrySet(Box<GeometrySetData>),
    /// Other types convert `to_geometry_set` lazily.
    Object(usize),
    Collection(usize),
}

impl InstanceReference {
    /// `reference.type()`.
    pub fn is_geometry_set(&self) -> bool {
        matches!(self, Self::GeometrySet(_))
    }
}

/// `GeometrySet` — components by type + optional bundle.
#[derive(Clone, Default)]
pub struct GeometrySetData {
    pub components: Vec<GeometryComponentData>,
    pub bundle: Option<Bundle>,
}

impl GeometrySetData {
    /// `geometry_set.has(type)`.
    pub fn has(&self, type_: ComponentType) -> bool {
        self.components.iter().any(|c| c.type_ == type_)
    }
    /// `geometry_set.has_bundle()`.
    pub fn has_bundle(&self) -> bool {
        self.bundle.is_some()
    }
    fn component(&self, type_: ComponentType) -> Option<&GeometryComponentData> {
        self.components.iter().find(|c| c.type_ == type_)
    }
    fn component_mut(&mut self, type_: ComponentType) -> Option<&mut GeometryComponentData> {
        self.components.iter_mut().find(|c| c.type_ == type_)
    }
}

/// `VisitParams` — injected per-kind callbacks.
///
/// `check_*` returns `Option<bool>` (None = absent, treated as
/// "not checked"); `edit_*` perform the edit in place.
pub struct VisitParams<'a> {
    /// Only check closures for editability (non-editable values).
    pub check_non_editable: bool,
    /// Skip `to_geometry_set` for non-geometry instance references.
    pub ignore_non_geometry_instances: bool,
    /// `check_AttributeAccessor(accessor) -> bool`.
    pub check_attribute_accessor: Option<&'a dyn Fn(usize) -> bool>,
    /// `edit_AttributeAccessor(accessor)`.
    pub edit_attribute_accessor: Option<&'a dyn Fn(usize)>,
    /// `check_GField(field) -> bool`.
    pub check_gfield: Option<&'a dyn Fn(usize) -> bool>,
    /// `edit_GField(field)`.
    pub edit_gfield: Option<&'a dyn Fn(usize)>,
    /// Resolve `GPtr` handle → `GeometrySetData` (check path).
    pub geometry_set_at: Option<&'a dyn Fn(usize) -> Option<GeometrySetData>>,
    /// `InstanceReference::to_geometry_set` for non-geometry refs.
    pub to_geometry_set: Option<&'a dyn Fn(&InstanceReference) -> GeometrySetData>,
    /// Resolve `GPtr` handle → `Bundle` (check path).
    pub bundle_at: Option<&'a dyn Fn(usize) -> Option<Bundle>>,
    /// Resolve `GPtr` handle → `Closure` (check path).
    pub closure_at: Option<&'a dyn Fn(usize) -> Option<Closure>>,
}

impl Default for VisitParams<'_> {
    fn default() -> Self {
        Self {
            check_non_editable: false,
            ignore_non_geometry_instances: false,
            check_attribute_accessor: None,
            edit_attribute_accessor: None,
            check_gfield: None,
            edit_gfield: None,
            geometry_set_at: None,
            to_geometry_set: None,
            bundle_at: None,
            closure_at: None,
        }
    }
}

/// `RecursiveVisitor`.
pub struct RecursiveVisitor<'a> {
    params: &'a VisitParams<'a>,
}

impl<'a> RecursiveVisitor<'a> {
    pub fn new(params: &'a VisitParams<'a>) -> Self {
        Self { params }
    }

    /// `check_SocketValueVariant`.
    pub fn check_socket_value_variant(&mut self, value: &SocketValueVariant) -> bool {
        match value {
            SocketValueVariant::Single(ptr) => self.check_gpointer(*ptr),
            SocketValueVariant::Field(field) => self.check_gfield(*field),
            SocketValueVariant::List(list) => self.check_glist(list),
        }
    }

    /// `edit_SocketValueVariant`.
    pub fn edit_socket_value_variant(&mut self, value: &mut SocketValueVariant) {
        match value {
            SocketValueVariant::Single(ptr) => self.edit_gpointer(*ptr),
            SocketValueVariant::Field(field) => self.edit_gfield(*field),
            SocketValueVariant::List(list) => self.edit_glist(list),
        }
    }

    /// `check_GPointer`.
    fn check_gpointer(&mut self, ptr: GPtr) -> bool {
        match ptr.type_tag {
            CppTypeTag::GeometrySet => {
                let Some(resolve) = self.params.geometry_set_at else {
                    return false;
                };
                match resolve(ptr.value) {
                    Some(gs) => self.check_geometry_set(&gs),
                    None => false,
                }
            }
            CppTypeTag::BundlePtr => match self.params.bundle_at.and_then(|f| f(ptr.value)) {
                Some(bundle) => self.check_bundle(&bundle),
                None => false,
            },
            CppTypeTag::ClosurePtr => {
                if self.params.check_non_editable {
                    match self.params.closure_at.and_then(|f| f(ptr.value)) {
                        Some(closure) => self.check_closure(&closure),
                        None => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// `edit_GPointer` — GeometrySet/Bundle/Closure edits happen on
    /// the caller-resolved store; closures can't be edited.
    fn edit_gpointer(&mut self, ptr: GPtr) {
        match ptr.type_tag {
            CppTypeTag::GeometrySet => {
                if let Some(resolve) = self.params.geometry_set_at {
                    if let Some(mut gs) = resolve(ptr.value) {
                        self.edit_geometry_set(&mut gs);
                    }
                }
            }
            CppTypeTag::BundlePtr => {
                if let Some(resolve) = self.params.bundle_at {
                    if let Some(mut bundle) = resolve(ptr.value) {
                        self.edit_bundle(&mut bundle);
                    }
                }
            }
            /* Can't edit closures. */
            _ => {}
        }
    }

    /// `check_GeometrySet`.
    pub fn check_geometry_set(&mut self, geometry_set: &GeometrySetData) -> bool {
        for type_ in ALL_COMPONENT_TYPES {
            if !geometry_set.has(type_) {
                continue;
            }
            let component = geometry_set.component(type_).unwrap();
            if self.check_geometry_component(component) {
                return true;
            }
        }
        if let Some(bundle) = &geometry_set.bundle {
            if self.check_bundle(bundle) {
                return true;
            }
        }
        false
    }

    /// `edit_GeometrySet` — checks bundle first (edit only when
    /// check passes), then per-component check-then-edit.
    pub fn edit_geometry_set(&mut self, geometry_set: &mut GeometrySetData) {
        if let Some(bundle) = &geometry_set.bundle {
            if self.check_bundle(bundle) {
                self.edit_bundle(geometry_set.bundle.as_mut().unwrap());
            }
        }
        for type_ in ALL_COMPONENT_TYPES {
            let need_edit = match geometry_set.component(type_) {
                Some(c) => self.check_geometry_component(c),
                None => continue,
            };
            if !need_edit {
                continue;
            }
            self.edit_geometry_component(geometry_set.component_mut(type_).unwrap());
        }
    }

    /// `check_GeometryComponent` — attributes + instance refs +
    /// GP drawings.
    fn check_geometry_component(&mut self, component: &GeometryComponentData) -> bool {
        if let Some(attr) = component.attributes {
            if self.check_attribute_accessor(attr) {
                return true;
            }
        }
        if component.type_ == ComponentType::Instance {
            for reference in &component.instance_references {
                if self.check_instance_reference(reference) {
                    return true;
                }
            }
        }
        if component.type_ == ComponentType::GreasePencil {
            for &drawing in &component.grease_pencil_drawings {
                if self.check_attribute_accessor(drawing) {
                    return true;
                }
            }
        }
        false
    }

    /// `edit_GeometryComponent`.
    fn edit_geometry_component(&mut self, component: &mut GeometryComponentData) {
        if let Some(attr) = component.attributes {
            self.edit_attribute_accessor(attr);
        }
        if component.type_ == ComponentType::Instance {
            for reference in &mut component.instance_references {
                self.edit_instance_reference(reference);
            }
        }
        if component.type_ == ComponentType::GreasePencil {
            for &drawing in &component.grease_pencil_drawings {
                self.edit_attribute_accessor(drawing);
            }
        }
    }

    /// `check_AttributeAccessor`.
    fn check_attribute_accessor(&self, accessor: usize) -> bool {
        self.params
            .check_attribute_accessor
            .map(|f| f(accessor))
            .unwrap_or(false)
    }

    /// `edit_AttributeAccessor`.
    fn edit_attribute_accessor(&self, accessor: usize) {
        if let Some(f) = self.params.edit_attribute_accessor {
            f(accessor);
        }
    }

    /// `check_InstanceReference` — GeometrySet refs recurse; others
    /// convert via `to_geometry_set` unless ignored.
    fn check_instance_reference(&mut self, reference: &InstanceReference) -> bool {
        match reference {
            InstanceReference::GeometrySet(gs) => self.check_geometry_set(gs),
            _ => {
                if !self.params.ignore_non_geometry_instances {
                    if let Some(to_gs) = self.params.to_geometry_set {
                        let gs = to_gs(reference);
                        return self.check_geometry_set(&gs);
                    }
                }
                false
            }
        }
    }

    /// `edit_InstanceReference` — only GeometrySet refs editable.
    fn edit_instance_reference(&mut self, reference: &mut InstanceReference) {
        if let InstanceReference::GeometrySet(gs) = reference {
            self.edit_geometry_set(gs);
        }
    }

    /// `check_Bundle` — SocketValue items only.
    fn check_bundle(&mut self, bundle: &Bundle) -> bool {
        for (_, item) in &bundle.items {
            if let BundleItemValue::SocketValue(v) = item {
                if self.check_socket_value_variant(v) {
                    return true;
                }
            }
        }
        false
    }

    /// `edit_Bundle`.
    fn edit_bundle(&mut self, bundle: &mut Bundle) {
        for (_, item) in bundle.items.iter_mut() {
            if let BundleItemValue::SocketValue(v) = item {
                self.edit_socket_value_variant(v);
            }
        }
    }

    /// `check_Closure` — captured + default input values.
    fn check_closure(&mut self, closure: &Closure) -> bool {
        for value in &closure.captured_values {
            if self.check_socket_value_variant(value) {
                return true;
            }
        }
        for value in &closure.default_input_values {
            if self.check_socket_value_variant(value) {
                return true;
            }
        }
        false
    }

    /// `check_GField`.
    fn check_gfield(&self, field: usize) -> bool {
        self.params.check_gfield.map(|f| f(field)).unwrap_or(false)
    }

    /// `edit_GField`.
    fn edit_gfield(&self, field: usize) {
        if let Some(f) = self.params.edit_gfield {
            f(field);
        }
    }

    /// `check_GList` — typed element walk, short-circuit.
    fn check_glist(&mut self, list: &GList) -> bool {
        match list {
            GList::SocketValueVariant(items) => items
                .iter()
                .any(|v| self.check_socket_value_variant(v)),
            GList::GeometrySet(items) => items.iter().any(|gs| self.check_geometry_set(gs)),
            GList::BundlePtr(items) => items.iter().flatten().any(|&ptr| {
                self.params
                    .bundle_at
                    .and_then(|f| f(ptr))
                    .map(|b| self.check_bundle(&b))
                    .unwrap_or(false)
            }),
            GList::ClosurePtr(items) => items.iter().flatten().any(|&ptr| {
                self.params
                    .closure_at
                    .and_then(|f| f(ptr))
                    .map(|c| self.check_closure(&c))
                    .unwrap_or(false)
            }),
            GList::Other => false,
        }
    }

    /// `edit_GList` — SocketValueVariant/GeometrySet/BundlePtr
    /// (ClosurePtr not editable).
    fn edit_glist(&mut self, list: &mut GList) {
        match list {
            GList::SocketValueVariant(items) => {
                for v in items.iter_mut() {
                    self.edit_socket_value_variant(v);
                }
            }
            GList::GeometrySet(items) => {
                for gs in items.iter_mut() {
                    self.edit_geometry_set(gs);
                }
            }
            GList::BundlePtr(items) => {
                for ptr in items.iter_mut().flatten() {
                    if let Some(resolve) = self.params.bundle_at {
                        if let Some(mut bundle) = resolve(*ptr) {
                            self.edit_bundle(&mut bundle);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// `edit_recursive(SocketValueVariant)`.
pub fn edit_recursive(value: &mut SocketValueVariant, params: &VisitParams) {
    RecursiveVisitor::new(params).edit_socket_value_variant(value);
}

/// `edit_recursive(GeometrySet)`.
pub fn edit_recursive_geometry_set(value: &mut GeometrySetData, params: &VisitParams) {
    RecursiveVisitor::new(params).edit_geometry_set(value);
}

/// `check_recursive(SocketValueVariant)`.
pub fn check_recursive(value: &SocketValueVariant, params: &VisitParams) -> bool {
    RecursiveVisitor::new(params).check_socket_value_variant(value)
}

/// `check_recursive(GeometrySet)`.
pub fn check_recursive_geometry_set(value: &GeometrySetData, params: &VisitParams) -> bool {
    RecursiveVisitor::new(params).check_geometry_set(value)
}
