#![expect(clippy::module_inception)]

use crate::{
    Component,
    registry::{
        ComponentId, ComponentIds, ComponentRegistryView, component_id_from_usize,
        traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType, PushBackArray},
    },
};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentRegistry<T, M = ()>
where
    M: ?Sized,
{
    components: T,
    mapping: M,
}

impl<T, M> ComponentRegistry<T, M> {
    #[inline]
    pub unsafe fn from_parts(components: T, mapping: M) -> Self {
        Self {
            components,
            mapping,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (T, M) {
        let Self {
            components,
            mapping,
        } = self;
        (components, mapping)
    }
}

impl<T, M> ComponentRegistry<T, M>
where
    T: Default,
    M: Default,
{
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, M> ComponentRegistry<T, M>
where
    T: PushBackArray,
    M: ?Sized,
{
    #[inline]
    pub fn register_component_with(&mut self, meta: T::Item) -> ComponentId {
        let Self { components, .. } = self;
        Self::register_inner(components, meta)
    }

    #[inline]
    fn register_inner(components: &mut T, meta: T::Item) -> ComponentId {
        let index = components.as_ref().len();
        let component_id = component_id_from_usize(index);

        components.push(meta);

        component_id
    }

    #[inline]
    pub fn as_view(&self) -> ComponentRegistryView<'_, T::Item, &M> {
        let Self {
            components,
            mapping,
        } = self;

        let components = components.as_ref();
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
    pub fn get_component_descriptor(&self, component_id: ComponentId) -> Option<&T::Item> {
        self.as_view().into_component_descriptor(component_id)
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds {
        self.as_view().component_ids()
    }
}

impl<T, M> ComponentRegistry<T, M>
where
    T: PushBackArray,
    M: ComponentIdFrom + ?Sized,
{
    #[inline]
    pub fn component_id_from(&self, key: M::Key) -> Option<ComponentId> {
        self.as_view().component_id_from(key)
    }
}

impl<T, M> ComponentRegistry<T, M>
where
    T: PushBackArray,
    M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
{
    #[inline]
    pub fn component_id<C>(&self) -> Option<ComponentId>
    where
        C: Component,
    {
        self.as_view().component_id::<C>()
    }

    #[inline]
    pub fn get_component_descriptor_of<C>(&self) -> Option<(ComponentId, &T::Item)>
    where
        C: Component,
    {
        self.as_view().into_component_descriptor_of::<C>()
    }
}

impl<T, M> ComponentRegistry<T, M>
where
    T: PushBackArray<Item: FromComponentType>,
    M: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
{
    #[inline]
    pub fn register_component<C>(&mut self) -> ComponentId
    where
        C: Component,
    {
        let Self {
            components,
            mapping,
        } = self;

        let key = M::Key::from_component::<C>();
        let new_meta = || {
            let meta = T::Item::from_component::<C>();
            Self::register_inner(components, meta)
        };
        mapping.component_id_from_or_insert_with(key, new_meta)
    }
}
