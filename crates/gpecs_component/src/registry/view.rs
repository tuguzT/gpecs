use crate::{
    Component,
    registry::{
        ComponentId, ComponentIds, ComponentInfo,
        traits::{ComponentIdFrom, FromComponentType},
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentRegistryView<'a, Meta, Mapping = ()>
where
    Mapping: ?Sized,
{
    components: &'a [Meta],
    mapping: Mapping,
}

impl<'a, Meta, Mapping> ComponentRegistryView<'a, Meta, Mapping> {
    #[inline]
    pub unsafe fn from_parts(components: &'a [Meta], mapping: Mapping) -> Self {
        Self {
            components,
            mapping,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (&'a [Meta], Mapping) {
        let Self {
            components,
            mapping,
        } = self;
        (components, mapping)
    }

    #[inline]
    pub fn into_get_component_info(self, id: ComponentId) -> Option<ComponentInfo<&'a Meta>> {
        let Self { components, .. } = self;
        get_component_info(components, id)
    }
}

impl<Meta, Mapping> ComponentRegistryView<'_, Meta, Mapping>
where
    Mapping: ?Sized,
{
    #[inline]
    pub fn len(&self) -> usize {
        let Self { components, .. } = self;
        components.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        let Self { components, .. } = self;
        components.is_empty()
    }

    #[inline]
    pub fn get_component_info(&self, id: ComponentId) -> Option<ComponentInfo<&Meta>> {
        let Self { components, .. } = self;
        get_component_info(components, id)
    }

    #[inline]
    pub fn component_ids(&self) -> ComponentIds {
        ComponentIds::new(self)
    }
}

impl<Meta, Mapping> ComponentRegistryView<'_, Meta, Mapping>
where
    Mapping: ComponentIdFrom + ?Sized,
{
    #[inline]
    pub fn component_id_from(&self, key: Mapping::Key) -> Option<ComponentId> {
        let Self { mapping, .. } = self;
        mapping.component_id_from(key)
    }
}

impl<Meta, Mapping> ComponentRegistryView<'_, Meta, Mapping>
where
    Mapping: ComponentIdFrom<Key: FromComponentType> + ?Sized,
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
    pub fn get_component_info_of<T>(&self) -> Option<ComponentInfo<&Meta>>
    where
        T: Component,
    {
        let component_id = self.component_id::<T>()?;
        self.get_component_info(component_id)
    }
}

impl<'a, Meta, Mapping> ComponentRegistryView<'a, Meta, Mapping>
where
    Mapping: ComponentIdFrom<Key: FromComponentType>,
{
    #[inline]
    pub fn into_get_component_info_of<T>(self) -> Option<ComponentInfo<&'a Meta>>
    where
        T: Component,
    {
        let component_id = self.component_id::<T>()?;
        self.into_get_component_info(component_id)
    }
}

impl<Meta, Mapping> Default for ComponentRegistryView<'_, Meta, Mapping>
where
    Mapping: Default,
{
    #[inline]
    fn default() -> Self {
        let components = &[];
        let mapping = Mapping::default();
        unsafe { Self::from_parts(components, mapping) }
    }
}

impl<Meta, Mapping> Clone for ComponentRegistryView<'_, Meta, Mapping>
where
    Mapping: Clone,
{
    fn clone(&self) -> Self {
        let Self {
            components,
            ref mapping,
        } = *self;

        let mapping = mapping.clone();
        unsafe { Self::from_parts(components, mapping) }
    }
}

impl<Meta, Mapping> Copy for ComponentRegistryView<'_, Meta, Mapping> where Mapping: Copy {}

fn get_component_info<Meta>(
    components: &[Meta],
    component_id: ComponentId,
) -> Option<ComponentInfo<&Meta>> {
    let index: usize = component_id.into_u32().try_into().ok()?;
    let meta = components.get(index)?;

    let info = ComponentInfo::new(component_id, meta);
    Some(info)
}
