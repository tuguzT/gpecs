use crate::component::Component;

use super::{
    ComponentId, ComponentIds, ComponentInfo, ComponentRegistryView, component_id_from_usize,
    traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentRegistry<Meta, Mapping = ()>
where
    Mapping: ?Sized,
{
    components: Vec<Meta>,
    mapping: Mapping,
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping> {
    #[inline]
    pub unsafe fn with_mapping(mapping: Mapping) -> Self {
        Self {
            components: Vec::new(),
            mapping,
        }
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Mapping: Default,
{
    #[inline]
    pub fn new() -> Self {
        let mapping = Mapping::default();
        unsafe { Self::with_mapping(mapping) }
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Mapping: ?Sized,
{
    #[inline]
    pub fn register_component_with(&mut self, meta: Meta) -> ComponentId {
        let Self { components, .. } = self;
        Self::register_inner(components, meta)
    }

    #[inline]
    fn register_inner(components: &mut Vec<Meta>, meta: Meta) -> ComponentId {
        let index = components.len();
        let component_id = component_id_from_usize(index);

        components.push(meta);

        component_id
    }

    #[inline]
    pub fn as_view(&self) -> ComponentRegistryView<'_, Meta, &Mapping> {
        let Self {
            components,
            mapping,
        } = self;
        unsafe { ComponentRegistryView::from_parts(components, mapping) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.as_view().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_view().is_empty()
    }

    #[inline]
    pub fn get_component_info(&self, component_id: ComponentId) -> Option<ComponentInfo<&Meta>> {
        self.as_view().into_get_component_info(component_id)
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds {
        self.as_view().component_ids()
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Mapping: ComponentIdFrom + ?Sized,
{
    #[inline]
    pub fn component_id_from(&self, key: Mapping::Key) -> Option<ComponentId> {
        self.as_view().component_id_from(key)
    }

    #[inline]
    pub fn component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
        Mapping::Key: FromComponentType,
    {
        self.as_view().component_id::<T>()
    }
}

impl<Meta, Mapping> ComponentRegistry<Meta, Mapping>
where
    Meta: FromComponentType,
    Mapping: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
{
    #[inline]
    pub fn register_component<T>(&mut self) -> ComponentId
    where
        T: Component,
    {
        let Self {
            components,
            mapping,
        } = self;

        let key = FromComponentType::from_component::<T>();
        let new_meta = || {
            let meta = Meta::from_component::<T>();
            Self::register_inner(components, meta)
        };
        mapping.component_id_from_or_insert_with(key, new_meta)
    }
}

impl<Meta, Mapping> Default for ComponentRegistry<Meta, Mapping>
where
    Mapping: Default,
{
    fn default() -> Self {
        Self::new()
    }
}
