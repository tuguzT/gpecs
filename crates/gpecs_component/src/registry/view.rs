use crate::{
    Component,
    registry::{
        ComponentId, ComponentIds, ComponentInfo,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentRegistryView<'a, D, M = ()>
where
    M: ?Sized,
{
    descriptors: &'a [D],
    mapping: M,
}

impl<'a, D, M> ComponentRegistryView<'a, D, M> {
    #[inline]
    pub unsafe fn from_parts(descriptors: &'a [D], mapping: M) -> Self {
        Self {
            descriptors,
            mapping,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [D], M) {
        let Self {
            descriptors,
            mapping,
        } = self;
        (descriptors, mapping)
    }

    #[inline]
    pub fn into_component_descriptor(self, id: ComponentId) -> Option<&'a D> {
        let Self { descriptors, .. } = self;
        get_component_descriptor(descriptors, id)
    }
}

impl<D, M> ComponentRegistryView<'_, D, M>
where
    M: ?Sized,
{
    #[inline]
    pub fn as_view(&self) -> ComponentRegistryView<'_, D, &M> {
        let Self {
            descriptors,
            mapping,
        } = self;
        unsafe { ComponentRegistryView::from_parts(descriptors, mapping) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { descriptors, .. } = self;
        descriptors.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { descriptors, .. } = self;
        descriptors.is_empty()
    }

    #[inline]
    pub fn get_component_descriptor(&self, id: ComponentId) -> Option<&D> {
        let Self { descriptors, .. } = self;
        get_component_descriptor(descriptors, id)
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds {
        ComponentIds::new(self)
    }
}

impl<D, M> ComponentRegistryView<'_, D, M>
where
    M: ComponentIdFrom + ?Sized,
{
    #[inline]
    pub fn component_id_from(&self, key: M::Key) -> Option<ComponentId> {
        let Self { mapping, .. } = self;
        mapping.component_id_from(key)
    }
}

impl<D, M> ComponentRegistryView<'_, D, M>
where
    M: ComponentIdFrom<Key: FromComponentType> + ?Sized,
{
    #[inline]
    pub fn component_id<T>(&self) -> Option<ComponentId>
    where
        T: Component,
    {
        let key = FromComponentType::from_component::<T>();
        self.component_id_from(key)
    }

    #[inline]
    pub fn get_component_descriptor_of<T>(&self) -> Option<ComponentInfo<&D>>
    where
        T: Component,
    {
        let component_id = self.component_id::<T>()?;
        let meta = self.get_component_descriptor(component_id)?;

        let info = ComponentInfo::new(component_id, meta);
        Some(info)
    }
}

impl<'a, Meta, Mapping> ComponentRegistryView<'a, Meta, Mapping>
where
    Mapping: ComponentIdFrom<Key: FromComponentType>,
{
    #[inline]
    pub fn into_component_descriptor_of<T>(self) -> Option<ComponentInfo<&'a Meta>>
    where
        T: Component,
    {
        let component_id = self.component_id::<T>()?;
        let meta = self.into_component_descriptor(component_id)?;

        let info = ComponentInfo::new(component_id, meta);
        Some(info)
    }
}

impl<Meta, Mapping> Default for ComponentRegistryView<'_, Meta, Mapping>
where
    Mapping: Default,
{
    #[inline]
    fn default() -> Self {
        let descriptors = &[];
        let mapping = Mapping::default();
        unsafe { Self::from_parts(descriptors, mapping) }
    }
}

impl<Meta, Mapping> Clone for ComponentRegistryView<'_, Meta, Mapping>
where
    Mapping: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            descriptors,
            ref mapping,
        } = *self;

        let mapping = mapping.clone();
        unsafe { Self::from_parts(descriptors, mapping) }
    }
}

impl<Meta, Mapping> Copy for ComponentRegistryView<'_, Meta, Mapping> where Mapping: Copy {}

fn get_component_descriptor<D>(descriptors: &[D], component_id: ComponentId) -> Option<&D> {
    let index: usize = component_id.into_u32().try_into().ok()?;
    descriptors.get(index)
}
