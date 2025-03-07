use std::{
    alloc::Layout,
    any::{type_name, TypeId},
    borrow::Cow,
    collections::HashMap,
};

pub trait Component: 'static {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct ComponentId(usize);

impl ComponentId {
    #[inline]
    pub const fn index(&self) -> usize {
        let Self(id) = *self;
        id
    }
}

impl From<ComponentId> for usize {
    #[inline]
    fn from(value: ComponentId) -> Self {
        value.index()
    }
}

#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    layout: Layout,
}

impl ComponentDescriptor {
    #[inline]
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        type_id: Option<TypeId>,
        layout: Layout,
    ) -> Self {
        Self {
            name: name.into(),
            type_id,
            layout,
        }
    }

    #[inline]
    pub fn of<T>() -> Self
    where
        T: Component,
    {
        Self {
            name: type_name::<T>().into(),
            type_id: Some(TypeId::of::<T>()),
            layout: Layout::new::<T>(),
        }
    }

    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        let Self { type_id, .. } = self;
        type_id.clone()
    }

    #[inline]
    pub fn name(&self) -> &str {
        let Self { name, .. } = self;
        name.as_ref()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = self;
        layout.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    id: ComponentId,
    descriptor: ComponentDescriptor,
}

impl ComponentInfo {
    #[inline]
    pub fn id(&self) -> ComponentId {
        let Self { id, .. } = self;
        id.clone()
    }

    #[inline]
    pub fn type_id(&self) -> Option<TypeId> {
        let Self { descriptor, .. } = self;
        descriptor.type_id()
    }

    #[inline]
    pub fn name(&self) -> &str {
        let Self { descriptor, .. } = self;
        descriptor.name()
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { descriptor, .. } = self;
        descriptor.layout()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ComponentRegistry {
    components: Vec<ComponentInfo>,
    type_ids: HashMap<TypeId, ComponentId>,
}

impl ComponentRegistry {
    #[inline]
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            type_ids: HashMap::new(),
        }
    }

    #[inline]
    pub fn register_component<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        let Self {
            components,
            type_ids,
        } = self;

        let type_id = TypeId::of::<T>();
        type_ids
            .entry(type_id)
            .or_insert_with(|| {
                let descriptor = ComponentDescriptor::of::<T>();
                Self::register_inner(components, descriptor)
            })
            .clone()
    }

    #[inline]
    pub fn register_component_with_descriptor(
        &mut self,
        descriptor: ComponentDescriptor,
    ) -> ComponentId {
        let Self { components, .. } = self;
        Self::register_inner(components, descriptor)
    }

    #[inline]
    fn register_inner(
        components: &mut Vec<ComponentInfo>,
        descriptor: ComponentDescriptor,
    ) -> ComponentId {
        let id = ComponentId(components.len());
        let info = ComponentInfo { id, descriptor };
        components.push(info);
        id
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.components.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    #[inline]
    pub fn get_info(&self, id: ComponentId) -> Option<&ComponentInfo> {
        let Self { components, .. } = self;

        let index = id.index();
        components.get(index)
    }

    #[inline]
    pub fn component_id_from(&self, type_id: TypeId) -> Option<ComponentId> {
        let Self { type_ids, .. } = self;
        type_ids.get(&type_id).cloned()
    }

    #[inline]
    pub fn component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
    {
        let type_id = TypeId::of::<T>();
        self.component_id_from(type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        _x: f32,
        _y: f32,
        _z: f32,
    }

    impl Component for Position {}

    #[test]
    fn new() {
        let components = ComponentRegistry::new();
        assert_eq!(components.len(), 0);
    }

    #[test]
    fn register_type() {
        let mut components = ComponentRegistry::new();
        assert_eq!(components.len(), 0);
        assert_eq!(components.component_id::<Position>(), None);

        let id = components.register_component::<Position>();
        assert_eq!(components.len(), 1);
        assert_eq!(id.index(), 0);
        assert_eq!(components.component_id::<Position>(), Some(id));

        assert_eq!(components.register_component::<Position>(), id);

        let info = components
            .get_info(id)
            .expect("info of just registered component should present");
        assert_eq!(info.id(), id);
        assert_eq!(info.type_id(), Some(TypeId::of::<Position>()));
        assert_eq!(info.name(), type_name::<Position>());
        assert_eq!(info.layout(), Layout::new::<Position>());
    }

    #[test]
    fn register_with_descriptor() {
        let mut components = ComponentRegistry::new();
        components.register_component::<Position>();
        assert_eq!(components.len(), 1);

        let descriptor = ComponentDescriptor::new("Mass", None, Layout::new::<f32>());
        let id = components.register_component_with_descriptor(descriptor);
        assert_eq!(components.len(), 2);
        assert_eq!(id.index(), 1);

        let info = components
            .get_info(id)
            .expect("info of just registered component should present");
        assert_eq!(info.id(), id);
        assert_eq!(info.type_id(), None);
        assert_eq!(info.name(), "Mass");
        assert_eq!(info.layout(), Layout::new::<f32>());
    }
}
