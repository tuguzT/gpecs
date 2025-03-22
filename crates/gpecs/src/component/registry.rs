use std::{
    any::{type_name, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use crate::soa::traits::FieldDescriptor;

use super::Component;

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
    desc: FieldDescriptor,
}

impl ComponentDescriptor {
    #[inline]
    pub fn new(name: impl Into<Cow<'static, str>>, desc: FieldDescriptor) -> Self {
        Self {
            name: name.into(),
            type_id: None,
            desc,
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
            desc: FieldDescriptor::of::<T>(),
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
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
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
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { descriptor, .. } = self;
        descriptor.descriptor()
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
