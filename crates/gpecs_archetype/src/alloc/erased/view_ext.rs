use gpecs_component::registry::{
    ComponentId, ComponentRegistryView,
    traits::{ComponentIdFrom, FromComponentType},
};

use crate::{
    bundle::Bundle,
    erased::{
        ErasedArchetype, ErasedArchetypeView,
        error::{IncompatibleArchetypeError, IncompatibleArchetypeExactError},
    },
};

pub trait ErasedArchetypeViewExt {
    fn check_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>;

    fn check_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized;

    fn check_exact_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>;

    fn check_exact_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized;
}

impl<U> ErasedArchetypeViewExt for &U
where
    U: ErasedArchetypeViewExt + ?Sized,
{
    #[inline]
    fn check_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        (**self).check_compatibility_for(components, component_ids)
    }

    #[inline]
    fn check_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        (**self).check_compatibility_of::<B, T>(components)
    }

    #[inline]
    fn check_exact_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        (**self).check_exact_compatibility_for(components, component_ids)
    }

    #[inline]
    fn check_exact_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        (**self).check_exact_compatibility_of::<B, T>(components)
    }
}

impl<Meta> ErasedArchetypeViewExt for ErasedArchetypeView<'_, Meta> {
    #[inline]
    fn check_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let other = ErasedArchetype::<()>::new(components, component_ids)?;
        self.check_compatibility(other.as_view())?;
        Ok(())
    }

    #[inline]
    fn check_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let other = ErasedArchetype::<()>::of::<B, _, _>(components)?;
        self.check_compatibility(other.as_view())?;
        Ok(())
    }

    #[inline]
    fn check_exact_compatibility_for<I>(
        &self,
        components: &ComponentRegistryView<impl Sized, impl ?Sized>,
        component_ids: I,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        I: IntoIterator<Item = ComponentId>,
    {
        let other = ErasedArchetype::<()>::new(components, component_ids)?;
        self.check_exact_compatibility(other.as_view())?;
        Ok(())
    }

    #[inline]
    fn check_exact_compatibility_of<B, T>(
        &self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<(), IncompatibleArchetypeExactError>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let other = ErasedArchetype::<()>::of::<B, _, _>(components)?;
        self.check_exact_compatibility(other.as_view())?;
        Ok(())
    }
}
