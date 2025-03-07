use std::{
    alloc::Layout,
    any::{type_name, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use crate::entity::Entity;

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

impl From<ComponentId> for Entity {
    #[inline]
    fn from(value: ComponentId) -> Self {
        Entity::new(value.index(), 0)
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
    pub fn new<T>() -> Self
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
                let descriptor = ComponentDescriptor::new::<T>();
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
